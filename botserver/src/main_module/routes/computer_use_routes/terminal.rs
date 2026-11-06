use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Json, State};
use axum::http::{HeaderMap, StatusCode};
use botcore::shared::state::AppState;
use botlib::security::command_guard::SafeCommand;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::audit::{audit_terminal, TerminalAuditRow};
use super::terminal_rules::blocked_pattern;
use super::{api_error, resolve_user, ApiError};
use crate::core::computer_use;

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
    /// #1645 — the action whose consent record authorises this execution.
    #[serde(default)]
    pub action_id: Option<Uuid>,
}

/// POST /api/terminal/exec
pub async fn terminal_exec(
    State(state): State<Arc<AppState>>,
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
    // #1645 — `approved=true` alone is forgeable. Execution requires the
    // action id whose consent record /approve created.
    let action_id = body.action_id.ok_or_else(|| {
        api_error(
            StatusCode::FORBIDDEN,
            "terminal execution requires an approved action_id",
        )
    })?;
    if !computer_use::is_approved(action_id, user_id).await {
        audit_terminal(
            &state.conn,
            TerminalAuditRow {
                user_id,
                session_id: body.session_id.as_deref().unwrap_or(""),
                action_id: Some(action_id),
                command: body.command.trim(),
                cwd: body.cwd.as_deref().unwrap_or(""),
                container: "",
                decision: "denied",
                exit_code: None,
                output: "no consent record",
            },
        );
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "no consent record for this action; approve it first",
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
        audit_terminal(
            &state.conn,
            TerminalAuditRow {
                user_id,
                session_id: body.session_id.as_deref().unwrap_or(""),
                action_id: Some(action_id),
                command,
                cwd: "",
                container: "",
                decision: "denied",
                exit_code: None,
                output: "blocked destructive pattern",
            },
        );
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
        audit_terminal(
            &state.conn,
            TerminalAuditRow {
                user_id,
                session_id: body.session_id.as_deref().unwrap_or(""),
                action_id: Some(action_id),
                command,
                cwd: body.cwd.as_deref().unwrap_or(""),
                container: "",
                decision: "denied",
                exit_code: None,
                output: "workspace container is not running",
            },
        );
        api_error(
            StatusCode::CONFLICT,
            "The agent workspace is not running. Finish provisioning (or restart the desktop app), then retry.",
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
        outcome
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    // #1645 — durable row, independent of the in-memory registry TTL.
    let exit_code = outcome
        .get("exit_code")
        .and_then(Value::as_i64)
        .map(|value| value as i32);
    let output_excerpt = format!(
        "{}{}",
        outcome.get("stdout").and_then(Value::as_str).unwrap_or(""),
        outcome.get("stderr").and_then(Value::as_str).unwrap_or("")
    );
    audit_terminal(
        &state.conn,
        TerminalAuditRow {
            user_id,
            session_id: body.session_id.as_deref().unwrap_or(""),
            action_id: Some(action_id),
            command,
            cwd: &cwd,
            container: &container,
            decision: "approved",
            exit_code,
            output: &output_excerpt,
        },
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
