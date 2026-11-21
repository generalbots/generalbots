//! `/api/ai-accounts` — per-user AI account status (#1654/#1656).
//!
//! Credentials never pass through this API: Claude stays in the agent VM and
//! ChatGPT tokens live in Vault. Connect/reconnect return an actionable error
//! until the provider-specific flows (#1652/#1653) are available.
//!
//! `handlers` carries the status/list surface, `claude` the Claude Code login
//! flows, `use_for` the Vibe/chat assignment and `shared` the common helpers.

mod antigravity;
mod claude;
mod codex;
mod handlers;
mod models;
mod shared;
mod use_for;

pub use antigravity::{antigravity_code, AntigravityCodeBody};
pub use claude::{claude_code, ClaudeCodeBody};
pub use codex::{codex_code, CodexCodeBody};
pub use handlers::{connect, disconnect, list, reconnect, status};
pub use models::{list_models, save_models, EnabledModelsBody};
pub use use_for::{use_for_chat, use_for_vibe, UseForVibeBody};

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use botcore::shared::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ai-accounts", get(list))
        .route("/api/ai-accounts/:provider/connect", post(connect))
        .route("/api/ai-accounts/:provider/reconnect", post(reconnect))
        .route("/api/ai-accounts/:provider/status", post(status))
        .route("/api/ai-accounts/:provider/disconnect", post(disconnect))
        .route(
            "/api/ai-accounts/:provider/models",
            get(list_models).put(save_models),
        )
        .route(
            "/api/ai-accounts/:provider/use-for-vibe",
            post(use_for_vibe),
        )
        .route(
            "/api/ai-accounts/:provider/use-for-chat",
            post(use_for_chat),
        )
        .route("/api/ai-accounts/claude-code/code", post(claude_code))
        .route("/api/ai-accounts/codex-cli/code", post(codex_code))
        .route("/api/ai-accounts/antigravity/code", post(antigravity_code))
}
