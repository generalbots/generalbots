//! The quota snapshot must not outlive a write that changes the stored bytes.

use super::cached_quota::{cached_quota, invalidate_quota, store_quota};
use crate::drive_types::QuotaResponse;

#[test]
fn a_write_drops_the_cached_quota_so_the_next_request_recounts() {
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
