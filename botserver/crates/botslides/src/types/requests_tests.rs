//! Offline unit tests for the shell-form number decoding (#2075).
//!
//! HTMX forms post numbers as strings ("0", "3"), while the JSON API sends real
//! numbers. The `deserialize_*` helpers accept both shapes and default instead
//! of rejecting the request — a wrong value here silently edits the wrong
//! slide, so the fallbacks are pinned.

use super::*;

#[test]
fn a_numeric_slide_index_is_accepted() {
    let request: UpdateSlideNotesRequest =
        serde_json::from_str(r#"{"presentation_id":"p","slide_index":2,"notes":"x"}"#)
            .expect("numeric form");
    assert_eq!(request.slide_index, 2);
}

#[test]
fn a_string_slide_index_is_accepted() {
    let request: UpdateSlideNotesRequest =
        serde_json::from_str(r#"{"presentation_id":"p","slide_index":" 3 ","notes":"x"}"#)
            .expect("string form");
    assert_eq!(request.slide_index, 3);
}

#[test]
fn a_missing_or_invalid_slide_index_defaults_to_the_first_slide() {
    for body in [
        r#"{"presentation_id":"p","notes":"x"}"#,
        r#"{"presentation_id":"p","slide_index":"","notes":"x"}"#,
        r#"{"presentation_id":"p","slide_index":"abc","notes":"x"}"#,
        r#"{"presentation_id":"p","slide_index":null,"notes":"x"}"#,
        r#"{"presentation_id":"p","slide_index":-1,"notes":"x"}"#,
    ] {
        let request: UpdateSlideNotesRequest =
            serde_json::from_str(body).expect("must not reject the request");
        assert_eq!(request.slide_index, 0, "{body}");
    }
}

#[test]
fn a_flat_transition_duration_accepts_both_shapes() {
    let numeric: SetTransitionRequest = serde_json::from_str(
        r#"{"presentation_id":"p","slide_index":1,"duration":1.5}"#,
    )
    .expect("numeric duration");
    assert_eq!(numeric.duration, Some(1.5));

    let textual: SetTransitionRequest = serde_json::from_str(
        r#"{"presentation_id":"p","slide_index":"1","duration":" 2.25 "}"#,
    )
    .expect("string duration");
    assert_eq!(textual.duration, Some(2.25));
    assert_eq!(textual.slide_index, 1);
}

#[test]
fn an_invalid_duration_becomes_none_instead_of_failing() {
    for body in [
        r#"{"presentation_id":"p","duration":"fast"}"#,
        r#"{"presentation_id":"p","duration":""}"#,
        r#"{"presentation_id":"p"}"#,
        r#"{"presentation_id":"p","duration":null}"#,
    ] {
        let request: SetTransitionRequest =
            serde_json::from_str(body).expect("must not reject the request");
        assert_eq!(request.duration, None, "{body}");
    }
}

#[test]
fn an_optional_slide_index_keeps_absence_distinct_from_zero() {
    let absent: UpdatePresenterRequest =
        serde_json::from_str(r#"{"session_id":"s"}"#).expect("no slide");
    assert_eq!(absent.current_slide, None);

    let zero: UpdatePresenterRequest =
        serde_json::from_str(r#"{"session_id":"s","current_slide":"0"}"#).expect("zero");
    assert_eq!(zero.current_slide, Some(0));

    let invalid: UpdatePresenterRequest =
        serde_json::from_str(r#"{"session_id":"s","current_slide":"last"}"#).expect("invalid");
    assert_eq!(invalid.current_slide, None);

    let numeric: UpdatePresenterRequest =
        serde_json::from_str(r#"{"session_id":"s","current_slide":4}"#).expect("numeric");
    assert_eq!(numeric.current_slide, Some(4));
}

#[test]
fn the_export_request_accepts_both_id_field_names() {
    let by_id: ExportRequest =
        serde_json::from_str(r#"{"id":"p1","format":"pdf"}"#).expect("id form");
    assert_eq!(by_id.id, "p1");

    let by_presentation: ExportRequest =
        serde_json::from_str(r#"{"presentation_id":"p2","format":"html"}"#)
            .expect("alias form");
    assert_eq!(by_presentation.id, "p2");
}
