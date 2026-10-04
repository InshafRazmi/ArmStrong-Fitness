// Temporary real SQLite + an explicitly mocked vault. OS acceptance is separate.
use super::*;
use std::collections::HashMap;
#[derive(Default)]
struct Vault {
    secrets: HashMap<String, String>,
    creates: usize,
    fail_read: bool,
    fail_write: bool,
    bad_readback: bool,
}
impl CredentialVault for Vault {
    fn read(&mut self, device: &str) -> Result<Option<String>> {
        if self.fail_read {
            return Err(VAULT_ERROR.into());
        }
        if self.bad_readback && self.creates > 0 {
            return Ok(None);
        }
        Ok(self.secrets.get(device).cloned())
    }
    fn create(&mut self, device: &str, secret: &str) -> Result<()> {
        self.creates += 1;
        if self.fail_write {
            return Err(VAULT_ERROR.into());
        }
        assert!(
            self.secrets.insert(device.into(), secret.into()).is_none(),
            "Must not overwrite"
        );
        Ok(())
    }
}
struct Fixture {
    directory: std::path::PathBuf,
    store: Store,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("armstrong-device-{}", id()));
        std::fs::create_dir(&directory).unwrap();
        let store = Store::open(&directory.join("test.sqlite3")).unwrap();
        Self { directory, store }
    }
    fn marker(&self) -> Option<String> {
        self.store
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key='native_device_secret_sha256'",
                [],
                |r| r.get(0),
            )
            .optional()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
