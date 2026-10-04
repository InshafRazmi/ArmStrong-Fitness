// Bounded access saved only after native online Auth + server enrollment. The
// operating-system login/keyring is the offline unlock factor, not a cached gym
// password. SQLite/backups contain only a random matching marker and UI expiry.
use super::member_worker::SyncScope;
use super::native_credentials::{OfflineVault, ACCESS_LIMIT};
use super::*;

const PERIOD_DAYS: i64 = 7;
const LOCKED: &str = "Offline access is unavailable or expired. Connect and sign in again. Local records were retained.";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Grant {
    version: u8,
    id: String,
    scope: SyncScope,
    auth_origin: String,
    subject: String,
    user_id: String,
    secret_sha256: String,
    can_write: bool,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
}
impl Grant {
    pub(super) fn new(
        store: &Store,
        scope: &SyncScope,
        subject: &str,
        can_write: bool,
        auth_origin: &str,
    ) -> Result<Self> {
        let (_, user_id) = store.native_identity(false)?;
        let now = Utc::now();
        let secret_sha256 = store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='native_device_secret_sha256'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        Ok(Self {
            version: 1,
            id: id(),
            scope: scope.clone(),
            auth_origin: auth_origin.into(),
            subject: subject.into(),
            user_id,
            secret_sha256,
            can_write,
            issued_at: now,
            expires_at: now + chrono::Duration::days(PERIOD_DAYS),
            last_seen_at: now,
        })
    }
    fn encode(&self) -> Result<String> {
        let value = serde_json::to_string(self).map_err(|_| LOCKED)?;
        if value.len() > ACCESS_LIMIT {
            return Err(LOCKED.into());
        }
        Ok(value)
    }
    pub(super) fn save(&self, store: &mut Store, vault: &mut impl OfflineVault) -> Result<()> {
        // Invalidate any previous grant before touching the OS item. A failed
        // update must not retain stale permissions from an earlier online login.
        invalidate(store)?;
        let value = self.encode()?;
        vault.save_access(&self.scope.device_id, &value)?;
        if vault.read_access(&self.scope.device_id)?.as_deref() != Some(&value) {
            return Err(LOCKED.into());
        }
        let marker = json!({"id":self.id,"expiresAt":self.expires_at}).to_string();
        store
            .conn
            .execute(
                "INSERT INTO metadata(key,value) VALUES('offline_access',?1)",
                [marker],
            )
            .map_err(db_error)?;
        Ok(())
    }
}
pub(super) fn invalidate(store: &mut Store) -> Result<()> {
    store
        .conn
        .execute("DELETE FROM metadata WHERE key='offline_access'", [])
        .map_err(db_error)?;
    Ok(())
}
pub(super) fn available_until(store: &Store) -> Option<DateTime<Utc>> {
    let marker: String = store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='offline_access'",
            [],
            |r| r.get(0),
        )
        .ok()?;
    let marker: Value = serde_json::from_str(&marker).ok()?;
    serde_json::from_value(marker["expiresAt"].clone())
        .ok()
        .filter(|time| *time > Utc::now())
}
pub(super) fn unlock(
    store: &mut Store,
    vault: &mut impl OfflineVault,
    auth_origin: &str,
    api_origin: &str,
    now: DateTime<Utc>,
) -> Result<()> {
    store.lock_native_session();
    let device: String = store
        .conn
        .query_row(
            "SELECT value FROM metadata WHERE key='device_id'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    super::native_credentials::checked_device(&device)?;
    let value = vault.read_access(&device)?.ok_or(LOCKED)?;
    if value.len() > ACCESS_LIMIT {
        return Err(LOCKED.into());
    }
    let mut grant: Grant = serde_json::from_str(&value).map_err(|_| LOCKED)?;
    let tx = store
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    let marker: Option<String> = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='offline_access'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    let marker: Value = serde_json::from_str(&marker.ok_or(LOCKED)?).map_err(|_| LOCKED)?;
    let saved: String = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='member_sync_scope'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let scope: SyncScope = serde_json::from_str(&saved).map_err(|_| LOCKED)?;
    let restored: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let hash: String = tx
        .query_row(
            "SELECT value FROM metadata WHERE key='native_device_secret_sha256'",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let active: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM users u JOIN user_roles ur ON ur.user_id=u.id JOIN roles r ON r.id=ur.role_id WHERE u.id=?1 AND u.subject=?2 AND u.active=1 AND r.name='Administrator' COLLATE NOCASE)", params![grant.user_id,grant.subject], |r| r.get(0)).map_err(db_error)?;
    if grant.version != 1
        || marker["id"] != grant.id
        || marker["expiresAt"] != json!(grant.expires_at)
        || grant.scope != scope
        || grant.scope.device_id != device
        || grant.auth_origin != auth_origin
        || grant.scope.server_origin != api_origin
        || hash != grant.secret_sha256
        || restored
        || !active
        || grant.expires_at <= now
        || grant.issued_at > now + chrono::Duration::seconds(60)
        || grant.last_seen_at > now + chrono::Duration::seconds(60)
        || grant.last_seen_at < grant.issued_at
        || grant.expires_at != grant.issued_at + chrono::Duration::days(PERIOD_DAYS)
    {
        return Err(LOCKED.into());
    }
    grant.last_seen_at = now.max(grant.last_seen_at);
    // Persist the time check before granting access; a locked vault fails closed.
    let encoded = grant.encode()?;
    vault.save_access(&device, &encoded)?;
    if vault.read_access(&device)?.as_deref() != Some(&encoded) {
        return Err(LOCKED.into());
    }
    let native_nonce = id();
    tx.execute("INSERT INTO metadata VALUES('native_session_nonce',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [&native_nonce]).map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    store.removal_session = Some(removal::Session {
        user_id: grant.user_id,
        expires_at: grant.expires_at,
        can_write: grant.can_write,
        native_nonce: Some(native_nonce),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Vault {
        value: Option<String>,
        locked: bool,
    }
    impl OfflineVault for Vault {
        fn read_access(&mut self, _: &str) -> Result<Option<String>> {
            if self.locked {
                Err("Vault locked".into())
            } else {
                Ok(self.value.clone())
            }
        }
        fn save_access(&mut self, _: &str, value: &str) -> Result<()> {
            if self.locked {
                Err("Vault locked".into())
            } else {
                self.value = Some(value.into());
                Ok(())
            }
        }
        fn clear_access(&mut self, _: &str) -> Result<()> {
            self.value = None;
            Ok(())
        }
    }
    struct Fixture {
        directory: std::path::PathBuf,
        store: Store,
        vault: Vault,
        now: DateTime<Utc>,
    }
    impl Fixture {
        fn new(can_write: bool) -> Self {
            let directory = std::env::temp_dir().join(format!("armstrong-offline-{}", id()));
            std::fs::create_dir(&directory).unwrap();
            let mut store = Store::open(&directory.join("test.sqlite3")).unwrap();
            let user = id();
            let role = id();
            store
                .conn
                .execute(
                    "INSERT INTO users VALUES(?1,?1,'admin@example.test','Verified Admin',1,1)",
                    [&user],
                )
                .unwrap();
            store
                .conn
                .execute("INSERT INTO roles VALUES(?1,'Administrator')", [&role])
                .unwrap();
            store
                .conn
                .execute("INSERT INTO user_roles VALUES(?1,?2)", params![user, role])
                .unwrap();
            store
                .conn
                .execute(
                    "INSERT INTO metadata VALUES('native_device_secret_sha256',?1)",
                    ["a".repeat(64)],
                )
                .unwrap();
            store.removal_session = Some(removal::Session {
                user_id: user.clone(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
                can_write,
                native_nonce: None,
            });
            let device = store
                .conn
                .query_row(
                    "SELECT value FROM metadata WHERE key='device_id'",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .unwrap();
            let scope = SyncScope::new("https://api.example", &id(), &device).unwrap();
            store.bind_member_scope(scope.clone()).unwrap();
            let mut vault = Vault::default();
            let grant =
                Grant::new(&store, &scope, &user, can_write, "https://auth.example").unwrap();
            let now = grant.issued_at;
            grant.save(&mut store, &mut vault).unwrap();
            store.lock_native_session();
            Self {
                directory,
                store,
                vault,
                now,
            }
        }
        fn unlock(&mut self, now: DateTime<Utc>) -> Result<()> {
            unlock(
                &mut self.store,
                &mut self.vault,
                "https://auth.example",
                "https://api.example",
                now,
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
    #[test]
    fn offline_restart_retains_records_and_requires_the_os_grant() {
        let mut f = Fixture::new(true);
        f.store = Store::open(&f.directory.join("test.sqlite3")).unwrap();
        assert!(f.store.native_identity(false).is_err());
        f.unlock(f.now + chrono::Duration::days(1)).unwrap();
        assert_eq!(f.store.native_identity(true).unwrap().0, "Verified Admin");
        f.vault.value = None;
        assert!(f.unlock(f.now + chrono::Duration::days(1)).is_err());
        assert!(f.store.native_identity(false).is_err());
        assert_eq!(
            f.store
                .conn
                .query_row("SELECT count(*) FROM users", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn offline_read_only_device_cannot_gain_write_permission() {
        let mut f = Fixture::new(false);
        f.unlock(f.now).unwrap();
        assert!(f.store.native_identity(false).is_ok());
        assert!(f.store.native_identity(true).is_err());
    }
    #[test]
    fn expiry_and_clock_rollback_refuse_to_unlock() {
        let mut f = Fixture::new(true);
        assert!(f.unlock(f.now + chrono::Duration::days(7)).is_err());
        assert!(f.unlock(f.now - chrono::Duration::minutes(2)).is_err());
        f.unlock(f.now + chrono::Duration::days(2)).unwrap();
        assert!(f.unlock(f.now + chrono::Duration::days(1)).is_err());
        assert!(f.store.native_identity(false).is_err());
    }
    #[test]
    fn copied_or_modified_sqlite_marker_cannot_grant_offline_access() {
        for change in [
            "UPDATE metadata SET value='{}' WHERE key='offline_access'",
            "UPDATE metadata SET value='wrong-hash' WHERE key='native_device_secret_sha256'",
            "UPDATE metadata SET value='{}' WHERE key='member_sync_scope'",
            "UPDATE users SET active=0",
            "DELETE FROM user_roles",
            "INSERT INTO metadata VALUES('restore_requires_reconciliation','1')",
        ] {
            let mut f = Fixture::new(true);
            f.store.conn.execute_batch(change).unwrap();
            assert!(f.unlock(f.now).is_err(), "{change}");
            assert!(f.store.native_identity(false).is_err());
        }
    }
    #[test]
    fn different_auth_or_api_cannot_unlock_an_existing_gym_grant() {
        let mut f = Fixture::new(true);
        assert!(unlock(
            &mut f.store,
            &mut f.vault,
            "https://other.example",
            "https://api.example",
            f.now
        )
        .is_err());
        assert!(unlock(
            &mut f.store,
            &mut f.vault,
            "https://auth.example",
            "https://other.example",
            f.now
        )
        .is_err());
    }
    #[test]
    fn locked_vault_or_invalid_response_fails_without_local_identity_grants() {
        let mut f = Fixture::new(true);
        f.vault.locked = true;
        assert!(f.unlock(f.now).is_err());
        f.vault.locked = false;
        for value in ["not-json".to_string(), "x".repeat(ACCESS_LIMIT + 1)] {
            f.vault.value = Some(value);
            assert!(f.unlock(f.now).is_err());
        }
        assert!(f.store.native_identity(false).is_err());
    }
    #[test]
    fn logout_marker_invalidation_and_failed_refresh_remove_old_access() {
        let mut f = Fixture::new(true);
        invalidate(&mut f.store).unwrap();
        assert!(f.unlock(f.now).is_err());
        assert!(available_until(&f.store).is_none());
        let mut f = Fixture::new(true);
        let grant: Grant = serde_json::from_str(f.vault.value.as_ref().unwrap()).unwrap();
        f.vault.locked = true;
        assert!(grant.save(&mut f.store, &mut f.vault).is_err());
        f.vault.locked = false;
        assert!(f.unlock(f.now).is_err());
    }
}
