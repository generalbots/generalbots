//! Cross-platform service controller stubs and target-specific dependency
//! catalog. The actual platform-specific implementations live in
//! feature-gated submodules so the workspace builds on Linux, macOS, and
//! Windows without forcing every developer to install all toolchains.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux as platform;
#[cfg(target_os = "macos")]
pub use macos as platform;
#[cfg(target_os = "windows")]
pub use windows as platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Macos,
    Windows,
    Other,
}

impl Platform {
    pub const fn current() -> Self {
        #[cfg(target_os = "linux")]
        { Platform::Linux }
        #[cfg(target_os = "macos")]
        { Platform::Macos }
        #[cfg(target_os = "windows")]
        { Platform::Windows }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        { Platform::Other }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::Macos => "macos",
            Platform::Windows => "windows",
            Platform::Other => "other",
        }
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    pub const SERVICE_MANAGER: &str = "systemd";
    pub const PACKAGE_MANAGER: &str = "apt";
    pub const POSTGRES_BIN: &str = "pg_ctlcluster";
    pub const REDIS_BIN: &str = "redis-server";
    pub const MINIO_BIN: &str = "minio";
}

#[cfg(target_os = "macos")]
pub mod macos {
    pub const SERVICE_MANAGER: &str = "launchd";
    pub const PACKAGE_MANAGER: &str = "brew";
    pub const POSTGRES_BIN: &str = "pg_ctl";
    pub const REDIS_BIN: &str = "redis-server";
    pub const MINIO_BIN: &str = "minio";
}

#[cfg(target_os = "windows")]
pub mod windows {
    pub const SERVICE_MANAGER: &str = "sc.exe";
    pub const PACKAGE_MANAGER: &str = "choco";
    pub const POSTGRES_BIN: &str = "pg_ctl.exe";
    pub const REDIS_BIN: &str = "redis-server.exe";
    pub const MINIO_BIN: &str = "minio.exe";
}

/// Documents the cross-platform build matrix for the workspace. The
/// recommended Cargo target triples to ship a release artifact for each
/// supported OS / arch combo.
pub const SUPPORTED_TARGETS: &[(&str, &str, &str)] = &[
    ("linux",   "x86_64-unknown-linux-gnu",  "primary"),
    ("linux",   "aarch64-unknown-linux-gnu", "arm servers"),
    ("macos",   "x86_64-apple-darwin",       "Intel macs"),
    ("macos",   "aarch64-apple-darwin",      "Apple silicon"),
    ("windows", "x86_64-pc-windows-msvc",    "primary"),
    ("windows", "aarch64-pc-windows-msvc",   "ARM laptops"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_platform_matches_compile_target() {
        let p = Platform::current();
        #[cfg(target_os = "linux")]
        assert_eq!(p, Platform::Linux);
        #[cfg(target_os = "macos")]
        assert_eq!(p, Platform::Macos);
        #[cfg(target_os = "windows")]
        assert_eq!(p, Platform::Windows);
    }

    #[test]
    fn supported_targets_include_linux_windows() {
        let os_list: Vec<&str> = SUPPORTED_TARGETS.iter().map(|(os, _, _)| *os).collect();
        assert!(os_list.contains(&"linux"));
        assert!(os_list.contains(&"windows"));
    }

    #[test]
    fn binaries_constant_is_nonempty() {
        let p = Platform::current();
        assert!(!p.as_str().is_empty());
    }
}
