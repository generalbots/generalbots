//! The quota snapshot must not outlive a write that changes the stored bytes.

use super::cached_quota::{
    cached_quota, invalidate_quota, store_quota, trash_changed, visible_used_bytes,
};
use crate::drive_types::QuotaResponse;
use botlib::traits::DriveObjectInfo;
use std::sync::Mutex;

/// The quota cache is one process-wide value; tests that fill and drop it
/// run one at a time.
static CACHE_LOCK: Mutex<()> = Mutex::new(());

fn object(key: &str, size: u64) -> DriveObjectInfo {
    DriveObjectInfo {
        key: key.to_string(),
        etag: None,
        size,
    }
}

#[test]
fn a_write_drops_the_cached_quota_so_the_next_request_recounts() {
    let _serial = CACHE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    // Initialises the cache cell; it may already hold nothing.
    cached_quota();
    store_quota(&QuotaResponse {
        used_bytes: 100,
        total_bytes: 1000,
        available_bytes: 900,
        percentage_used: 10.0,
    });
    assert_eq!(cached_quota().map(|quota| quota.used_bytes), Some(100));

    invalidate_quota();

    assert!(cached_quota().is_none());
}

#[test]
fn trashing_or_restoring_drops_the_cached_quota() {
    let _serial = CACHE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    cached_quota();
    store_quota(&QuotaResponse {
        used_bytes: 500,
        total_bytes: 1000,
        available_bytes: 500,
        percentage_used: 50.0,
    });
    assert!(cached_quota().is_some());

    assert!(trash_changed().0.success);

    assert!(cached_quota().is_none());
}

#[test]
fn files_waiting_in_trash_do_not_count_as_used() {
    let objects = vec![
        object("users/u1/report.pdf", 1_000),
        object(".trash/u1/20261127T100000000Z/users/u1/old.pdf", 4_000),
        object(".trash/u1/20261127T100001000Z/photos/a.png", 2_000),
        object("users/u1/.trash-notes.txt", 30),
    ];

    assert_eq!(visible_used_bytes(&objects), 1_030);
    assert_eq!(visible_used_bytes(&[]), 0);
}
