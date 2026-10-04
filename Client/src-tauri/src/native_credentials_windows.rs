// Minimal Windows SDK bindings. Credential storage belongs to the logged-in
// Windows user, survives app restarts and does not roam to other computers.
#![cfg_attr(not(windows), allow(dead_code))]
use super::*;
use std::{ffi::c_void, ptr};
#[repr(C)]
struct FileTime {
    low: u32,
    high: u32,
}
#[repr(C)]
struct Credential {
    flags: u32,
    kind: u32,
    target_name: *mut u16,
    comment: *mut u16,
    last_written: FileTime,
    blob_size: u32,
    blob: *mut u8,
    persist: u32,
    attribute_count: u32,
    attributes: *mut c_void,
    target_alias: *mut u16,
    user_name: *mut u16,
}
#[cfg_attr(windows, link(name = "advapi32"))]
extern "system" {
    fn CredReadW(
        target: *const u16,
        kind: u32,
        flags: u32,
        credential: *mut *mut Credential,
    ) -> i32;
    fn CredWriteW(credential: *const Credential, flags: u32) -> i32;
    fn CredFree(buffer: *mut c_void);
}
#[cfg_attr(windows, link(name = "kernel32"))]
extern "system" {
    fn GetLastError() -> u32;
}
fn target(device: &str) -> Result<Vec<u16>> {
    checked_device(device)?;
    Ok(format!("lk.armstrong.fitness/device-v1/{device}\0")
        .encode_utf16()
        .collect())
}
struct ReadCredential(*mut Credential);
impl Drop for ReadCredential {
    fn drop(&mut self) {
        // SAFETY: CredReadW returned this single owned allocation, freed exactly
        // once with the SDK's matching allocator after all borrows end.
        unsafe {
            CredFree(self.0.cast());
        }
    }
}
pub(super) struct WindowsVault;
impl CredentialVault for WindowsVault {
    fn read(&mut self, device: &str) -> Result<Option<String>> {
        let target = target(device)?;
        let mut raw = ptr::null_mut();
        // SAFETY: target is NUL-terminated UTF-16, raw is a writable out pointer.
        let ok = unsafe { CredReadW(target.as_ptr(), 1, 0, &mut raw) };
        if ok == 0 {
            // Only ERROR_NOT_FOUND means absent. A locked/unavailable OS store
            // must never cause a replacement credential to be generated.
            return if unsafe { GetLastError() } == 1168 {
                Ok(None)
            } else {
                Err(VAULT_ERROR.into())
            };
        }
        if raw.is_null() {
            return Err(VAULT_ERROR.into());
        }
        let allocation = ReadCredential(raw);
        // SAFETY: successful CredReadW provides a valid CREDENTIALW and blob
        // for the lifetime of allocation. Size/pointer are checked before slice.
        let credential = unsafe { &*allocation.0 };
        if credential.kind != 1 || credential.blob_size != 64 || credential.blob.is_null() {
            return Err(RECONCILE.into());
        }
        let bytes = unsafe { std::slice::from_raw_parts(credential.blob, 64) };
        let secret = std::str::from_utf8(bytes)
            .map_err(|_| RECONCILE)?
            .to_owned();
        checked_secret(&secret)?;
        Ok(Some(secret))
    }
    fn create(&mut self, device: &str, secret: &str) -> Result<()> {
        let mut target = target(device)?;
        checked_secret(secret)?;
        if self.read(device)?.is_some() {
            return Err(RECONCILE.into());
        }
        let mut blob = secret.as_bytes().to_vec();
        let credential = Credential {
            flags: 0,
            kind: 1,
            target_name: target.as_mut_ptr(),
            comment: ptr::null_mut(),
            last_written: FileTime { low: 0, high: 0 },
            blob_size: 64,
            blob: blob.as_mut_ptr(),
            persist: 2,
            attribute_count: 0,
            attributes: ptr::null_mut(),
            target_alias: ptr::null_mut(),
            user_name: ptr::null_mut(),
        };
        // SAFETY: all pointers remain live throughout the synchronous call.
        // SDK copies the blob; no pointers are retained after CredWriteW returns.
        let ok = unsafe { CredWriteW(&credential, 0) };
        wipe(&mut blob);
        if ok != 0 {
            Ok(())
        } else {
            Err(VAULT_ERROR.into())
        }
    }
}
#[cfg(all(test, target_pointer_width = "64"))]
mod tests {
    use super::*;
    #[test]
    fn windows_sdk_credential_layout_on_64_bit_hosts() {
        assert_eq!(std::mem::size_of::<Credential>(), 80);
        assert_eq!(std::mem::offset_of!(Credential, blob_size), 32);
        assert_eq!(std::mem::offset_of!(Credential, blob), 40);
        assert_eq!(std::mem::offset_of!(Credential, user_name), 72);
    }
}
