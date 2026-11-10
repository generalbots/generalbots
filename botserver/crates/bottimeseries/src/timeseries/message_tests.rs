//! Offline unit tests for the metric builders (#2075).
//!
//! `Metrics::*` are the only constructors the runtime uses to write a metric
//! point. A wrong measurement name or tag key silently creates a new series in
//! Influx, so the wire shape of each builder is pinned through
//! `to_line_protocol`.

use super::*;

fn field_i64(point: &MetricPoint, key: &str) -> Option<i64> {
    match point.fields.get(key) {
        Some(FieldValue::Integer(value)) => Some(*value),
        _ => None,
    }
}

fn field_f64(point: &MetricPoint, key: &str) -> Option<f64> {
    match point.fields.get(key) {
        Some(FieldValue::Float(value)) => Some(*value),
        _ => None,
    }
}

fn tag<'a>(point: &'a MetricPoint, key: &str) -> Option<&'a str> {
    point.tags.get(key).map(String::as_str)
}

#[test]
fn every_builder_stamps_the_current_time() {
    let points = [
        Metrics::message("b", "web", "in"),
        Metrics::response_time("b", 1.0),
        Metrics::llm_tokens("b", "m", 1, 1),
        Metrics::active_sessions("b", 1),
        Metrics::error("b", "t", "m"),
        Metrics::storage_usage("b", 1, 1),
        Metrics::api_request("/x", "GET", 200, 1.0),
        Metrics::system(1.0, 2.0, 3.0),
    ];
    for point in points {
        assert!(point.timestamp.is_some(), "{} has no timestamp", point.measurement);
    }
}

#[test]
fn message_points_carry_the_three_tags() {
    let point = Metrics::message("bot-1", "whatsapp", "outbound");
    assert_eq!(point.measurement, "messages");
    assert_eq!(tag(&point, "bot_id"), Some("bot-1"));
    assert_eq!(tag(&point, "channel"), Some("whatsapp"));
    assert_eq!(tag(&point, "direction"), Some("outbound"));
    assert_eq!(field_i64(&point, "count"), Some(1));
}

#[test]
fn llm_tokens_sum_the_two_directions() {
    let point = Metrics::llm_tokens("bot-1", "gpt", 120, 30);
    assert_eq!(point.measurement, "llm_tokens");
    assert_eq!(tag(&point, "model"), Some("gpt"));
    assert_eq!(field_i64(&point, "prompt_tokens"), Some(120));
    assert_eq!(field_i64(&point, "completion_tokens"), Some(30));
    assert_eq!(field_i64(&point, "total_tokens"), Some(150));
}

#[test]
fn error_points_escape_the_message_field() {
    // A quoted/backslashed message must survive the line protocol encoding.
    let point = Metrics::error("bot-1", "llm", "said \"no\" \\ twice");
    assert_eq!(point.measurement, "errors");
    assert_eq!(tag(&point, "error_type"), Some("llm"));
    assert_eq!(field_i64(&point, "count"), Some(1));

    let line = point.to_line_protocol();
    assert!(line.contains("message=\"said \\\"no\\\" \\\\ twice\""), "{line}");

    // A newline inside a field value would split the point into two lines and
    // make the second half unparsable, so it must be escaped too.
    let point = Metrics::error("bot-1", "llm", "line one\nline two\r\nline three");
    let line = point.to_line_protocol();
    assert_eq!(line.lines().count(), 1, "{line}");
    assert!(line.contains("line one\\nline two\\r\\nline three"), "{line}");
}

#[test]
fn response_time_and_api_request_keep_their_numeric_fields() {
    let point = Metrics::response_time("bot-1", 12.5);
    assert_eq!(point.measurement, "response_time");
    assert_eq!(field_f64(&point, "duration_ms"), Some(12.5));

    let point = Metrics::api_request("/api/x", "POST", 201, 3.25);
    assert_eq!(point.measurement, "api_requests");
    assert_eq!(tag(&point, "endpoint"), Some("/api/x"));
    assert_eq!(tag(&point, "method"), Some("POST"));
    assert_eq!(field_i64(&point, "status_code"), Some(201));
    assert_eq!(field_i64(&point, "count"), Some(1));
    assert_eq!(field_f64(&point, "duration_ms"), Some(3.25));
}

#[test]
fn storage_and_session_points_use_unsigned_and_signed_integers() {
    let point = Metrics::storage_usage("bot-1", 4096, 7);
    assert_eq!(point.measurement, "storage_usage");
    assert!(matches!(
        point.fields.get("bytes_used"),
        Some(FieldValue::UnsignedInteger(4096))
    ));
    assert!(matches!(
        point.fields.get("file_count"),
        Some(FieldValue::UnsignedInteger(7))
    ));

    let point = Metrics::active_sessions("bot-1", -1);
    assert_eq!(point.measurement, "active_sessions");
    assert_eq!(field_i64(&point, "count"), Some(-1));
}

#[test]
fn system_metrics_carry_the_three_percentages_without_tags() {
    let point = Metrics::system(12.5, 40.0, 77.25);
    assert_eq!(point.measurement, "system_metrics");
    assert!(point.tags.is_empty());
    assert_eq!(field_f64(&point, "cpu_percent"), Some(12.5));
    assert_eq!(field_f64(&point, "memory_percent"), Some(40.0));
    assert_eq!(field_f64(&point, "disk_percent"), Some(77.25));
}
