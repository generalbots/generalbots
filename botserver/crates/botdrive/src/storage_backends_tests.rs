//! Unit tests for `super` (storage_backends.rs).
//!
//! Split out of `storage_backends.rs` to keep the file inside the project size
//! limit; the production code stays beside it.

use super::*;

use super::*;

#[test]
fn backend_names_round_trip() {
    for backend in [
        StorageBackend::Minio,
        StorageBackend::BackblazeB2,
        StorageBackend::Wasabi,
        StorageBackend::CloudflareR2,
    ] {
        assert_eq!(
            StorageBackend::from_name(backend.config_name()),
            backend,
            "{}",
            backend.config_name()
        );
    }
}

#[test]
fn vendor_spellings_resolve_to_the_same_backend() {
    assert_eq!(StorageBackend::from_name("B2"), StorageBackend::BackblazeB2);
    assert_eq!(StorageBackend::from_name("backblaze"), StorageBackend::BackblazeB2);
    assert_eq!(StorageBackend::from_name("Cloudflare-R2"), StorageBackend::CloudflareR2);
    assert_eq!(StorageBackend::from_name("R2"), StorageBackend::CloudflareR2);
}

#[test]
fn an_unknown_name_falls_back_to_minio_rather_than_failing_the_boot() {
    assert_eq!(StorageBackend::from_name("nonsense"), StorageBackend::Minio);
    assert_eq!(StorageBackend::from_name(""), StorageBackend::Minio);
    assert_eq!(StorageBackend::from_name("   "), StorageBackend::Minio);
}

#[test]
fn b2_and_r2_get_a_default_region_because_they_require_one() {
    assert!(StorageBackend::BackblazeB2.requires_explicit_region());
    assert!(StorageBackend::CloudflareR2.requires_explicit_region());
    assert!(!StorageBackend::Minio.requires_explicit_region());

    let selection = StorageSelection::new("b2", "https://s3.us-west-004.backblazeb2.com", "", "acme.gbai", None);
    assert_eq!(selection.region, "us-west-004");
}

#[test]
fn an_explicit_region_is_never_overridden() {
    let selection = StorageSelection::new("b2", "https://s3.eu-west-001.backblazeb2.com", "eu-west-001", "acme.gbai", None);
    assert_eq!(selection.region, "eu-west-001");
}

#[test]
fn minio_keeps_the_conventional_auto_region() {
    let selection = StorageSelection::new("minio", "http://127.0.0.1:9100", "", "acme.gbai", None);
    assert_eq!(selection.region, "auto");
}

#[test]
fn endpoints_are_normalised() {
    let selection = StorageSelection::new("minio", "http://127.0.0.1:9100/", "", "acme.gbai", None);
    assert_eq!(selection.endpoint, "http://127.0.0.1:9100");
}

#[test]
fn an_empty_fallback_endpoint_is_treated_as_absent() {
    let with_blank = StorageSelection::new("minio", "http://a", "", "b", Some("  ".to_string()));
    assert!(!with_blank.has_fallback());
    let with_real = StorageSelection::new("minio", "http://a", "", "b", Some("http://c/".to_string()));
    assert!(with_real.has_fallback());
    assert_eq!(with_real.fallback_endpoint.as_deref(), Some("http://c"));
}

#[test]
fn wasabi_is_flagged_as_unsuitable_for_a_runtime_bucket() {
    assert!(!StorageBackend::Wasabi.suits_runtime_buckets());
    assert!(StorageBackend::Wasabi.runtime_warning().is_some());
    assert!(StorageBackend::Minio.suits_runtime_buckets());
    assert!(StorageBackend::BackblazeB2.suits_runtime_buckets());
    assert!(StorageBackend::Wasabi.minimum_billable_gb() == 1_024);
    assert_eq!(StorageBackend::Wasabi.minimum_retention_days(), 90);
}

#[test]
fn the_minimum_billing_floor_is_documented_only_for_wasabi() {
    assert_eq!(StorageBackend::Minio.minimum_billable_gb(), 0);
    assert_eq!(StorageBackend::BackblazeB2.minimum_billable_gb(), 0);
    assert_eq!(StorageBackend::CloudflareR2.minimum_billable_gb(), 0);
    assert_eq!(StorageBackend::Minio.minimum_retention_days(), 0);
}

#[test]
fn egress_counters_accumulate() {
    let meter = EgressMeter::new();
    meter.record_read(100);
    meter.record_read(50);
    meter.record_write(7);
    assert_eq!(meter.bytes_read(), 150);
    assert_eq!(meter.bytes_written(), 7);
    assert_eq!(meter.snapshot().bytes_read, 150);
}

#[test]
fn egress_under_the_allowance_is_not_billable() {
    let meter = EgressMeter::new();
    meter.record_read(1_000);
    let stored = 1_000;
    assert_eq!(meter.billable_egress(StorageBackend::BackblazeB2, stored), 0, "3x allowance");
}

#[test]
fn egress_past_the_allowance_is_reported() {
    let meter = EgressMeter::new();
    meter.record_read(5_000);
    assert_eq!(meter.billable_egress(StorageBackend::BackblazeB2, 1_000), 2_000);
}

#[test]
fn backends_with_unmetered_egress_report_nothing() {
    let meter = EgressMeter::new();
    meter.record_read(10_000_000);
    assert_eq!(meter.billable_egress(StorageBackend::CloudflareR2, 1_000), 0);
    assert_eq!(meter.billable_egress(StorageBackend::Minio, 1_000), 0);
}
