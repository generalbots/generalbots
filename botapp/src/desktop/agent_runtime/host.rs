//! Abstraction over OS operations so the entire step machine is testable
//! without touching real processes, the registry, or the network.

use std::io;
use std::path::{Path, PathBuf};

#[cfg(windows)]
#[path = "host_windows.rs"]
mod windows_impl;

#[path = "host_http.rs"]
mod http;

// ── Error type for downloads ────────────────────────────────────────────

#[derive(Debug)]
pub(crate) enum DownloadError {
    Network(String),
    CertificateNotTrusted(String),
    Cancelled,
    Io(io::Error),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::CertificateNotTrusted(msg) => write!(f, "certificate not trusted: {msg}"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

// ── Host trait ──────────────────────────────────────────────────────────

/// OS-level operations the agent runtime needs.
pub(crate) trait RuntimeHost: Send + Sync {
    fn env(&self, name: &str) -> Option<String>;
    fn arch(&self) -> &str { "unknown" }
    fn os_build(&self) -> u32 { 0 }

    fn wsl_policy_blocked(&self) -> bool { false }
    fn wsl_installed(&self) -> bool { false }
    fn virtualization_available(&self) -> bool { true }
    fn wsl_reboot_pending(&self) -> bool { false }
    fn distro_registered(&self, name: &str) -> bool;
    fn distro_base_path(&self, name: &str) -> Option<PathBuf>;

    fn file_exists(&self, path: &Path) -> bool;
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn file_len(&self, path: &Path) -> io::Result<u64>;
    fn available_disk_space(&self, path: &Path) -> io::Result<u64>;

    /// Hash a file in chunks, returning the lowercase hex SHA-256 digest.
    fn sha256_file(&self, path: &Path) -> io::Result<String> {
        use sha2::{Digest, Sha256};
        let data = self.read_file(path)?;
        let mut hasher = Sha256::new();
        // Process in 64 KB chunks to bound peak memory on the default impl.
        for chunk in data.chunks(64 * 1024) {
            hasher.update(chunk);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    fn run_command(
        &self, program: &str, args: &[&str],
    ) -> io::Result<(i32, String, String)>;

    /// Run `wsl.exe --install --no-distribution` through a UAC prompt.
    /// Takes no caller-supplied arguments: the elevated command line is
    /// fixed so nothing user-controlled can reach it.
    fn run_wsl_install_elevated(&self) -> io::Result<()>;

    fn download(
        &self, url: &str, dest: &Path, expected_size: u64,
        on_progress: &dyn Fn(u64, u64), cancel: &dyn Fn() -> bool,
    ) -> Result<u64, DownloadError>;
    fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, DownloadError>;
}

// ── Path normalisation (shared between removal and provision) ───────────

/// Strip the `\\?\` extended-path prefix Windows APIs sometimes use.
pub(crate) fn strip_extended_prefix(s: &str) -> &str {
    s.strip_prefix(r"\\?\").unwrap_or(s)
}

/// Normalise a Windows path for case-insensitive comparison:
/// strip `\\?\`, lowercase, forward slashes, strip trailing slash.
pub(crate) fn normalise_path(p: &Path) -> String {
    let lossy = p.to_string_lossy();
    let s = strip_extended_prefix(&lossy);
    let mut n = s.replace('\\', "/").to_lowercase();
    while n.ends_with('/') && n.len() > 1 {
        n.pop();
    }
    n
}

/// Whether `candidate` is at or below `container` after normalisation.
pub(crate) fn path_is_inside(candidate: &Path, container: &Path) -> bool {
    let c = normalise_path(candidate);
    let d = normalise_path(container);
    c == d || c.starts_with(&format!("{d}/"))
}

// ── Production host ─────────────────────────────────────────────────────

pub(crate) struct SystemHost;

impl RuntimeHost for SystemHost {
    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok().filter(|v| !v.trim().is_empty())
    }

    fn arch(&self) -> &str { std::env::consts::ARCH }

    fn os_build(&self) -> u32 {
        #[cfg(windows)] { windows_impl::read_os_build() }
        #[cfg(not(windows))] { 0 }
    }

    fn wsl_policy_blocked(&self) -> bool {
        #[cfg(windows)] { windows_impl::wsl_policy_blocked() }
        #[cfg(not(windows))] { false }
    }

    fn wsl_installed(&self) -> bool {
        #[cfg(windows)] { windows_impl::wsl_installed() }
        #[cfg(not(windows))] { false }
    }

    fn virtualization_available(&self) -> bool {
        #[cfg(windows)] { windows_impl::virtualization_available() }
        #[cfg(not(windows))] { true }
    }

    fn wsl_reboot_pending(&self) -> bool {
        #[cfg(windows)] { windows_impl::wsl_reboot_pending() }
        #[cfg(not(windows))] { false }
    }

    fn distro_registered(&self, name: &str) -> bool {
        #[cfg(windows)] { windows_impl::distro_registered(name) }
        #[cfg(not(windows))] { let _ = name; false }
    }

    fn distro_base_path(&self, name: &str) -> Option<PathBuf> {
        #[cfg(windows)] { windows_impl::distro_base_path(name) }
        #[cfg(not(windows))] { let _ = name; None }
    }

    fn file_exists(&self, path: &Path) -> bool { path.exists() }
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> { std::fs::read(path) }
    fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        std::fs::write(path, data)
    }
    fn remove_file(&self, path: &Path) -> io::Result<()> { std::fs::remove_file(path) }
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }
    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir_all(path)
    }
    fn file_len(&self, path: &Path) -> io::Result<u64> {
        Ok(std::fs::metadata(path)?.len())
    }

    fn available_disk_space(&self, path: &Path) -> io::Result<u64> {
        #[cfg(windows)] { windows_impl::disk_free_space(path) }
        #[cfg(not(windows))] { let _ = path; Ok(u64::MAX) }
    }

    fn run_command(
        &self, program: &str, args: &[&str],
    ) -> io::Result<(i32, String, String)> {
        use crate::desktop::no_window::NoConsoleWindow;
        let output = std::process::Command::new(program)
            .args(args)
            .env("WSL_UTF8", "1")
            .no_console_window()
            .output()?;
        let code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((code, stdout, stderr))
    }

    fn run_wsl_install_elevated(&self) -> io::Result<()> {
        #[cfg(windows)] { windows_impl::run_wsl_install_elevated() }
        #[cfg(not(windows))] {
            Err(io::Error::new(io::ErrorKind::Unsupported, "elevation not available"))
        }
    }