#[test]
fn preparation_keeps_device_and_secret_across_retries_and_restart() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    let before = f.store.snapshot().unwrap();
    let first = f.store.prepare_device_with(&mut vault).unwrap();
    let secret = vault.secrets[&first.device_id].clone();
    checked_secret(&secret).unwrap();
    assert_eq!(digest(&secret), first.secret_sha256);
    assert_ne!(secret, first.secret_sha256);
    assert_eq!(first.sqlite_path, f.store.path.to_string_lossy());
    assert_eq!(f.store.snapshot().unwrap(), before);
    assert_eq!(vault.creates, 1);
    let repeat = f.store.prepare_device_with(&mut vault).unwrap();
    assert_eq!(repeat.secret_sha256, first.secret_sha256);
    let mut restart = Store::open(&f.store.path).unwrap();
    let repeat = restart.prepare_device_with(&mut vault).unwrap();
    assert_eq!(repeat.device_id, first.device_id);
    assert_eq!(repeat.secret_sha256, first.secret_sha256);
    assert_eq!(vault.creates, 1);
}
#[test]
fn missing_changed_or_corrupt_credentials_never_rotate() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    let first = f.store.prepare_device_with(&mut vault).unwrap();
    vault.secrets.clear();
    assert_eq!(
        f.store.prepare_device_with(&mut vault).err().unwrap(),
        RECONCILE
    );
    vault
        .secrets
        .insert(first.device_id.clone(), "b".repeat(64));
    assert_eq!(
        f.store.prepare_device_with(&mut vault).err().unwrap(),
        RECONCILE
    );
    vault
        .secrets
        .insert(first.device_id.clone(), "corrupt-private-value".into());
    let error = f.store.prepare_device_with(&mut vault).err().unwrap();
    assert!(!error.contains("corrupt-private-value"));
    assert_eq!(vault.creates, 1);
    assert_eq!(f.marker(), Some(first.secret_sha256));
}
#[test]
fn locked_store_write_failure_and_failed_readback_leave_sqlite_unprepared() {
    for kind in 0..3 {
        let mut f = Fixture::new();
        let mut vault = Vault {
            fail_read: kind == 0,
            fail_write: kind == 1,
            bad_readback: kind == 2,
            ..Vault::default()
        };
        assert!(f.store.prepare_device_with(&mut vault).is_err());
        assert!(f.marker().is_none());
        assert!(f.store.removal_session.is_none());
        assert_eq!(
            f.store.snapshot().unwrap()["memberSync"]["available"],
            false
        );
    }
}
#[test]
fn orphaned_os_write_is_reused_after_local_transaction_failure() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    f.store.conn.execute_batch("CREATE TRIGGER reject_device_marker BEFORE INSERT ON metadata WHEN NEW.key='native_device_secret_sha256' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(f.store.prepare_device_with(&mut vault).is_err());
    assert!(f.marker().is_none());
    let stored = vault.secrets.values().next().unwrap().clone();
    f.store
        .conn
        .execute_batch("DROP TRIGGER reject_device_marker;")
        .unwrap();
    assert_eq!(
        f.store
            .prepare_device_with(&mut vault)
            .unwrap()
            .secret_sha256,
        digest(&stored)
    );
    assert_eq!(vault.creates, 1);
}
#[test]
fn restored_bound_or_changed_device_requires_reconciliation() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('member_sync_scope','test')",
            [],
        )
        .unwrap();
    assert_eq!(
        f.store.prepare_device_with(&mut vault).err().unwrap(),
        RECONCILE
    );
    assert_eq!(vault.creates, 0);
    f.store
        .conn
        .execute("DELETE FROM metadata WHERE key='member_sync_scope'", [])
        .unwrap();
    let first = f.store.prepare_device_with(&mut vault).unwrap();
    f.store
        .conn
        .execute("UPDATE metadata SET value=?1 WHERE key='device_id'", [id()])
        .unwrap();
    assert_eq!(
        f.store.prepare_device_with(&mut vault).err().unwrap(),
        RECONCILE
    );
    f.store
        .conn
        .execute(
            "UPDATE metadata SET value=?1 WHERE key='device_id'",
            [&first.device_id],
        )
        .unwrap();
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('restore_requires_reconciliation','true')",
            [],
        )
        .unwrap();
    assert!(f
        .store
        .prepare_device_with(&mut vault)
        .err()
        .unwrap()
        .contains("Restored database"));
    assert_eq!(vault.creates, 1);
}
#[test]
fn corrupt_identity_or_hash_never_recreates_a_secret() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    f.store
        .conn
        .execute(
            "UPDATE metadata SET value='invalid' WHERE key='device_id'",
            [],
        )
        .unwrap();
    assert!(f.store.prepare_device_with(&mut vault).is_err());
    assert_eq!(vault.creates, 0);
    f.store
        .conn
        .execute("UPDATE metadata SET value=?1 WHERE key='device_id'", [id()])
        .unwrap();
    f.store
        .conn
        .execute(
            "INSERT INTO metadata VALUES('native_device_secret_sha256','invalid')",
            [],
        )
        .unwrap();
    assert!(f.store.prepare_device_with(&mut vault).is_err());
    assert_eq!(vault.creates, 0);
}
#[test]
fn approval_snapshot_and_actual_backup_bytes_contain_no_device_secret() {
    let mut f = Fixture::new();
    let mut vault = Vault::default();
    let approval = f.store.prepare_device_with(&mut vault).unwrap();
    let secret = vault.secrets[&approval.device_id].clone();
    let value = serde_json::to_value(&approval).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert!(!value.to_string().contains(&secret));
    assert!(!f.store.snapshot().unwrap().to_string().contains(&secret));
    let export = f.store.export_backup().unwrap();
    let file = std::fs::read(export["path"].as_str().unwrap()).unwrap();
    let envelope: BackupEnvelope = serde_json::from_slice(&file).unwrap();
    assert!(!envelope
        .data
        .windows(secret.len())
        .any(|window| window == secret.as_bytes()));
    assert!(envelope
        .data
        .windows(approval.secret_sha256.len())
        .any(|window| window == approval.secret_sha256.as_bytes()));
}
#[test]
fn generated_credentials_are_independent_and_full_length() {
    let a = generate().unwrap();
    let b = generate().unwrap();
    checked_secret(&a).unwrap();
    checked_secret(&b).unwrap();
    assert_ne!(a, b);
}
