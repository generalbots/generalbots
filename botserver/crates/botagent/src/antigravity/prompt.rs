use axum::http::StatusCode;

use crate::AgentService;

use super::internal;

pub async fn run_prompt(
    _state: &AgentService,
    _vm_name: &str,
    _prompt: &str,
    _model: &str,
) -> Result<String, (StatusCode, String)> {
    Err(internal("Antigravity prompt execution not implemented yet".to_string()))
}
