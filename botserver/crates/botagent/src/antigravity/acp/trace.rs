//! What the ACP session did, for the server log: every tool call and its
//! updates, every permission request with its answer, and every other peer
//! request or notification. Only ids, kinds, statuses and the agent's own
//! short title are logged; never the tool's input or output.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde_json::{Map, Value};
use tokio::time::Instant;

use super::proto::Choice;

/// Longest title kept in a log line.
const TITLE_KEEP: usize = 80;
/// How long an acting tool call may wait without a permission request
/// before the log says so.
pub(super) const UNASKED_WAIT: Duration = Duration::from_secs(10);

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("-")
}

/// The agent's title of a tool call, shortened, on one line.
#[must_use]
pub(super) fn title_of(call: &Value) -> String {
    let title: String = text(call, "title")
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(TITLE_KEEP)
        .collect();
    title.trim().to_string()
}

pub(super) fn tool_update(call: &Value) {
    tracing::info!(
        "antigravity bridge: {} id={} kind={} status={} title=\"{}\"",
        text(call, "sessionUpdate"),
        text(call, "toolCallId"),
        text(call, "kind"),
        text(call, "status"),
        title_of(call)
    );
}

pub(super) fn permission_asked(params: &Value, actions: usize) {
    let call = params.get("toolCall").unwrap_or(&Value::Null);
    let options: Vec<&str> = params
        .get("options")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|option| option.get("kind").and_then(Value::as_str))
        .collect();
    tracing::info!(
        "antigravity bridge: session/request_permission id={} kind={} title=\"{}\" actions={} options=[{}]",
        text(call, "toolCallId"),
        text(call, "kind"),
        title_of(call),
        actions,
        options.join(",")
    );
}

pub(super) fn permission_answered(tool_id: &str, choice: Choice, since: Instant, delivered: bool) {
    tracing::info!(
        "antigravity bridge: permission answered id={tool_id} choice={choice:?} after {} ms delivered={delivered}",
        since.elapsed().as_millis()
    );
}

pub(super) fn request_refused(method: &str) {
    tracing::info!("antigravity bridge: peer request {method} refused (this client does not offer it)");
}

/// A notification other than `session/update`, logged once per method.
pub(super) fn other_notification(seen: &mut HashSet<String>, method: &str) {
    if seen.insert(method.to_string()) {
        tracing::info!("antigravity bridge: notification {method} ignored");
    }
}

/// The permission request's tool call completed with what the session's
/// earlier updates said about it: ACP sends a `ToolCallUpdate` there, whose
/// kind, title, input and locations may be left out.
#[must_use]
pub(super) fn with_known_call(params: &Value, known: Option<&Value>) -> Value {
    let Some(Value::Object(known)) = known else {
        return params.clone();
    };
    let mut params = params.clone();
    let Some(Value::Object(call)) = params.get_mut("toolCall") else {
        return params;
    };
    for key in ["kind", "title", "rawInput", "locations"] {
        let missing = call.get(key).map_or(true, Value::is_null);
        if missing {
            if let Some(value) = known.get(key).filter(|value| !value.is_null()) {
                call.insert(key.to_string(), value.clone());
            }
        }
    }
    params
}

/// Folds one tool-call update into what is known about that call.
pub(super) fn remember(calls: &mut HashMap<String, (Instant, Value)>, id: &str, update: &Value) {
    let entry = calls
        .entry(id.to_string())
        .or_insert_with(|| (Instant::now(), Value::Object(Map::new())));
    if let (Value::Object(known), Value::Object(update)) = (&mut entry.1, update) {
        for (key, value) in update {
            if !value.is_null() {
                known.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Acting tool calls still open that never asked, waiting longer than
/// [`UNASKED_WAIT`]; each is logged once.
pub(super) fn waiting_unasked(
    calls: &HashMap<String, (Instant, Value)>,
    asked: &HashSet<String>,
    warned: &mut HashSet<String>,
) {
    for (id, (since, call)) in calls {
        let open = !matches!(text(call, "status"), "completed" | "failed");
        let acts = super::proto::acts(text(call, "kind"));
        if open && acts && !asked.contains(id) && since.elapsed() > UNASKED_WAIT && warned.insert(id.clone()) {
            tracing::warn!(
                "antigravity bridge: tool call id={id} kind={} title=\"{}\" open {} s without a permission request",
                text(call, "kind"),
                title_of(call),
                since.elapsed().as_secs()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::antigravity::acp::proto::actions_of;
    use crate::antigravity::acp::AcpAction;

    #[test]
    fn a_bare_permission_request_is_completed_from_the_tool_call_update() {
        let mut calls = HashMap::new();
        remember(&mut calls, "t1", &json!({"toolCallId": "t1", "kind": "execute", "title": "uname -a",
            "rawInput": {"command": "uname -a"}, "status": "pending"}));
        remember(&mut calls, "t1", &json!({"toolCallId": "t1", "status": "in_progress", "kind": null}));
        let bare = json!({"toolCall": {"toolCallId": "t1"}, "options": []});
        let merged = with_known_call(&bare, calls.get("t1").map(|(_, call)| call));
        assert_eq!(actions_of(&merged), vec![AcpAction::Command { command: "uname -a".into(), cwd: None }]);
        assert_eq!(calls["t1"].1["status"], "in_progress");
        let own = json!({"toolCall": {"toolCallId": "t1", "kind": "read", "locations": [{"path": "/a"}]}});
        assert_eq!(with_known_call(&own, calls.get("t1").map(|(_, call)| call))["toolCall"]["kind"], "read");
        assert_eq!(with_known_call(&bare, None), bare);
    }

    #[test]
    fn titles_are_short_single_lines() {
        let long = "x".repeat(200);
        assert_eq!(title_of(&json!({"title": format!("a\nb {long}")})).chars().count(), TITLE_KEEP);
        assert!(!title_of(&json!({"title": "a\nb"})).contains('\n'));
        assert_eq!(title_of(&json!({})), "-");
    }
}
