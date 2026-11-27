//! Protocol, action and verdict tests, on lines recorded from Claude Code
//! 2.1.330 in the live spike.

use serde_json::json;

use super::protocol::{self, Incoming};
use super::{exit_code, interpret, output_text, ClaudeAction, DriveReport, ToolRequest, TurnResult};

const CAN_USE_TOOL: &str = r#"{"type": "control_request", "request_id": "6633d675-c0b1-498d-9e25-10a1418baca1", "request": {"subtype": "can_use_tool", "tool_name": "Bash", "display_name": "Bash", "input": {"command": "touch /home/claude/t1", "description": "touch"}, "description": "touch", "permission_suggestions": [{"type": "addDirectories", "directories": ["/home/claude"], "destination": "session"}], "tool_use_id": "toolu_06"}}"#;
const BASH_FAILED: &str = r#"{"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "content": "Exit code 2\nout\nls: cannot access '/nonexistent': No such file or directory", "is_error": true, "tool_use_id": "toolu_01"}]}, "parent_tool_use_id": null, "session_id": "36f8b26f-1568-4d72-826c-aa5f59e193b5"}"#;
const NOT_LOGGED_IN: &str = r#"{"type": "assistant", "message": {"id": "07465d01", "model": "<synthetic>", "role": "assistant", "content": [{"type": "text", "text": "Not logged in · Please run /login"}]}, "parent_tool_use_id": null}"#;

fn request(tool: &str, input: serde_json::Value) -> ToolRequest {
    ToolRequest { tool_name: tool.to_string(), tool_use_id: Some("t".into()), input }
}

#[test]
fn can_use_tool_is_parsed_with_its_ids_and_exact_input() {
    let parsed = protocol::parse(CAN_USE_TOOL);
    let [Incoming::CanUseTool { request_id, request }] = parsed.as_slice() else {
        panic!("not a permission request: {parsed:?}");
    };
    assert_eq!(request_id, "6633d675-c0b1-498d-9e25-10a1418baca1");
    assert_eq!(request.tool_name, "Bash");
    assert_eq!(request.tool_use_id.as_deref(), Some("toolu_06"));
    assert_eq!(request.input["command"], "touch /home/claude/t1");
}

#[test]
fn answers_have_the_shape_the_cli_accepted() {
    let input = json!({"command": "ls"});
    assert_eq!(
        protocol::allow("r1", &input),
        json!({"type": "control_response", "response": {"subtype": "success", "request_id": "r1",
               "response": {"behavior": "allow", "updatedInput": {"command": "ls"}}}})
    );
    assert_eq!(protocol::deny("r1", "no")["response"]["response"], json!({"behavior": "deny", "message": "no"}));
    assert_eq!(protocol::unsupported("h1")["response"]["subtype"], "error");
    assert_eq!(protocol::interrupt()["request"]["subtype"], "interrupt");
    assert_eq!(protocol::user_message("oi", &[])["message"], json!({"role": "user", "content": "oi"}));
}

#[test]
fn tool_results_and_cli_text_are_told_apart_from_model_text() {
    let parsed = protocol::parse(BASH_FAILED);
    let [Incoming::ToolResult { tool_use_id, is_error, text }] = parsed.as_slice() else {
        panic!("not a tool result: {parsed:?}");
    };
    assert_eq!((tool_use_id.as_str(), *is_error), ("toolu_01", true));
    assert_eq!(exit_code(*is_error, text), Some(2));
    assert_eq!(output_text(text), "out\nls: cannot access '/nonexistent': No such file or directory");
    assert_eq!(protocol::parse(NOT_LOGGED_IN), vec![Incoming::CliText("Not logged in · Please run /login".into())]);
    assert_eq!(protocol::parse("not json"), vec![Incoming::Ignored]);
    let other = r#"{"type":"control_request","request_id":"m1","request":{"subtype":"mcp_message"}}"#;
    assert_eq!(protocol::parse(other), vec![Incoming::OtherControl { request_id: "m1".into(), subtype: "mcp_message".into() }]);
}

