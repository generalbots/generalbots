//! Per-chat agent VM lifecycle on Incus (#1167). Every invocation goes
//! through `botlib::security::command_guard::SafeCommand` and runs blocking
//! work inside `tokio::task::spawn_blocking`.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AgentSessionRow;
use crate::schema::agent_sessions;
use crate::AgentService;

pub const STATUS_PROVISIONING: &str = "provisioning";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_ERROR: &str = "error";

const LAUNCH_WAIT_SECS: u64 = 60;
const POLL_INTERVAL_SECS: u64 = 3;

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Agent VM operation failed".to_string())
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

/// Sanitized session identifier used for lookups and name derivation.
pub fn sanitize_session_id(session_id: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = session_id.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(bad_request("Invalid session id"));
    }
    let clean: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() {
        return Err(bad_request("Invalid session id"));
    }
    Ok(clean)
}

/// `ag-{session8}-{rand4}` — short, unique, DNS-safe container name.
fn generate_vm_name(session_id: &str) -> String {
    let session8: String = session_id.chars().take(8).collect();
    let rand4 = Uuid::new_v4().simple().to_string();
    let rand4: String = rand4.chars().take(4).collect();
    format!("ag-{session8}-{rand4}")
}

fn incus_command(state: &AgentService) -> Result<SafeCommand, (StatusCode, String)> {
    SafeCommand::new(state.incus_bin()).map_err(|e| {
        tracing::error!("incus command rejected by guard: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "VM subsystem unavailable".to_string(),
        )
    })
}

/// Run an `incus` subcommand and capture its output off the async runtime.
async fn run_incus(state: &AgentService, args: Vec<String>) -> Result<std::process::Output, (StatusCode, String)> {
    let mut cmd = incus_command(state)?;
    for arg in &args {
        cmd = cmd.arg(arg).map_err(|e| internal(format!("incus arg guard: {e}")))?;
    }
    tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| internal(format!("incus execution: {e}")))
}

/// True when the container appears in `incus list --format json` with status RUNNING.
fn running_from_list(stdout: &str, vm_name: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(stdout) {
        Ok(serde_json::Value::Array(items)) => items.iter().any(|item| {
            item.get("name").and_then(|n| n.as_str()) == Some(vm_name)
                && item
                    .get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s.eq_ignore_ascii_case("running"))
                    .unwrap_or(false)
        }),
        Ok(_) => false,
        Err(e) => {
            tracing::error!("incus list parse: {e}");
            false
        }
    }
}

async fn is_running(state: &AgentService, vm_name: &str) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(
        state,
        vec![
            "list".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
    )
    .await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(running_from_list(&stdout, vm_name))
}

/// Poll until the container reports RUNNING or the 60s budget elapses.
async fn wait_for_running(
    state: &AgentService,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(LAUNCH_WAIT_SECS);
    while std::time::Instant::now() < deadline {
        if is_running(state, vm_name).await? {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
    Err(internal(format!("container {vm_name} did not reach RUNNING within {LAUNCH_WAIT_SECS}s")))
}

fn load_session(
    conn: &mut diesel::PgConnection,
    session_id: &str,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use crate::schema::agent_sessions::dsl::{agent_sessions, session_id as sid};
    agent_sessions
        .filter(sid.eq(session_id))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))
}

fn touch_last_active(
    conn: &mut diesel::PgConnection,
    row_id: Uuid,
) -> Result<(), (StatusCode, String)> {
    use crate::schema::agent_sessions::dsl::{agent_sessions, id, last_active_at};
    diesel::update(agent_sessions.filter(id.eq(row_id)))
        .set(last_active_at.eq(Utc::now()))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions touch: {e}")))?;
    Ok(())
}

/// Idempotently guarantee a RUNNING VM for the chat session. Redis-free:
/// deduplication relies solely on the UNIQUE `session_id` column.
pub async fn ensure_vm(
    state: &Arc<AgentService>,
    session_id: &str,
    user_id: &Uuid,
    bot_id: &Uuid,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let clean = sanitize_session_id(session_id)?;

    let existing = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        load_session(&mut conn, &clean)?
    };

    if let Some(row) = existing {
        match row.status.as_str() {
            STATUS_RUNNING => {
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                touch_last_active(&mut conn, row.id)?;
                let mut fresh = row;
                fresh.last_active_at = Utc::now();
                return Ok(fresh);
            }
            STATUS_PROVISIONING => return Ok(row),
            _ => {
                // stopped / error — attempt to revive the same container when it exists.
                if is_running(state, &row.vm_name).await? {
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh.last_active_at = Utc::now();
                    return Ok(fresh);
                }
                if container_exists(state, &row.vm_name).await? {
                    run_incus(state, vec!["start".to_string(), row.vm_name.clone()]).await?;
                    wait_for_running(state, &row.vm_name).await?;
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    return Ok(fresh);
                }
                // Container vanished externally: fall through and provision anew
                // under a fresh name, replacing the stale row afterwards.
                return provision(state, &clean, user_id, bot_id, Some(row)).await;
            }
        }
    }

    provision(state, &clean, user_id, bot_id, None).await
}

