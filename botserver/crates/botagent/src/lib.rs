//! #1167 / #1172 — Always-On Agent Mode backend.
//!
//! Per-chat Incus VMs (`agent_sessions`, `agent_snapshots`), org API keys
//! (`org_api_keys`) and the sandboxed code-execution service (`sandbox_runs`).
//! All Incus invocations go through `botlib::security::command_guard::SafeCommand`.

pub mod api;
pub mod keys;
pub mod models;
pub mod sandbox;
pub mod schema;
pub mod snapshots;
pub mod state;
pub mod vm;

use std::sync::Arc;

/// Shared service handle wired into axum routers by the integrator.
pub struct AgentService {
    pool: state::DbPool,
    incus_bin: String,
    max_snapshots_per_session: i64,
    idle_timeout_secs: i64,
}

impl AgentService {
    pub fn new(pool: state::DbPool) -> Self {
        let incus_bin = std::env::var("AGENT_INCUS_BIN").unwrap_or_else(|_| "incus".to_string());
        let max_snapshots_per_session = std::env::var("AGENT_MAX_SNAPSHOTS_PER_SESSION")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(10);
        let idle_timeout_secs = std::env::var("AGENT_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(3600);
        Self {
            pool,
            incus_bin,
            max_snapshots_per_session,
            idle_timeout_secs,
        }
    }

    pub fn pool(&self) -> &state::DbPool {
        &self.pool
    }

    pub fn incus_bin(&self) -> &str {
        &self.incus_bin
    }

    pub fn max_snapshots_per_session(&self) -> i64 {
        self.max_snapshots_per_session
    }

    pub fn idle_timeout_secs(&self) -> i64 {
        self.idle_timeout_secs
    }
}

pub fn configure_routes() -> axum::Router<Arc<AgentService>> {
    use axum::routing::{delete, get, post};
    axum::Router::new()
        .route("/api/agent/sessions/mode", post(api::set_agent_mode))
        .route("/api/agent/sessions/current", get(api::current_session))
        .route(
            "/api/agent/sessions/:id/snapshots",
            get(snapshots::list_snapshots).post(snapshots::create_snapshot),
        )
        .route("/api/agent/snapshots/:id/restore", post(snapshots::restore_snapshot))
        .route("/api/agent/snapshots/:id", delete(snapshots::delete_snapshot))
        .route("/api/v1/sandbox/exec", post(api::sandbox_exec))
        .route(
            "/api/cloud/orgkeys",
            get(keys::list_org_keys).post(keys::create_org_key),
        )
        .route("/api/cloud/orgkeys/:id", delete(keys::revoke_org_key))
}
