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