async fn container_exists(
    state: &AgentService,
    vm_name: &str,
) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(
        state,
        vec![
            "list".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
    )
    .await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(match serde_json::from_str::<serde_json::Value>(&stdout) {
        Ok(serde_json::Value::Array(items)) => items
            .iter()
            .any(|item| item.get("name").and_then(|n| n.as_str()) == Some(vm_name)),
        _ => false,
    })
}

async fn provision(
    state: &Arc<AgentService>,
    session_id: &str,
    user_id: &Uuid,
    bot_id: &Uuid,
    previous: Option<AgentSessionRow>,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let vm_name = generate_vm_name(session_id);

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let now: DateTime<Utc> = Utc::now();
    let no_expiry: Option<DateTime<Utc>> = None;
    let no_metadata: Option<serde_json::Value> = None;
    let row_id = match previous {
        Some(prev) => {
            use crate::schema::agent_sessions::dsl::{
                agent_sessions, expires_at, id, last_active_at, metadata, status, vm_name as vcol,
            };
            diesel::update(agent_sessions.filter(id.eq(prev.id)))
                .set((
                    vcol.eq(&vm_name),
                    status.eq(STATUS_PROVISIONING),
                    last_active_at.eq(now),
                    expires_at.eq(no_expiry),
                    metadata.eq(no_metadata),
                ))
                .execute(&mut conn)
                .map_err(|e| internal(format!("agent_sessions reprovision: {e}")))?;
            prev.id
        }
        None => {
            let new_id = Uuid::new_v4();
            let row = AgentSessionRow {
                id: new_id,
                session_id: session_id.to_string(),
                user_id: *user_id,
                org_id: None,
                branch_id: None,
                bot_id: *bot_id,
                vm_name: vm_name.clone(),
                status: STATUS_PROVISIONING.to_string(),
                last_active_at: now,
                expires_at: None,
                metadata: None,
                created_at: now,
            };
            diesel::insert_into(agent_sessions::table)
                .values(&row)
                .execute(&mut conn)
                .map_err(|e| internal(format!("agent_sessions insert: {e}")))?;
            new_id
        }
    };
    drop(conn);

    if let Err(e) = launch_and_wait(state, &vm_name).await {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        let _ = set_status(&mut conn, row_id, STATUS_ERROR);
        return Err(e);
    }

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row_id, STATUS_RUNNING)?;
    load_session(&mut conn, session_id)?.ok_or_else(|| internal(format!("agent session {session_id} vanished after provision")))
}

async fn launch_and_wait(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    run_incus(
        state,
        vec![
            "launch".to_string(),
            "images:ubuntu/24.04".to_string(),
            vm_name.to_string(),
        ],
    )
    .await?;
    wait_for_running(state, vm_name).await
}

fn set_status(
    conn: &mut diesel::PgConnection,
    row_id: Uuid,
    status_value: &str,
) -> Result<(), (StatusCode, String)> {
    use crate::schema::agent_sessions::dsl::{agent_sessions, id, status};
    diesel::update(agent_sessions.filter(id.eq(row_id)))
        .set(status.eq(status_value))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set status: {e}")))?;
    Ok(())
}

/// Force-stop the VM bound to the session (idempotent).
pub async fn stop_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_session_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(
            state,
            vec![
                "stop".to_string(),
                "--force".to_string(),
                row.vm_name.clone(),
            ],
        )
        .await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

/// Delete the container and its session row (snapshots cascade).
pub async fn delete_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_session_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(
            state,
            vec![
                "delete".to_string(),
                "--force".to_string(),
                row.vm_name.clone(),
            ],
        )
        .await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    use crate::schema::agent_sessions::dsl::{agent_sessions, id};
    diesel::delete(agent_sessions.filter(id.eq(row.id)))
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_sessions delete: {e}")))?;
    Ok(())
}

/// Execute a command inside the VM; returns (exit success, stdout, stderr).
pub async fn exec_in_vm(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
) -> Result<(bool, String, String), (StatusCode, String)> {
    let mut args = vec!["exec".to_string(), vm_name.to_string(), "--".to_string()];
    args.extend(cmd_args.iter().cloned());
    let output = run_incus(state, args).await?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

/// Idle expiry timestamp for newly provisioned sessions.
pub fn expiry_from_now(idle_timeout_secs: i64) -> Option<DateTime<Utc>> {
    Utc::now().checked_add_signed(Duration::seconds(idle_timeout_secs))
}
