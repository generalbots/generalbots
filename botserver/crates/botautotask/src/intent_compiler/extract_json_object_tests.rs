//! Offline tests for the balanced-JSON extractor (issue #2075). Models wrap
//! their JSON in prose or fences, and the pipeline must recover exactly the
//! first balanced object.

use super::extract_json_object;

#[test]
fn a_bare_object_is_returned_unchanged() {
    let input = r#"{"basic_program":"TALK \"hi\""}"#;
    assert_eq!(extract_json_object(input), input);
}

#[test]
fn surrounding_prose_is_trimmed_to_the_object() {
    let input = "Sure! Here is the plan:\n{\"a\":1}\nAnything else?";
    assert_eq!(extract_json_object(input), "{\"a\":1}");
}

#[test]
fn a_code_fence_is_stripped() {
    let input = "```json\n{\"a\":{\"b\":2}}\n```";
    assert_eq!(extract_json_object(input), "{\"a\":{\"b\":2}}");
}

#[test]
fn trailing_junk_after_the_first_object_is_dropped() {
    let input = r#"{"first":true} {"second":true}"#;
    assert_eq!(extract_json_object(input), r#"{"first":true}"#);
}

#[test]
fn braces_inside_strings_do_not_change_the_depth() {
    let input = r#"prefix {"text":"a } b { c","nested":{"k":"}"}} suffix"#;
    assert_eq!(
        extract_json_object(input),
        r#"{"text":"a } b { c","nested":{"k":"}"}}"#
    );
}

#[test]
fn escaped_quotes_inside_strings_are_respected() {
    let input = r#"{"text":"say \"}\" now"}"#;
    assert_eq!(extract_json_object(input), input);
}

#[test]
fn an_unbalanced_object_returns_the_original_input() {
    let input = "prefix {\"a\":1";
    assert_eq!(extract_json_object(input), input);
}

#[test]
fn input_without_an_object_is_returned_unchanged() {
    for input in ["", "no json here", "[1,2,3]"] {
        assert_eq!(extract_json_object(input), input, "{input}");
    }
}

#[test]
fn the_extracted_slice_parses_as_json() {
    let input = "The plan is {\"steps\":[{\"n\":1}],\"ok\":true} and that is it";
    let value: serde_json::Value =
        serde_json::from_str(extract_json_object(input)).expect("valid JSON");

    assert_eq!(value["ok"], true);
    assert_eq!(value["steps"][0]["n"], 1);
}
