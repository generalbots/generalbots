//! Codex CLI subscription provider (#2574).
//!
//! Mirrors the Claude flow: CLI/login helper runs inside the agent VM,
//! auth URL is surfaced to the UI, user pastes code back, credentials stay in VM.

mod login;
mod paths;
mod prompt;
mod pty;
mod status;

use axum::http::StatusCode;
use base64::Engine;

pub use login::{logout, start_login, submit_code};
pub use paths::{CODEX_HOME, CODEX_USER, CREDENTIALS_PATH};
pub use prompt::run_prompt;
pub use status::{extract_device_code, extract_device_url, status, CodexStatus};

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Codex account operation failed".to_string(),
    )
}

fn b64(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}
