use std::path::PathBuf;

pub mod linux;
pub mod macos;
pub mod windows;

/// Target operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

impl Platform {
    /// Detect the current platform at compile time.
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }

    /// Filename extension for executables on this platform.
    #[must_use]
    pub fn executable_extension(&self) -> &'static str {
        if *self == Platform::Windows {
            ".exe"
        } else {
            ""
        }
    }

    /// Path separator character for this platform.
    #[must_use]
    pub fn path_separator(&self) -> &'static str {
        if *self == Platform::Windows {
            "\\"
        } else {
            "/"
        }
    }

    /// Line ending sequence for text files on this platform.
    #[must_use]
    pub fn line_ending(&self) -> &'static str {
        if *self == Platform::Windows {
            "\r\n"
        } else {
            "\n"
        }
    }
}

/// Abstracted platform-system operations.
pub trait OsAbstraction {
    fn platform(&self) -> Platform;
    fn is_admin(&self) -> bool;
    fn set_permissions(&self, path: &std::path::Path, mode: u32) -> Result<(), String>;
    fn set_executable(&self, path: &std::path::Path) -> Result<(), String>;
    fn set_readonly_owner(&self, path: &std::path::Path) -> Result<(), String>;
    fn default_data_dir(&self) -> PathBuf;
    fn default_config_dir(&self) -> PathBuf;
    fn shell_command(&self) -> (&'static str, &'static str);
    fn process_grep_command(&self) -> &'static str;
}

/// Returns the platform-specific OS abstraction implementation.
#[must_use]
pub fn get_abstraction() -> Box<dyn OsAbstraction> {
    match Platform::current() {
        Platform::Linux => Box::new(linux::LinuxAbstraction),
        Platform::Windows => Box::new(windows::WindowsAbstraction),
        Platform::MacOS => Box::new(macos::MacOsAbstraction),
    }
}

/// Detect the current operating system platform.
#[must_use]
pub fn detect_platform() -> Platform {
    Platform::current()
}

/// Convert from the package_manager `OsType` to `Platform`.
#[must_use]
pub fn from_os_type(os_type: crate::package_manager::OsType) -> Platform {
    match os_type {
        crate::package_manager::OsType::Linux => Platform::Linux,
        crate::package_manager::OsType::MacOS => Platform::MacOS,
        crate::package_manager::OsType::Windows => Platform::Windows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current_returns_something() {
        let p = Platform::current();
        assert!(matches!(p, Platform::Linux | Platform::Windows | Platform::MacOS));
    }

    #[test]
    fn test_executable_extension() {
        assert_eq!(Platform::Windows.executable_extension(), ".exe");
        assert_eq!(Platform::Linux.executable_extension(), "");
        assert_eq!(Platform::MacOS.executable_extension(), "");
    }

    #[test]
    fn test_get_abstraction() {
        let abs = get_abstraction();
        assert_eq!(abs.platform(), Platform::current());
    }

    #[test]
    fn test_detect_platform_matches_current() {
        assert_eq!(detect_platform(), Platform::current());
    }

    #[test]
    fn test_from_os_type() {
        assert_eq!(from_os_type(crate::package_manager::OsType::Linux), Platform::Linux);
        assert_eq!(from_os_type(crate::package_manager::OsType::Windows), Platform::Windows);
        assert_eq!(from_os_type(crate::package_manager::OsType::MacOS), Platform::MacOS);
    }
}
