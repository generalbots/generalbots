//! Persisted step-machine state and shared types.

use serde::{Deserialize, Serialize};

pub(crate) const PINNED_RUNTIME_VERSION: &str = "1.0.1";

pub(crate) const RELEASE_BASE_URL: &str =
    "https://github.com/generalbots/generalbots/releases/download";

pub(crate) const DISTRO_NAME: &str = "GeneralBotsLite";

pub(crate) const MIN_BUILD: u32 = 19041;

pub(crate) fn manifest_url() -> String {
    format!(
        "{}/gbgov-agent-runtime-v{}/agent-runtime-manifest.json",
        RELEASE_BASE_URL, PINNED_RUNTIME_VERSION
    )
}

pub(crate) fn asset_url(file_name: &str) -> String {
    format!(
        "{}/gbgov-agent-runtime-v{}/{}",
        RELEASE_BASE_URL, PINNED_RUNTIME_VERSION, file_name
    )
}

// ── Detection states ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DetectionState {
    Unsupported,
    PolicyBlocked,
    NeedsVirtualization,
    NeedsWsl,
    NeedsReboot,
    NotInstalled,
    Installing,
    Broken,
    Ready,
}

impl DetectionState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::PolicyBlocked => "policy_blocked",
            Self::NeedsVirtualization => "needs_virtualization",
            Self::NeedsWsl => "needs_wsl",
            Self::NeedsReboot => "needs_reboot",
            Self::NotInstalled => "not_installed",
            Self::Installing => "installing",
            Self::Broken => "broken",
            Self::Ready => "ready",
        }
    }
}

// ── Status returned to the UI ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RuntimeStatus {
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_version: Option<String>,
    pub download_bytes: u64,
    pub disk_bytes: u64,
    pub needs_admin: bool,
    pub needs_reboot: bool,
    pub managed: bool,
    /// The last failure of the current or last install attempt.
    /// Cleared when an install starts or succeeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<LastError>,
    /// Current provisioning step (while state is "installing").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    /// Current progress percent (while state is "installing").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
    /// Bytes downloaded so far (while state is "installing").
    pub done_bytes: u64,
    /// Total bytes to download (while state is "installing").
    pub total_bytes: u64,
    /// True after a cancel when a resumable state is on disk.
    pub paused: bool,
}

/// Persisted error from the last install attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LastError {
    pub step: String,
    pub code: String,
    pub detail: String,
}

// ── Progress event ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProgressEvent {
    pub step: &'static str,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub percent: u8,
    pub state: &'static str,
}

// ── Error event ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ErrorEvent {
    pub step: &'static str,
    pub code: &'static str,
    pub detail: String,
}

// ── Provisioning step machine state (persisted) ─────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum ProvisionStep {
    WslInstall = 1,
    RootfsDownload = 2,
    DistroImport = 3,
    FirstBoot = 4,
    ImageImport = 5,
    Finish = 6,
    Done = 7,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedState {
    pub completed_step: Option<ProvisionStep>,
    pub rootfs_file: Option<String>,
    pub rootfs_sha256: Option<String>,
    pub image_file: Option<String>,
    pub image_sha256: Option<String>,
    pub release_tag: String,
    pub runtime_version: String,
    /// Persisted last error so it survives an app restart.
    #[serde(default)]
    pub last_error: Option<LastError>,
}

impl PersistedState {
    pub(crate) fn new() -> Self {
        Self {
            completed_step: None,
            rootfs_file: None,
            rootfs_sha256: None,
            image_file: None,
            image_sha256: None,
            release_tag: format!("gbgov-agent-runtime-v{PINNED_RUNTIME_VERSION}"),
            runtime_version: PINNED_RUNTIME_VERSION.to_string(),
            last_error: None,
        }
    }

    pub(crate) fn load(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("invalid state file: {e}"))
    }

    pub(crate) fn to_json(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(self)
            .map_err(|e| format!("cannot serialise state: {e}"))
    }
}

// ── ready.json (the marker file) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ReadyMarker {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub runtime_version: String,
    #[serde(default)]
    pub distro: String,
    #[serde(default)]
    pub installed_at: String,
    #[serde(default)]
    pub release_tag: String,
    #[serde(default)]
    pub image_fingerprint: String,
}

impl ReadyMarker {
    pub(crate) fn is_valid(&self) -> bool {
        self.schema_version == 1
            && !self.runtime_version.is_empty()
            && self.distro == DISTRO_NAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_url_uses_pinned_version() {
        let url = manifest_url();
        assert!(url.contains(PINNED_RUNTIME_VERSION));
        assert!(url.ends_with("agent-runtime-manifest.json"));
    }

    #[test]
    fn asset_url_builds_correctly() {
        let url = asset_url("rootfs.tar.gz");
        assert!(url.contains(PINNED_RUNTIME_VERSION));
        assert!(url.ends_with("rootfs.tar.gz"));
    }

    #[test]
    fn persisted_state_round_trips() {
        let state = PersistedState::new();
        let json = state.to_json().expect("serialise");
        let loaded = PersistedState::load(&json).expect("deserialise");
        assert_eq!(loaded.runtime_version, PINNED_RUNTIME_VERSION);
    }

    #[test]
    fn persisted_state_with_last_error_round_trips() {
        let mut state = PersistedState::new();
        state.last_error = Some(LastError {
            step: "rootfs_download".into(),
            code: "offline".into(),
            detail: "network error".into(),
        });
        let json = state.to_json().expect("serialise");
        let loaded = PersistedState::load(&json).expect("deserialise");
        assert_eq!(loaded.last_error.as_ref().map(|e| e.code.as_str()), Some("offline"));
    }

    #[test]
    fn ready_marker_validation() {
        let valid = ReadyMarker {
            schema_version: 1,
            runtime_version: "1.0.0".into(),
            distro: "GeneralBotsLite".into(),
            installed_at: "2026-01-01T00:00:00Z".into(),
            release_tag: "gbgov-agent-runtime-v1.0.0".into(),
            image_fingerprint: "abc".into(),
        };
        assert!(valid.is_valid());
        let bad_schema = ReadyMarker { schema_version: 2, ..valid.clone() };
        assert!(!bad_schema.is_valid());
        let bad_distro = ReadyMarker { distro: "Other".into(), ..valid };
        assert!(!bad_distro.is_valid());
    }

    #[test]
    fn detection_state_as_str() {
        assert_eq!(DetectionState::Unsupported.as_str(), "unsupported");
        assert_eq!(DetectionState::Ready.as_str(), "ready");
        assert_eq!(DetectionState::PolicyBlocked.as_str(), "policy_blocked");
    }
}
