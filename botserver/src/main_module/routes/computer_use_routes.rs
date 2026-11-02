//! Computer-use HTTP surface (GB6-19 / GB6-20).
//!
//! `POST /api/computer/actions` lets the assistant (or a tool) propose a
//! browser or terminal action for a conversation. The suite polls
//! `/api/computer/actions/pending`, renders the consent card, executes the
//! action and posts the outcome to `/api/computer/actions/:id/result`.
//!
//! `POST /api/terminal/exec` is the server-side executor for terminal actions:
//! it refuses to run without an explicit approval flag, refuses destructive
//! patterns, and always executes inside the user's own agent container — never
//! on the bot host.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Json, Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use botcore::shared::state::AppState;
use botlib::security::command_guard::SafeCommand;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::computer_use;

type ApiError = (StatusCode, Json<Value>);

fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn resolve_user(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _roles) =
        crate::main_module::routes::anonymous_auth::resolve_token_user(token)
            .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid token"))?;
    Ok(crate::security::user_role::derive_stable_user_uuid(
        &claims_user,
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub session_id: String,
    /// `browser` or `terminal`.
    pub kind: String,
    pub action: Value,
}

/// POST /api/computer/actions
pub async fn create_action(
    headers: HeaderMap,
    Json(body): Json<CreateActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if body.session_id.trim().is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "session_id is required"));
    }
    if body.kind != "browser" && body.kind != "terminal" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "kind must be 'browser' or 'terminal'",
        ));
    }
    let action =
        computer_use::create(body.session_id.trim(), user_id, &body.kind, body.action).await;
    Ok(Json(action.to_json()))
}

#[derive(Debug, Deserialize)]
pub struct PendingQuery {
    pub session_id: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// GET /api/computer/actions/pending?session_id=&kind=
pub async fn pending(
    headers: HeaderMap,
    Query(query): Query<PendingQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let kind = query.kind.as_deref().unwrap_or("browser");
    let actions = computer_use::pending_for(query.session_id.trim(), user_id, kind).await;
    Ok(Json(json!({
        "actions": actions.iter().map(computer_use::ComputerAction::to_json).collect::<Vec<_>>(),
    })))
}

/// POST /api/computer/actions/:id/result
pub async fn submit_result(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(result): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    match computer_use::complete(id, user_id, result).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(StatusCode::NOT_FOUND, "action not found")),
    }
}

/// GET /api/computer/actions/:id
pub async fn get_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    match computer_use::get(id, user_id).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(StatusCode::NOT_FOUND, "action not found")),
    }
}

#[derive(Debug, Deserialize)]
pub struct TerminalExecRequest {
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Explicit user consent. A request without it is refused: nothing runs
    /// silently on the user's machine.
    #[serde(default)]
    pub approved: bool,
    #[serde(default)]
    pub session_id: Option<String>,
}

const DENIED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "dd if=",
    "shutdown",
    "reboot",
    ":(){",
    "> /dev/sda",
    "chmod -R 000 /",
];

/// Server-side re-check of destructive patterns. The consent card enforces the
/// same list client-side, but the server must not rely on it.
pub(crate) fn blocked_pattern(command: &str) -> Option<&'static str> {
    let lowered = command.to_ascii_lowercase();
    DENIED_PATTERNS
        .iter()
        .find(|pattern| lowered.contains(&pattern.to_ascii_lowercase()))
        .copied()
}

/// POST /api/terminal/exec
pub async fn terminal_exec(
    headers: HeaderMap,
    Json(body): Json<TerminalExecRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if !body.approved {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "terminal execution requires explicit user approval",
        ));
    }
    let command = body.command.trim();
    if command.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "command is required"));
    }
    if command.len() > 16_000 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "command is limited to 16000 characters",
        ));
    }
    if blocked_pattern(command).is_some() {
        log::warn!("terminal exec refused a blocked pattern for user {user_id}");
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "command matches a blocked destructive pattern",
        ));
    }
    #[cfg(feature = "agent-vm")]
    let container = crate::core::bot::agent_vm_hook::user_vm_name(user_id).await;
    #[cfg(not(feature = "agent-vm"))]
    let container: Option<String> = None;
    let container = container.ok_or_else(|| {
        api_error(
            StatusCode::FORBIDDEN,
            "no workspace container: send a message in chat first",
        )
    })?;

    let cwd = body
        .cwd
        .clone()
        .filter(|value| value.starts_with('/') && !value.contains(".."))
        .unwrap_or_else(|| "/opt/vibe/app".to_string());
    let timeout = Duration::from_millis(body.timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000));
    let command_owned = command.to_string();
    let container_for_exec = container.clone();
    let cwd_for_exec = cwd.clone();

    let execution = tokio::task::spawn_blocking(move || {
        let guarded = SafeCommand::new("incus")
            .and_then(|cmd| cmd.arg("exec"))
            .and_then(|cmd| cmd.arg(&container_for_exec))
            .and_then(|cmd| cmd.arg("--cwd"))
            .and_then(|cmd| cmd.arg(&cwd_for_exec))
            .and_then(|cmd| cmd.arg("--"))
            .and_then(|cmd| cmd.arg("sh"))
            .and_then(|cmd| cmd.arg("-c"))
            .and_then(|cmd| cmd.trusted_shell_script_arg(&command_owned))?;
        guarded.execute()
    });

    let outcome = match tokio::time::timeout(timeout, execution).await {
        Ok(Ok(Ok(output))) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let truncated = stdout.len() > 16_000 || stderr.len() > 16_000;
            json!({
                "status": if output.status.success() { "ok" } else { "failed" },
                "exit_code": output.status.code(),
                "stdout": stdout.chars().take(16_000).collect::<String>(),
                "stderr": stderr.chars().take(16_000).collect::<String>(),
                "truncated": truncated,
            })
        }
        Ok(Ok(Err(error))) => json!({ "status": "failed", "error": error.to_string() }),
        Ok(Err(error)) => json!({ "status": "failed", "error": error.to_string() }),
        Err(_) => json!({ "status": "failed", "error": "command timed out" }),
    };
    log::info!(
        "terminal exec for {user_id} in {container} ({} chars) -> {}",
        command.len(),
        outcome.get("status").and_then(Value::as_str).unwrap_or("unknown")
    );
    let recorded = computer_use::record_terminal(
        body.session_id.as_deref().unwrap_or(""),
        user_id,
        json!({ "command": command, "cwd": cwd, "timeout_ms": timeout.as_millis() }),
        outcome,
    )
    .await;
    Ok(Json(recorded.to_json()))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/computer/actions", post(create_action))
        .route("/api/computer/actions/pending", get(pending))
        .route("/api/computer/actions/:id", get(get_action))
        .route("/api/computer/actions/:id/result", post(submit_result))
        .route("/api/terminal/exec", post(terminal_exec))
}

#[cfg(test)]
mod tests {
    use super::blocked_pattern;

    #[test]
    fn destructive_commands_are_refused_server_side() {
        assert_eq!(blocked_pattern("rm -rf /"), Some("rm -rf /"));
        assert_eq!(blocked_pattern("sudo SHUTDOWN now"), Some("shutdown"));
        assert_eq!(blocked_pattern("dd if=/dev/zero of=/dev/sda"), Some("dd if="));
        assert_eq!(blocked_pattern(":(){ :|:& };:"), Some(":(){"));
    }

    #[test]
    fn ordinary_commands_are_not_blocked() {
        for command in ["ls -la", "python3 make_pdf.py", "echo hi", "git status"] {
            assert_eq!(blocked_pattern(command), None, "{command} must be allowed");
        }
    }
}
