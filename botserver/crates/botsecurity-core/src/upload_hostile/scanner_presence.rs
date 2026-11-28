//! Whether the ClamAV scanner binary exists on this machine, and the
//! process-wide switch an edition sets at boot to accept uploads when no
//! scanner is installed.
//!
//! The switch only changes how [`MalwareScanDecision::NotConfigured`] is
//! judged. A registered scanner is never bypassed: an infected verdict and a
//! scan error stay refusals whatever the switch says.
//!
//! [`MalwareScanDecision::NotConfigured`]: super::MalwareScanDecision::NotConfigured

use std::ffi::OsStr;
use std::sync::OnceLock;

/// Binary the ClamAV adapter launches.
pub const CLAMSCAN_BINARY: &str = "clamscan";

static MISSING_SCANNER_ALLOWED: OnceLock<bool> = OnceLock::new();

/// Lets uploads through when no scanner is registered. One-shot; returns
/// `false` when the switch was already set.
pub fn allow_missing_scanner() -> bool {
    MISSING_SCANNER_ALLOWED.set(true).is_ok()
}

/// True once [`allow_missing_scanner`] has run in this process.
pub fn missing_scanner_allowed() -> bool {
    MISSING_SCANNER_ALLOWED.get().copied().unwrap_or(false)
}

/// True when `clamscan` can be found on the process `PATH`.
pub fn clamscan_on_path() -> bool {
    std::env::var_os("PATH").is_some_and(|path| binary_in_path_list(&path, CLAMSCAN_BINARY))
}

/// True when `name` (or `name.exe` on Windows) is a file in one of the
/// directories of `path_list`, a `PATH`-style list.
pub fn binary_in_path_list(path_list: &OsStr, name: &str) -> bool {
    let candidates = candidate_file_names(name);
    std::env::split_paths(path_list)
        .filter(|dir| !dir.as_os_str().is_empty())
        .any(|dir| candidates.iter().any(|file| dir.join(file).is_file()))
}

fn candidate_file_names(name: &str) -> Vec<String> {
    if cfg!(windows) {
        vec![format!("{name}.exe"), name.to_string()]
    } else {
        vec![name.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn scratch_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gb-scanner-presence-{tag}-{}",
            uuid::Uuid::new_v4()
        ));
        assert!(std::fs::create_dir_all(&dir).is_ok());
        dir
    }

    #[test]
    fn binary_is_found_in_a_listed_directory() {
        let empty = scratch_dir("empty");
        let with_binary = scratch_dir("bin");
        let file_name = candidate_file_names(CLAMSCAN_BINARY)[0].clone();
        assert!(std::fs::write(with_binary.join(&file_name), b"").is_ok());

        let list = std::env::join_paths([empty.clone(), with_binary.clone()])
            .unwrap_or_else(|_| OsString::new());
        assert!(binary_in_path_list(&list, CLAMSCAN_BINARY));

        let _ = std::fs::remove_dir_all(&empty);
        let _ = std::fs::remove_dir_all(&with_binary);
    }

    #[test]
    fn binary_is_absent_from_directories_without_it() {
        let empty = scratch_dir("none");
        let list = std::env::join_paths([empty.clone()]).unwrap_or_else(|_| OsString::new());
        assert!(!binary_in_path_list(&list, CLAMSCAN_BINARY));
        assert!(!binary_in_path_list(OsStr::new(""), CLAMSCAN_BINARY));
        let _ = std::fs::remove_dir_all(&empty);
    }

    #[test]
    fn a_directory_named_like_the_binary_does_not_count() {
        let parent = scratch_dir("dir");
        for name in candidate_file_names(CLAMSCAN_BINARY) {
            assert!(std::fs::create_dir_all(parent.join(name)).is_ok());
        }
        let list = std::env::join_paths([parent.clone()]).unwrap_or_else(|_| OsString::new());
        assert!(!binary_in_path_list(&list, CLAMSCAN_BINARY));
        let _ = std::fs::remove_dir_all(&parent);
    }
}
