//! Tests for the provisioning step machine.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::desktop::agent_runtime::host::RuntimeHost;
use crate::desktop::agent_runtime::provision::{self, ProvisionEvents};
use crate::desktop::agent_runtime::state::{
    ErrorEvent, PersistedState, ProgressEvent, ProvisionStep,
};
use crate::desktop::agent_runtime::tests::FakeHost;

struct TestEvents {
    progress: Mutex<Vec<ProgressEvent>>,
    errors: Mutex<Vec<ErrorEvent>>,
}

impl TestEvents {
    fn new() -> Self {
        Self {
            progress: Mutex::new(Vec::new()),
            errors: Mutex::new(Vec::new()),
        }
    }
    fn error_codes(&self) -> Vec<String> {
        self.errors.lock().expect("lock").iter().map(|e| e.code.to_string()).collect()
    }
    fn progress_steps(&self) -> Vec<String> {
        self.progress.lock().expect("lock").iter().map(|p| p.step.to_string()).collect()
    }
}

impl ProvisionEvents for TestEvents {
    fn progress(&self, event: ProgressEvent) {
        self.progress.lock().expect("lock").push(event);
    }
    fn error(&self, event: ErrorEvent) {
        self.errors.lock().expect("lock").push(event);
    }
}

fn manifest_json() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "runtime_version": "1.0.1",
        "release_tag": "gbgov-agent-runtime-v1.0.1",
        "commit": "a".repeat(40),
        "built_at": "2026-01-01T00:00:00Z",
        "debian_snapshot": "20261120T000000Z",
        "assets": {
            "wsl_rootfs": {
                "kind": "wsl-rootfs", "file": "rootfs.tar.gz",
                "size": 10, "sha256": rootfs_sha256(),
                "arch": "x86_64", "distro_name": "GeneralBotsLite",
                "compression": "gzip", "debian_suite": "trixie",
                "debian_snapshot": "20261120T000000Z",
                "incus_version": "6.0.4", "systemd_version": "256"
            },
            "workspace_image": {
                "kind": "incus-image", "file": "image.tar.xz",
                "size": 20, "sha256": image_sha256(),
                "arch": "x86_64", "format": "unified", "compression": "xz",
                "incus_fingerprint": image_sha256(),
                "alias": "gb-workspace-office", "profile": "office",
                "provisioned_marker": "v7-office"
            }
        }
    })).expect("json")
}

fn rootfs_data() -> Vec<u8> { b"rootfs1234".to_vec() }
fn image_data() -> Vec<u8> { b"imagedata12345678901".to_vec() }

fn rootfs_sha256() -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(&rootfs_data()))
}
fn image_sha256() -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(&image_data()))
}

fn manifest_url() -> String { crate::desktop::agent_runtime::state::manifest_url() }
fn rootfs_url() -> String { crate::desktop::agent_runtime::state::asset_url("rootfs.tar.gz") }
fn image_url() -> String { crate::desktop::agent_runtime::state::asset_url("image.tar.xz") }

fn full_host() -> FakeHost {
    FakeHost::new()
        .with_wsl_installed(true)
        .with_download(&manifest_url(), manifest_json())
        .with_download(&rootfs_url(), rootfs_data())
        .with_download(&image_url(), image_data())
        .with_command_result("wsl", 0, "", "")
}

#[test]
fn full_install_when_wsl_already_present() {
    let fp = image_sha256();
    let host = full_host()
        .with_command_result("wsl", 0, &format!("ok\nFingerprint: {fp}"), "");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    let result = provision::run(&host, Path::new("/stack"), &events, &cancel);
    assert!(result.is_ok(), "provision failed: {:?}", result);
    assert!(host.file_exists(Path::new("/stack/agent-runtime/ready.json")));
    let steps = events.progress_steps();
    assert!(steps.contains(&"rootfs_download".to_string()));
    assert!(steps.contains(&"finish".to_string()));
}

#[test]
fn wsl_install_triggers_elevation_and_reboot() {
    let host = FakeHost::new()
        .with_download(&manifest_url(), manifest_json());
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    let result = provision::run(&host, Path::new("/stack"), &events, &cancel);
    assert_eq!(result, Err("needs_reboot"));
    let calls = host.elevated_calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("--install"));
}

#[test]
fn resumes_from_rootfs_download_after_reboot() {
    let mut ps = PersistedState::new();
    ps.completed_step = Some(ProvisionStep::WslInstall);
    let fp = image_sha256();
    let host = full_host()
        .with_file("/stack/agent-runtime/state.json", ps.to_json().expect("json"))
        .with_command_result("wsl", 0, &format!("ok\nFingerprint: {fp}"), "");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    let result = provision::run(&host, Path::new("/stack"), &events, &cancel);
    assert!(result.is_ok());
    assert!(host.elevated_calls().is_empty());
}

#[test]
fn cancel_during_provisioning() {
    let host = full_host();
    let events = TestEvents::new();
    let cancel = AtomicBool::new(true);
    assert_eq!(provision::run(&host, Path::new("/stack"), &events, &cancel), Err("cancelled"));
}

