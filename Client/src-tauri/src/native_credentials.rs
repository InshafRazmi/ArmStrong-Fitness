// Device secrets live only in the current OS user's credential store. SQLite
// retains a hash to detect missing/replaced credentials; backups carry no secret.
use super::*;
use sha2::{Digest, Sha256};

const VAULT_ERROR: &str = "OS credential storage is unavailable or locked; unlock it and retry. No device credential was replaced.";
const RECONCILE: &str = "Native device credential is missing or changed; administrator reconciliation is required. No credential was replaced.";
pub(super) trait CredentialVault {
    fn read(&mut self, device: &str) -> Result<Option<String>>;
    fn create(&mut self, device: &str, secret: &str) -> Result<()>;
}
pub(super) struct OsVault;
// Separate OS credential item for a bounded authorization grant. It never
// shares or replaces the device possession secret and is absent from backups.
pub(super) trait OfflineVault {
    fn read_access(&mut self, device: &str) -> Result<Option<String>>;
    fn save_access(&mut self, device: &str, value: &str) -> Result<()>;
    fn clear_access(&mut self, device: &str) -> Result<()>;
}
pub(super) const ACCESS_LIMIT: usize = 2048;

pub(super) fn checked_device(device: &str) -> Result<()> {
    let uuid = Uuid::parse_str(device).map_err(|_| "Invalid saved device identity")?;
    if uuid.to_string() != device || uuid.get_version_num() != 4 {
        return Err("Invalid saved device identity; keep database for recovery".into());
    }
    Ok(())
}
pub(super) fn checked_secret(secret: &str) -> Result<()> {
    if secret.len() != 64
        || !secret
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(RECONCILE.into());
    }
    Ok(())
}
pub(super) fn digest(secret: &str) -> String {
    format!("{:x}", Sha256::digest(secret.as_bytes()))
}
fn generate() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| "OS randomness unavailable; no credential created")?;
    let secret = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    wipe(&mut bytes);
    Ok(secret)
}
// Best effort cleanup of our own buffers, never a claim that all OS/library
// copies or allocator pages have been erased.
fn wipe(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: byte is a live exclusive reference. Volatile prevents dead
        // store elimination when the backing allocation is about to be freed.
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceApproval {
    device_id: String,
    sqlite_path: String,
    secret_sha256: String,
}
impl Store {
    pub(super) fn native_device_secret(&self) -> Result<(String, String)> {
        self.read_existing_device_secret(false)
    }
    // Recovery reads the existing possession proof only. It cannot prepare or
    // replace a credential, clear the restore guard, or authorize local access.
    pub(super) fn native_restore_device_secret(&self) -> Result<(String, String)> {
        self.read_existing_device_secret(true)
    }
    fn read_existing_device_secret(&self, reconciliation: bool) -> Result<(String, String)> {
        let tx = self.conn.unchecked_transaction().map_err(db_error)?;
        let restored: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if restored != reconciliation {
            return Err(
                "Restored database requires verified online reconciliation before sign-in".into(),
            );
        }
        if reconciliation && super::member_worker::scope(&tx)?.is_none() {
            return Err(
                "Restored backup has no verified server scope; keep it for Administrator recovery"
                    .into(),
            );
        }
        let device: String = tx
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        checked_device(&device)?;
        let hash: Option<String> = tx
            .query_row(
                "SELECT value FROM metadata WHERE key='native_device_secret_sha256'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let hash =
            hash.ok_or("Prepare this computer in Server synchronization before signing in")?;
        let secret = OsVault.read(&device)?.ok_or(RECONCILE)?;
        checked_secret(&secret)?;
        if digest(&secret) != hash {
            return Err(RECONCILE.into());
        }
        tx.commit().map_err(db_error)?;
        Ok((device, secret))
    }
    pub fn database_path(&self) -> std::path::PathBuf {
        self.path.clone()
    }
    // Invoke on a dedicated connection/background thread. The SQLite write lock
    // serializes creation across processes, so two app instances cannot rotate
    // each other's secret. Nothing binds a gym, grants access or enables sync.
    pub fn prepare_native_device(&mut self) -> Result<DeviceApproval> {
        self.prepare_device_with(&mut OsVault)
    }
    fn prepare_device_with(&mut self, vault: &mut impl CredentialVault) -> Result<DeviceApproval> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let restored: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='restore_requires_reconciliation')",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if restored {
            return Err(
                "Restored database requires administrator reconciliation before device preparation"
                    .into(),
            );
        }
        let device: String = tx
            .query_row(
                "SELECT value FROM metadata WHERE key='device_id'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        checked_device(&device)?;
        let saved_hash: Option<String> = tx
            .query_row(
                "SELECT value FROM metadata WHERE key='native_device_secret_sha256'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let bound: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='member_sync_scope')",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let secret = match vault.read(&device)? {
            Some(secret) => secret,
            None if saved_hash.is_some() || bound => return Err(RECONCILE.into()),
            None => {
                let secret = generate()?;
                vault.create(&device, &secret)?;
                // A successful OS write alone is insufficient: verify the exact
                // persistent value before recording an approval hash locally.
                if vault.read(&device)?.as_deref() != Some(secret.as_str()) {
                    return Err(VAULT_ERROR.into());
                }
                secret
            }
        };
        checked_secret(&secret)?;
        let hash = digest(&secret);
        if saved_hash.as_ref().is_some_and(|saved| saved != &hash) {
            return Err(RECONCILE.into());
        }
        tx.execute("INSERT INTO metadata(key,value) VALUES('native_device_secret_sha256',?1) ON CONFLICT(key) DO NOTHING", [&hash]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(DeviceApproval {
            device_id: device,
            sqlite_path: self.path.to_string_lossy().into(),
            secret_sha256: hash,
        })
    }
}

#[cfg(target_os = "linux")]
#[path = "native_credentials_linux.rs"]
mod platform;
#[cfg(windows)]
impl CredentialVault for OsVault {
    fn read(&mut self, device: &str) -> Result<Option<String>> {
        windows::WindowsVault.read(device)
    }
    fn create(&mut self, device: &str, secret: &str) -> Result<()> {
        windows::WindowsVault.create(device, secret)
    }
}
#[cfg(windows)]
impl OfflineVault for OsVault {
    fn read_access(&mut self, device: &str) -> Result<Option<String>> {
        windows::WindowsVault.read_access(device)
    }
    fn save_access(&mut self, device: &str, value: &str) -> Result<()> {
        windows::WindowsVault.save_access(device, value)
    }
    fn clear_access(&mut self, device: &str) -> Result<()> {
        windows::WindowsVault.clear_access(device)
    }
}
// Also typecheck SDK binding code in Linux tests; this is not Windows execution.
#[cfg(any(windows, test))]
#[path = "native_credentials_windows.rs"]
mod windows;
#[cfg(not(any(target_os = "linux", windows)))]
impl CredentialVault for OsVault {
    fn read(&mut self, _: &str) -> Result<Option<String>> {
        Err(VAULT_ERROR.into())
    }
    fn create(&mut self, _: &str, _: &str) -> Result<()> {
        Err(VAULT_ERROR.into())
    }
}
#[cfg(not(any(target_os = "linux", windows)))]
impl OfflineVault for OsVault {
    fn read_access(&mut self, _: &str) -> Result<Option<String>> {
        Err(VAULT_ERROR.into())
    }
    fn save_access(&mut self, _: &str, _: &str) -> Result<()> {
        Err(VAULT_ERROR.into())
    }
    fn clear_access(&mut self, _: &str) -> Result<()> {
        Err(VAULT_ERROR.into())
    }
}
#[cfg(test)]
#[path = "native_credentials_tests.rs"]
mod tests;
