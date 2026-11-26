//! The parts of the `codex app-server` protocol (v0.171) the bridge speaks:
//! the params it sends and readers for what the server sends back.
//!
//! The bridge never sends `acceptForSession`, an execpolicy amendment or a
//! network policy amendment: those store rules inside the CLI, and every
//! "allow this kind" grant lives in the server's gate instead.

use serde_json::{json, Value};

/// The approval policy that makes every command and file change ask.
pub const APPROVAL_POLICY: &str = "untrusted";
pub const SANDBOX: &str = "workspace-write";

pub const COMMAND_APPROVAL: &str = "item/commandExecution/requestApproval";
pub const FILE_APPROVAL: &str = "item/fileChange/requestApproval";

/// The only server requests the bridge takes; the transport refuses every
/// other one (permissions, user input, MCP elicitation, legacy approvals).
pub const HANDLED_REQUESTS: [&str; 2] = [COMMAND_APPROVAL, FILE_APPROVAL];

/// The answer to one approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Accept,
    Decline,
    /// Declines and ends the turn (no answer in time, turn stopped).
    Cancel,
}

impl Answer {
    #[must_use]
    pub fn result(self) -> Value {
        let decision = match self {
            Self::Accept => "accept",
            Self::Decline => "decline",
            Self::Cancel => "cancel",
        };
        json!({ "decision": decision })
    }
}

