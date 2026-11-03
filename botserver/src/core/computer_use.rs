//! Computer-use action registry (GB6-19/GB6-20).
//!
//! The assistant proposes a browser or terminal action; the suite polls the
//! pending queue, asks the user for consent, executes it and posts the result
//! back. Actions are short-lived state, so an in-memory registry is the right
//! scope: nothing here is worth surviving a restart, and the execution itself
//! is audited through the response the caller receives.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

/// How long an action stays queryable before it is dropped.
const TTL_SECS: u64 = 30 * 60;

#[derive(Debug, Clone)]
pub struct ComputerAction {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: Uuid,
    pub kind: String,
    pub action: Value,
    pub status: String,
    pub result: Option<Value>,
    pub created_at: u64,
    /// #1644/#1645 — approval is a server-side record, not a flag the client
    /// can assert. Set by `approve`; checked by every executor.
    pub approved_at: Option<u64>,
    /// #1644 — the next turn has already been told about this outcome.
    pub reported: bool,
}

impl ComputerAction {
    pub fn to_json(&self) -> Value {
        json!({
            "action_id": self.id,
            "session_id": self.session_id,
            "user_id": self.user_id,
            "kind": self.kind,
            "action": self.action,
            "status": self.status,
            "result": self.result,
            "created_at": self.created_at,
            "approved": self.approved_at.is_some(),
        })
    }
}

static REGISTRY: OnceLock<Mutex<HashMap<Uuid, ComputerAction>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<Uuid, ComputerAction>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn prune(map: &mut HashMap<Uuid, ComputerAction>) {
    let cutoff = now().saturating_sub(TTL_SECS);
    map.retain(|_, action| action.created_at >= cutoff);
}

/// Registers a proposed action for a conversation.
pub async fn create(session_id: &str, user_id: Uuid, kind: &str, action: Value) -> ComputerAction {
    let entry = ComputerAction {
        id: Uuid::new_v4(),
        session_id: session_id.to_string(),
        user_id,
        kind: kind.to_string(),
        action,
        status: "pending".to_string(),
        result: None,
        created_at: now(),
        approved_at: None,
        reported: false,
    };
    let mut map = registry().lock().await;
    prune(&mut map);
    map.insert(entry.id, entry.clone());
    entry
}

/// Actions the suite has not collected yet. Collecting marks them delivered so
/// a reconnecting tab does not render the same consent card twice.
pub async fn pending_for(session_id: &str, user_id: Uuid, kind: &str) -> Vec<ComputerAction> {
    let mut map = registry().lock().await;
    prune(&mut map);
    let mut collected = Vec::new();
    for action in map.values_mut() {
        if action.session_id != session_id || action.user_id != user_id || action.kind != kind {
            continue;
        }
        if action.status == "pending" {
            action.status = "delivered".to_string();
            collected.push(action.clone());
        }
    }
    collected.sort_by_key(|action| action.created_at);
    collected
}

/// Stores the outcome reported by the suite.
pub async fn complete(id: Uuid, user_id: Uuid, result: Value) -> Option<ComputerAction> {
    let mut map = registry().lock().await;
    let entry = map.get_mut(&id)?;
    if entry.user_id != user_id {
        return None;
    }
    let status = result
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();
    entry.status = if status == "denied" { "denied" } else { "completed" }.to_string();
    entry.result = Some(result);
    Some(entry.clone())
}

pub async fn get(id: Uuid, user_id: Uuid) -> Option<ComputerAction> {
    let mut map = registry().lock().await;
    prune(&mut map);
    map.get(&id)
        .filter(|action| action.user_id == user_id)
        .cloned()
}

/// Records a terminal execution so the assistant can read the outcome by id.
pub async fn record_terminal(
    session_id: &str,
    user_id: Uuid,
    action: Value,
    result: Value,
) -> ComputerAction {
    let entry = create(session_id, user_id, "terminal_exec", action).await;
    complete(entry.id, user_id, result).await.unwrap_or(entry)
}

/// Records the user's approval for one action. Executors require this record;
/// `approved=true` in a request body is no longer sufficient (#1644/#1645).
pub async fn approve(id: Uuid, user_id: Uuid) -> Option<ComputerAction> {
    let mut map = registry().lock().await;
    let entry = map.get_mut(&id)?;
    if entry.user_id != user_id || entry.status == "denied" {
        return None;
    }
    entry.approved_at = Some(now());
    entry.status = "approved".to_string();
    Some(entry.clone())
}

pub async fn is_approved(id: Uuid, user_id: Uuid) -> bool {
    let map = registry().lock().await;
    map.get(&id)
        .filter(|action| action.user_id == user_id)
        .is_some_and(|action| action.approved_at.is_some())
}

/// Completed outcomes not yet shown to the model, oldest first. Marks them
/// reported so one result never leaks into two turns (#1644).
pub async fn take_completed_for_prompt(
    session_id: &str,
    user_id: Uuid,
) -> Vec<ComputerAction> {
    let mut map = registry().lock().await;
    prune(&mut map);
    let mut collected = Vec::new();
    for action in map.values_mut() {
        if action.session_id != session_id || action.user_id != user_id {
            continue;
        }
        if action.reported || action.result.is_none() {
            continue;
        }
        action.reported = true;
        collected.push(action.clone());
    }
    collected.sort_by_key(|action| action.created_at);
    collected
}