#[test]
fn each_known_tool_becomes_the_action_the_card_shows() {
    let cwd = "/home/claude";
    let cases = [
        (request("Bash", json!({"command": "ls -la"})), Some(ClaudeAction::Command { command: "ls -la".into() })),
        (
            request("Bash", json!({"command": "pip install --quiet requests"})),
            Some(ClaudeAction::Install {
                manager: "pip".into(),
                packages: vec!["requests".into()],
                command: "pip install --quiet requests".into(),
            }),
        ),
        (
            request("Bash", json!({"command": "pip install x && rm -rf ~"})),
            Some(ClaudeAction::Command { command: "pip install x && rm -rf ~".into() }),
        ),
        (
            request("Bash", json!({"command": "pip install --index-url=http://mirror.test x"})),
            Some(ClaudeAction::Command { command: "pip install --index-url=http://mirror.test x".into() }),
        ),
        (request("Read", json!({"file_path": ".claude/.credentials.json"})), Some(ClaudeAction::ReadFile { path: "/home/claude/.claude/.credentials.json".into() })),
        (request("Write", json!({"file_path": "/home/claude/a.txt", "content": "x"})), Some(ClaudeAction::WriteFile { path: "/home/claude/a.txt".into(), edit: false })),
        (request("Edit", json!({"file_path": "./a.txt"})), Some(ClaudeAction::WriteFile { path: "/home/claude/a.txt".into(), edit: true })),
        (request("NotebookEdit", json!({"notebook_path": "/home/claude/n.ipynb"})), Some(ClaudeAction::WriteFile { path: "/home/claude/n.ipynb".into(), edit: true })),
        (request("WebFetch", json!({"url": "https://example.com/", "prompt": "t"})), Some(ClaudeAction::Fetch { url: "https://example.com/".into() })),
        (request("WebFetch", json!({"url": "file:///etc/passwd"})), None),
        (request("WebSearch", json!({"query": "rust 2026"})), Some(ClaudeAction::Search { query: "rust 2026".into() })),
        (request("WebSearch", json!({"query": " "})), None),
        (request("Bash", json!({})), None),
        (request("CronCreate", json!({"cron": "*"})), None),
        (request("Task", json!({"prompt": "x"})), None),
    ];
    for (request, expected) in cases {
        assert_eq!(ClaudeAction::from_request(&request, cwd), expected, "{request:?}");
    }
}

fn report(answer: &str, cli: &str, result: TurnResult) -> DriveReport {
    DriveReport {
        answer: answer.to_string(),
        cli_text: cli.to_string(),
        result: Some(result),
        ..DriveReport::default()
    }
}

#[test]
fn a_spent_plan_keeps_the_limit_card_path() {
    let result = TurnResult { subtype: "success".into(), is_error: true, text: "You've hit your limit · resets 3pm".into(), ..TurnResult::default() };
    let error = interpret(report("", "You've hit your limit · resets 3pm", result)).map(|_| ()).unwrap_err();
    assert_eq!(error.error, "rate_limited");
    assert!(error.message.starts_with(crate::plan_limit::MARKER));
}

#[test]
fn a_good_turn_keeps_its_session_id_and_an_empty_one_fails() {
    let ok = TurnResult { subtype: "success".into(), session_id: Some("s-1".into()), ..TurnResult::default() };
    let mut good = report("Hello", "", ok.clone());
    good.session_id = Some("s-1".into());
    assert_eq!(interpret(good).expect("ok").session_id.as_deref(), Some("s-1"));
    assert_eq!(interpret(report("", "", ok)).map(|_| ()).unwrap_err().error, "internal_error");
    let failed = TurnResult { subtype: "error_max_turns".into(), is_error: true, ..TurnResult::default() };
    assert_eq!(interpret(report("partial", "", failed)).map(|_| ()).unwrap_err().message, "Claude Code failed to answer.");
}

#[test]
fn attachments_go_before_the_text_as_the_blocks_the_cli_passes_on() {
    use super::protocol::Attachment;
    let files = [
        Attachment::Image { media_type: "image/png".into(), data: "iVBO".into() },
        Attachment::Pdf { data: "JVBE".into() },
        Attachment::Text { path: "/opt/vibe/conversations/c1/data/notes.txt".into(), text: "hello".into() },
    ];
    let content = &protocol::user_message("Describe them.", &files)["message"]["content"];
    assert_eq!(
        content[0],
        json!({"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "iVBO"}})
    );
    assert_eq!(
        content[1],
        json!({"type": "document", "source": {"type": "base64", "media_type": "application/pdf", "data": "JVBE"}})
    );
    let quoted = content[2]["text"].as_str().unwrap_or_default();
    assert!(quoted.starts_with("--- BEGIN ATTACHMENT /opt/vibe/conversations/c1/data/notes.txt ---\nhello\n"));
    assert_eq!(content[3], json!({"type": "text", "text": "Describe them."}));
    assert_eq!(content.as_array().map(Vec::len), Some(4));
}
