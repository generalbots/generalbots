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
    "The VM is switched off for this conversation. Turn on Agent (VM) in the chat controls to run commands or generate files.";

/// The single decision every VM/workspace route asks before dispatching a run
/// (#2239/#2264). `false` means the chat's Agent (VM) switch is off for this
/// turn: no workspace provisioning, no VM bridge call and no agent/workspace
/// run may start.
#[must_use]
pub fn workspace_route_allowed(agent_vm_enabled: bool) -> bool {
    agent_vm_enabled
}

/// Commands whose work happens inside the user's workspace VM (or that start
/// an agent run). The Agent (VM) switch gates them at the command edge, and
/// the VM bridge refuses them again centrally (`agent_vm_hook`).
#[must_use]
pub fn command_runs_in_workspace(name: &str, params: &serde_json::Value) -> bool {
    match name {
        "terminal.run" | "terminal.exec" | "terminal.session.open" | "vibe.project.change"
        | "tasks.autotask.create" => true,
        // `api.exec` reaches any registered endpoint; the workspace ones must
        // stay behind the switch even though the command name looks generic.
        "api.exec" => api_exec_targets_workspace(params),
        _ => crate::apps::commands::command_by_name(name)
            .map(|command| command.app == "terminal" || command.app == "vibe")
            .unwrap_or(false),
    }
}

/// Workspace/agent endpoints reachable through `api.exec`. Matching is by
/// prefix so a path param or trailing slash cannot slip past the gate.
fn api_exec_targets_workspace(params: &serde_json::Value) -> bool {
    const WORKSPACE_PATH_PREFIXES: [&str; 6] = [
        "/api/vibe/",
        "/api/terminal/",
        "/api/agent/",
        "/api/agents/",
        "/api/computer-use/",
        "/api/runs/",
    ];
    let path = params
        .get("path")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();
    WORKSPACE_PATH_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn defaults_to_enabled_when_absent() {
        let id = Uuid::new_v4();
        clear(id);
        assert!(is_enabled(id));
    }

    #[test]
    fn records_and_clears_turn_flag() {
        let id = Uuid::new_v4();
        set(id, false);
        assert!(!is_enabled(id));
        clear(id);
        assert!(is_enabled(id));
    }

    #[test]
    fn guard_restores_default_on_drop() {
        let id = Uuid::new_v4();
        clear(id);
        {
            let _guard = TurnGuard::new(id, false);
            assert!(!is_enabled(id));
        }
        assert!(is_enabled(id));
    }

    #[test]
    fn workspace_route_refused_when_agent_switch_is_off() {
        let id = Uuid::new_v4();
        let _guard = TurnGuard::new(id, false);
        let enabled = is_enabled(id);
        assert!(!workspace_route_allowed(enabled));
    }

    #[test]
    fn workspace_route_allowed_when_agent_switch_is_on() {
        let id = Uuid::new_v4();
        let _guard = TurnGuard::new(id, true);
        let enabled = is_enabled(id);
        assert!(workspace_route_allowed(enabled));
    }

    #[test]
    fn refusal_message_tells_the_user_how_to_enable_the_vm() {
        assert!(DISABLED_MESSAGE.contains("Agent (VM)"));
        assert!(!DISABLED_MESSAGE.is_empty());
    }

    #[test]
    fn workspace_commands_are_recognised() {
        let empty = serde_json::json!({});
        for name in [
            "terminal.run",
            "terminal.exec",
            "terminal.session.open",
            "vibe.project.change",
            "tasks.autotask.create",
        ] {
            assert!(
                command_runs_in_workspace(name, &empty),
                "{name} must require the Agent/VM switch"
            );
        }
    }

    #[test]
    fn api_exec_payloads_are_classified_by_path() {
        let workspace_run = serde_json::json!({
            "method": "POST",
            "path": "/api/vibe/run",
        });
        let terminal = serde_json::json!({
            "method": "POST",
            "path": "/api/terminal/exec",
        });
        let crm = serde_json::json!({
            "method": "GET",
            "path": "/api/crm/contacts",
        });
        assert!(command_runs_in_workspace("api.exec", &workspace_run));
        assert!(command_runs_in_workspace("api.exec", &terminal));
        assert!(!command_runs_in_workspace("api.exec", &crm));
    }
}