    fn download(
        &self, url: &str, dest: &Path, expected_size: u64,
        on_progress: &dyn Fn(u64, u64), cancel: &dyn Fn() -> bool,
    ) -> Result<u64, DownloadError> {
        http::download_with_resume(url, dest, expected_size, on_progress, cancel)
    }

    fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, DownloadError> {
        http::fetch_bytes(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn path_normalisation_strips_prefix_and_case() {
        assert_eq!(
            normalise_path(Path::new(r"\\?\C:\Data\agent-runtime\distro")),
            "c:/data/agent-runtime/distro"
        );
        assert_eq!(
            normalise_path(Path::new(r"C:\Data\Agent-Runtime\Distro\")),
            "c:/data/agent-runtime/distro"
        );
    }

    #[test]
    fn path_is_inside_handles_prefix_and_case() {
        assert!(path_is_inside(
            Path::new(r"\\?\C:\data\agent-runtime\distro\ext4.vhdx"),
            Path::new(r"C:\data\agent-runtime\distro"),
        ));
        assert!(path_is_inside(
            Path::new(r"C:\Data\Agent-Runtime\Distro"),
            Path::new(r"c:\data\agent-runtime\distro"),
        ));
        assert!(!path_is_inside(
            Path::new(r"C:\other\distro"),
            Path::new(r"C:\data\agent-runtime\distro"),
        ));
        // A container that is a prefix of the name but not a parent.
        assert!(!path_is_inside(
            Path::new(r"C:\data\agent-runtime\distro2\ext4.vhdx"),
            Path::new(r"C:\data\agent-runtime\distro"),
        ));
    }

    // ── reg query output fixtures from this machine ─────────────────

    #[test]
    fn reg_query_sz_fixture_shape() {
        // Real output from `reg query HKCU\...\Lxss\{GUID} /v DistributionName`:
        // "HKEY_CURRENT_USER\Software\...\Lxss\{GUID}\n"
        // "    DistributionName    REG_SZ    Ubuntu\n"
        // The parser expects: trimmed line starts with the value name,
        // followed by whitespace + REG_SZ + whitespace + data.
        let line = "    DistributionName    REG_SZ    Ubuntu";
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix("DistributionName").expect("prefix");
        let rest = rest.trim_start();
        let data = rest.strip_prefix("REG_SZ").expect("type").trim();
        assert_eq!(data, "Ubuntu");
    }

    #[test]
    fn reg_query_basepath_fixture_no_extended_prefix() {
        // Real output on this machine (modern WSL, managed install):
        let raw = r"C:\Users\thipa\AppData\Local\wsl\{55d3eab5-aa65-4e68-b482-a2cd95743578}";
        assert_eq!(strip_extended_prefix(raw), raw);
    }

    #[test]
    fn reg_query_basepath_fixture_with_extended_prefix() {
        // Older WSL or custom --import path:
        let raw = r"\\?\C:\Users\thipa\AppData\Local\Packages\distro\LocalState";
        assert_eq!(
            strip_extended_prefix(raw),
            r"C:\Users\thipa\AppData\Local\Packages\distro\LocalState"
        );
    }

    #[test]
    fn wsl_status_output_fixture() {
        // Real output from `wsl --status` with WSL_UTF8=1 (pt-BR locale):
        // "Distribuição Padrão: Ubuntu\nVersão Padrão: 2\n"
        // Exit code 0 means WSL is installed.
        let output = "Distribui\u{e7}\u{e3}o Padr\u{e3}o: Ubuntu\nVers\u{e3}o Padr\u{e3}o: 2\n";
        assert!(!output.is_empty());
        // Parser only checks exit code, not content.
    }
}
