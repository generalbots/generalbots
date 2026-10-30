//! Per-user agent VM lifecycle (#1580). One running VM per logged-in user;
//! each conversation gets its own folder inside the VM at
//! `/opt/vibe/conversations/{conversation_id}/`. When the user switches
//! conversations the previous job is stopped. Every invocation goes through
//! `botlib::security::command_guard::SafeCommand` and runs blocking work
//! inside `tokio::task::spawn_blocking`.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AgentConversationFolderRow, AgentSessionRow};
use crate::schema::{agent_conversation_folders, agent_sessions};
use crate::AgentService;

pub const STATUS_PROVISIONING: &str = "provisioning";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_ERROR: &str = "error";

const LAUNCH_WAIT_SECS: u64 = 60;
const POLL_INTERVAL_SECS: u64 = 3;
const CONVERSATIONS_ROOT: &str = "/opt/vibe/conversations";

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Agent VM operation failed".to_string())
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

pub fn sanitize_id(raw: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(bad_request("Invalid identifier"));
    }
    let clean: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() {
        return Err(bad_request("Invalid identifier"));
    }
    Ok(clean)
}

#[deprecated(note = "use sanitize_id instead")]
pub fn sanitize_session_id(session_id: &str) -> Result<String, (StatusCode, String)> {
    sanitize_id(session_id)
}

/// `ag-u{user8}-{rand4}` — short, unique, DNS-safe container name.
fn generate_vm_name(user_id: &Uuid) -> String {
    let user8: String = user_id.simple().to_string().chars().take(8).collect();
    let rand4: String = Uuid::new_v4().simple().to_string().chars().take(4).collect();
    format!("ag-u{user8}-{rand4}")
}

fn incus_command(state: &AgentService) -> Result<SafeCommand, (StatusCode, String)> {
    SafeCommand::new(state.incus_bin()).map_err(|e| {
        tracing::error!("incus command rejected by guard: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "VM subsystem unavailable".to_string())
    })
}

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
        _ => false,
    }
}

pub(crate) async fn is_running(state: &AgentService, vm_name: &str) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(state, vec!["list".into(), "--format".into(), "json".into()]).await?;
    Ok(running_from_list(&String::from_utf8_lossy(&output.stdout), vm_name))
}

pub(crate) async fn wait_for_running(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(LAUNCH_WAIT_SECS);
    while std::time::Instant::now() < deadline {
        if is_running(state, vm_name).await? {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
    Err(internal(format!("container {vm_name} did not reach RUNNING within {LAUNCH_WAIT_SECS}s")))
}

// ─── DB helpers ────────────────────────────────────────────────────────────

fn load_session_by_user(
    conn: &mut diesel::PgConnection,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::user_id.eq(user_id))
        .filter(dsl::status.ne(STATUS_STOPPED))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select by user: {e}")))
}

fn load_session(
    conn: &mut diesel::PgConnection,
    session_id: &str,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::session_id.eq(session_id))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))
}

fn touch_last_active(conn: &mut diesel::PgConnection, row_id: Uuid) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::last_active_at.eq(Utc::now()))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions touch: {e}")))?;
    Ok(())
}

fn set_status(conn: &mut diesel::PgConnection, row_id: Uuid, status_value: &str) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::status.eq(status_value))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set status: {e}")))?;
    Ok(())
}

fn set_active_conversation(
    conn: &mut diesel::PgConnection,
    row_id: Uuid,
    conversation_id: &str,
) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set((
            dsl::active_conversation_id.eq(Some(conversation_id)),
            dsl::last_active_at.eq(Utc::now()),
        ))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set conversation: {e}")))?;
    Ok(())
}

async fn container_exists(state: &AgentService, vm_name: &str) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(state, vec!["list".into(), "--format".into(), "json".into()]).await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(match serde_json::from_str::<serde_json::Value>(&stdout) {
        Ok(serde_json::Value::Array(items)) => items
            .iter()
            .any(|item| item.get("name").and_then(|n| n.as_str()) == Some(vm_name)),
        _ => false,
    })
}

// ─── Core: per-user VM guarantee ──────────────────────────────────────────

