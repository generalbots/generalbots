//! The Agent Client Protocol messages the Antigravity bridge sends and reads
//! (`agy acp`, protocol version 1): builders for the requests and parsers
//! for the answers, updates and permission requests. No I/O here.

use serde_json::{json, Value};

use crate::codex::app_server::ActionOutcome;

pub(super) const PROTOCOL_VERSION: u64 = 1;
pub(super) const PERMISSION_REQUEST: &str = "session/request_permission";
/// The only peer requests the bridge answers; any other is refused.
pub(super) const HANDLED_REQUESTS: [&str; 1] = [PERMISSION_REQUEST];
/// Session modes in which the agent acts without asking first.
const AUTO_MODES: [&str; 5] = ["auto", "accept-edits", "acceptedits", "bypasspermissions", "yolo"];
/// Longest action output kept for a step row.
const OUTPUT_KEEP: usize = 2_000;

/// What `initialize` says the agent can do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentInfo {
    pub version: String,
    pub protocol: u64,
    pub load_session: bool,
    pub resume_session: bool,
    pub images: bool,
    pub embedded_context: bool,
}

fn flag(value: &Value, camel: &str, snake: &str) -> bool {
    value.get(camel).or_else(|| value.get(snake)).and_then(Value::as_bool).unwrap_or(false)
}

fn section<'a>(value: &'a Value, camel: &str, snake: &str) -> Option<&'a Value> {
    value.get(camel).or_else(|| value.get(snake)).filter(|inner| inner.is_object())
}

#[must_use]
pub(super) fn initialize_params() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "clientCapabilities": {"fs": {"readTextFile": false, "writeTextFile": false}, "terminal": false},
        "clientInfo": {"name": "gb_bridge", "version": env!("CARGO_PKG_VERSION")},
    })
}

#[must_use]
pub fn agent_info(init: &Value) -> AgentInfo {
    let caps = init.get("agentCapabilities").cloned().unwrap_or(Value::Null);
    let prompt = section(&caps, "promptCapabilities", "prompt_capabilities").cloned().unwrap_or(Value::Null);
    let sessions = section(&caps, "sessionCapabilities", "session_capabilities").cloned().unwrap_or(Value::Null);
    let version = init
        .get("agentInfo")
        .and_then(|info| info.get("version"))
        .and_then(Value::as_str)
        .filter(|version| version.len() <= 40 && version.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+')))
        .unwrap_or("unknown")
        .to_string();
    AgentInfo {
        version,
        protocol: init.get("protocolVersion").and_then(Value::as_u64).unwrap_or(0),
        load_session: flag(&caps, "loadSession", "load_session") || sessions.get("load").is_some(),
        resume_session: sessions.get("resume").is_some(),
        images: flag(&prompt, "image", "image"),
        embedded_context: flag(&prompt, "embeddedContext", "embedded_context"),
    }
}

/// A session id the bridge stores and passes on: letters, digits, `-`, `_`.
#[must_use]
pub fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

#[must_use]
pub(super) fn session_new_params(cwd: &str) -> Value {
    json!({"cwd": cwd, "mcpServers": []})
}

#[must_use]
pub(super) fn session_open_params(session_id: &str, cwd: &str) -> Value {
    json!({"sessionId": session_id, "cwd": cwd, "mcpServers": []})
}

#[must_use]
pub(super) fn session_id_of(result: &Value) -> Option<String> {
    result.get("sessionId").and_then(Value::as_str).filter(|id| safe_id(id)).map(str::to_string)
}

/// The session's mode when it is one that acts without asking.
#[must_use]
pub(super) fn auto_mode(result: &Value) -> Option<String> {
    let current = result.get("modes")?.get("currentModeId")?.as_str()?;
    AUTO_MODES.contains(&current.to_ascii_lowercase().as_str()).then(|| current.to_string())
}

/// An available mode that asks before acting, to switch an auto session to.
#[must_use]
pub(super) fn asking_mode(result: &Value) -> Option<String> {
    result
        .get("modes")?
        .get("availableModes")?
        .as_array()?
        .iter()
        .filter_map(|mode| mode.get("id").and_then(Value::as_str))
        .find(|id| !AUTO_MODES.contains(&id.to_ascii_lowercase().as_str()) && !id.eq_ignore_ascii_case("plan"))
        .map(str::to_string)
}

/// A picture for the prompt, already sized for the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImagePart {
    pub mime_type: String,
    /// Base64 of the picture.
    pub data: String,
}

