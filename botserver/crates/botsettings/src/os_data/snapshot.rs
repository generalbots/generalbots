//! Bounded, non-destructive Drive-backed snapshot descriptor adapter (#2470).
//!
//! This is a *descriptor* adapter for the OS Settings snapshot contract. It
//! stages a deterministic receipt describing where a future `tar.zst` archive
//! would live in Drive and which retention policy would apply. It never reads
//! or writes the data root, never calls Drive, never deletes anything and never
//! emits credentials, host paths, bucket names or the raw user reason.

use super::{
    manifest_id, retention_payload, SnapshotBackend, SnapshotReceipt, ARCHIVE_FORMAT,
    ARCHIVE_SCOPE_FULL, DATA_ROOT,
};
use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Logical prefix (object key, not a bucket) for staged OS archives. No host
/// path, endpoint or account identifier is ever included.
const ARCHIVE_PREFIX: &str = "os-snapshots";

/// Fixed number of hex characters kept from a digest in a key or job id. This
/// bounds the size of every generated identifier regardless of input length.
const HEX_PREFIX_LEN: usize = 12;

/// Drive descriptor adapter. A unit struct: it holds no credentials and no
/// mutable state, so it is safe to share as a single static instance.
pub(crate) struct DriveSnapshotBackend;

/// Safe, panic-free prefix of a hex digest (digests are ASCII, but `.get`
/// avoids any reliance on that for slicing).
fn hex_prefix(value: &str, len: usize) -> &str {
    match value.get(..len) {
        Some(prefix) => prefix,
        None => value,
    }
}

impl DriveSnapshotBackend {
    /// Deterministic, bounded object key. The user-supplied reason is hashed,
    /// so sensitive text typed into the request never reaches the response or
    /// any log. The UTC date groups descriptors by day for the daily/weekly/
    /// monthly retention policy; the key length is constant.
    fn object_key(&self, reason: &str) -> String {
        let reason_hash = hex::encode(Sha256::digest(reason.as_bytes()));
        let stamp = Utc::now().format("%Y%m%d").to_string();
        let manifest = manifest_id();
        format!(
            "{ARCHIVE_PREFIX}/{}/{stamp}/{}.{ARCHIVE_FORMAT}",
            hex_prefix(&manifest, HEX_PREFIX_LEN),
            hex_prefix(&reason_hash, HEX_PREFIX_LEN)
        )
    }

    /// Stable job id derived only from the (already sanitized) object key.
    fn job_id(&self, reason: &str) -> String {
        let digest = hex::encode(Sha256::digest(self.object_key(reason).as_bytes()));
        format!("snap-{}", hex_prefix(&digest, HEX_PREFIX_LEN))
    }

    /// Bounded, self-contained descriptor: archive manifest plus retention
    /// policy. Contains no user data, no credentials and performs no I/O.
    pub(crate) fn descriptor(&self, reason: &str) -> Value {
        json!({
            "manifest_id": manifest_id(),
            "status": "descriptor_only",
            "scope": ARCHIVE_SCOPE_FULL,
            "format": ARCHIVE_FORMAT,
            "source_root": DATA_ROOT,
            "drive_object": self.object_key(reason),
            "retention": retention_payload(),
            "destructive": false,
            "archive_created": false,
            "bytes_copied": 0,
            "note": "Bounded, non-destructive Drive archive descriptor. No bytes are read or written, no Drive call is made and no source data is deleted."
        })
    }
}

impl SnapshotBackend for DriveSnapshotBackend {
    fn id(&self) -> &'static str {
        "drive-archive"
    }

    fn medium(&self) -> &'static str {
        "drive-archive"
    }

    fn prepare(&self, reason: &str) -> Option<SnapshotReceipt> {
        Some(SnapshotReceipt {
            job_id: self.job_id(reason),
            location: self.object_key(reason),
        })
    }

    fn manifest(&self, reason: &str) -> Value {
        self.descriptor(reason)
    }
}

/// Single shared instance of the Drive descriptor adapter.
pub(crate) fn backend() -> &'static dyn SnapshotBackend {
    static BACKEND: DriveSnapshotBackend = DriveSnapshotBackend;
    &BACKEND
}