/// Turns an explicit "open/read <url>" request into a browser proposal. This
/// is the producer the desktop build lacked: the assistant can act on a URL
/// without the model having to know an HTTP endpoint.
pub fn propose_from_text(text: &str) -> Option<Value> {
    let lower = text.to_lowercase();
    let intent = [
        "open ", "abrir ", "read ", "leia ", "extract ", "extraia ", "visit ", "aceda ",
        "navigate to ", "acesse ",
    ]
    .iter()
    .any(|verb| lower.contains(verb));
    if !intent {
        return None;
    }
    let url = text
        .split_whitespace()
        .find(|word| word.starts_with("http://") || word.starts_with("https://"))?
        .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ')' | ']' | '>'))
        .to_string();
    Some(json!({
        "action": "open_url",
        "url": url,
        "extract": "title_and_text",
    }))
}

/// Turns an explicit "run/execute <command>" request into a terminal
/// proposal (#1645). A backticked command wins; otherwise the text after the
/// verb is used. The consent card still gates execution.
pub fn propose_terminal_from_text(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    let prefixes = [
        "run the command ",
        "execute the command ",
        "run ",
        "execute ",
        "rode ",
        "roda ",
        "executar ",
        "executa ",
    ];
    let rest = prefixes.iter().find_map(|prefix| {
        lower
            .starts_with(prefix)
            .then(|| trimmed.chars().skip(prefix.chars().count()).collect::<String>())
    })?;
    let rest = rest.trim();
    let command = if let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        after
            .find('`')
            .map(|end| after[..end].trim().to_string())?
    } else {
        rest.trim_matches(|c: char| c == '"' || c == '\'' || c == '.' || c == ';')
            .trim()
            .to_string()
    };
    if command.is_empty() || command.len() > 16_000 {
        return None;
    }
    Some(json!({
        "command": command,
        "cwd": "/opt/vibe/app",
        "reason": "requested in chat",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> Uuid {
        Uuid::new_v4()
    }

    /// The suite collects each proposed action once, and only for the user and
    /// conversation it belongs to.
    #[tokio::test]
    async fn pending_is_scoped_and_delivered_once() {
        let owner = user();
        let session = format!("s-{}", Uuid::new_v4());
        let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
        let other = user();

        assert!(pending_for(&session, other, "browser").await.is_empty());
        assert!(pending_for(&session, owner, "terminal").await.is_empty());

        let collected = pending_for(&session, owner, "browser").await;
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].id, action.id);
        assert!(
            pending_for(&session, owner, "browser").await.is_empty(),
            "a reconnecting tab must not receive the same consent card twice"
        );
    }

    #[tokio::test]
    async fn results_are_scoped_to_the_owner() {
        let owner = user();
        let session = format!("s-{}", Uuid::new_v4());
        let action = create(&session, owner, "browser", json!({"action": "extract_text"})).await;

        assert!(complete(action.id, user(), json!({"status": "ok"})).await.is_none());
        assert!(get(action.id, user()).await.is_none());

        let done = complete(action.id, owner, json!({"status": "ok", "text": "hello"}))
            .await
            .expect("the owner can record the outcome");
        assert_eq!(done.status, "completed");
        assert_eq!(done.result.unwrap()["text"], "hello");

        let denied = create(&session, owner, "browser", json!({"action": "click_element"})).await;
        let denied = complete(denied.id, owner, json!({"status": "denied"}))
            .await
            .expect("denial is recorded");
        assert_eq!(denied.status, "denied");
    }

    #[tokio::test]
    async fn terminal_outcome_is_queryable_by_id() {
        let owner = user();
        let session = format!("s-{}", Uuid::new_v4());
        let recorded = record_terminal(
            &session,
            owner,
            json!({"command": "echo hi"}),
            json!({"status": "ok", "stdout": "hi\n", "exit_code": 0}),
        )
        .await;
        assert_eq!(recorded.status, "completed");
        assert_eq!(recorded.kind, "terminal_exec");
        let found = get(recorded.id, owner).await.expect("recorded action is readable");
        assert_eq!(found.result.unwrap()["stdout"], "hi\n");
    }

    /// #1644 — a URL in an explicit "open/read" request becomes a browser
    /// proposal; ordinary text with no verb or no URL stays with the model.
    #[test]
    fn explicit_url_requests_become_proposals() {
        let proposal = propose_from_text("please open https://example.com/docs.")
            .expect("a URL with an open verb must propose an action");
        assert_eq!(proposal["action"], "open_url");
        assert_eq!(proposal["url"], "https://example.com/docs");
        assert!(propose_from_text("what is the weather?").is_none());
        assert!(propose_from_text("open the settings window").is_none());
    }

    #[tokio::test]
    async fn execution_requires_a_stored_approval() {
        let owner = user();
        let session = format!("s-{}", Uuid::new_v4());
        let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
        assert!(!is_approved(action.id, owner).await);
        assert!(approve(action.id, user()).await.is_none());
        let approved = approve(action.id, owner).await.expect("owner can approve");
        assert!(approved.approved_at.is_some());
        assert!(is_approved(action.id, owner).await);
    }

    #[tokio::test]
    async fn completed_outcomes_are_reported_once() {
        let owner = user();
        let session = format!("s-{}", Uuid::new_v4());
        let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
        complete(action.id, owner, json!({"status": "ok", "title": "Example"})).await;
        let first = take_completed_for_prompt(&session, owner).await;
        assert_eq!(first.len(), 1);
        assert!(
            take_completed_for_prompt(&session, owner).await.is_empty(),
            "the same outcome must not enter two turns"
        );
    }

    #[test]
    fn explicit_terminal_requests_become_proposals() {
        let proposal = propose_terminal_from_text("run `ls -la` in the workspace")
            .expect("a backticked command must propose a terminal action");
        assert_eq!(proposal["command"], "ls -la");
        let plain = propose_terminal_from_text("execute echo hello")
            .expect("a plain command after the verb is accepted");
        assert_eq!(plain["command"], "echo hello");
        assert!(propose_terminal_from_text("what is the weather?").is_none());
        assert!(propose_terminal_from_text("run ").is_none());
    }
}