/// Idempotently guarantee a RUNNING VM for the user (#1580). One VM per
/// logged-in user; the `conversation_id` is used to set up the working
/// folder inside the VM.
pub async fn ensure_vm(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    bot_id: &Uuid,
    conversation_id: Option<&str>,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let existing = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        load_session_by_user(&mut conn, user_id)?
    };

    let row = if let Some(row) = existing {
        match row.status.as_str() {
            STATUS_RUNNING => {
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                touch_last_active(&mut conn, row.id)?;
                row
            }
            STATUS_PROVISIONING => row,
            _ => {
                if is_running(state, &row.vm_name).await? {
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh
                } else if container_exists(state, &row.vm_name).await? {
                    run_incus(state, vec!["start".into(), row.vm_name.clone()]).await?;
                    wait_for_running(state, &row.vm_name).await?;
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh
                } else {
                    provision(state, user_id, bot_id, Some(row)).await?
                }
            }
        }
    } else {
        provision(state, user_id, bot_id, None).await?
    };

    if let Some(cid) = conversation_id {
        let _ = ensure_conversation_folder(state, &row, cid).await;
    }

    Ok(row)
}

/// Legacy wrapper for code still passing session_id. Routes through user_id.
pub async fn ensure_vm_by_session(
    state: &Arc<AgentService>,
    _session_id: &str,
    user_id: &Uuid,
    bot_id: &Uuid,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    ensure_vm(state, user_id, bot_id, None).await
}

