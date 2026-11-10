//! Settings payload parsing: the HTML escaper, the boolean coercion used by the
//! HTMX forms and the dual form/JSON body reader.

use super::{form_or_json_bool, html_escape, parse_notification_body};
use serde_json::json;

#[test]
fn escaping_covers_the_markup_breakers() {
    assert_eq!(
        html_escape("<b>\"x\" & y</b>"),
        "&lt;b&gt;&quot;x&quot; &amp; y&lt;/b&gt;"
    );
    assert_eq!(html_escape("plain"), "plain");
    assert_eq!(html_escape(""), "");
}

#[test]
fn the_escaper_leaves_entities_alone_until_the_ampersand_pass() {
    assert_eq!(html_escape("&lt;"), "&amp;lt;");
}

#[test]
fn booleans_accept_every_wire_spelling() {
    assert!(form_or_json_bool(Some(&json!(true))));
    assert!(!form_or_json_bool(Some(&json!(false))));
    assert!(form_or_json_bool(Some(&json!("on"))));
    assert!(form_or_json_bool(Some(&json!("true"))));
    assert!(form_or_json_bool(Some(&json!("1"))));
    assert!(!form_or_json_bool(Some(&json!("off"))));
    assert!(!form_or_json_bool(Some(&json!("false"))));
    assert!(!form_or_json_bool(Some(&json!("0"))));
    assert!(!form_or_json_bool(Some(&json!(""))));
}

#[test]
fn numbers_are_truthy_only_when_non_zero() {
    assert!(form_or_json_bool(Some(&json!(1))));
    assert!(form_or_json_bool(Some(&json!(-3))));
    assert!(!form_or_json_bool(Some(&json!(0))));
}

#[test]
fn absent_and_unsupported_values_are_false() {
    assert!(!form_or_json_bool(None));
    assert!(!form_or_json_bool(Some(&json!(null))));
    assert!(!form_or_json_bool(Some(&json!([true]))));
    assert!(!form_or_json_bool(Some(&json!({ "on": true }))));
    // A float has no integer representation, so it is not truthy.
    assert!(!form_or_json_bool(Some(&json!(1.5))));
}

#[test]
fn a_json_object_body_is_accepted() {
    let body = br#"{"email_mentions":true,"email_digest":false}"#;
    let parsed = parse_notification_body("application/json", body).expect("parses");
    assert_eq!(parsed.get("email_mentions"), Some(&json!(true)));
    assert_eq!(parsed.get("email_digest"), Some(&json!(false)));
}

#[test]
fn a_json_body_must_be_an_object() {
    let error = parse_notification_body("application/json", b"[1,2]").expect_err("must fail");
    assert_eq!(error, "Expected a JSON object");
    let error = parse_notification_body("application/json", b"not json").expect_err("must fail");
    assert!(error.starts_with("Invalid JSON:"));
}

#[test]
fn a_form_body_becomes_string_values() {
    let parsed =
        parse_notification_body("application/x-www-form-urlencoded", b"email_digest=on&time=08%3A00")
            .expect("parses");
    assert_eq!(parsed.get("email_digest"), Some(&json!("on")));
    assert_eq!(parsed.get("time"), Some(&json!("08:00")));
}

#[test]
fn an_unchecked_checkbox_is_absent_from_the_form_payload() {
    let parsed = parse_notification_body("application/x-www-form-urlencoded", b"")
        .expect("parses");
    assert!(parsed.is_empty());
    assert!(!form_or_json_bool(parsed.get("email_digest")));
}

#[test]
fn a_malformed_form_body_is_reported() {
    // `%FF` is not valid UTF-8, so the decoder must refuse the body instead of
    // silently dropping the cell.
    let error = parse_notification_body("application/x-www-form-urlencoded", b"email_digest=%FF")
        .expect_err("must fail");
    assert!(error.starts_with("Invalid form:"));
}
