use super::incus::run_incus;
use super::internal;
use crate::AgentService;
use axum::http::StatusCode;
use serde_json::{json, Value};

/// Only return resource counters for the VM resolved from the authenticated user.
pub async fn resource_summary(service: &AgentService, name: &str) -> Result<Value, (StatusCode, String)> {
    let out = run_incus(service, vec!["list".into(), name.into(), "--format=json".into()]).await?;
    if !out.status.success() {
        return Err(internal("workspace metrics unavailable".into()));
    }
    let instances: Vec<Value> = serde_json::from_slice(&out.stdout)
        .map_err(|_| internal("invalid workspace metrics".into()))?;
    let vm = instances.iter().find(|item| item["name"].as_str() == Some(name))
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workspace unavailable".into()))?;
    let state = &vm["state"];
    Ok(json!({
        "status": vm["status"],
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
