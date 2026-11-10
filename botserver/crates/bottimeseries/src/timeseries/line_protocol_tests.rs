//! Offline unit tests for the metrics encoder and Influx CSV decoder (#2071).
//!
//! `to_line_protocol` is what the metrics pipeline actually writes: tags and
//! fields must be sorted (so a line is reproducible), integers need the `i`
//! suffix, strings need quoting and escaping, and the timestamp is nanoseconds.
//! `parse_csv_result` is the read path and must classify every cell.

use super::*;
use chrono::TimeZone;
use serde_json::json;

#[test]
fn a_bare_point_renders_measurement_and_fields() {
    let point = MetricPoint::new("messages").field_i64("count", 1);
    assert_eq!(point.to_line_protocol(), "messages count=1i");
}

#[test]
fn tags_and_fields_are_sorted_alphabetically() {
    let point = MetricPoint::new("messages")
        .tag("channel", "web")
        .tag("bot_id", "b1")
        .field_i64("z", 2)
        .field_i64("a", 1);
    assert_eq!(point.to_line_protocol(), "messages,bot_id=b1,channel=web a=1i,z=2i");
}

#[test]
fn every_field_type_gets_its_influx_suffix() {
    let point = MetricPoint::new("m")
        .field_i64("i", -3)
        .field_u64("u", 7)
        .field_f64("f", 1.5)
        .field_bool("b", true)
        .field_str("s", "hi");
    assert_eq!(
        point.to_line_protocol(),
        "m b=true,f=1.5,i=-3i,s=\"hi\",u=7u"
    );
}

#[test]
fn string_values_escape_quotes_and_backslashes() {
    let point = MetricPoint::new("m").field_str("s", "a\"b\\c");
    assert_eq!(point.to_line_protocol(), "m s=\"a\\\"b\\\\c\"");
}

#[test]
fn tag_keys_and_values_escape_protocol_separators() {
    let point = MetricPoint::new("m").tag("k,= ", "v,= ").field_i64("n", 1);
    assert_eq!(point.to_line_protocol(), "m,k\\,\\=\\ =v\\,\\=\\  n=1i");
}

#[test]
fn field_keys_escape_protocol_separators() {
    let point = MetricPoint::new("m").field_i64("k,= ", 1);
    assert_eq!(point.to_line_protocol(), "m k\\,\\=\\ =1i");
}

#[test]
fn the_timestamp_is_appended_in_nanoseconds() {
    let at = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let point = MetricPoint::new("m").field_i64("n", 1).at(at);
    let line = point.to_line_protocol();
    assert_eq!(
        line,
        format!("m n=1i {}", at.timestamp_nanos_opt().unwrap_or(0))
    );
    assert!(line.ends_with("1767225600000000000"), "got {line}");
}

#[test]
fn the_builder_helpers_fill_the_expected_slots() {
    let point = Metrics::llm_tokens("b1", "gpt", 10, 5);
    assert_eq!(point.measurement, "llm_tokens");
    assert_eq!(point.tags.get("bot_id").map(String::as_str), Some("b1"));
    assert_eq!(point.tags.get("model").map(String::as_str), Some("gpt"));
    // `FieldValue` has no `PartialEq`; read the discriminant explicitly.
    let total = match point.fields.get("total_tokens") {
        Some(FieldValue::Integer(v)) => *v,
        _ => -1,
    };
    assert_eq!(total, 15);
    assert!(point.timestamp.is_some());

    let message = Metrics::message("b1", "web", "in");
    assert_eq!(message.measurement, "messages");
    assert_eq!(message.tags.len(), 3);
}

#[test]
fn csv_decoder_skips_annotations_and_types_every_cell() {
    let csv = "#datatype,string,long,double,boolean\n\
               #group,false,false,false,false\n\
               ,result,count,ratio,ok\n\
               ,mean,3,0.5,true\n";
    let parsed = TimeSeriesClient::parse_csv_result(csv).expect("valid csv");
    assert_eq!(parsed.columns, vec!["", "result", "count", "ratio", "ok"]);
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0], vec![json!(""), json!("mean"), json!(3), json!(0.5), json!(true)]);
}

#[test]
fn csv_decoder_handles_an_empty_document() {
    let parsed = TimeSeriesClient::parse_csv_result("").expect("empty is not an error");
    assert!(parsed.columns.is_empty());
    assert!(parsed.rows.is_empty());
}

#[test]
fn csv_decoder_keeps_only_the_header_when_there_are_no_rows() {
    let parsed = TimeSeriesClient::parse_csv_result("#comment\n\n,result,value\n").expect("header only");
    assert_eq!(parsed.columns, vec!["", "result", "value"]);
    assert!(parsed.rows.is_empty());
}

#[test]
fn csv_decoder_reads_false_and_negative_numbers() {
    let csv = ",ok,delta\n,false,-12\n";
    let parsed = TimeSeriesClient::parse_csv_result(csv).expect("valid csv");
    assert_eq!(parsed.rows[0][1], json!(false));
    assert_eq!(parsed.rows[0][2], json!(-12));
}
