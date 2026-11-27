//! Detection of the agent runtime state without admin rights.
//!
//! Checks, in order: architecture, Windows build, WSL policy, WSL installed,
//! virtualization, reboot pending, distro registered, ready.json valid, and
//! whether a manual `AGENT_INCUS_BIN` is set (which makes everything a
//! no-op report of `managed=false`).

use std::path::Path;

use super::host::RuntimeHost;
use super::state::{DetectionState, ReadyMarker, RuntimeStatus, DISTRO_NAME, MIN_BUILD, PINNED_RUNTIME_VERSION};

/// Full detection result.
pub(crate) struct Detection {
    pub state: DetectionState,
    pub detail: Option<String>,
    pub managed: bool,
    pub runtime_version: Option<String>,
    pub needs_admin: bool,
    pub needs_reboot: bool,
}

/// Run detection against the given host.
pub(crate) fn detect(host: &dyn RuntimeHost, stack: &std::path::Path) -> Detection {
    // A person's own AGENT_INCUS_BIN means a manual setup: report it, offer
    // nothing.  The wave-1 agent_env module only sets it when the bundled
    // wrapper exists; a user override always wins.
    if host.env("AGENT_INCUS_BIN").is_some() {
        // Check if it was set by the product (GBO_AGENT_RUNTIME_MARKER also set)
        // or by the person (marker not set = manual).
        if host.env("GBO_AGENT_RUNTIME_MARKER").is_none() {
            return Detection {
                state: DetectionState::Ready,
                detail: Some("manual AGENT_INCUS_BIN".into()),
                managed: false,
                runtime_version: None,
                needs_admin: false,
                needs_reboot: false,
            };
        }
    }

    if host.arch() != "x86_64" {
        return Detection {
            state: DetectionState::Unsupported,
            detail: Some(format!("architecture {} not supported", host.arch())),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        };
    }

    let build = host.os_build();
    if build < MIN_BUILD {
        return Detection {
            state: DetectionState::Unsupported,
            detail: Some(format!("Windows build {build} < {MIN_BUILD}")),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        };
    }

    if host.wsl_policy_blocked() {
        return Detection {
            state: DetectionState::PolicyBlocked,
            detail: Some("machine policy forbids WSL".into()),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        };
    }

    if !host.virtualization_available() {
        return Detection {
            state: DetectionState::NeedsVirtualization,
            detail: Some("hardware virtualization not available".into()),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        };
    }

    if !host.wsl_installed() {
        // Check if a reboot is pending from a prior WSL install.
        if host.wsl_reboot_pending() {
            return Detection {
                state: DetectionState::NeedsReboot,
                detail: Some("WSL install pending reboot".into()),
                managed: true,
                runtime_version: None,
                needs_admin: false,
                needs_reboot: true,
            };
        }
        return Detection {
            state: DetectionState::NeedsWsl,
            detail: Some("WSL2 is not installed".into()),
            managed: true,
            runtime_version: None,
            needs_admin: true,
            needs_reboot: false,
        };
    }

    if host.wsl_reboot_pending() {
        return Detection {
            state: DetectionState::NeedsReboot,
            detail: Some("reboot pending".into()),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: true,
        };
    }

    // Check for the persisted state.json (installation in progress).
    let state_path = stack
        .join("agent-runtime")
        .join("state.json");
    if host.file_exists(&state_path) {
        // Check if the provisioning state file indicates in-progress.
        if let Ok(data) = host.read_file(&state_path) {
            if let Ok(persisted) = super::state::PersistedState::load(&data) {
                if persisted.completed_step != Some(super::state::ProvisionStep::Done) {
                    return Detection {
                        state: DetectionState::Installing,
                        detail: Some("provisioning in progress".into()),
                        managed: true,
                        runtime_version: None,
                        needs_admin: false,
                        needs_reboot: false,
                    };
                }
            }
        }
    }

    if !host.distro_registered(DISTRO_NAME) {
        return Detection {
            state: DetectionState::NotInstalled,
            detail: Some(format!("distro {DISTRO_NAME} not registered")),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        };
    }

    // Check the ready.json marker.
    let marker_path = stack
        .join("agent-runtime")
        .join("ready.json");
    match read_marker(host, &marker_path) {
        Some(marker) if marker.is_valid() => Detection {
            state: DetectionState::Ready,
            detail: None,
            managed: true,
            runtime_version: Some(marker.runtime_version),
            needs_admin: false,
            needs_reboot: false,
        },
        Some(_) => Detection {
            state: DetectionState::Broken,
            detail: Some("ready.json has invalid content".into()),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        },
        None => Detection {
            state: DetectionState::NotInstalled,
            detail: Some("ready.json not found".into()),
            managed: true,
            runtime_version: None,
            needs_admin: false,
            needs_reboot: false,
        },
    }
}

