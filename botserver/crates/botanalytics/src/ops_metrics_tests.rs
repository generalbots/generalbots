use super::*;

#[test]
fn classify_endpoint_prefers_the_specific_prefix() {
    assert_eq!(classify_endpoint("/api/ui/analytics/ops/slo"), "analytics");
    assert_eq!(classify_endpoint("/api/analytics/messages/count"), "analytics");
    assert_eq!(classify_endpoint("/api/ui/drive/files"), "ui");
    assert_eq!(classify_endpoint("/api/cloud/signup"), "cloud");
    assert_eq!(classify_endpoint("/api/auth/login"), "auth");
    assert_eq!(classify_endpoint("/api/ops/status"), "ops");
    assert_eq!(classify_endpoint("/api/bots"), "api");
    assert_eq!(classify_endpoint("/ws"), "websocket");
    assert_eq!(classify_endpoint("/suite/analytics/analytics.js"), "static");
    assert_eq!(classify_endpoint("/"), "other");
}

#[test]
fn classify_status_groups_codes() {
    assert_eq!(classify_status(101), "1xx");
    assert_eq!(classify_status(200), "2xx");
    assert_eq!(classify_status(304), "3xx");
    assert_eq!(classify_status(404), "4xx");
    assert_eq!(classify_status(503), "5xx");
}

#[test]
fn bucket_index_respects_the_bounds() {
    assert_eq!(bucket_index(10.0), 0);
    assert_eq!(bucket_index(50.0), 0);
    assert_eq!(bucket_index(51.0), 1);
    assert_eq!(bucket_index(500.0), 3);
    assert_eq!(bucket_index(9_000.0), 7);
}

#[test]
fn percentile_is_none_without_observations() {
    let empty = [0i64; BUCKET_COUNT];
    assert!(percentile_from_histogram(&empty, 0.95).is_none());
    assert!(ratio_above_histogram(&empty, 500.0).is_none());
}

#[test]
fn percentile_lands_in_the_observed_bucket() {
    let mut buckets = [0i64; BUCKET_COUNT];
    buckets[0] = 95; // 95 requests under 50 ms
    buckets[7] = 5; // 5 requests over 5 s
    assert_eq!(percentile_from_histogram(&buckets, 0.95), Some(50.0));
    assert_eq!(percentile_from_histogram(&buckets, 0.99), Some(5_000.0));
}

#[test]
fn ratio_above_counts_the_open_bucket() {
    let mut buckets = [0i64; BUCKET_COUNT];
    buckets[3] = 90; // <= 500 ms
    buckets[4] = 10; // <= 1000 ms
    let ratio = ratio_above_histogram(&buckets, 500.0).unwrap_or(-1.0);
    assert!((ratio - 0.1).abs() < f64::EPSILON);
}

#[test]
fn record_and_drain_keep_the_count() {
    let before: i64 = drain().iter().map(|row| row.count).sum();
    record("/api/ui/analytics/ops/slo", 200, Duration::from_millis(12));
    record("/api/ui/analytics/ops/slo", 200, Duration::from_millis(700));
    record("/api/ui/analytics/ops/slo", 500, Duration::from_millis(1_200));
    let rows = drain();
    let total: i64 = rows.iter().map(|row| row.count).sum();
    assert_eq!(total - before, 3);
    assert!(rows
        .iter()
        .any(|row| row.status_class == "5xx" && row.endpoint_class == "analytics"));
    // Draining twice yields nothing new.
    assert_eq!(drain().iter().map(|row| row.count).sum::<i64>(), 0);
}
