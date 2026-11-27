//! What the agent-runtime check reads from the host: environment, files,
//! PATH and, on Windows, the registry. No process is started.

use std::path::Path;

/// The product's own WSL distro (the `incus.exe` wrapper's default too).
pub const DEFAULT_WSL_DISTRO: &str = "GeneralBotsLite";

/// Same rule as the wrapper: 1 to 64 characters from `[A-Za-z0-9._-]`, not
/// starting with `-`.
pub fn valid_distro_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub(crate) trait RuntimeHost {
    fn platform(&self) -> &str;
    fn env(&self, name: &str) -> Option<String>;
    /// A path that is a file, or a bare name found on PATH.
    fn program_exists(&self, program: &str) -> bool;
    fn file_exists(&self, path: &Path) -> bool;
    fn wsl_installed(&self) -> bool;
    fn distro_registered(&self, name: &str) -> bool;
}

pub(crate) struct SystemHost;

impl RuntimeHost for SystemHost {
    fn platform(&self) -> &str {
        std::env::consts::OS
    }

    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn program_exists(&self, program: &str) -> bool {
        binary_on_path(program)
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn wsl_installed(&self) -> bool {
        registry::wsl_service_registered()
    }

    fn distro_registered(&self, name: &str) -> bool {
        registry::distro_names()
            .iter()
            .any(|registered| registered.eq_ignore_ascii_case(name))
    }
}

/// Checks for an executable on `PATH` without spawning a process.
pub(crate) fn binary_on_path(binary: &str) -> bool {
    if binary.contains('/') || binary.contains('\\') {
        return Path::new(binary).is_file();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| executable_in(&dir, binary))
}

fn executable_in(dir: &Path, binary: &str) -> bool {
    if dir.join(binary).is_file() {
        return true;
    }
    #[cfg(windows)]
    {
        ["exe", "cmd", "bat", "com"]
            .iter()
            .any(|extension| dir.join(format!("{binary}.{extension}")).is_file())
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
mod registry {
    use windows_sys::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
        HKEY_LOCAL_MACHINE, KEY_READ, RRF_RT_REG_SZ,
    };

    const LXSS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Lxss";
    /// The Store WSL service, and the inbox optional-feature service.
    const WSL_SERVICE_KEYS: [&str; 2] = [
        r"SYSTEM\CurrentControlSet\Services\WSLService",
        r"SYSTEM\CurrentControlSet\Services\LxssManager",
    ];

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// An open registry key, closed on drop.
    struct Key(HKEY);

    impl Key {
        fn open(root: HKEY, path: &str) -> Option<Self> {
            let path = wide(path);
            let mut key: HKEY = std::ptr::null_mut();
            // SAFETY: `path` is NUL-terminated and outlives the call; `key`
            // is a valid out pointer.
            let status = unsafe { RegOpenKeyExW(root, path.as_ptr(), 0, KEY_READ, &mut key) };
            (status == ERROR_SUCCESS).then_some(Self(key))
        }

        fn subkey_names(&self) -> Vec<Vec<u16>> {
            let mut names = Vec::new();
            for index in 0..1024u32 {
                let mut buffer = [0u16; 256];
                let mut len = buffer.len() as u32;
                // SAFETY: `buffer` holds `len` u16s; the optional out
                // parameters are null, which the API allows.
                let status = unsafe {
                    RegEnumKeyExW(
                        self.0,
                        index,
                        buffer.as_mut_ptr(),
                        &mut len,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                };
                if status == ERROR_NO_MORE_ITEMS {
                    break;
                }
                if status != ERROR_SUCCESS {
                    continue;
                }
                let mut name = buffer[..len as usize].to_vec();
                name.push(0);
                names.push(name);
            }
            names
        }

        fn string_value(&self, subkey: &[u16], value: &str) -> Option<String> {
            let value = wide(value);
            let mut buffer = [0u16; 512];
            let mut size = std::mem::size_of_val(&buffer) as u32;
            // SAFETY: `subkey` and `value` are NUL-terminated; `buffer` has
            // `size` bytes; RRF_RT_REG_SZ makes the API NUL-terminate it.
            let status = unsafe {
                RegGetValueW(
                    self.0,
                    subkey.as_ptr(),
                    value.as_ptr(),
                    RRF_RT_REG_SZ,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr().cast(),
                    &mut size,
                )
            };
            if status != ERROR_SUCCESS {
                return None;
            }
            let chars = (size as usize / 2).min(buffer.len());
            let text = &buffer[..chars];
            let end = text.iter().position(|&c| c == 0).unwrap_or(text.len());
            Some(String::from_utf16_lossy(&text[..end]))
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful RegOpenKeyExW.
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }

    pub(super) fn wsl_service_registered() -> bool {
        WSL_SERVICE_KEYS
            .iter()
            .any(|path| Key::open(HKEY_LOCAL_MACHINE, path).is_some())
    }

    /// `DistributionName` of every WSL distro registered for this user.
    pub(super) fn distro_names() -> Vec<String> {
        let Some(lxss) = Key::open(HKEY_CURRENT_USER, LXSS_KEY) else {
            return Vec::new();
        };
        lxss.subkey_names()
            .iter()
            .filter_map(|subkey| lxss.string_value(subkey, "DistributionName"))
            .collect()
    }
}

#[cfg(not(windows))]
mod registry {
    pub(super) fn wsl_service_registered() -> bool {
        false
    }

    pub(super) fn distro_names() -> Vec<String> {
        Vec::new()
    }
}
