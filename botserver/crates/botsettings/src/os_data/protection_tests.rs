//! Inline tests for the data-protection surface (#2470). Included as a child
//! of `os_data_protection` via `#[path]`, so `super::*` is that module.

use super::*;
use serde_json::json;

struct B(bool);
impl SnapshotBackend for B {
    fn id(&self) -> &'static str {
        "btrfs"
    }
    fn medium(&self) -> &'static str {
        "local-subvolume"
    }
    fn prepare(&self, reason: &str) -> Option<SnapshotReceipt> {
        if !self.0 {
            return None;
        }
        Some(SnapshotReceipt {
            job_id: format!("j-{reason}"),
            location: DATA_ROOT.into(),
        })
    }
}

#[test]
fn status_is_honest_and_publishes_contracts() {
    let p = status_payload();
    assert_eq!(p["status"], "not_implemented");
    assert_eq!(p["encryption"]["implemented"], false);
    assert_eq!(p["backup"]["implemented"], false);
    assert_eq!(p["export"]["enabled"], false);
    assert_eq!(p["export"]["descriptor_available"], true);
    assert_eq!(p["reset"]["enabled"], false);
    assert_eq!(p["reset"]["confirmation_token"], RESET_CONFIRMATION_TOKEN);
    assert_eq!(p["backup"]["contract"]["path"], SNAPSHOT_CONTRACT_PATH);
    assert_eq!(p["export"]["contract"]["path"], EXPORT_CONTRACT_PATH);
    assert_eq!(p["reset"]["contract"]["path"], RESET_CONTRACT_PATH);
    let r = retention_payload();
    assert_eq!(r["daily_keep"], RETENTION_DAILY_KEEP);
    let t = p.to_string().to_ascii_lowercase();
    assert!(!t.contains("tpm 2.0 sealing of the root volume is performed"));
    assert!(!t.contains("enabled\":true"));
}

#[test]
fn manifest_is_deterministic_and_non_destructive() {
    let m = archive_manifest();
    assert_eq!(m["manifest_id"], archive_manifest()["manifest_id"]);
    assert_eq!(m["manifest_id"].as_str().map(str::len), Some(64));
    assert_eq!(m["status"], "descriptor_only");
    assert_eq!(m["archive_created"], false);
    assert_eq!(m["destructive"], false);
    assert_eq!(m["excludes"].as_array().map(Vec::len), Some(3));
}

#[test]
fn snapshot_fails_closed_and_reports_a_prepared_backend() {
    assert!(registered_snapshot_backends(false).is_empty());
    assert!(snapshot_descriptor(&registered_snapshot_backends(false), "r").is_none());
    assert!(snapshot_descriptor(&[&B(false)], "r").is_none());
    let d = snapshot_descriptor(&[&B(true)], "r");
    assert!(d.is_some());
    if let Some(d) = d {
        assert_eq!(d["status"], "staged");
        assert_eq!(d["adapter"], "btrfs");
        assert_eq!(d["destructive"], false);
        assert_eq!(d["retention"]["daily_keep"], RETENTION_DAILY_KEEP);
        assert!(d["manifest"].is_object());
    }
}

#[test]
fn inputs_are_validated() {
    assert!(valid_reason("nightly backup"));
    assert!(!valid_reason("   "));
    assert!(!valid_reason(&"x".repeat(MAX_REASON_BYTES + 1)));
    assert!(reset_confirmed(Some(RESET_CONFIRMATION_TOKEN)));
    assert!(!reset_confirmed(None));
}

#[test]
fn drive_backend_is_registered_only_when_drive_is_available() {
    assert!(registered_snapshot_backends(false).is_empty());
    let backends = registered_snapshot_backends(true);
    assert_eq!(backends.len(), 1);
    assert_eq!(backends[0].id(), "drive-archive");
    assert_eq!(backends[0].medium(), "drive-archive");
    assert!(backends[0].prepare("nightly").is_some());
}

#[test]
fn drive_descriptor_is_bounded_and_non_destructive() {
    let backend = drive_snapshot::backend();
    let secret_reason = "token=supersecret password=hunter2";
    let d = backend.manifest(secret_reason);

    assert_eq!(d["status"], "descriptor_only");
    assert_eq!(d["archive_created"], false);
    assert_eq!(d["destructive"], false);
    assert_eq!(d["bytes_copied"], 0);
    assert_eq!(d["scope"], ARCHIVE_SCOPE_FULL);
    assert_eq!(d["format"], ARCHIVE_FORMAT);
    assert_eq!(d["source_root"], DATA_ROOT);
    assert_eq!(d["retention"]["daily_keep"], RETENTION_DAILY_KEEP);
    assert_eq!(d["manifest_id"].as_str().map(str::len), Some(64));

    // The raw reason and obvious secret markers must never be echoed.
    let rendered = d.to_string().to_ascii_lowercase();
    assert!(!rendered.contains("supersecret"));
    assert!(!rendered.contains("hunter2"));
    assert!(!rendered.contains("token="));
    assert!(!rendered.contains("password="));

    // The object key is a bounded, fixed-layout logical path.
    let key = d["drive_object"].as_str().unwrap_or_default();
    assert!(key.starts_with("os-snapshots/"));
    assert!(key.ends_with(".tar.zst"));
    assert!(key.len() < 96);
}

#[test]
fn drive_receipt_uses_hashes_not_the_raw_reason() {
    let backend = drive_snapshot::backend();
    let receipt = backend
        .prepare("reason-with-sensitive-text")
        .unwrap_or_else(|| SnapshotReceipt {
            job_id: String::new(),
            location: String::new(),
        });
    assert!(receipt.job_id.starts_with("snap-"));
    assert_eq!(receipt.job_id.len(), "snap-".len() + 12);
    assert!(!receipt.job_id.contains("sensitive"));
    assert!(!receipt.location.contains("sensitive"));
    assert_eq!(
        receipt.location,
        backend.manifest("reason-with-sensitive-text")["drive_object"]
    );
}

#[test]
fn drive_snapshot_descriptor_stages_with_manifest_and_retention() {
    let d = snapshot_descriptor(&registered_snapshot_backends(true), "nightly backup");
    let d = d.unwrap_or_else(|| json!({}));
    assert_eq!(d["status"], "staged");
    assert_eq!(d["adapter"], "drive-archive");
    assert_eq!(d["medium"], "drive-archive");
    assert_eq!(d["destructive"], false);
    assert_eq!(d["source_root"], DATA_ROOT);
    assert_eq!(d["retention"]["daily_keep"], RETENTION_DAILY_KEEP);
    assert_eq!(d["manifest"]["archive_created"], false);
    assert_eq!(d["manifest"]["destructive"], false);
    assert!(d["manifest"]["drive_object"].is_string());
    assert!(d["manifest"]["retention"].is_object());
    assert!(!d
        .to_string()
        .to_ascii_lowercase()
        .contains("nightly backup"));
}
