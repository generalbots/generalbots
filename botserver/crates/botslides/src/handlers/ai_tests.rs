//! #2923 — offline contract tests for the Slides AI handler.

use super::*;

const ACTION_COMMANDS: &[&str] = &[
    "add slide",
    "duplicate slide",
    "delete slide",
    "insert image",
    "add chart",
    "apply theme",
    "set transition",
];

fn request(command: &str) -> SlidesAiRequest {
    SlidesAiRequest {
        command: command.to_string(),
        slide_index: Some(2),
        presentation_id: Some("deck-1".to_string()),
    }
}

#[test]
fn typed_action_is_extracted_with_deck_context() {
    let answer = r#"{"response":"Inserted a content slide","action":"slides.slide.add","data":{"layout":"content","position":3}}"#;
    let parsed = parse_llm_answer(answer, &request("add slide"));
    assert_eq!(parsed.action.as_deref(), Some("slides.slide.add"));
    assert!(parsed.data.is_some(), "typed action carries data");
    let data = parsed.data.unwrap_or_default();
    assert_eq!(data["presentation_id"], serde_json::json!("deck-1"));
    assert_eq!(data["layout"], serde_json::json!("content"));
}

#[test]
fn unlisted_action_is_never_executable() {
    let answer =
        r#"{"response":"Trying to run a script","action":"shell.exec","data":{"cmd":"rm"}}"#;
    let parsed = parse_llm_answer(answer, &request("do something unsafe"));
    assert!(parsed.action.is_none(), "unlisted verb must be stripped");
    assert!(parsed.data.is_none(), "stripped verb must carry no data");
}

#[test]
fn plain_llm_answer_is_returned_without_an_action() {
    let parsed = parse_llm_answer("Here are three slide ideas.", &request("ideas"));
    assert_eq!(parsed.response, "Here are three slide ideas.");
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
fn prompts_carry_every_allowed_verb_and_the_request_context() {
    let prompt = system_prompt();
    for verb in ALLOWED_ACTIONS {
        assert!(
            prompt.contains(verb),
            "missing typed verb in system prompt: {verb}"
        );
    }
    for command in ACTION_COMMANDS {
        let text = user_prompt(&request(command));
        assert!(
            text.contains(command),
            "command dropped from prompt: {command}"
        );
        assert!(text.contains("deck-1"));
        assert!(text.contains("2"));
    }
}

#[test]
fn handler_has_no_keyword_hint_table() {
    let source = include_str!("ai.rs");
    let canned = [
        concat!("cannot modify the ", "deck"),
        concat!("only explains how to use the ", "editor"),
        concat!("This chat endpoint ", "cannot"),
        "Use the + button",
    ];
    for hint in canned {
        assert!(
            !source.contains(hint),
            "canned hint still in handler: {hint}"
        );
    }
    assert!(!source.contains("fn ai_reply_for"), "keyword table removed");
}
