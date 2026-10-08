use super::native_auth::{AuthConfig, VerifiedEnrollment};
use super::native_https::NativeHttps;
use super::*;
use std::io::Read;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigFile {
    auth_origin: String,
    api_origin: String,
    publishable_key: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopAuthStatus {
    requires_login: bool,
    configured: bool,
    authenticated: bool,
    user_name: Option<String>,
    can_write: bool,
    role: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    offline_until: Option<DateTime<Utc>>,
    offline: bool,
    reason: String,
}
// Opaque result passed between native threads, never serialized into the webview.
pub struct PendingDesktopLogin {
    epoch: u64,
    verified: VerifiedEnrollment,
    restore: Option<super::recovery::NativeRestore>,
}
// An opaque native job. The webview cannot supply its session, account, scope,
// database path or credentials. A dedicated connection avoids I/O under UI locks.
pub struct PendingMemberSync {
    path: std::path::PathBuf,
    session: super::removal::Session,
    epoch: u64,
}
pub struct PendingSessionRenewal {
    path: std::path::PathBuf,
    session: super::removal::Session,
    epoch: u64,
}
pub struct SessionRenewal {
    epoch: u64,
    verified: Option<VerifiedEnrollment>,
    refused: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberSyncOutcome {
    state: &'static str,
    pushed: usize,
    pages: usize,
    reason: String,
    #[serde(skip)]
    denied: bool,
    #[serde(skip)]
    epoch: u64,
}
pub struct DesktopAuth {
    config: Option<AuthConfig>,
    config_error: Option<String>,
    epoch: AtomicU64,
    online_transport: AtomicBool,
    transport: Mutex<Option<super::member_http::MemberApi<NativeHttps>>>,
    renewal: Mutex<Option<super::native_auth::RefreshState>>,
}
impl DesktopAuth {
    pub fn load(directory: &Path) -> Self {
        Self::load_with_bundled(directory, None)
    }
    // Installers carry only HTTPS origins and a public Auth key. Never read a
    // server env file or ask the person installing the app to configure one.
    #[cfg(feature = "packaged-auth")]
    pub fn load_packaged(directory: &Path) -> Self {
        Self::load_with_bundled(
            directory,
            Some(include_str!("../../desktop-auth.production.json")),
        )
    }
    fn parse_config(bytes: &[u8]) -> Result<ConfigFile> {
        if bytes.len() > 16 * 1024 {
            return Err("Native sign-in configuration exceeds its limit; values withheld".into());
        }
        let file: ConfigFile = serde_json::from_slice(bytes)
            .map_err(|_| "Invalid native sign-in configuration; values withheld")?;
        AuthConfig::new(&file.auth_origin, &file.publishable_key, &file.api_origin)?;
        Ok(file)
    }
    fn load_with_bundled(directory: &Path, bundled: Option<&str>) -> Self {
        let path = directory.join("desktop-auth.json");
        let config = (|| -> Result<Option<AuthConfig>> {
            let bundled = bundled
                .map(|bytes| Self::parse_config(bytes.as_bytes()))
                .transpose()?;
            let file = match std::fs::File::open(path) {
                Ok(file) => Some(file),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(_) => {
                    return Err(
                        "Native sign-in configuration cannot be read; values withheld".into(),
                    )
                }
            };
            let local = file
                .map(|file| {
                    let mut bytes = Vec::new();
                    file.take(16 * 1024 + 1)
                        .read_to_end(&mut bytes)
                        .map_err(|_| {
                            "Native sign-in configuration cannot be read; values withheld"
                        })?;
                    Self::parse_config(&bytes)
                })
                .transpose()?;
            // Preserve a pre-existing workstation configuration and refuse to
            // redirect an installed gym build to another API/project/key.
            if bundled
                .as_ref()
                .zip(local.as_ref())
                .is_some_and(|(a, b)| a != b)
            {
                return Err("This computer's server settings differ from the installed app; contact the Administrator. Existing settings were retained.".into());
            }
            let Some(file) = bundled.or(local) else {
                return Ok(None);
            };
            Ok(Some(AuthConfig::new(
                &file.auth_origin,
                &file.publishable_key,
                &file.api_origin,
            )?))
        })();
        let (config, config_error) = match config {
            Ok(config) => (config, None),
            Err(error) => (None, Some(error)),
        };
        Self {
            config,
            config_error,
            epoch: AtomicU64::new(0),
            online_transport: AtomicBool::new(false),
            transport: Mutex::new(None),
            renewal: Mutex::new(None),
        }
    }
    pub fn initialize(&self, store: &mut Store) -> Result<()> {
        if self.config.is_some() || self.config_error.is_some() {
            store.conn.execute("INSERT INTO metadata VALUES('native_auth_required','1') ON CONFLICT(key) DO NOTHING",[]).map_err(db_error)?;
        }
        self.lock(store)
    }
    fn requires_login(&self, store: &Store) -> Result<bool> {
        let saved:bool=store.conn.query_row("SELECT EXISTS(SELECT 1 FROM metadata WHERE key IN ('native_auth_required','member_sync_scope'))",[],|r|r.get(0)).map_err(db_error)?;
        Ok(saved || self.config.is_some() || self.config_error.is_some())
    }
    pub fn status(&self, store: &Store) -> Result<DesktopAuthStatus> {
        let identity = store.native_identity(false).ok();
        let requires_login = self.requires_login(store)?;
        let restored = store.requires_restore_reconciliation()?;
        let reason=self.config_error.clone().unwrap_or_else(|| {
            if self.config.is_none() {
                "Native sign-in needs desktop-auth.json with the approved HTTPS API and public Auth settings".into()
            } else if identity.is_none() && restored {
                "This restored backup needs online recovery. Sign in with the approved account on this computer to reconcile retained changes and download server history. Failed or conflicting recovery keeps the backup guarded.".into()
            } else if identity.is_none() {
                "Sign in with the approved Administrator account. Internet access is required for sign-in".into()
            } else { String::new() }
        });
        Ok(DesktopAuthStatus {
            requires_login,
            configured: self.config.is_some(),
            authenticated: identity.is_some(),
            can_write: identity.is_some()
                && store
                    .removal_session
                    .as_ref()
                    .is_some_and(|session| session.can_write),
            user_name: identity.as_ref().map(|(name, _)| name.clone()),
            role: identity.as_ref().map(|_| "Administrator".into()),
            expires_at: identity.as_ref().and_then(|_| {
                store
                    .removal_session
                    .as_ref()
                    .map(|session| session.expires_at)
            }),
            offline_until: super::offline_access::available_until(store),
            offline: identity.is_some() && !self.online_transport.load(Ordering::SeqCst),
            reason,
        })
    }
    // Every production business IPC calls this inside its Store mutex, including
    // read/export/restore operations. No UI-provided role or UUID is accepted.
    pub fn authorize(&self, store: &mut Store, write: bool) -> Result<()> {
        store
            .conn
            .execute("DELETE FROM temp.native_actor", [])
            .map_err(db_error)?;
        if !self.requires_login(store)? {
            return Ok(());
        }
        let (name, user) = store.native_identity(write)?;
        store
            .conn
            .execute(
                "INSERT INTO temp.native_actor VALUES(?1,?2)",
                params![user, format!("{name} ({user})")],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn begin(&self, store: &mut Store) -> Result<u64> {
        self.lock(store)?;
        if let Some(error) = &self.config_error {
            return Err(error.clone());
        }
        self.config
            .as_ref()
            .ok_or("Configure native sign-in before using the account login")?;
        Ok(self.epoch.load(Ordering::SeqCst))
    }
    pub fn authenticate(
        &self,
        path: &Path,
        epoch: u64,
        email: &str,
        password: &str,
    ) -> Result<PendingDesktopLogin> {
        let config = self
            .config
            .as_ref()
            .ok_or("Native sign-in is not configured")?;
        let mut store = Store::open(path)?;
        let mut restore = if store.requires_restore_reconciliation()? {
            Some(store.prepare_restore_reconciliation()?)
        } else {
            None
        };
        let (device, secret) = if restore.is_some() {
            store.native_restore_device_secret()?
        } else {
            store.prepare_native_device()?;
            store.native_device_secret()?
        };
        drop(store);
        let mut exchange = NativeHttps::new(&config.auth_origin, &config.api_origin);
        let verified =
            super::native_auth::login(&mut exchange, config, &device, secret, email, password)?;
        if let Some(candidate) = restore.as_mut() {
            let mut transport = verified
                .recovery_transport(NativeHttps::new(&config.auth_origin, &config.api_origin))?;
            candidate.run(&verified, &mut transport, || {
                if epoch != self.epoch.load(Ordering::SeqCst) {
                    return Err("Online restore recovery was cancelled".into());
                }
                Ok(())
            })?;
        }
        Ok(PendingDesktopLogin {
            epoch,
            verified,
            restore,
        })
    }
    pub fn finish(
        &self,
        store: &mut Store,
        pending: PendingDesktopLogin,
    ) -> Result<DesktopAuthStatus> {
        self.finish_with_vault(store, pending, &mut super::native_credentials::OsVault)
    }
    fn finish_with_vault(
        &self,
        store: &mut Store,
        pending: PendingDesktopLogin,
        vault: &mut impl super::native_credentials::OfflineVault,
    ) -> Result<DesktopAuthStatus> {
        if pending.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("Sign-in was cancelled or superseded; permissions remain locked".into());
        }
        if !pending.verified.is_administrator() {
            return Err("The approved Administrator account is required for this desktop".into());
        }
        if pending.verified.valid_until() <= Utc::now() {
            return Err("Verified account session expired; sign in again".into());
        }
        // Take lock before committing local enrollment; poisoning cannot leave a
        // new authorized session after the caller received a failure.
        let mut renewal = self
            .renewal
            .try_lock()
            .map_err(|_| "Native session renewal is running; retry shortly")?;
        let mut transport = self
            .transport
            .try_lock()
            .map_err(|_| "Native session unavailable")?;
        if let Some(recovery) = pending.restore {
            recovery.commit(store)?;
        }
        store.enroll_native(&pending.verified)?;
        // Only the agreed Administrator can operate this desktop. Reception
        // enrollment remains an API role, without desktop control grants.
        if let Err(error) = store.native_identity(false) {
            store.lock_native_session();
            return Err(error);
        }
        let config = self
            .config
            .as_ref()
            .ok_or("Native sign-in is not configured")?;
        // Online access can continue if OS storage is locked, but the UI must
        // not advertise offline restart access. Stale grants are invalidated.
        super::offline_access::invalidate(store)?;
        let saved = pending
            .verified
            .offline_access(store, &config.auth_origin)
            .and_then(|grant| grant.save(store, vault));
        *renewal = pending.verified.renewal();
        *transport = Some(
            pending
                .verified
                .member_transport(NativeHttps::new(&config.auth_origin, &config.api_origin))?,
        );
        self.online_transport.store(true, Ordering::SeqCst);
        drop(transport);
        let mut status = self.status(store)?;
        if saved.is_err() {
            status.reason = "Signed in online. Offline access could not be saved; unlock the OS credential store and sign in again.".into();
        }
        Ok(status)
    }
    pub fn prepare_session_renewal(&self, store: &Store) -> Result<Option<PendingSessionRenewal>> {
        let Some(session) = store.removal_session.as_ref() else {
            return Ok(None);
        };
        super::removal::current_session(&store.conn, session)?;
        let active: bool = store.conn.query_row("SELECT EXISTS(SELECT 1 FROM users u WHERE u.id=?1 AND u.active=1 AND EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=u.id AND r.name='Administrator' COLLATE NOCASE))", [&session.user_id], |r| r.get(0)).map_err(db_error)?;
        if !active {
            return Err("An active enrolled Administrator account is required".into());
        }
        Ok(Some(PendingSessionRenewal {
            path: store.database_path(),
            session: session.clone(),
            epoch: self.epoch.load(Ordering::SeqCst),
        }))
    }
    pub fn run_session_renewal(&self, job: PendingSessionRenewal) -> Result<SessionRenewal> {
        let mut slot = self
            .renewal
            .try_lock()
            .map_err(|_| "Native session renewal is already running")?;
        let unchanged = |store: &Store| -> Result<()> {
            if job.epoch != self.epoch.load(Ordering::SeqCst) {
                return Err("Session renewal cancelled".into());
            }
            super::removal::current_session(&store.conn, &job.session)
        };
        let store = Store::open(&job.path)?;
        unchanged(&store)?;
        let mut result = SessionRenewal {
            epoch: job.epoch,
            verified: None,
            refused: false,
        };
        let Some(state) = slot.as_ref() else {
            return Ok(result);
        };
        if state.expires_at > Utc::now() + chrono::Duration::seconds(120) {
            return Ok(result);
        }
        let config = self
            .config
            .as_ref()
            .ok_or("Native sign-in is not configured")?;
        let mut exchange = NativeHttps::new(&config.auth_origin, &config.api_origin);
        let refreshed = super::native_auth::renew(&mut exchange, config, state);
        if let Err(error) = unchanged(&store) {
            *slot = None;
            return Err(error);
        }
        match refreshed {
            Ok(verified) => {
                // Retain the rotated credential even when committing the native
                // result is interrupted; keep it due until finish succeeds.
                let due = state.expires_at;
                *slot = verified.renewal();
                if let Some(next) = slot.as_mut() {
                    next.expires_at = due;
                }
                result.verified = Some(verified);
            }
            Err(super::native_auth::RefreshFailure::Refused) => {
                *slot = None;
                result.refused = true;
            }
            Err(super::native_auth::RefreshFailure::Unavailable) => (), // do not extend access or discard offline data
        }
        Ok(result)
    }
    pub fn finish_session_renewal(
        &self,
        store: &mut Store,
        result: SessionRenewal,
    ) -> Result<DesktopAuthStatus> {
        if result.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("Session renewal cancelled".into());
        }
        if result.refused {
            self.logout(store)?;
        }
        if let Some(verified) = result.verified {
            return self.finish(
                store,
                PendingDesktopLogin {
                    epoch: result.epoch,
                    verified,
                    restore: None,
                },
            );
        }
        self.status(store)
    }
    pub fn unlock_offline(&self, store: &mut Store) -> Result<DesktopAuthStatus> {
        self.lock(store)?;
        let config = self
            .config
            .as_ref()
            .ok_or("Native sign-in is not configured")?;
        // Verify that the separate device possession secret still matches this
        // database before accepting the cached OS authorization item.
        store.native_device_secret()?;
        super::offline_access::unlock(
            store,
            &mut super::native_credentials::OsVault,
            &config.auth_origin,
            &config.api_origin,
            Utc::now(),
        )?;
        self.status(store)
    }
    pub fn logout(&self, store: &mut Store) -> Result<()> {
        self.lock(store)?;
        super::offline_access::invalidate(store)?;
        let device: String = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        use super::native_credentials::OfflineVault;
        super::native_credentials::OsVault.clear_access(&device)
    }
    pub fn prepare_member_sync(&self, store: &Store) -> Result<PendingMemberSync> {
        store.native_identity(false)?;
        if !self.online_transport.load(Ordering::SeqCst) {
            return Err(
                "Connect and sign in online to synchronize members. Offline changes were retained."
                    .into(),
            );
        }
        Ok(PendingMemberSync {
            path: store.database_path(),
            session: store
                .removal_session
                .as_ref()
                .ok_or("Verified sign-in is required")?
                .clone(),
            epoch: self.epoch.load(Ordering::SeqCst),
        })
    }
    pub fn run_member_sync(&self, job: PendingMemberSync) -> Result<MemberSyncOutcome> {
        use super::member_worker::{Limits, Run};
        if job.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("Synchronization cancelled because the account session changed".into());
        }
        let mut slot = self
            .transport
            .try_lock()
            .map_err(|_| "Member synchronization is already running or unavailable")?;
        let transport = slot
            .as_mut()
            .ok_or("Sign in online before synchronizing members")?;
        let mut worker = Store::open(&job.path)?;
        worker.removal_session = Some(job.session);
        // The durable session nonce is rechecked inside every receipt/page
        // transaction, so a logout or another app cannot commit a late reply.
        let run = worker.run_business_sync(
            transport,
            Utc::now(),
            Limits {
                pushes: 10,
                pages: 5,
            },
        );
        if job.epoch != self.epoch.load(Ordering::SeqCst) {
            *slot = None;
            return Err("Synchronization cancelled because the account session changed; unconfirmed changes were retained".into());
        }
        let (state, pushed, pages, reason) = match run? {
            Run::Complete { pushed, pages } => (
                "complete",
                pushed,
                pages,
                "Gym transactions confirmed by the server.".into(),
            ),
            Run::Yielded { pushed, pages } => (
                "yielded",
                pushed,
                pages,
                "Gym synchronization will continue on the next run.".into(),
            ),
            Run::Deferred { .. } => (
                "deferred",
                0,
                0,
                "Waiting before retrying the server. Local changes were retained.".into(),
            ),
            Run::Failed => (
                "failed",
                0,
                0,
                "Member synchronization failed. Local changes were retained.".into(),
            ),
            Run::Blocked => (
                "blocked",
                0,
                0,
                super::business_sync::status(&worker.conn)?["lastError"].as_str().map(|s|format!("{s} Open Settings → Server synchronization to review and retry the retained transaction.")).unwrap_or_else(||"A retained gym transaction needs review in Settings → Server synchronization.".into()),
            ),
        };
        let denied = worker.removal_session.is_none();
        if denied {
            *slot = None;
            self.online_transport.store(false, Ordering::SeqCst);
        }
        Ok(MemberSyncOutcome {
            state,
            pushed,
            pages,
            reason,
            denied,
            epoch: job.epoch,
        })
    }
    pub fn finish_member_sync(
        &self,
        store: &mut Store,
        outcome: MemberSyncOutcome,
    ) -> Result<MemberSyncOutcome> {
        if outcome.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("Synchronization account session changed".into());
        }
        if outcome.denied {
            self.logout(store)?;
        }
        Ok(outcome)
    }
    pub fn member_sync_status(&self, store: &Store, snapshot: &mut Value) {
        let available =
            self.online_transport.load(Ordering::SeqCst) && store.native_identity(false).is_ok();
        snapshot["memberSync"]["available"] = json!(available);
        snapshot["businessSync"]["available"] = json!(available);
        snapshot["memberSync"]["reason"] = json!(if available {
            "Gym synchronization is connected. Changes require actual server receipts."
        } else {
            "Sign in online to synchronize members. Offline changes are retained."
        });
    }
    pub fn lock(&self, store: &mut Store) -> Result<()> {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.online_transport.store(false, Ordering::SeqCst);
        store.lock_native_session();
        store.conn.execute("INSERT INTO metadata VALUES('native_session_nonce',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [id()]).map_err(db_error)?;
        store
            .conn
            .execute("DELETE FROM temp.native_actor", [])
            .map_err(db_error)?;
        match self.transport.try_lock() {
            Ok(mut transport) => *transport = None,
            Err(std::sync::TryLockError::WouldBlock) => (), // running job sees the epoch/nonce revocation
            Err(_) => return Err("Native session unavailable".into()),
        }
        match self.renewal.try_lock() {
            Ok(mut renewal) => *renewal = None,
            Err(std::sync::TryLockError::WouldBlock) => (),
            Err(_) => return Err("Native session renewal unavailable".into()),
        }
        Ok(())
    }
}
impl Store {
    pub(super) fn native_identity(&self, write: bool) -> Result<(String, String)> {
        let session = self
            .removal_session
            .as_ref()
            .ok_or("Verified Administrator sign-in is required")?;
        super::removal::current_session(&self.conn, session)?;
        if session.expires_at <= Utc::now() {
            return Err("Account session expired; sign in again".into());
        }
        if write && !session.can_write {
            return Err("This approved computer has read-only access".into());
        }
        let name:Option<String>=self.conn.query_row("SELECT display_name FROM users u WHERE id=?1 AND active=1 AND EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON r.id=ur.role_id WHERE ur.user_id=u.id AND r.name='Administrator' COLLATE NOCASE)",[&session.user_id],|r|r.get(0)).optional().map_err(db_error)?;
        Ok((
            name.ok_or("An active enrolled Administrator account is required")?,
            session.user_id.clone(),
        ))
    }
}
pub(super) fn actor_label(conn: &Connection) -> Result<String> {
    Ok(conn
        .query_row("SELECT label FROM temp.native_actor LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(db_error)?
        .unwrap_or_else(|| ACTOR.into()))
}

#[cfg(test)]
#[path = "desktop_auth_tests.rs"]
mod tests;
