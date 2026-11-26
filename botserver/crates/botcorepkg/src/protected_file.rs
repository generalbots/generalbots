//! Secret files kept beside the local stack (the Vault unseal keys and the
//! Vault init data). On Windows they are sealed with DPAPI for the current
//! user, so a copy of the folder alone does not reveal them; a plain-text file
//! left by an older install is sealed the first time it is read. Elsewhere the
//! files stay plain text with the permissions their writers already set.

use std::io;
use std::path::Path;

/// Marks a sealed file. Plain-text secret files never start with it.
pub const SEALED_MAGIC: &[u8] = b"GBSEALED1\n";

pub fn is_sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(SEALED_MAGIC)
}

fn invalid(e: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| "secret file".to_string(), |n| n.to_string_lossy().into_owned())
}

/// Reads a secret file written by [`write_secret`] or by an older install.
pub fn read_secret(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    if is_sealed(&bytes) {
        let plain = os::unseal(&bytes[SEALED_MAGIC.len()..])?;
        return String::from_utf8(plain).map_err(invalid);
    }
    let text = String::from_utf8(bytes).map_err(invalid)?;
    if os::SEALS {
        match write_secret(path, &text) {
            Ok(()) => log::info!("{} is now protected for the current user", file_label(path)),
            Err(e) => log::warn!("could not protect {}: {e}", file_label(path)),
        }
    }
    Ok(text)
}

/// Writes a secret file, sealed where the platform supports it. The new
/// content replaces the old one in a single rename.
pub fn write_secret(path: &Path, content: &str) -> io::Result<()> {
    if !os::SEALS {
        return std::fs::write(path, content);
    }
    let mut sealed = SEALED_MAGIC.to_vec();
    sealed.extend(os::seal(content.as_bytes())?);
    let staging = path.with_file_name(format!("{}.sealing", file_label(path)));
    std::fs::write(&staging, &sealed)?;
    std::fs::rename(&staging, path).inspect_err(|_| {
        if let Err(e) = std::fs::remove_file(&staging) {
            log::debug!("cannot remove staging file {}: {e}", staging.display());
        }
    })
}

#[cfg(windows)]
mod os {
    use std::io;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    pub(super) const SEALS: bool = true;
    const ENTROPY: &[u8] = b"generalbots-local-stack-secret";

    fn blob(data: &[u8]) -> io::Result<CRYPT_INTEGER_BLOB> {
        let len = u32::try_from(data.len()).map_err(super::invalid)?;
        Ok(CRYPT_INTEGER_BLOB { cbData: len, pbData: data.as_ptr().cast_mut() })
    }

    fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        if out.pbData.is_null() {
            return Vec::new();
        }
        // SAFETY: DPAPI returned a LocalAlloc'd buffer of `cbData` bytes that
        // this function owns; it is copied once and then freed.
        unsafe {
            let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
            LocalFree(out.pbData.cast());
            bytes
        }
    }

    pub(super) fn seal(data: &[u8]) -> io::Result<Vec<u8>> {
        let input = blob(data)?;
        let entropy = blob(ENTROPY)?;
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        // SAFETY: every pointer refers to a live local or is null where the
        // API allows it; DPAPI only reads the input blobs.
        let ok = unsafe {
            CryptProtectData(
                &input,
                std::ptr::null(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(take(out))
    }

    pub(super) fn unseal(data: &[u8]) -> io::Result<Vec<u8>> {
        let input = blob(data)?;
        let entropy = blob(ENTROPY)?;
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        // SAFETY: as in `seal`.
        let ok = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(take(out))
    }
}

#[cfg(not(windows))]
mod os {
    use std::io;

    pub(super) const SEALS: bool = false;

    pub(super) fn seal(data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    pub(super) fn unseal(data: &[u8]) -> io::Result<Vec<u8>> {
        Err(super::invalid(format!(
            "a sealed secret file ({} bytes) can only be opened on Windows",
            data.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_not_mistaken_for_a_sealed_file() {
        assert!(!is_sealed(b"VAULT_UNSEAL_KEY_1=abc\n"));
        assert!(!is_sealed(b"{\"root_token\":\"x\"}"));
        assert!(is_sealed(b"GBSEALED1\n\x01\x02"));
    }

    #[test]
    fn written_secret_reads_back_and_is_not_plain_on_windows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault-unseal-keys");
        write_secret(&path, "VAULT_UNSEAL_KEY_1=k1\n").unwrap();
        assert_eq!(read_secret(&path).unwrap(), "VAULT_UNSEAL_KEY_1=k1\n");
        let raw = std::fs::read(&path).unwrap();
        assert_eq!(is_sealed(&raw), cfg!(windows));
        assert!(!dir.path().join("vault-unseal-keys.sealing").exists());
    }

    #[test]
    fn legacy_plain_file_is_read_and_sealed_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("init.json");
        std::fs::write(&path, "{\"root_token\":\"t\"}").unwrap();
        assert_eq!(read_secret(&path).unwrap(), "{\"root_token\":\"t\"}");
        assert_eq!(is_sealed(&std::fs::read(&path).unwrap()), cfg!(windows));
        assert_eq!(read_secret(&path).unwrap(), "{\"root_token\":\"t\"}");
    }
}
