use super::incus::run_incus;
use super::incus_run::{is_timeout, within_budget, INCUS_TIMEOUT_CODE, PROBE_DEADLINE};
use super::lifecycle::start_in_progress;
use super::STATUS_RUNNING;
use crate::AgentService;
use axum::http::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

/// What a status read can say about the workspace without starting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceState {
    Running,
    Stopped,
    Starting,
    /// The backend did not answer (`incus_timeout`) or failed (`incus_error`),
    /// or the container is gone (`workspace_missing`).
    Unavailable(&'static str),
}

impl WorkspaceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Unavailable(_) => "unavailable",
        }
    }

    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::Unavailable(code) => Some(code),
            _ => None,
        }
    }

    /// The JSON body a status endpoint answers with instead of blocking.
    pub fn to_json(&self) -> Value {
        match self.code() {
            Some(code) => json!({ "status": self.as_str(), "code": code }),
            None => json!({ "status": self.as_str() }),
        }
    }
}

/// The container's `incus list` entry within the probe budget.
/// `Err` carries the [`WorkspaceState`] to report instead.
async fn probe_instance(service: &AgentService, name: &str) -> Result<Value, WorkspaceState> {
    let args = vec!["list".to_string(), name.to_string(), "--format=json".to_string()];
    let output = match within_budget(PROBE_DEADLINE, run_incus(service, args)).await {
        None => return Err(WorkspaceState::Unavailable(INCUS_TIMEOUT_CODE)),
        Some(Err(error)) if is_timeout(&error) => {
            return Err(WorkspaceState::Unavailable(INCUS_TIMEOUT_CODE))
        }
        Some(Err((code, detail))) => {
            tracing::warn!("workspace probe for {name} failed ({code}): {detail}");
            return Err(WorkspaceState::Unavailable("incus_error"));
        }
        Some(Ok(output)) => output,
    };
    if !output.status.success() {
        return Err(WorkspaceState::Unavailable("incus_error"));
    }
    let instances: Vec<Value> = serde_json::from_slice(&output.stdout)
        .map_err(|_| WorkspaceState::Unavailable("incus_error"))?;
    instances
        .into_iter()
        .find(|item| item["name"].as_str() == Some(name))
        .ok_or(WorkspaceState::Unavailable("workspace_missing"))
}

fn state_of(instance: &Value) -> WorkspaceState {
    match instance["status"].as_str().map(str::to_ascii_lowercase).as_deref() {
        Some("running") => WorkspaceState::Running,
        Some("starting") => WorkspaceState::Starting,
        _ => WorkspaceState::Stopped,
    }
}

/// A start in progress answers "starting" without another Incus call; a
/// workspace the database already calls running is still probed, because a
/// chat turn holds the same lock for a moment while it checks it.
fn starting_without_probe(user_id: &Uuid, row_status: &str) -> bool {
    row_status != STATUS_RUNNING && start_in_progress(user_id)
}

/// The workspace state for a status read, answered within the probe budget.
/// Never starts the workspace.
pub async fn workspace_state(
    service: &AgentService,
    user_id: &Uuid,
    vm_name: &str,
    row_status: &str,
) -> WorkspaceState {
    if starting_without_probe(user_id, row_status) {
        return WorkspaceState::Starting;
    }
    match probe_instance(service, vm_name).await {
        Ok(instance) => state_of(&instance),
        Err(_) if start_in_progress(user_id) => WorkspaceState::Starting,
        Err(state) => state,
    }
}

/// Only return resource counters for the VM resolved from the authenticated
/// user. A workspace that is starting, stopped or not answering gets a JSON
/// state within the probe budget instead of a request that hangs.
pub async fn resource_summary(
    service: &AgentService,
    user_id: &Uuid,
    name: &str,
    row_status: &str,
) -> Result<Value, (StatusCode, String)> {
    if starting_without_probe(user_id, row_status) {
        return Ok(WorkspaceState::Starting.to_json());
    }
    let vm = match probe_instance(service, name).await {
        Ok(vm) => vm,
        Err(WorkspaceState::Unavailable("workspace_missing")) => {
            return Err((StatusCode::NOT_FOUND, "Workspace unavailable".into()))
        }
        Err(state) if start_in_progress(user_id) => {
            tracing::debug!("workspace {name} probe: {:?} while starting", state);
            return Ok(WorkspaceState::Starting.to_json());
        }
        Err(state) => return Ok(state.to_json()),
    };
    let status = vm["status"].as_str().map(str::to_ascii_lowercase);
    let state = &vm["state"];
    Ok(json!({
        "status": status,
        "cpu_ns": state.pointer("/cpu/usage"),
        "memory_bytes": state.pointer("/memory/usage"),
        "disk_bytes": state.pointer("/disk/root/usage"),
        "disk_total_bytes": state.pointer("/disk/root/total"),
        "processes": state["processes"],
        "cpu_limit": vm.pointer("/expanded_config/limits.cpu"),
        "memory_limit": vm.pointer("/expanded_config/limits.memory"),
        "disk_limit": vm.pointer("/expanded_devices/root/size"),
        "architecture": vm["architecture"],
        "image": vm.pointer("/config/image.description")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_serialise_to_the_values_the_interface_knows() {
        assert_eq!(WorkspaceState::Starting.to_json(), json!({ "status": "starting" }));
        assert_eq!(WorkspaceState::Stopped.to_json(), json!({ "status": "stopped" }));
        assert_eq!(
            WorkspaceState::Unavailable(INCUS_TIMEOUT_CODE).to_json(),
            json!({ "status": "unavailable", "code": "incus_timeout" })
        );
    }

    #[test]
    fn the_incus_status_maps_to_a_workspace_state() {
        assert_eq!(state_of(&json!({ "status": "Running" })), WorkspaceState::Running);
        assert_eq!(state_of(&json!({ "status": "Stopped" })), WorkspaceState::Stopped);
        assert_eq!(state_of(&json!({ "status": "Frozen" })), WorkspaceState::Stopped);
        assert_eq!(state_of(&json!({})), WorkspaceState::Stopped);
    }

    #[test]
    fn a_start_in_progress_skips_the_probe_unless_the_row_is_running() {
        let user = Uuid::new_v4();
        assert!(!starting_without_probe(&user, "stopped"));
        let lock = super::super::lifecycle::provision_lock(&user);
        let guard = lock.try_lock();
        assert!(guard.is_ok());
        assert!(starting_without_probe(&user, "stopped"));
        assert!(starting_without_probe(&user, "provisioning"));
        assert!(!starting_without_probe(&user, STATUS_RUNNING));
    }
}