/// Build the status response from a detection result.
pub(crate) fn status_from_detection(d: &Detection) -> RuntimeStatus {
    RuntimeStatus {
        state: d.state.as_str(),
        detail: d.detail.clone(),
        runtime_version: d.runtime_version.clone(),
        available_version: Some(PINNED_RUNTIME_VERSION.to_string()),
        download_bytes: 0,
        disk_bytes: 0,
        needs_admin: d.needs_admin,
        needs_reboot: d.needs_reboot,
        managed: d.managed,
        last_error: None,
        step: None,
        percent: None,
        done_bytes: 0,
        total_bytes: 0,
        paused: false,
    }
}

fn read_marker(host: &dyn RuntimeHost, path: &Path) -> Option<ReadyMarker> {
    if !host.file_exists(path) {
        return None;
    }
    let data = host.read_file(path).ok()?;
    serde_json::from_slice(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::agent_runtime::tests::FakeHost;

    #[test]
    fn manual_incus_bin_reports_unmanaged() {
        let host = FakeHost::new()
            .with_env("AGENT_INCUS_BIN", "/usr/bin/incus");
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::Ready);
        assert!(!d.managed);
    }

    #[test]
    fn product_incus_bin_still_checks_marker() {
        let host = FakeHost::new()
            .with_env("AGENT_INCUS_BIN", "/app/incus.exe")
            .with_env("GBO_AGENT_RUNTIME_MARKER", "/stack/agent-runtime/ready.json")
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_installed(true);
        let d = detect(&host, std::path::Path::new("/stack"));
        // No distro, no marker -> NotInstalled
        assert_eq!(d.state, DetectionState::NotInstalled);
        assert!(d.managed);
    }

    #[test]
    fn wrong_arch_is_unsupported() {
        let host = FakeHost::new().with_arch("aarch64");
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::Unsupported);
    }

    #[test]
    fn old_build_is_unsupported() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(18362);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::Unsupported);
    }

    #[test]
    fn policy_blocked() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_policy_blocked(true);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::PolicyBlocked);
    }

    #[test]
    fn needs_virtualization() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_virtualization(false);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::NeedsVirtualization);
    }

    #[test]
    fn needs_wsl() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::NeedsWsl);
        assert!(d.needs_admin);
    }

    #[test]
    fn needs_reboot_when_wsl_pending() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_reboot_pending(true);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::NeedsReboot);
        assert!(d.needs_reboot);
    }

    #[test]
    fn not_installed_when_distro_missing() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_installed(true);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::NotInstalled);
    }

    #[test]
    fn ready_when_distro_and_marker_ok() {
        let marker = serde_json::to_vec(&ReadyMarker {
            schema_version: 1,
            runtime_version: "1.0.0".into(),
            distro: "GeneralBotsLite".into(),
            installed_at: "2026-01-01T00:00:00Z".into(),
            release_tag: "gbgov-agent-runtime-v1.0.0".into(),
            image_fingerprint: "abc".into(),
        })
        .expect("json");

        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_installed(true)
            .with_distro("GeneralBotsLite", "/stack/agent-runtime/distro")
            .with_file("/stack/agent-runtime/ready.json", marker);
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::Ready);
        assert!(d.managed);
        assert_eq!(d.runtime_version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn broken_when_marker_invalid() {
        let host = FakeHost::new()
            .with_arch("x86_64")
            .with_os_build(22000)
            .with_wsl_installed(true)
            .with_distro("GeneralBotsLite", "/stack/agent-runtime/distro")
            .with_file("/stack/agent-runtime/ready.json", b"{}".to_vec());
        let d = detect(&host, std::path::Path::new("/stack"));
        assert_eq!(d.state, DetectionState::Broken);
    }
}
