//! Antigravity subscription provider (#2576).
//!
//! Mirrors the Claude flow: CLI/login helper runs inside the agent VM,
//! auth URL is surfaced to the UI, user pastes code back, credentials stay in VM.
//! Chat turns run over ACP ([`acp::run_acp_turn`]); print mode
//! ([`run_prompt`]) answers only when the CLI does not speak ACP.

pub mod acp;
mod login;
mod paths;
mod prompt;
mod pty;
mod status;

use axum::http::StatusCode;
use base64::Engine;

pub use login::{logout, start_login, submit_code};
pub use paths::{credentials_test, ANTIGRAVITY_HOME, ANTIGRAVITY_USER, CREDENTIALS_DIRS};
pub use prompt::run_prompt;
pub use status::{extract_auth_code, extract_auth_url, status, AntigravityStatus};

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Antigravity account operation failed".to_string(),
    )
}

fn b64(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}
