//! HTTP handlers for agent sessions, per-user VMs, and the sandbox.
//!
//! JWT handling: claims are read with a dependency-free base64url split of the
//! token payload (`b64_json_sub`). Signature verification is expected to be
//! performed by the upstream gateway/integrator.

mod auth;
mod files;
mod sandbox;
mod sessions;
mod vm;

use axum::http::StatusCode;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::state::RateLimiter;

pub use auth::{authorize_session_access, b64_json_sub, bearer_token, is_admin_claims, jwt_payload};
pub use files::{vm_file, vm_files};
pub use sandbox::sandbox_exec;
pub use sessions::{current_session, set_agent_mode};
pub use vm::{switch_conversation, user_vm_status, vm_exec};

static MODE_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);
static EXEC_LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::new);

/// Resolves a non-JWT suite token (`gb_<uuid>_<ts>`) to a user id. The server
/// wires this at boot because the session store lives outside this crate.
static EXTERNAL_TOKEN_VERIFIER: LazyLock<
    std::sync::Mutex<Option<fn(&str) -> Option<Uuid>>>,
> = LazyLock::new(|| std::sync::Mutex::new(None));

pub fn set_external_token_verifier(resolver: fn(&str) -> Option<Uuid>) {
    if let Ok(mut slot) = EXTERNAL_TOKEN_VERIFIER.lock() {
        *slot = Some(resolver);
    }
}

fn external_token_user(raw: &str) -> Option<Uuid> {
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();
    let slot = EXTERNAL_TOKEN_VERIFIER.lock().ok()?;
    let resolver = (*slot)?;
    resolver(token)
}

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Agent request failed".to_string())
}

fn unauthorized(msg: &str) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.to_string())
}
