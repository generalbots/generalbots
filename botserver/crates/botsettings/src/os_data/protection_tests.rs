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
    let key = d["drive_object"].as_str().unwrap_or("");
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

#[test]
fn drive_unavailable_fails_closed_with_no_descriptor() {
    // A Drive-less build registers no backend, so even a valid reason yields
    // no descriptor. The handler maps that decision to 503 /
    // os_backend_unavailable: nothing is staged, copied or deleted.
    let reason = "nightly backup";
    assert!(valid_reason(reason));
    let backends = registered_snapshot_backends(false);
    assert!(backends.is_empty());
    assert!(snapshot_descriptor(&backends, reason).is_none());
    assert!(snapshot_descriptor(&[], reason).is_none());
    // An unprovisioned backend (prepare -> None) also cannot stage a job.
    assert!(snapshot_descriptor(&[&B(false)], reason).is_none());
    let status = status_payload();
    assert_eq!(status["backup"]["descriptor_adapter"], "drive-archive");
    let result = status["backup"]["contract"]["result"]
        .as_str()
        .unwrap_or("");
    assert!(result.contains("503"));
}

#[test]
fn drive_configured_stages_202_descriptor_with_retention_and_no_destruction() {
    // The configured path is the descriptor the handler answers with 202:
    // staged status, retention counts, a manifest and destructive:false.
    let reason = "nightly backup";
    assert!(valid_reason(reason));
    let backends = registered_snapshot_backends(true);
    assert_eq!(backends.len(), 1);
    let staged = snapshot_descriptor(&backends, reason).unwrap_or_else(|| json!({}));

    assert_eq!(staged["status"], "staged");
    assert_eq!(staged["adapter"], "drive-archive");
    assert_eq!(staged["medium"], "drive-archive");
    assert_eq!(staged["source_root"], DATA_ROOT);
    assert_eq!(staged["destructive"], false);
    assert_eq!(
        staged["job_id"].as_str().map(str::len),
        Some("snap-".len() + 12)
    );
    assert!(staged["location"].is_string());
    assert!(staged["message"]
        .as_str()
        .unwrap_or("")
        .contains("no archive was created"));

    let retention = &staged["retention"];
    assert_eq!(retention["daily_keep"], RETENTION_DAILY_KEEP);
    assert_eq!(retention["weekly_keep"], RETENTION_WEEKLY_KEEP);
    assert_eq!(retention["monthly_keep"], RETENTION_MONTHLY_KEEP);
    assert_eq!(retention["unit"], "snapshots");

    let manifest = &staged["manifest"];
    assert_eq!(manifest["status"], "descriptor_only");
    assert_eq!(manifest["scope"], ARCHIVE_SCOPE_FULL);
    assert_eq!(manifest["format"], ARCHIVE_FORMAT);
    assert_eq!(manifest["source_root"], DATA_ROOT);
    assert_eq!(manifest["archive_created"], false);
    assert_eq!(manifest["destructive"], false);
    assert_eq!(manifest["bytes_copied"], 0);
    assert_eq!(manifest["manifest_id"].as_str().map(str::len), Some(64));
    assert_eq!(manifest["retention"]["daily_keep"], RETENTION_DAILY_KEEP);
    assert!(manifest["drive_object"].is_string());

    let status = status_payload();
    let result = status["backup"]["contract"]["result"]
        .as_str()
        .unwrap_or("");
    assert!(result.contains("202"));
}

#[test]
fn drive_reason_never_leaks_into_descriptor_or_receipt() {
    let backend = drive_snapshot::backend();
    let reasons = [
        "token=alpha-secret-999 password=bravo-secret-888",
        "api_key=sk_live_51Hx payload=eyJhbGciOiJIUzI1NiJ9",
        RESET_CONFIRMATION_TOKEN,
    ];
    let markers = [
        "alpha-secret",
        "bravo-secret",
        "sk_live",
        "eyjhb",
        "reset this pc",
    ];
    for reason in reasons {
        let descriptor = backend.manifest(reason);
        let receipt = backend.prepare(reason).unwrap_or_else(|| SnapshotReceipt {
            job_id: String::new(),
            location: String::new(),
        });
        let fields = [
            descriptor["manifest_id"].as_str().unwrap_or(""),
            descriptor["drive_object"].as_str().unwrap_or(""),
            receipt.job_id.as_str(),
            receipt.location.as_str(),
        ];
        let rendered = fields.join(" ").to_ascii_lowercase();
        for marker in markers {
            assert!(
                !rendered.contains(marker),
                "reason fragment leaked into a Drive descriptor: {marker}"
            );
        }
    }
}

