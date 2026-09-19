//! #1500/#1504 — Process-wide hooks used by the Vibe reform.
//!
//! `botvibe` (project registry, publish pipeline) cannot depend on the main
//! botserver binary crate where the git monitor and the BasicCompiler live,
//! and the signup flow lives in `botcloud`. Instead of adding cross-crate
//! edges that would cycle, the binaries register closures here at boot and
//! lower-level crates invoke them defensively (a no-op when unset).
//!
//! Two hooks:
//! - `bot_project_ops` — "run-test" materializes the workspace into the
//!   `{bot}-test` runtime layout and recompiles the TEST twin bot;
//!   "deploy-prod" promotes the same into the production `{bot}` layout.
//!   Registered by the main botserver (git monitor module).
//! - `workspace_resync` — asks the git monitor to pull/refresh one project
//!   checkout right away (used by provisioning paths).

use std::sync::OnceLock;

/// Operation dispatched to the main binary for a bot-kind Vibe project.
pub type BotProjectOpsHook = std::sync::Arc<
    dyn Fn(&str, uuid::Uuid) -> Result<(), String> + Send + Sync,
>;

/// Called with the project's ALM repo slug after a workspace changed so the
/// git monitor can sync/compile immediately instead of waiting for its tick.
pub type WorkspaceResyncHook = std::sync::Arc<
    dyn Fn(&str) -> Result<(), String> + Send + Sync,
>;

/// Called with (branch_id, branch/bot name) after a branch gains its default
/// bot — creates the default-bot Vibe project + PROD/TEST bot rows (#1500).
pub type WorkspaceBootstrapHook =
    std::sync::Arc<dyn Fn(uuid::Uuid, String) -> Result<(), String> + Send + Sync>;

static BOT_PROJECT_OPS: OnceLock<BotProjectOpsHook> = OnceLock::new();
static WORKSPACE_RESYNC: OnceLock<WorkspaceResyncHook> = OnceLock::new();
static WORKSPACE_BOOTSTRAP: OnceLock<WorkspaceBootstrapHook> = OnceLock::new();

/// Register the bot project ops hook (main binary at boot). The first
/// registration wins; a second attempt is logged and ignored.
pub fn register_bot_project_ops_hook(hook: BotProjectOpsHook) {
    match BOT_PROJECT_OPS.set(hook) {
        Ok(()) => log::info!("hooks: bot_project_ops registered"),
        Err(_) => log::warn!("hooks: bot_project_ops already registered — ignored"),
    }
}

/// Register the workspace resync hook (main binary at boot).
pub fn register_workspace_resync_hook(hook: WorkspaceResyncHook) {
    match WORKSPACE_RESYNC.set(hook) {
        Ok(()) => log::info!("hooks: workspace_resync registered"),
        Err(_) => log::warn!("hooks: workspace_resync already registered — ignored"),
    }
}

/// Dispatch a bot project operation ("run-test" | "deploy-prod"). Returns a
/// descriptive error when no hook is registered (e.g. server built without
/// the `vibe` feature) so callers surface it to the user/agent.
pub fn call_bot_project_ops(op: &str, project_id: uuid::Uuid) -> Result<(), String> {
    match BOT_PROJECT_OPS.get() {
        Some(hook) => hook(op, project_id),
        None => Err(format!(
            "bot project op '{op}' unavailable: git monitor hook not registered (server built without the vibe feature?)"
        )),
    }
}

/// Ask the git monitor to resync a workspace by repo slug (best effort).
pub fn call_workspace_resync(repo_slug: &str) -> Result<(), String> {
    match WORKSPACE_RESYNC.get() {
        Some(hook) => hook(repo_slug),
        None => Ok(()), // no monitor in this build — nothing to resync
    }
}

/// Register the branch-default-project bootstrap hook (main binary at boot).
pub fn register_workspace_bootstrap_hook(hook: WorkspaceBootstrapHook) {
    match WORKSPACE_BOOTSTRAP.set(hook) {
        Ok(()) => log::info!("hooks: workspace_bootstrap registered"),
        Err(_) => log::warn!("hooks: workspace_bootstrap already registered — ignored"),
    }
}

/// Take (consume for this call) the bootstrap hook; `None` when this build
/// has no vibe wiring (signup then simply skips the Vibe bootstrap).
pub fn take_workspace_bootstrap_hook() -> Option<WorkspaceBootstrapHook> {
    WORKSPACE_BOOTSTRAP.get().cloned()
}
