//! #2923 — offline contract tests for the Sheet AI handler.

use super::*;

const ACTION_COMMANDS: &[&str] = &[
    "sum column B",
    "format as currency",
    "create a chart",
    "sort ascending",
    "freeze rows",
    "merge cells",
];

fn request(command: &str) -> SheetAiRequest {
    SheetAiRequest {
        command: command.to_string(),
        selection: Some(serde_json::json!({"range": "B2:B10"})),
        active_cell: Some(serde_json::json!("B11")),
        sheet_id: Some("sheet-1".to_string()),
    }
}

#[test]
fn typed_action_is_extracted_with_workbook_context() {
    let answer =
        r#"{"response":"Wrote =SUM(B2:B10)","action":"sheet.formula.set","data":{"cell":"B11"}}"#;
    let parsed = parse_llm_answer(answer, &request("sum column B"));
    assert_eq!(parsed.action.as_deref(), Some("sheet.formula.set"));
    assert!(parsed.data.is_some(), "typed action carries data");
    let data = parsed.data.unwrap_or_default();
    assert_eq!(data["file_id"], serde_json::json!("sheet-1"));
    assert_eq!(data["cell"], serde_json::json!("B11"));
}

#[test]
fn unlisted_action_is_never_executable() {
    let answer = r#"{"response":"Running a script","action":"shell.exec","data":{"cmd":"rm"}}"#;
    let parsed = parse_llm_answer(answer, &request("do something unsafe"));
    assert!(parsed.action.is_none(), "unlisted verb must be stripped");
    assert!(parsed.data.is_none(), "stripped verb must carry no data");
}

#[test]
fn plain_llm_answer_is_returned_without_an_action() {
    let parsed = parse_llm_answer("B2:B10 totals 42.", &request("what is the total?"));
    assert_eq!(parsed.response, "B2:B10 totals 42.");
    assert!(parsed.action.is_none());
    assert!(parsed.data.is_none());
}

#[test]
fn unavailability_is_typed_without_instructions() {
    for reason in [Unavailable::NotConfigured, Unavailable::Provider] {
        let body = unavailable_body(reason);
        assert_eq!(body["error"], serde_json::json!("ai_unavailable"));
        assert_eq!(body["reason"], serde_json::json!(reason.reason()));
        assert!(
            body.get("response").is_none(),
            "no instruction reply on 503"
        );
        assert_eq!(
            unavailable_response(reason).status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}

#[test]
fn every_action_command_reaches_the_llm_prompt() {
    let prompt = build_prompt(&request("sum column B"));
    for verb in ALLOWED_ACTIONS {
        assert!(
            prompt.contains(verb),
            "missing typed verb in prompt: {verb}"
        );
    }
    assert!(prompt.contains("B2:B10"), "selection must reach the LLM");
    assert!(prompt.contains("B11"), "active cell must reach the LLM");
    for command in ACTION_COMMANDS {
        assert!(
            build_prompt(&request(command)).contains(command),
            "command dropped from prompt: {command}"
        );
    }
}

#[test]
fn handler_has_no_keyword_hint_table() {
    let source = include_str!("ai.rs");
    let canned = [
        concat!("cannot modify the ", "workbook"),
        concat!("only explains how to use the ", "editor"),
        concat!("This chat endpoint ", "cannot"),
        "AutoSum in the toolbar",
    ];
    for hint in canned {
        assert!(
            !source.contains(hint),
            "canned hint still in handler: {hint}"
        );
    }
    assert!(!source.contains("fn ai_reply_for"), "keyword table removed");
}
