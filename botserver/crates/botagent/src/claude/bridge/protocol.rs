//! The stream-json control protocol of `claude -p --input-format stream-json
//! --output-format stream-json --permission-prompt-tool stdio`, as Claude
//! Code 2.1.330 speaks it (recorded live, see the module docs of
//! [`super`]): the lines it writes, parsed into [`Incoming`], and the lines
//! the bridge writes back.

use serde_json::{json, Value};

/// Request id of the bridge's own `initialize` control request.
pub(super) const INIT_REQUEST: &str = "gb_init";
/// Request id of the bridge's `interrupt` control request (Stop).
pub(super) const INTERRUPT_REQUEST: &str = "gb_interrupt";

/// One tool use Claude Code asks permission for (`can_use_tool`).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolRequest {
    pub tool_name: String,
    /// The `tool_use` block this request is for; its result carries the same id.
    pub tool_use_id: Option<String>,
    /// The tool's arguments exactly as the model sent them.
    pub input: Value,
}

/// The end of a turn (`type: result`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurnResult {
    pub subtype: String,
    pub is_error: bool,
    /// The CLI's final text: the last answer, or its own error sentence.
    pub text: String,
    pub session_id: Option<String>,
    pub denials: usize,
}

/// What one line of the CLI means to the bridge.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Incoming {
    Init { session_id: String, cwd: Option<String> },
    /// Model text for the answer.
    Text(String),
    /// The model's thinking, for the reasoning card only.
    Thinking(String),
    /// Text the CLI wrote itself (`model: "<synthetic>"`): sign-in and plan
    /// messages, never streamed into the answer.
    CliText(String),
    ToolUse { id: String, name: String },
    ToolResult { tool_use_id: String, is_error: bool, text: String },
    CanUseTool { request_id: String, request: ToolRequest },
    /// Any other control request: answered with an error, so nothing the
    /// bridge does not understand is approved.
    OtherControl { request_id: String, subtype: String },
    Result(TurnResult),
    Ignored,
}

fn text_of(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_string()
}

/// Text of a `tool_result` content: a string or a list of text blocks.
fn result_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn blocks(message: &Value) -> &[Value] {
    message
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn assistant(message: &Value) -> Vec<Incoming> {
    let synthetic = message
        .get("message")
        .and_then(|m| m.get("model"))
        .and_then(Value::as_str)
        == Some("<synthetic>");
    let nested = message.get("parent_tool_use_id").is_some_and(|p| !p.is_null());
    let mut out = Vec::new();
    for block in blocks(message) {
        match block.get("type").and_then(Value::as_str) {
            Some("text") if !nested => {
                let text = text_of(block.get("text"));
                if text.trim().is_empty() {
                    continue;
                }
                out.push(if synthetic { Incoming::CliText(text) } else { Incoming::Text(text) });
            }
            Some("thinking") if !nested && !synthetic => {
                let text = text_of(block.get("thinking"));
                if !text.trim().is_empty() {
                    out.push(Incoming::Thinking(text));
                }
            }
            Some("tool_use") => out.push(Incoming::ToolUse {
                id: text_of(block.get("id")),
                name: text_of(block.get("name")),
            }),
            _ => {}
        }
    }
    out
}

fn tool_results(message: &Value) -> Vec<Incoming> {
    blocks(message)
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
        .map(|b| Incoming::ToolResult {
            tool_use_id: text_of(b.get("tool_use_id")),
            is_error: b.get("is_error").and_then(Value::as_bool).unwrap_or(false),
            text: result_text(b.get("content")),
        })
        .collect()
}

fn control_request(message: &Value) -> Incoming {
    let request_id = text_of(message.get("request_id"));
    let request = message.get("request").cloned().unwrap_or(Value::Null);
    let subtype = text_of(request.get("subtype"));
    if subtype == "can_use_tool" {
        return Incoming::CanUseTool {
            request_id,
            request: ToolRequest {
                tool_name: text_of(request.get("tool_name")),
                tool_use_id: request.get("tool_use_id").and_then(Value::as_str).map(str::to_string),
                input: request.get("input").cloned().unwrap_or(Value::Null),
            },
        };
    }
    Incoming::OtherControl { request_id, subtype }
}

fn result(message: &Value) -> Incoming {
    Incoming::Result(TurnResult {
        subtype: text_of(message.get("subtype")),
        is_error: message.get("is_error").and_then(Value::as_bool).unwrap_or(false),
        text: text_of(message.get("result")),
        session_id: message.get("session_id").and_then(Value::as_str).map(str::to_string),
        denials: message
            .get("permission_denials")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
    })
}

/// Parses one stdout line. A line that is not JSON, or of a type the
/// bridge does not use, is [`Incoming::Ignored`].
pub(super) fn parse(line: &str) -> Vec<Incoming> {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return vec![Incoming::Ignored];
    };
    match message.get("type").and_then(Value::as_str) {
        Some("system") if message.get("subtype").and_then(Value::as_str) == Some("init") => {
            vec![Incoming::Init {
                session_id: text_of(message.get("session_id")),
                cwd: message.get("cwd").and_then(Value::as_str).map(str::to_string),
            }]
        }
        Some("assistant") => assistant(&message),
        Some("user") => tool_results(&message),
        Some("control_request") => vec![control_request(&message)],
        Some("result") => vec![result(&message)],
        _ => vec![Incoming::Ignored],
    }
}