#[must_use]
pub fn initialize_params() -> Value {
    json!({
        "clientInfo": {
            "name": "gb_bridge",
            "title": "General Bots",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": { "experimentalApi": true },
    })
}

fn with_model(mut params: Value, model: &str) -> Value {
    if !model.is_empty() {
        params["model"] = Value::String(model.to_string());
    }
    params
}

#[must_use]
pub fn thread_start_params(cwd: &str, model: &str, ephemeral: bool) -> Value {
    let mut params = json!({
        "cwd": cwd,
        "approvalPolicy": APPROVAL_POLICY,
        "sandbox": SANDBOX,
        "config": super::unasked::thread_config(),
    });
    if ephemeral {
        params["ephemeral"] = Value::Bool(true);
    }
    with_model(params, model)
}

/// `instructions` is this turn's system prompt: a resumed thread receives
/// only the newest message, so a changed prompt reaches it this way.
#[must_use]
pub fn thread_resume_params(thread_id: &str, cwd: &str, model: &str, instructions: Option<&str>) -> Value {
    let mut params = json!({
        "threadId": thread_id,
        "cwd": cwd,
        "approvalPolicy": APPROVAL_POLICY,
        "sandbox": SANDBOX,
        "config": super::unasked::thread_config(),
    });
    if let Some(instructions) = instructions.map(str::trim).filter(|i| !i.is_empty()) {
        params["developerInstructions"] = Value::String(instructions.to_string());
    }
    with_model(params, model)
}

/// The turn repeats the approval policy, so a resumed thread whose stored
/// settings differ still asks for every action.
#[must_use]
pub fn turn_start_params(thread_id: &str, prompt: &str, model: &str) -> Value {
    turn_start_with_images(thread_id, prompt, &[], model)
}

/// [`turn_start_params`] with pictures the person attached, as `localImage`
/// inputs: absolute paths inside the VM that Codex reads and shows to the
/// model as images, not as text.
#[must_use]
pub fn turn_start_with_images(thread_id: &str, prompt: &str, images: &[String], model: &str) -> Value {
    let mut input = vec![json!({ "type": "text", "text": prompt })];
    input.extend(images.iter().map(|path| json!({ "type": "localImage", "path": path })));
    with_model(
        json!({
            "threadId": thread_id,
            "input": input,
            "approvalPolicy": APPROVAL_POLICY,
        }),
        model,
    )
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// The CLI version from the `initialize` result's user agent
/// (`gb_bridge/0.171.0 (Debian 13; x86_64) …`).
#[must_use]
pub fn version_of(initialized: &Value) -> String {
    text(initialized, "userAgent")
        .and_then(|agent| agent.split_once('/'))
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .filter(|version| version.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+')))
        .unwrap_or("unknown")
        .to_string()
}

/// `account/read`: no account while the CLI needs an OpenAI sign-in.
#[must_use]
pub fn signed_in(account: &Value) -> bool {
    let missing = account.get("account").map_or(true, Value::is_null);
    let needs_auth = account
        .get("requiresOpenaiAuth")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    !(missing && needs_auth)
}

#[must_use]
pub fn thread_id_of(result: &Value) -> Option<String> {
    result
        .get("thread")
        .and_then(|thread| text(thread, "id"))
        .filter(|id| safe_id(id))
        .map(str::to_string)
}

#[must_use]
pub fn turn_id_of(result: &Value) -> Option<String> {
    result
        .get("turn")
        .and_then(|turn| text(turn, "id"))
        .map(str::to_string)
}

/// Codex thread ids are UUIDs; anything else is not stored or sent.
#[must_use]
pub fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The command as the person reads it on the card: Codex wraps each one
/// as `/bin/bash -lc '<command>'`; the wrapper is dropped only when the
/// inner text holds no single quote, so both run exactly the same.
#[must_use]
pub fn shown_command(command: &str) -> &str {
    let command = command.trim();
    for wrapper in ["/bin/bash -lc '", "bash -lc '"] {
        if let Some(inner) = command.strip_prefix(wrapper).and_then(|rest| rest.strip_suffix('\'')) {
            if !inner.contains('\'') && !inner.trim().is_empty() {
                return inner;
            }
        }
    }
    command
}

/// One changed path of a `fileChange` item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub delete: bool,
}

/// The paths of a `fileChange` item (`changes: [{path, kind}]`, where
/// `kind` is `"add"|"delete"|"update"` or `{type: …}`).
#[must_use]
pub fn file_changes(item: &Value) -> Vec<FileChange> {
    let Some(changes) = item.get("changes").and_then(Value::as_array) else {
        return Vec::new();
    };
    changes
        .iter()
        .filter_map(|change| {
            let path = text(change, "path")?.trim();
            if path.is_empty() {
                return None;
            }
            let kind = change.get("kind");
            let kind = kind
                .and_then(Value::as_str)
                .or_else(|| kind.and_then(|k| text(k, "type")))
                .unwrap_or("update");
            Some(FileChange { path: path.to_string(), delete: kind.eq_ignore_ascii_case("delete") })
        })
        .collect()
}

/// How an approved action ended, for its step row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionOutcome {
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub output: String,
    pub files: Vec<String>,
}

/// Longest output kept for a step row (the row itself shows three lines).
const OUTPUT_KEEP: usize = 2_000;

impl ActionOutcome {
    /// From a finished `commandExecution` or `fileChange` item.
    #[must_use]
    pub fn of_item(item: &Value) -> Self {
        let status = text(item, "status").unwrap_or("");
        let exit_code = item
            .get("exitCode")
            .and_then(Value::as_i64)
            .and_then(|code| i32::try_from(code).ok());
        let output: String = text(item, "aggregatedOutput").unwrap_or("").chars().take(OUTPUT_KEEP).collect();
        let files = file_changes(item).into_iter().map(|change| change.path).collect();
        Self {
            ok: status == "completed" && exit_code.map_or(true, |code| code == 0),
            exit_code,
            output: if output.trim().is_empty() && status != "completed" {
                format!("Codex reported the action as {status}.")
            } else {
                output
            },
            files,
        }
    }

    /// An allowed path that did not run because another path of the same
    /// change was refused.
    #[must_use]
    pub fn not_run(path: &str) -> Self {
        Self {
            ok: false,
            exit_code: None,
            output: "Not written: another file in the same change was not allowed.".to_string(),
            files: vec![path.to_string()],
        }
    }
}

/// The reason a failed turn gives: the turn's error, else the last `error`
/// notification.
#[must_use]
pub fn turn_failure(turn: &Value, last_error: Option<&str>) -> String {
    let error = turn.get("error");
    let message = error.and_then(|e| text(e, "message")).unwrap_or("");
    let details = error.and_then(|e| text(e, "additionalDetails")).unwrap_or("");
    let info = error
        .and_then(|e| e.get("codexErrorInfo"))
        .map(|info| match info {
            Value::String(s) => s.clone(),
            other => other.as_object().and_then(|o| o.keys().next().cloned()).unwrap_or_default(),
        })
        .unwrap_or_default();
    let joined = [message, details, info.as_str()]
        .iter()
        .filter(|part| !part.trim().is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        last_error.unwrap_or("the Codex turn failed").to_string()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comes_from_the_user_agent() {
        let init = json!({"userAgent": "gb_bridge/0.171.0 (Debian 13.0.0; x86_64) xterm (gb_bridge; 1)"});
        assert_eq!(version_of(&init), "0.171.0");
        assert_eq!(version_of(&json!({})), "unknown");
        assert_eq!(version_of(&json!({"userAgent": "x/1;rm (y)"})), "unknown");
    }

    #[test]
    fn no_account_with_required_auth_is_not_signed_in() {
        assert!(!signed_in(&json!({"account": null, "requiresOpenaiAuth": true})));
        assert!(signed_in(&json!({"account": {"type": "chatgpt"}, "requiresOpenaiAuth": true})));
        assert!(signed_in(&json!({"account": null, "requiresOpenaiAuth": false})));
    }

    #[test]
    fn decisions_never_store_rules_in_the_cli() {
        for answer in [Answer::Accept, Answer::Decline, Answer::Cancel] {
            let text = answer.result().to_string();
            assert!(!text.contains("ForSession") && !text.contains("Amendment"));
        }
        assert_eq!(Answer::Accept.result(), json!({"decision": "accept"}));
    }

    #[test]
    fn every_turn_and_thread_asks() {
        assert_eq!(thread_start_params("/home/codex", "", false)["approvalPolicy"], "untrusted");
        assert_eq!(thread_start_params("/home/codex", "", false)["sandbox"], "workspace-write");
        let turn = turn_start_params("t1", "olá", "gpt-5-codex");
        assert_eq!(turn["approvalPolicy"], "untrusted");
        assert_eq!(turn["input"][0]["text"], "olá");
        assert_eq!(turn["model"], "gpt-5-codex");
        assert!(turn_start_params("t1", "x", "").get("model").is_none());
        let images = ["/opt/vibe/conversations/c/data/Tusks Poster.jpg".to_string()];
        let with = turn_start_with_images("t1", "veja", &images, "");
        assert_eq!(with["input"][0]["text"], "veja");
        assert_eq!(with["input"][1]["type"], "localImage");
        assert_eq!(with["input"][1]["path"], images[0].as_str());
        assert_eq!(turn_start_params("t1", "x", "")["input"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn file_change_paths_and_kinds() {
        let item = json!({"changes": [
            {"path": "/home/codex/a.txt", "kind": "add"},
            {"path": "/home/codex/b.txt", "kind": {"type": "delete"}},
            {"path": " ", "kind": "add"},
        ]});
        let changes = file_changes(&item);
        assert_eq!(changes.len(), 2);
        assert!(!changes[0].delete && changes[1].delete);
    }

    #[test]
    fn a_finished_command_keeps_its_exit_code_and_output() {
        let done = json!({"status": "completed", "exitCode": 0, "aggregatedOutput": "açúcar\n"});
        let outcome = ActionOutcome::of_item(&done);
        assert!(outcome.ok);
        assert_eq!(outcome.output, "açúcar\n");
        let failed = ActionOutcome::of_item(&json!({"status": "failed", "exitCode": 2}));
        assert!(!failed.ok);
        assert_eq!(failed.exit_code, Some(2));
    }

    #[test]
    fn a_failed_turn_says_why() {
        let turn = json!({"status": "failed", "error": {"message": "You've hit your usage limit.", "codexErrorInfo": "usageLimitExceeded"}});
        assert_eq!(turn_failure(&turn, None), "You've hit your usage limit. usageLimitExceeded");
        assert_eq!(turn_failure(&json!({"error": null}), Some("401")), "401");
    }

    #[test]
    fn the_card_shows_the_command_without_its_shell_wrapper() {
        assert_eq!(shown_command("/bin/bash -lc 'touch a && echo b'"), "touch a && echo b");
        let quoted = r"/bin/bash -lc 'echo '\''x'\'''";
        assert_eq!(shown_command(quoted), quoted);
        assert_eq!(shown_command(" ls -la "), "ls -la");
    }

    #[test]
    fn only_uuid_like_thread_ids_are_kept() {
        assert!(safe_id("01a20425-ca18-7971-8f4f-6e9edf5bcf43"));
        assert!(!safe_id("a b") && !safe_id("../x") && !safe_id(""));
    }
}