/// A text file given whole to the agent as an embedded resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextResource {
    /// Absolute path inside the workspace.
    pub path: String,
    pub mime_type: String,
    pub text: String,
}

/// `session/prompt`: the text, then pictures and text files when the agent
/// accepts them.
#[must_use]
pub(super) fn prompt_params(
    session_id: &str,
    text: &str,
    images: &[ImagePart],
    resources: &[TextResource],
    info: &AgentInfo,
) -> Value {
    let mut prompt = vec![json!({"type": "text", "text": text})];
    if info.images {
        prompt.extend(
            images.iter().map(|image| json!({"type": "image", "mimeType": image.mime_type, "data": image.data})),
        );
    }
    if info.embedded_context {
        prompt.extend(resources.iter().map(|file| {
            json!({"type": "resource", "resource": {
                "uri": format!("file://{}", file.path),
                "mimeType": file.mime_type,
                "text": file.text,
            }})
        }));
    }
    json!({"sessionId": session_id, "prompt": prompt})
}

/// One `session/update`, reduced to what the bridge uses.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Update {
    Text(String),
    Thought(String),
    /// A tool call appeared or changed: id, kind, status, and the call.
    Tool { id: String, kind: Option<String>, status: Option<String>, call: Value },
    Other,
}

fn chunk_text(update: &Value) -> String {
    let content = update.get("content").cloned().unwrap_or(Value::Null);
    match content.get("type").and_then(Value::as_str) {
        Some("text") => content.get("text").and_then(Value::as_str).unwrap_or("").to_string(),
        _ => String::new(),
    }
}

#[must_use]
pub(super) fn update_of(params: &Value) -> Update {
    let update = params.get("update").cloned().unwrap_or(Value::Null);
    let text = |key: &str| update.get(key).and_then(Value::as_str).map(str::to_string);
    match update.get("sessionUpdate").and_then(Value::as_str) {
        Some("agent_message_chunk") => Update::Text(chunk_text(&update)),
        Some("agent_thought_chunk") => Update::Thought(chunk_text(&update)),
        Some("tool_call" | "tool_call_update") => Update::Tool {
            id: text("toolCallId").unwrap_or_default(),
            kind: text("kind"),
            status: text("status"),
            call: update.clone(),
        },
        _ => Update::Other,
    }
}

/// A tool kind that changes the workspace or runs something.
#[must_use]
pub(super) fn acts(kind: &str) -> bool {
    matches!(kind, "execute" | "edit" | "delete" | "move")
}

/// What one permission request asks to do, as the gate sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpAction {
    Command { command: String, cwd: Option<String> },
    Read { path: String },
    Write { path: String },
    Delete { path: String },
    Fetch { url: String },
    /// Anything else, described by the agent's own title.
    Other { title: String },
}