#[test]
fn checksum_mismatch_deletes_rootfs() {
    let host = FakeHost::new()
        .with_wsl_installed(true)
        .with_download(&manifest_url(), manifest_json())
        .with_download(&rootfs_url(), b"wrong_data".to_vec());
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert!(provision::run(&host, Path::new("/stack"), &events, &cancel).is_err());
    assert!(events.error_codes().contains(&"checksum".to_string()));
    assert!(!host.file_exists(Path::new("/stack/agent-runtime/rootfs.tar.gz")));
}

#[test]
fn disk_full_reported() {
    let host = FakeHost::new()
        .with_wsl_installed(true)
        .with_disk_space(1024)
        .with_download(&manifest_url(), manifest_json());
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert_eq!(provision::run(&host, Path::new("/stack"), &events, &cancel), Err("disk_full"));
}

#[test]
fn offline_when_manifest_unreachable() {
    let host = FakeHost::new().with_wsl_installed(true).with_download_fail();
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert_eq!(provision::run(&host, Path::new("/stack"), &events, &cancel), Err("offline"));
}

#[test]
fn uac_denied_reports_wsl_failed() {
    let host = FakeHost::new().with_elevated_fail();
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert_eq!(provision::run(&host, Path::new("/stack"), &events, &cancel), Err("wsl_failed"));
}

#[test]
fn import_reuses_already_imported_distro() {
    let mut ps = PersistedState::new();
    ps.completed_step = Some(ProvisionStep::RootfsDownload);
    ps.rootfs_file = Some("rootfs.tar.gz".into());
    ps.rootfs_sha256 = Some(rootfs_sha256());
    ps.image_file = Some("image.tar.xz".into());
    ps.image_sha256 = Some(image_sha256());
    let fp = image_sha256();
    let host = full_host()
        .with_file("/stack/agent-runtime/state.json", ps.to_json().expect("json"))
        .with_distro("GeneralBotsLite", "/stack/agent-runtime/distro")
        .with_file("/stack/agent-runtime/distro/ext4.vhdx", b"disk".to_vec())
        .with_command_result("wsl", 0, &format!("ok\nFingerprint: {fp}"), "");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert!(provision::run(&host, Path::new("/stack"), &events, &cancel).is_ok());
}

#[test]
fn import_refuses_foreign_distro_with_our_name() {
    let mut ps = PersistedState::new();
    ps.completed_step = Some(ProvisionStep::RootfsDownload);
    ps.rootfs_file = Some("rootfs.tar.gz".into());
    ps.image_file = Some("image.tar.xz".into());
    ps.image_sha256 = Some(image_sha256());
    let host = full_host()
        .with_file("/stack/agent-runtime/state.json", ps.to_json().expect("json"))
        .with_distro("GeneralBotsLite", "/other/location");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert_eq!(provision::run(&host, Path::new("/stack"), &events, &cancel), Err("import_failed"));
}

#[test]
fn import_unregisters_half_made_entry() {
    let mut ps = PersistedState::new();
    ps.completed_step = Some(ProvisionStep::RootfsDownload);
    ps.rootfs_file = Some("rootfs.tar.gz".into());
    ps.rootfs_sha256 = Some(rootfs_sha256());
    ps.image_file = Some("image.tar.xz".into());
    ps.image_sha256 = Some(image_sha256());
    let fp = image_sha256();
    let host = full_host()
        .with_file("/stack/agent-runtime/state.json", ps.to_json().expect("json"))
        .with_distro("GeneralBotsLite", "/stack/agent-runtime/distro")
        .with_command_result("wsl", 0, &format!("ok\nFingerprint: {fp}"), "");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert!(provision::run(&host, Path::new("/stack"), &events, &cancel).is_ok());
}

#[test]
fn last_error_persisted_on_failure() {
    let host = FakeHost::new().with_elevated_fail();
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert!(provision::run(&host, Path::new("/stack"), &events, &cancel).is_err());
    let data = host.read_file(Path::new("/stack/agent-runtime/state.json")).unwrap();
    let ps = PersistedState::load(&data).unwrap();
    assert_eq!(ps.last_error.as_ref().map(|e| e.code.as_str()), Some("wsl_failed"));
}

#[test]
fn last_error_cleared_on_success() {
    let fp = image_sha256();
    let host = full_host()
        .with_command_result("wsl", 0, &format!("ok\nFingerprint: {fp}"), "");
    let events = TestEvents::new();
    let cancel = AtomicBool::new(false);
    assert!(provision::run(&host, Path::new("/stack"), &events, &cancel).is_ok());
    let data = host.read_file(Path::new("/stack/agent-runtime/state.json")).unwrap();
    let ps = PersistedState::load(&data).unwrap();
    assert!(ps.last_error.is_none());
}

#[test]
fn only_our_distro_name_in_commands() {
    assert_eq!(crate::desktop::agent_runtime::state::DISTRO_NAME, "GeneralBotsLite");
}
