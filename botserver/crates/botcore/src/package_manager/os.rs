use crate::os_abstraction::Platform;
use crate::package_manager::OsType;

#[must_use]
pub fn detect_os() -> OsType {
    match Platform::current() {
        Platform::Linux => OsType::Linux,
        Platform::MacOS => OsType::MacOS,
        Platform::Windows => OsType::Windows,
    }
}

#[must_use]
pub fn platform_to_os_type(platform: Platform) -> OsType {
    match platform {
        Platform::Linux => OsType::Linux,
        Platform::MacOS => OsType::MacOS,
        Platform::Windows => OsType::Windows,
    }
}
