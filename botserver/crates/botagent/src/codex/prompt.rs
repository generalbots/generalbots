//! Run a Codex prompt inside the VM (placeholder).

use axum::http::StatusCode;

use crate::AgentService;
use crate::vm;

use super::internal;

pub async fn run_prompt(
    _state: &AgentService,
    _vm_name: &str,
    _prompt: &str,
    _model: &str,
) -> Result<String, (StatusCode, String)> {
    Err(internal("Codex prompt execution not implemented yet".to_string()))
}
