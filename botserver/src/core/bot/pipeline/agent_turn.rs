//! Per-turn agent/VM enable flag (#2239).
//!
//! The chat WS payload carries `agent_enabled` for each turn. Pipeline stages
//! and native tools run inside that turn need to know whether the workspace
//! may be provisioned or invoked. The flag is scoped to the session for the
//! duration of `process_message_internal` and defaults to enabled when unset
//! (non-chat paths, legacy callers).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use uuid::Uuid;

fn map() -> &'static Mutex<HashMap<Uuid, bool>> {
    static MAP: OnceLock<Mutex<HashMap<Uuid, bool>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records whether the agent/VM is enabled for the current turn.
pub fn set(session_id: Uuid, enabled: bool) {
    if let Ok(mut guard) = map().lock() {
        guard.insert(session_id, enabled);
    }
}

/// Clears the turn flag (no-op if absent).
pub fn clear(session_id: Uuid) {
    if let Ok(mut guard) = map().lock() {
        guard.remove(&session_id);
    }
}

/// Returns true when the agent/VM is allowed for this session turn.
///
/// Absent entries default to `true` so non-chat and legacy paths keep their
/// existing behavior.
pub fn is_enabled(session_id: Uuid) -> bool {
    map()
        .lock()
        .ok()
        .and_then(|guard| guard.get(&session_id).copied())
        .unwrap_or(true)
}

/// RAII guard that clears the per-turn flag on drop.
pub struct TurnGuard {
    session_id: Uuid,
}

impl TurnGuard {
    pub fn new(session_id: Uuid, enabled: bool) -> Self {
        set(session_id, enabled);
        Self { session_id }
    }
}

impl Drop for TurnGuard {
    fn drop(&mut self) {
        clear(self.session_id);
    }
}

pub const DISABLED_MESSAGE: &str =
    "Agent/VM is disabled for this conversation.";
