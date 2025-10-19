use crate::package_manager::OsType;

pub fn detect_os() -> OsType {
    if cfg!(target_os = "linux") {
        OsType::Linux
    } else if cfg!(target_os = "macos") {
        OsType::MacOS
    } else if cfg!(target_os = "windows") {
        OsType::Windows
    } else {
        OsType::Linux
    }
}