fn first_text<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn paths_of(call: &Value) -> Vec<String> {
    let mut paths: Vec<String> = call
        .get("locations")
        .and_then(Value::as_array)
        .map(|locations| {
            locations
                .iter()
                .filter_map(|location| location.get("path").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if paths.is_empty() {
        let raw = call.get("rawInput").cloned().unwrap_or(Value::Null);
        if let Some(path) = first_text(&raw, &["path", "file_path", "filePath", "AbsolutePath", "TargetFile"]) {
            paths.push(path.to_string());
        }
    }
    paths.dedup();
    paths
}

/// The actions of one `session/request_permission` (several paths of one
/// edit are asked one by one).
#[must_use]
pub fn actions_of(params: &Value) -> Vec<AcpAction> {
    let call = params.get("toolCall").cloned().unwrap_or(Value::Null);
    let raw = call.get("rawInput").cloned().unwrap_or(Value::Null);
    let title = first_text(&call, &["title"]).unwrap_or("").to_string();
    let other = || vec![AcpAction::Other { title: if title.is_empty() { "action".to_string() } else { title.clone() } }];
    let paths = paths_of(&call);
    match call.get("kind").and_then(Value::as_str).unwrap_or("other") {
        "execute" => {
            let command = first_text(&raw, &["command", "cmd", "CommandLine", "commandLine"])
                .map(str::to_string)
                .unwrap_or_else(|| title.clone());
            if command.is_empty() {
                return other();
            }
            let cwd = first_text(&raw, &["cwd", "Cwd", "workdir", "directory"]).map(str::to_string);
            vec![AcpAction::Command { command, cwd }]
        }
        "read" | "search" if !paths.is_empty() => paths.into_iter().map(|path| AcpAction::Read { path }).collect(),
        "edit" | "move" if !paths.is_empty() => paths.into_iter().map(|path| AcpAction::Write { path }).collect(),
        "delete" if !paths.is_empty() => paths.into_iter().map(|path| AcpAction::Delete { path }).collect(),
        "fetch" => match first_text(&raw, &["url", "Url", "uri"]) {
            Some(url) => vec![AcpAction::Fetch { url: url.to_string() }],
            None => other(),
        },
        _ => other(),
    }
}

/// The gate's decision on a permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Choice {
    Allow,
    Reject,
    Cancelled,
}

/// The answer to `session/request_permission`: the agent's own option of
/// the chosen kind, once (a lasting grant is the server's "Permitir tudo",
/// never the agent's), or `cancelled` when no option fits.
#[must_use]
pub(super) fn permission_result(params: &Value, choice: Choice) -> Value {
    let options = params.get("options").and_then(Value::as_array).cloned().unwrap_or_default();
    let find = |kind: &str| {
        options
            .iter()
            .find(|option| option.get("kind").and_then(Value::as_str) == Some(kind))
            .and_then(|option| option.get("optionId").and_then(Value::as_str))
            .map(str::to_string)
    };
    let picked = match choice {
        Choice::Allow => find("allow_once"),
        Choice::Reject => find("reject_once").or_else(|| find("reject_always")),
        Choice::Cancelled => None,
    };
    match picked {
        Some(option_id) => json!({"outcome": {"outcome": "selected", "optionId": option_id}}),
        None => json!({"outcome": {"outcome": "cancelled"}}),
    }
}

/// The tool call id a permission request is about.
#[must_use]
pub(super) fn permission_tool_id(params: &Value) -> String {
    params
        .get("toolCall")
        .and_then(|call| call.get("toolCallId"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// The result of a finished tool call, for its step row.
#[must_use]
pub(super) fn outcome_of(call: &Value, status: &str, files: Vec<String>) -> ActionOutcome {
    let mut output = String::new();
    for block in call.get("content").and_then(Value::as_array).into_iter().flatten() {
        if let Some(text) = block.get("content").and_then(|inner| inner.get("text")).and_then(Value::as_str) {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(text);
        }
    }
    let raw = call.get("rawOutput").cloned().unwrap_or(Value::Null);
    if output.is_empty() {
        if let Some(text) = first_text(&raw, &["output", "stdout"]) {
            output = text.to_string();
        }
    }
    let exit_code = ["exitCode", "exit_code"]
        .iter()
        .find_map(|key| raw.get(*key).and_then(Value::as_i64))
        .and_then(|code| i32::try_from(code).ok());
    let ok = status == "completed" && exit_code.map_or(true, |code| code == 0);
    ActionOutcome {
        ok,
        exit_code,
        output: output.chars().take(OUTPUT_KEEP).collect(),
        files: if ok { files } else { Vec::new() },
    }
}

/// Why the prompt ended (`session/prompt` result).
#[must_use]
pub(super) fn stop_reason(result: &Value) -> String {
    result.get("stopReason").and_then(Value::as_str).unwrap_or("end_turn").to_string()
}

/// The peer asked for sign-in (`Authentication required`).
#[must_use]
pub(super) fn needs_sign_in(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("authentication required") || lower.contains("not authenticated") || lower.contains("login required")
}
