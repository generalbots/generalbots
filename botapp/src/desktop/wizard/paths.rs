//! Filesystem locations and first-run detection.
//!
//! The wizard never reads botserver state: detection is a marker file and every
//! path is derived from the sidecar stack directory.

use std::path::{Path, PathBuf};

use super::super::sidecar::Sidecar;

pub(crate) fn stack_path() -> PathBuf {
    Sidecar::stack_path()
}

/// Chooses the installers directory.
///
/// An explicit path wins (development runs point at the checkout's bundled
/// installers); a packaged MSI/NSIS install uses its resource directory and the
/// self-extracting executable uses the payload extracted into the stack. Only
/// an existing resource directory is passed in: preferring a missing one hid
/// the self-extractor's `stack/installers/` (#1793).
fn pick_installers(
    explicit: Option<&str>,
    resource_installers: Option<PathBuf>,
    stack: &Path,
) -> PathBuf {
    if let Some(value) = explicit {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    if let Some(dir) = resource_installers {
        return dir;
    }
    stack.join("installers")
}

pub(crate) fn installers_path(resource_dir: Option<&Path>) -> PathBuf {
    let explicit = std::env::var("BOTSERVER_INSTALLERS_PATH").ok();
    let resource_installers = resource_dir
        .map(|dir| dir.join("installers"))
        .filter(|dir| dir.exists());
    pick_installers(explicit.as_deref(), resource_installers, &stack_path())
}

pub(crate) fn marker_path() -> PathBuf {
    stack_path().join(".initialized")
}

pub(crate) fn progress_path() -> PathBuf {
    stack_path().join(".wizard-progress.json")
}

pub(crate) fn draft_path() -> PathBuf {
    stack_path().join(".wizard-draft.json")
}

pub(crate) fn log_path() -> PathBuf {
    stack_path().join("logs")
}

pub(crate) fn workspace_path() -> PathBuf {
    if let Ok(value) = std::env::var("GB_WIZARD_WORKSPACE") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    stack_path().join("workspace")
}

/// Desktop mode is explicit: the desktop-local feature or the desktop product
/// file. No heuristics based on a slow API.
pub fn is_desktop_mode() -> bool {
    cfg!(feature = "desktop-local")
        || std::env::var("GB_DESKTOP_MODE")
            .map(|v| v == "1")
            .unwrap_or(false)
        || stack_path().join(".product.desktop-local").exists()
}

/// First run is the absence of the persisted marker. An existing folder without
/// the marker is still a first run (it may be a corrupt or partial stack).
pub fn is_first_run() -> bool {
    first_run(is_desktop_mode(), marker_path().exists())
}

/// The first-run decision, with the desktop mode and the marker injected.
fn first_run(desktop_mode: bool, marker_exists: bool) -> bool {
    desktop_mode && !marker_exists
}

#[cfg(test)]
mod tests {
    use super::{first_run, pick_installers};
    use std::path::{Path, PathBuf};

    #[test]
    fn first_run_needs_desktop_mode_and_a_missing_marker() {
        assert!(first_run(true, false));
        assert!(!first_run(true, true));
        // A server deployment never runs the desktop wizard.
        assert!(!first_run(false, false));
        assert!(!first_run(false, true));
    }

    #[test]
    fn an_explicit_installers_path_wins() {
        let picked = pick_installers(
            Some("/opt/gb/installers"),
            Some(PathBuf::from("/res/installers")),
            Path::new("/home/u/stack"),
        );
        assert_eq!(picked, PathBuf::from("/opt/gb/installers"));
    }

    #[test]
    fn a_blank_explicit_path_is_ignored() {
        let picked = pick_installers(
            Some("   "),
            Some(PathBuf::from("/res/installers")),
            Path::new("/home/u/stack"),
        );
        assert_eq!(picked, PathBuf::from("/res/installers"));
    }

    #[test]
    fn the_resource_directory_is_used_when_it_exists() {
        let picked = pick_installers(None, Some(PathBuf::from("/res/installers")), Path::new("/s"));
        assert_eq!(picked, PathBuf::from("/res/installers"));
    }

    #[test]
    fn the_stack_payload_is_the_last_resort() {
        let picked = pick_installers(None, None, Path::new("/home/u/stack"));
        assert_eq!(picked, PathBuf::from("/home/u/stack/installers"));
    }
}
