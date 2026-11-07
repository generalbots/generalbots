//! Claude Code subscription provider (#1652).
//!
//! The CLI and its credentials live inside the user's agent VM:
//! `claude setup-token` runs under the `claude` Unix user, prints an OAuth
//! URL, and accepts the authorization code through a small PTY bridge. The
//! botserver never sees the token itself — it only reads the status of
//! `/home/claude/.claude/.credentials.json`.
//!
//! Every VM command goes through `vm::exec_in_vm`, which uses `SafeCommand`.
//! User-supplied text (the authorization code and the prompt) is base64
//! encoded before it reaches a shell.

mod login;
mod paths;
mod prompt;
mod pty;
mod status;

use axum::http::StatusCode;
use base64::Engine;

pub use login::{logout, start_login, submit_code};
pub use paths::{CLAUDE_HOME, CLAUDE_USER, CREDENTIALS_PATH};
pub use prompt::run_prompt;
pub use status::{extract_oauth_url, status, ClaudeStatus};

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Claude account operation failed".to_string(),
    )
}

fn b64(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}
