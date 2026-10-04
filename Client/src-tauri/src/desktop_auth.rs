use super::native_auth::{AuthConfig, VerifiedEnrollment};
use super::native_https::NativeHttps;
use super::*;
use std::io::Read;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

#[derive(Deserialize)]
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
    reason: String,
}
// Opaque result passed between native threads, never serialized into the webview.
pub struct PendingDesktopLogin {
    epoch: u64,
    verified: VerifiedEnrollment,
}
pub struct DesktopAuth {
    config: Option<AuthConfig>,
    config_error: Option<String>,
    epoch: AtomicU64,
    transport: Mutex<Option<super::member_http::MemberApi<NativeHttps>>>,
}
impl DesktopAuth {
    pub fn load(directory: &Path) -> Self {
        let path = directory.join("desktop-auth.json");
        let config = (|| -> Result<Option<AuthConfig>> {
            let file = match std::fs::File::open(path) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(_) => {
                    return Err(
                        "Native sign-in configuration cannot be read; values withheld".into(),
                    )
                }
            };
            let mut bytes = Vec::new();
            file.take(16 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| "Native sign-in configuration cannot be read; values withheld")?;
            if bytes.len() > 16 * 1024 {
                return Err(
                    "Native sign-in configuration exceeds its limit; values withheld".into(),
                );
            }
            let file: ConfigFile = serde_json::from_slice(&bytes)
                .map_err(|_| "Invalid native sign-in configuration; values withheld")?;
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
            transport: Mutex::new(None),
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
        let reason=self.config_error.clone().unwrap_or_else(|| {
            if self.config.is_none() {
                "Native sign-in needs desktop-auth.json with the approved HTTPS API and public Auth settings".into()
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
        let store = Store::open(path)?;
        let (device, secret) = store.native_device_secret()?;
        drop(store);
        let mut exchange = NativeHttps::new(&config.auth_origin, &config.api_origin);
        let verified =
            super::native_auth::login(&mut exchange, config, &device, secret, email, password)?;
        Ok(PendingDesktopLogin { epoch, verified })
    }
    pub fn finish(
        &self,
        store: &mut Store,
        pending: PendingDesktopLogin,
    ) -> Result<DesktopAuthStatus> {
        if pending.epoch != self.epoch.load(Ordering::SeqCst) {
            return Err("Sign-in was cancelled or superseded; permissions remain locked".into());
        }
        if !pending.verified.is_administrator() {
            return Err("The approved Administrator account is required for this desktop".into());
        }
        // Take lock before committing local enrollment; poisoning cannot leave a
        // new authorized session after the caller received a failure.
        let mut transport = self
            .transport
            .lock()
            .map_err(|_| "Native session unavailable")?;
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
        *transport = Some(
            pending
                .verified
                .member_transport(NativeHttps::new(&config.auth_origin, &config.api_origin))?,
        );
        self.status(store)
    }
    pub fn lock(&self, store: &mut Store) -> Result<()> {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        store.lock_native_session();
        store
            .conn
            .execute("DELETE FROM temp.native_actor", [])
            .map_err(db_error)?;
        *self
            .transport
            .lock()
            .map_err(|_| "Native session unavailable")? = None;
        Ok(())
    }
}
impl Store {
    fn native_identity(&self, write: bool) -> Result<(String, String)> {
        let session = self
            .removal_session
            .as_ref()
            .ok_or("Verified Administrator sign-in is required")?;
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