#[cfg(test)]
mod tests {
    //! Inline tests for the Drive snapshot descriptor adapter (#2470). They
    //! exercise the concrete adapter directly — not the HTTP layer — to prove
    //! the four safety properties of the descriptor contract: a bounded,
    //! deterministic, secret-free object key; carried retention plus explicit
    //! non-destructive flags; idempotent preparation; and no source deletion.

    use super::*;

    fn adapter() -> DriveSnapshotBackend {
        DriveSnapshotBackend
    }

    #[test]
    fn object_key_is_bounded_and_deterministic() {
        let a = adapter();
        let reason = "nightly backup of customer data";
        let first = a.object_key(reason);
        let second = a.object_key(reason);
        assert_eq!(first, second, "object key must be deterministic per reason");

        // The reason is hashed, so a very long reason cannot grow the key.
        let long_key = a.object_key(&"x".repeat(10_000));
        assert_eq!(long_key.len(), first.len());
        assert!(long_key.len() < 96);

        // Fixed four-segment shape: prefix / 12-hex / YYYYMMDD / 12-hex.tar.zst.
        let parts: Vec<&str> = first.split('/').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], ARCHIVE_PREFIX);
        assert_eq!(parts[1].len(), HEX_PREFIX_LEN);
        assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(parts[2].len(), 8);
        assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
        assert_eq!(parts[3].len(), HEX_PREFIX_LEN + 1 + ARCHIVE_FORMAT.len());
        assert!(first.ends_with(ARCHIVE_FORMAT));
    }

    #[test]
    fn object_key_is_secret_free() {
        let a = adapter();
        let secret = "token=sk_live_SECRET password=hunter2 eyJhbGciOiJIUzI1NiJ9";
        let key = a.object_key(secret);
        let lower = key.to_ascii_lowercase();
        for marker in ["token", "sk_live", "secret", "password", "hunter2", "eyjhb"] {
            assert!(!lower.contains(marker), "object key leaked {marker}");
        }
        // No host path, bucket, endpoint or account identifier either.
        for forbidden in ["/var/lib", "otterio", "bucket", "s3", "http", "https"] {
            assert!(!lower.contains(forbidden), "object key leaked {forbidden}");
        }
    }

    #[test]
    fn descriptor_carries_retention_and_non_destructive_flags() {
        let a = adapter();
        let d = a.descriptor("nightly backup");
        assert_eq!(d["status"], "descriptor_only");
        assert_eq!(d["retention"], retention_payload());
        assert_eq!(d["destructive"], false);
        assert_eq!(d["archive_created"], false);
        assert_eq!(d["bytes_copied"], 0);
        assert_eq!(d["manifest_id"], manifest_id());
        assert_eq!(d["drive_object"], a.object_key("nightly backup"));
    }

    #[test]
    fn prepare_is_idempotent() {
        let a = adapter();
        let reason = "nightly backup";
        let first = a.prepare(reason).expect("prepare must stage a receipt");
        let second = a.prepare(reason).expect("prepare must stage a receipt");
        assert_eq!(first.job_id, second.job_id);
        assert_eq!(first.location, second.location);
        // Bounded and derived from the key, never from the raw reason.
        assert_eq!(first.job_id.len(), "snap-".len() + HEX_PREFIX_LEN);
        assert_eq!(first.location, a.object_key(reason));
        assert!(!first.job_id.contains(reason));
        assert!(!first.location.contains(reason));
    }

    #[test]
    fn descriptor_never_deletes_source_data() {
        // The adapter is pure computation: it reads no file, makes no Drive
        // call and issues no delete. Its only output is a descriptor that
        // explicitly copies zero bytes and creates no archive. Even a reason
        // asking for deletion cannot flip those non-destructive flags.
        let a = adapter();
        for reason in ["wipe everything", "delete /var/lib/gbdesk", "purge all"] {
            let d = a.descriptor(reason);
            assert_eq!(d["status"], "descriptor_only");
            assert_eq!(d["source_root"], DATA_ROOT);
            assert_eq!(d["bytes_copied"], 0);
            assert_eq!(d["archive_created"], false);
            assert_eq!(d["destructive"], false);
        }
    }
}
