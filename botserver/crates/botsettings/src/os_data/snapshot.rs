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
