//! Filesystem locations and first-run detection.
//!
//! The wizard never reads botserver state: detection is a marker file and every
//! path is derived from the sidecar stack directory.

use std::path::{Path, PathBuf};

use super::super::sidecar::Sidecar;

pub(crate) fn stack_path() -> PathBuf {
    Sidecar::stack_path()
}

pub(crate) fn installers_path(resource_dir: Option<&Path>) -> PathBuf {
    // An explicit path wins (development runs point at the checkout's bundled
    // installers); packaged builds fall back to the resource directory.
    if let Ok(value) = std::env::var("BOTSERVER_INSTALLERS_PATH") {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    resource_dir
        .map(|dir| dir.join("installers"))
        .unwrap_or_else(|| stack_path().join("installers"))
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
    is_desktop_mode() && !marker_path().exists()
}