async fn provision(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    bot_id: &Uuid,
    previous: Option<AgentSessionRow>,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let vm_name = generate_vm_name(user_id);
    let session_id_str = user_id.to_string();

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let now: DateTime<Utc> = Utc::now();
    let row_id = match previous {
        Some(prev) => {
            use agent_sessions::dsl;
            diesel::update(dsl::agent_sessions.filter(dsl::id.eq(prev.id)))
                .set((
                    dsl::vm_name.eq(&vm_name),
                    dsl::status.eq(STATUS_PROVISIONING),
                    dsl::last_active_at.eq(now),
                    dsl::expires_at.eq(None::<DateTime<Utc>>),
                    dsl::metadata.eq(None::<serde_json::Value>),
                    dsl::active_conversation_id.eq(None::<String>),
                ))
                .execute(&mut conn)
                .map_err(|e| internal(format!("agent_sessions reprovision: {e}")))?;
            prev.id
        }
        None => {
            let new_id = Uuid::new_v4();
            let row = AgentSessionRow {
                id: new_id,
                session_id: session_id_str.clone(),
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
                active_conversation_id: None,
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

    // Bootstrap the conversations root inside the VM.
    let _ = exec_in_vm(state, &vm_name, &["mkdir".into(), "-p".into(), CONVERSATIONS_ROOT.into()]).await;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row_id, STATUS_RUNNING)?;
    load_session_by_user(&mut conn, user_id)?
        .ok_or_else(|| internal(format!("agent session for user {user_id} vanished after provision")))
}

async fn launch_and_wait(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    run_incus(state, vec!["launch".into(), "images:ubuntu/24.04".into(), vm_name.into()]).await?;
    wait_for_running(state, vm_name).await
}

// ─── Conversation folder management ──────────────────────────────────────

/// Create `/opt/vibe/conversations/{conversation_id}/` in the VM and record
/// it in `agent_conversation_folders`. Idempotent.
pub async fn ensure_conversation_folder(
    state: &AgentService,
    session: &AgentSessionRow,
    conversation_id: &str,
) -> Result<String, (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;
    let folder_path = format!("{CONVERSATIONS_ROOT}/{clean_cid}");

    // Check DB first.
    let exists = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        use agent_conversation_folders::dsl as cf;
        cf::agent_conversation_folders
            .filter(cf::agent_session_id.eq(session.id))
            .filter(cf::conversation_id.eq(&clean_cid))
            .first::<AgentConversationFolderRow>(&mut conn)
            .optional()
            .map_err(|e| internal(format!("conv_folders select: {e}")))?
    };

    if let Some(row) = exists {
        return Ok(row.folder_path);
    }

    exec_in_vm(state, &session.vm_name, &["mkdir".into(), "-p".into(), folder_path.clone()]).await?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = AgentConversationFolderRow {
        id: Uuid::new_v4(),
        agent_session_id: session.id,
        conversation_id: clean_cid,
        folder_path: folder_path.clone(),
        created_at: Utc::now(),
    };
    diesel::insert_into(agent_conversation_folders::table)
        .values(&row)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .map_err(|e| internal(format!("conv_folders insert: {e}")))?;

    Ok(folder_path)
}

/// Switch active conversation: stop running jobs in the previous conversation
/// folder, then activate the new one (#1580).
pub async fn switch_conversation(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    new_conversation_id: &str,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let clean_cid = sanitize_id(new_conversation_id)?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let session = load_session_by_user(&mut conn, user_id)?
        .ok_or_else(|| bad_request("No active VM for this user"))?;
    drop(conn);

    // Kill any running processes from the previous conversation folder.
    if let Some(ref prev_cid) = session.active_conversation_id {
        if *prev_cid != clean_cid {
            let prev_folder = format!("{CONVERSATIONS_ROOT}/{prev_cid}");
            let _ = exec_in_vm(
                state,
                &session.vm_name,
                &[
                    "sh".into(), "-c".into(),
                    format!("pkill -f '{prev_folder}' 2>/dev/null || true"),
                ],
            ).await;
        }
    }

    ensure_conversation_folder(state, &session, &clean_cid).await?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_active_conversation(&mut conn, session.id, &clean_cid)?;

    let mut updated = session;
    updated.active_conversation_id = Some(clean_cid);
    Ok(updated)
}

/// Execute a program in the conversation folder. The code is written to a
/// temp file inside the folder and run with the appropriate interpreter.
/// Returns (success, stdout, stderr).
pub async fn exec_in_conversation(
    state: &AgentService,
    vm_name: &str,
    conversation_id: &str,
    language: &str,
    code: &str,
    timeout_ms: u64,
) -> Result<(bool, String, String), (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;
    let folder = format!("{CONVERSATIONS_ROOT}/{clean_cid}");

    let (interpreter, ext) = match language {
        "python" => ("python3", "py"),
        "node" => ("node", "js"),
        "shell" => ("sh", "sh"),
        _ => return Err(bad_request("Unsupported language")),
    };

    let run_id: String = Uuid::new_v4().simple().to_string().chars().take(8).collect();
    let script_path = format!("{folder}/run_{run_id}.{ext}");

    let b64 = crate::sandbox::b64_encode(code.as_bytes());
    let setup = format!(
        "mkdir -p {folder} && echo {b64} | base64 -d > {script_path} && cd {folder} && {interpreter} {script_path}"
    );

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), setup]),
    ).await;

    match result {
        Err(_) => Ok((false, String::new(), "Execution timed out".to_string())),
        Ok(r) => r,
    }
}

// ─── Low-level VM operations ─────────────────────────────────────────────

pub async fn stop_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["stop".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn stop_vm_for_user(state: &Arc<AgentService>, user_id: &Uuid) -> Result<(), (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = match load_session_by_user(&mut conn, user_id)? {
        Some(r) => r,
        None => return Ok(()),
    };
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["stop".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn delete_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["delete".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    use agent_sessions::dsl;
    diesel::delete(dsl::agent_sessions.filter(dsl::id.eq(row.id)))
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_sessions delete: {e}")))?;
    Ok(())
}

pub async fn exec_in_vm(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
) -> Result<(bool, String, String), (StatusCode, String)> {
    let mut args = vec!["exec".into(), vm_name.into(), "--".into()];
    args.extend(cmd_args.iter().cloned());
    let output = run_incus(state, args).await?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

pub fn expiry_from_now(idle_timeout_secs: i64) -> Option<DateTime<Utc>> {
    Utc::now().checked_add_signed(Duration::seconds(idle_timeout_secs))
}

/// Return the user's active VM if one exists, without provisioning.
pub fn get_user_vm(
    state: &AgentService,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    load_session_by_user(&mut conn, user_id)
}
