use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::{HeaderMap, StatusCode};
use botcore::shared::state::AppState;
use serde_json::Value;
use uuid::Uuid;

use super::audit::{audit_browser, audit_terminal, BrowserAuditRow, TerminalAuditRow};
use super::{api_error_key, resolve_user, ApiError};
use crate::core::computer_use;

/// POST /api/computer/actions/:id/result
pub async fn submit_result(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(result): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    // A denial is an auditable decision even though nothing ran.
    if result.get("status").and_then(Value::as_str) == Some("denied") {
        if let Some(action) = computer_use::get(id, user_id).await {
            if action.kind == "terminal" {
                let command = action
                    .action
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                audit_terminal(
                    &state.conn,
                    TerminalAuditRow {
                        user_id,
                        session_id: &action.session_id,
                        action_id: Some(id),
                        command,
                        cwd: action
                            .action
                            .get("cwd")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                        container: "",
                        decision: "denied",
                        exit_code: None,
                        output: "user denied consent",
                    },
                );
            } else if action.kind == "browser" {
                let action_name = action
                    .action
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or("browser");
                let target = action
                    .action
                    .get("url")
                    .and_then(Value::as_str)
                    .or_else(|| action.action.get("selector").and_then(Value::as_str))
                    .unwrap_or("");
                audit_browser(
                    &state.conn,
                    BrowserAuditRow {
                        user_id,
                        session_id: Some(&action.session_id),
                        action_id: Some(id),
                        target,
                        action: action_name,
                        decision: "denied",
                        result: Some("user denied consent"),
                        screenshot_path: None,
                    },
                );
            }
        }
    }
    let submitted = result.clone();
    match computer_use::complete(id, user_id, result).await {
        Some(action) => {
            if action.kind == "browser" {
                let action_name = action
                    .action
                    .get("action")
                    .and_then(Value::as_str)
                    .unwrap_or("browser");
                let target = action
                    .action
                    .get("url")
                    .and_then(Value::as_str)
                    .or_else(|| action.action.get("selector").and_then(Value::as_str))
                    .or_else(|| action.action.get("text").and_then(Value::as_str))
                    .unwrap_or("");
                let decision = submitted
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                let result_text = submitted
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| submitted.get("error").and_then(Value::as_str))
                    .or_else(|| submitted.get("title").and_then(Value::as_str));
                let screenshot_path = submitted.get("screenshot_path").and_then(Value::as_str);
                audit_browser(
                    &state.conn,
                    BrowserAuditRow {
                        user_id,
                        session_id: Some(&action.session_id),
                        action_id: Some(id),
                        target,
                        action: action_name,
                        decision,
                        result: result_text,
                        screenshot_path,
                    },
                );
            }
            Ok(Json(action.to_json()))
        }
        None => Err(api_error_key(
            &botcore::i18n::locale_from_headers(&headers),
            StatusCode::NOT_FOUND,
            "error-computer-use-action-not-found",
        )),
    }
}