#[test]
fn drive_descriptor_key_is_bounded_logical_and_deterministic() {
    let backend = drive_snapshot::backend();
    let reason = "nightly backup";
    let first = backend.manifest(reason);
    let second = backend.manifest(reason);
    assert_eq!(first["drive_object"], second["drive_object"]);
    assert_eq!(first["manifest_id"], second["manifest_id"]);

    let key = first["drive_object"].as_str().unwrap_or("");
    assert!(key.starts_with("os-snapshots/"));
    assert!(key.ends_with(".tar.zst"));
    let parts: Vec<&str> = key.split('/').collect();
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0], "os-snapshots");
    assert_eq!(parts[1].len(), 12); // truncated manifest digest
    assert_eq!(parts[2].len(), 8); // YYYYMMDD
    assert_eq!(parts[3].len(), 20); // 12 hex + ".tar.zst"

    // The object key is a logical path only — never a host path, bucket,
    // endpoint or account identifier.
    let key_lower = key.to_ascii_lowercase();
    for forbidden in [
        "/var/lib", "minio", "s3://", "http://", "https://", "bucket",
    ] {
        assert!(
            !key_lower.contains(forbidden),
            "object key leaked {forbidden}"
        );
    }
    // Credential markers must never appear anywhere in the descriptor. The
    // declared source_root constant is expected and checked separately above.
    let rendered = first.to_string().to_ascii_lowercase();
    for forbidden in ["secret", "password", "token"] {
        assert!(
            !rendered.contains(forbidden),
            "descriptor leaked {forbidden}"
        );
    }
}

#[test]
fn reason_validation_boundaries_are_stable() {
    assert!(valid_reason("a"));
    assert!(valid_reason("  nightly backup  "));
    assert!(valid_reason(&"x".repeat(MAX_REASON_BYTES)));
    assert!(!valid_reason(&"x".repeat(MAX_REASON_BYTES + 1)));
    assert!(!valid_reason("   "));
    assert!(!valid_reason("line\nbreak"));
    assert!(!valid_reason("nightly\rbackup"));
}

#[test]
fn export_contract_remains_descriptor_only_and_never_destructive() {
    let manifest = archive_manifest();
    assert_eq!(manifest["status"], "descriptor_only");
    assert_eq!(manifest["archive_created"], false);
    assert_eq!(manifest["destructive"], false);
    assert_eq!(manifest["bytes_read"], 0);
    assert_eq!(manifest["source_root"], DATA_ROOT);
    assert_eq!(manifest["scope"], ARCHIVE_SCOPE_FULL);
    assert_eq!(manifest["format"], ARCHIVE_FORMAT);
    assert_eq!(manifest["manifest_id"].as_str().map(str::len), Some(64));
    assert_eq!(manifest["includes"].as_array().map(Vec::len), Some(4));
    assert_eq!(manifest["excludes"].as_array().map(Vec::len), Some(3));

    let status = status_payload();
    assert_eq!(status["export"]["enabled"], false);
    assert_eq!(status["export"]["implemented"], false);
    assert_eq!(status["export"]["descriptor_available"], true);
    let result = status["export"]["contract"]["result"]
        .as_str()
        .unwrap_or("");
    assert!(result.contains("200"));
    assert!(result.contains("no archive is created"));
}

#[test]
fn reset_contract_stays_disabled_and_requires_the_exact_token() {
    assert!(reset_confirmed(Some(RESET_CONFIRMATION_TOKEN)));
    assert!(!reset_confirmed(None));
    assert!(!reset_confirmed(Some("")));
    assert!(!reset_confirmed(Some("reset this pc")));
    assert!(!reset_confirmed(Some("RESET THIS PC ")));
    assert!(!reset_confirmed(Some("RESET")));

    let status = status_payload();
    assert_eq!(status["reset"]["enabled"], false);
    assert_eq!(status["reset"]["implemented"], false);
    assert_eq!(
        status["reset"]["confirmation_token"],
        RESET_CONFIRMATION_TOKEN
    );
    let result = status["reset"]["contract"]["result"]
        .as_str()
        .unwrap_or("");
    assert!(result.contains("400"));
    assert!(result.contains("501"));
}