pub(super) fn initialize() -> Value {
    json!({"type": "control_request", "request_id": INIT_REQUEST,
           "request": {"subtype": "initialize", "hooks": null}})
}

/// A file the person attached, sent inside the user message so the model
/// sees it without opening anything in the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attachment {
    /// A picture, already shrunk to the API's per-image cap; `data` is base64.
    Image { media_type: String, data: String },
    /// A PDF; `data` is base64.
    Pdf { data: String },
    /// A text file whose text the message does not already quote.
    Text { path: String, text: String },
}

impl Attachment {
    fn block(&self) -> Value {
        match self {
            Self::Image { media_type, data } => {
                json!({"type": "image", "source": {"type": "base64", "media_type": media_type, "data": data}})
            }
            Self::Pdf { data } => {
                json!({"type": "document", "source": {"type": "base64", "media_type": "application/pdf", "data": data}})
            }
            Self::Text { path, text } => json!({"type": "text", "text": format!(
                "--- BEGIN ATTACHMENT {path} ---\n{text}\n--- END ATTACHMENT {path} ---"
            )}),
        }
    }
}

/// The user message: plain text, or the attachments' blocks then the text.
pub(super) fn user_message(text: &str, attachments: &[Attachment]) -> Value {
    let content = if attachments.is_empty() {
        Value::String(text.to_string())
    } else {
        let mut blocks: Vec<Value> = attachments.iter().map(Attachment::block).collect();
        blocks.push(json!({"type": "text", "text": text}));
        Value::Array(blocks)
    };
    json!({"type": "user", "message": {"role": "user", "content": content},
           "parent_tool_use_id": null, "session_id": ""})
}

/// Allows the tool use with its arguments unchanged.
pub(super) fn allow(request_id: &str, input: &Value) -> Value {
    json!({"type": "control_response", "response": {"subtype": "success", "request_id": request_id,
           "response": {"behavior": "allow", "updatedInput": input}}})
}

/// Denies the tool use; `message` becomes the tool result the model reads.
pub(super) fn deny(request_id: &str, message: &str) -> Value {
    json!({"type": "control_response", "response": {"subtype": "success", "request_id": request_id,
           "response": {"behavior": "deny", "message": message}}})
}

/// Refuses a control request the bridge does not handle.
pub(super) fn unsupported(request_id: &str) -> Value {
    json!({"type": "control_response", "response": {"subtype": "error", "request_id": request_id,
           "error": "not supported by this host"}})
}

pub(super) fn interrupt() -> Value {
    json!({"type": "control_request", "request_id": INTERRUPT_REQUEST,
           "request": {"subtype": "interrupt"}})
}
