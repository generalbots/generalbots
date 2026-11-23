//! Codex CLI subscription provider (#2574).
//!
//! Mirrors the Claude flow: CLI/login helper runs inside the agent VM,
//! auth URL is surfaced to the UI, user pastes code back, credentials stay in VM.
//! Chat turns run through the app-server bridge ([`run_bridged`]), where
//! every command and file change waits for the server's permission gate.

pub mod app_server;
pub mod bridge;
mod bridge_turn;
mod limits;
mod live_limits;
mod login;
mod paths;
mod prompt;
mod pty;
mod selfcheck;
mod status;
mod turn_loop;
mod unasked;

#[cfg(test)]
mod bridge_tests;
#[cfg(test)]
mod bridge_live_tests;

use axum::http::StatusCode;
use base64::Engine;

pub use limits::{limits, parse_rate_limits_line, CodexLimits, CodexWindow};
pub use live_limits::{live_limits, parse_snapshot};
pub use login::{logout, start_login, submit_code};
pub use paths::{CODEX_HOME, CODEX_USER, CREDENTIALS_PATH};
pub use bridge::{Approvals, BridgeError, Verdict};
pub use bridge_turn::{run_bridged, CodexTurn, BRIDGE_DEADLINE};
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
