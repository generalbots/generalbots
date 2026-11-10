//! Offline tests for the model reply dialects (issue #2072): JSON tool calls,
//! the DSML/XML variants and the recovery paths for malformed payloads.

use super::{
    extract_tool_call, parse_dsml_tool_call, parse_xml_tool_use, recover_tool_name,
    salvage_tool_call,
};
use serde_json::Value;

fn field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .expect("string field")
}

#[test]
fn extracts_the_documented_json_envelope() {
    let raw = r#"thinking… {"__tool_call__": true, "name": "vm_exec", "arguments": "{\"code\":\"print(1)\"}"} trailing"#;
    let call = extract_tool_call(raw).expect("call parsed");
    assert_eq!(field(&call, "name"), "vm_exec");
    assert_eq!(field(&call, "arguments"), r#"{"code":"print(1)"}"#);
}

#[test]
fn unwraps_calls_nested_inside_the_marker() {
    let nested = r#"{"__tool_call__": {"name": "drive_save", "arguments": {"path": "a.txt"}}}"#;
    let call = extract_tool_call(nested).expect("call parsed");
    assert_eq!(field(&call, "name"), "drive_save");
    assert_eq!(call["arguments"]["path"], "a.txt");

    let legacy = r#"{"tool_call": {"name": "drive_fetch", "arguments": "{}"}}"#;
    let call = extract_tool_call(legacy).expect("call parsed");
    assert_eq!(field(&call, "name"), "drive_fetch");
}

#[test]
fn repairs_raw_control_characters_inside_arguments() {
    let raw = "{\"__tool_call__\": true, \"name\": \"vm_exec\", \"arguments\": \"line1\nline2\tend\"}";
    let call = extract_tool_call(raw).expect("repaired call parsed");
    assert_eq!(field(&call, "arguments"), "line1\nline2\tend");
}

#[test]
fn keeps_braces_and_quotes_inside_strings() {
    let braces = r#"{"name": "vm_exec", "arguments": "a}b{}"}"#;
    let call = extract_tool_call(braces).expect("call parsed");
    assert_eq!(field(&call, "arguments"), "a}b{}");

    let escaped = r#"{"name": "vm_exec", "arguments": "say \"hi\""}"#;
    let call = extract_tool_call(escaped).expect("call parsed");
    assert_eq!(field(&call, "arguments"), "say \"hi\"");
}

#[test]
fn missing_or_unbalanced_objects_yield_none() {
    assert!(extract_tool_call("no json here").is_none());
    assert!(extract_tool_call(r#"{"name": "vm_exec""#).is_none());
}

#[test]
fn salvage_recovers_arguments_with_unescaped_quotes() {
    let raw = r#"{"__tool_call__": true, "name": "vm_exec", "arguments": "print("x")"}"#;
    let call = salvage_tool_call(raw).expect("salvaged call");
    assert_eq!(field(&call, "name"), "vm_exec");
    assert_eq!(field(&call, "arguments"), r#"print("x")"#);

    let escaped = r#"{"name": "vm_exec", "arguments": "a\nb"}"#;
    let call = salvage_tool_call(escaped).expect("salvaged call");
    assert_eq!(field(&call, "arguments"), "a\nb");

    assert!(salvage_tool_call(r#"{"arguments": "{}"}"#).is_none());
}

#[test]
fn recovers_a_missing_tool_name() {
    let empty = serde_json::json!({});
    assert_eq!(
        recover_tool_name(&empty, r#"please call "drive_save" now"#),
        "drive_save"
    );

    let nested = serde_json::json!({"function": {"name": " vm_files "}});
    assert_eq!(recover_tool_name(&nested, ""), "vm_files");

    let flat = serde_json::json!({"tool": "classify_media"});
    assert_eq!(recover_tool_name(&flat, ""), "classify_media");

    assert_eq!(recover_tool_name(&empty, "nothing known"), "");
}

#[test]
fn parses_the_dsml_dialect_into_the_native_wrapper() {
    let raw = r#"<|DSML|tool_calls>
<|DSML|invoke name="vm_exec">
<|DSML|parameter name="arguments">{"language":"python","code":"print(1)"}<|/DSML|parameter>
<|/DSML|invoke>"#;
    let call = parse_dsml_tool_call(raw).expect("dsml parsed");
    assert_eq!(field(&call, "name"), "vm_exec");
    let arguments: Value =
        serde_json::from_str(field(&call, "arguments")).expect("arguments are JSON");
    assert_eq!(arguments["code"], "print(1)");

    let envelope = r#"invoke name="__tool_call__" parameter name="arguments" {"code":"x"}"#;
    let call = parse_dsml_tool_call(envelope).expect("envelope parsed");
    assert_eq!(field(&call, "name"), "vm_exec");

    assert!(parse_dsml_tool_call("no invoke block").is_none());
}

#[test]
fn parses_both_xml_tool_use_dialects() {
    let named = r#"<tool_use>{"name": "vm_exec", "arguments": {"language": "python"}}</tool_use>"#;
    let call = parse_xml_tool_use(named).expect("named form parsed");
    assert_eq!(field(&call, "name"), "vm_exec");
    assert_eq!(field(&call, "arguments"), r#"{"language":"python"}"#);

    let attribute = r#"<tool_use name="drive_save">{"path": "notes.txt"}</tool_use>"#;
    let call = parse_xml_tool_use(attribute).expect("attribute form parsed");
    assert_eq!(field(&call, "name"), "drive_save");
    let arguments: Value =
        serde_json::from_str(field(&call, "arguments")).expect("arguments are JSON");
    assert_eq!(arguments["path"], "notes.txt");
}

#[test]
fn xml_without_arguments_or_close_tag_is_not_a_call() {
    assert!(parse_xml_tool_use(r#"<tool_use>{"name": "vm_exec"}</tool_use>"#).is_none());
    assert!(parse_xml_tool_use(r#"<tool_use name="vm_exec">{"path": "a"}"#).is_none());
    assert!(parse_xml_tool_use("plain prose").is_none());
}
