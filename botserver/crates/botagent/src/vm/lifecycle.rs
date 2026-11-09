//! Per-user container lifecycle: guarantee, provision, launch, stop and delete.

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, OnceLock, Weak};
use uuid::Uuid;

use crate::models::AgentSessionRow;
use crate::schema::agent_sessions;
use crate::AgentService;

use super::bootstrap::{bootstrap_script, schedule_workspace_preparation};
use super::db::{load_session, load_session_by_user, set_status, touch_last_active};
use super::identity::{generate_vm_name, require_authenticated_identity, sanitize_id};
use super::incus::{
    cold_launch_wait_secs, container_exists, exec_in_vm, instance_log_tail, is_running, run_incus,
    wait_for_running, wait_for_running_within,
};
use super::retry::{retry_incus, start_container};
use super::workspace::ensure_conversation_folder;
use super::{
    bad_request, internal, CONVERSATIONS_ROOT, STATUS_ERROR, STATUS_PROVISIONING, STATUS_RUNNING,
    STATUS_STOPPED,
};

/// One in-process lock per user. Two chat turns can reach `ensure_vm` at the
/// same time (the prompt-context hook and the `vm_exec` tool); without this
/// lock both ran `incus launch` for the same container, the loser failed its
/// launch/row update and the user saw the generic "Agent VM operation failed"
/// on the first cold start (issue #1798). The second call now waits for the
/// first provisioning to finish and then observes the RUNNING row.
fn provision_lock(user_id: &Uuid) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<StdMutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>> =
        OnceLock::new();
    let map = LOCKS.get_or_init(|| StdMutex::new(HashMap::new()));
    let key = user_id.to_string();
    let mut guard = match map.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Weak entries disappear once the last waiter drops its Arc, so the map
    // does not grow for the lifetime of the process.
    guard.retain(|_, weak| weak.strong_count() > 0);
    if let Some(lock) = guard.get(&key).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(tokio::sync::Mutex::new(()));
    guard.insert(key, Arc::downgrade(&lock));
    lock
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
    require_authenticated_identity(state, user_id)?;
    // Serialize cold starts per user: the second concurrent turn waits here
    // instead of launching a duplicate container (issue #1798).
    let lock = provision_lock(user_id);
    let _provision_guard = lock.lock().await;

    let existing = {
        let mut conn = state
            .pool()
            .get()
            .map_err(|e| internal(format!("DB pool: {e}")))?;
        load_session_by_user(&mut conn, user_id)?
    };

    let row = if let Some(row) = existing {
        match row.status.as_str() {
            // Trust the row only when the container behind it still exists:
            // after a cleanup or a failed provisioning the database could say
            // "running" while `incus list` was empty, so every vm_exec returned
            // nothing and the workspace looked permanently unavailable.
            STATUS_RUNNING if is_running(state, &row.vm_name).await? => {
                let mut conn = state
                    .pool()
                    .get()
                    .map_err(|e| internal(format!("DB pool: {e}")))?;
                touch_last_active(&mut conn, row.id)?;
                row
            }
            STATUS_RUNNING | STATUS_PROVISIONING
                if container_exists(state, &row.vm_name).await? =>
            {
                // `start` is retried with backoff and tolerates "already
                // running": a concurrent turn may have started it first.
                start_container(state, &row.vm_name).await?;
                wait_for_running(state, &row.vm_name).await?;
                let mut conn = state
                    .pool()
                    .get()
                    .map_err(|e| internal(format!("DB pool: {e}")))?;
                set_status(&mut conn, row.id, STATUS_RUNNING)?;
                let mut fresh = row;
                fresh.status = STATUS_RUNNING.to_string();
                fresh
            }
            _ => {
                if is_running(state, &row.vm_name).await? {
                    let mut conn = state
                        .pool()
                        .get()
                        .map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh
                } else if container_exists(state, &row.vm_name).await? {
                    start_container(state, &row.vm_name).await?;
                    wait_for_running(state, &row.vm_name).await?;
                    let mut conn = state
                        .pool()
                        .get()
                        .map_err(|e| internal(format!("DB pool: {e}")))?;
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

    if row.status == STATUS_RUNNING {
        schedule_workspace_preparation(state.clone(), row.vm_name.clone());
    }

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

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
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
        let mut conn = state
            .pool()
            .get()
            .map_err(|e| internal(format!("DB pool: {e}")))?;
        let _ = set_status(&mut conn, row_id, STATUS_ERROR);
        return Err(e);
    }

    // Bootstrap the conversations root inside the VM.
    let _ = exec_in_vm(
        state,
        &vm_name,
        &["mkdir".into(), "-p".into(), CONVERSATIONS_ROOT.into()],
    )
    .await;

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row_id, STATUS_RUNNING)?;
    load_session_by_user(&mut conn, user_id)?.ok_or_else(|| {
        internal(format!(
            "agent session for user {user_id} vanished after provision"
        ))
    })
}

/// Provision the container the same way the Vibe runtime does: a Debian
/// system container with explicit CPU/memory limits attached to the managed
/// bridge (internet egress). Runtime packages (python3/node) are installed
/// in the background so the first user message is not blocked by apt.
async fn launch_and_wait(
    state: &Arc<AgentService>,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    let image = std::env::var("AGENT_VM_IMAGE").unwrap_or_else(|_| "images:debian/13".to_string());
    let cpu = std::env::var("AGENT_VM_CPU").unwrap_or_else(|_| "1".to_string());
    let mem = std::env::var("AGENT_VM_MEMORY").unwrap_or_else(|_| "1GiB".to_string());
    let bridge = std::env::var("AGENT_VM_BRIDGE").unwrap_or_else(|_| "incusbr0".to_string());

    let launch_args = vec![
        "launch".to_string(),
        image,
        vm_name.to_string(),
        "--config".to_string(),
        format!("limits.cpu={cpu}"),
        "--config".to_string(),
        format!("limits.memory={mem}"),
    ];
    // A cold launch pulls and unpacks the image and can leave the daemon busy.
    // Retry with backoff instead of turning the first transient failure into a
    // terminal workspace error (issue #1798).
    if let Err(e) = retry_incus(state, &launch_args, "launch").await {
        // A host without Incus (or without socket permission) never recovers
        // by retrying `incus start`; return the actionable setup error now
        // instead of waiting out the cold-launch budget.
        if e.0 == StatusCode::SERVICE_UNAVAILABLE {
            return Err(e);
        }
        // A previous attempt can leave the container created but stopped; the
        // launch then fails with "already exists" and the workspace stayed
        // permanently unavailable. Start the existing container instead.
        tracing::warn!(
            "incus launch for {vm_name} failed after retries ({}); starting the existing container",
            e.1
        );
        if let Err(start_error) = start_container(state, vm_name).await {
            tracing::warn!(
                "container {vm_name} is not available to start yet ({}); waiting for the cold launch",
                start_error.1
            );
        }
    }

    // Attach the managed bridge. The default profile often ships no NIC;
    // ignore the error when a NIC already exists (idempotent provisioning).
    let _ = run_incus(
        state,
        vec![
            "config".into(),
            "device".into(),
            "add".into(),
            vm_name.into(),
            "eth0".into(),
            "nic".into(),
            format!("network={bridge}"),
            "name=eth0".into(),
        ],
    )
    .await;

    // The first launch of a fresh image can download/unpack for minutes; use
    // the cold budget before declaring the workspace unavailable.
    if wait_for_running_within(state, vm_name, cold_launch_wait_secs())
        .await
        .is_err()
    {
        // The container may still be created but stopped; start it explicitly
        // and wait again instead of failing the whole workspace.
        tracing::warn!("container {vm_name} did not start in time; retrying incus launch/start");
        // The instance may never have been created if the original launch kept
        // failing while the image was still downloading. Re-run launch (it is
        // idempotent when the instance now exists) before starting it.
        if let Err(launch_error) = retry_incus(state, &launch_args, "launch").await {
            if launch_error.0 == StatusCode::SERVICE_UNAVAILABLE {
                return Err(launch_error);
            }
            tracing::warn!(
                "container {vm_name} launch retry failed ({}); trying incus start",
                launch_error.1
            );
        }
        if let Err(start_error) = start_container(state, vm_name).await {
            tracing::warn!(
                "container {vm_name} still cannot start ({}); checking the instance log",
                start_error.1
            );
        }
        if let Err(wait_error) = wait_for_running(state, vm_name).await {
            // A created-but-never-running container keeps its reason inside
            // the instance log (bpf/mount/cgroup host-side failures); the
            // daemon can even return exit 0 from `start` while the instance
            // stays STOPPED. Attach that evidence so callers surface the
            // true cause instead of a bare timeout.
            let reason = match instance_log_tail(state, vm_name).await {
                Some(tail) => format!("{}; instance log: {tail}", wait_error.1),
                None => wait_error.1,
            };
            return Err((wait_error.0, reason));
        }
    }

    // Best-effort runtime bootstrap. Logs land inside the container; a
    // failure is reported by the first exec, never by provisioning.
    let state_for_bootstrap = Arc::clone(state);
    let name_for_bootstrap = vm_name.to_string();
    tokio::spawn(async move {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            exec_in_vm(
                &state_for_bootstrap,
                &name_for_bootstrap,
                &["sh".into(), "-c".into(), bootstrap_script()],
            ),
        )
        .await;
        match result {
            Ok(Ok((_, _, stderr))) if !stderr.trim().is_empty() => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap}: {stderr}");
            }
            Ok(Err((code, err))) => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap} ({code}): {err}");
            }
            Err(_) => tracing::warn!("agent runtime bootstrap {name_for_bootstrap} timed out"),
            _ => {}
        }
    });

    Ok(())
}

pub async fn stop_vm(
    state: &Arc<AgentService>,
    session_id: &str,
) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?.ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(
            state,
            vec!["stop".into(), "--force".into(), row.vm_name.clone()],
        )
        .await?;
    }
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn stop_vm_for_user(
    state: &Arc<AgentService>,
    user_id: &Uuid,
) -> Result<(), (StatusCode, String)> {
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = match load_session_by_user(&mut conn, user_id)? {
        Some(r) => r,
        None => return Ok(()),
    };
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(
            state,
            vec!["stop".into(), "--force".into(), row.vm_name.clone()],
        )
        .await?;
    }
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn delete_vm(
    state: &Arc<AgentService>,
    session_id: &str,
) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?.ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(
            state,
            vec!["delete".into(), "--force".into(), row.vm_name.clone()],
        )
        .await?;
    }
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    use agent_sessions::dsl;
    diesel::delete(dsl::agent_sessions.filter(dsl::id.eq(row.id)))
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_sessions delete: {e}")))?;
    Ok(())
}

pub fn expiry_from_now(idle_timeout_secs: i64) -> Option<DateTime<Utc>> {
    Utc::now().checked_add_signed(Duration::seconds(idle_timeout_secs))
}

/// Return the user's active VM if one exists, without provisioning.
pub fn get_user_vm(
    state: &AgentService,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    load_session_by_user(&mut conn, user_id)
}

#[cfg(test)]
mod provision_lock_tests {
    use super::provision_lock;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn the_same_user_shares_one_provision_lock() {
        let user = Uuid::new_v4();
        let first = provision_lock(&user);
        let second = provision_lock(&user);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn different_users_provision_independently() {
        let first = provision_lock(&Uuid::new_v4());
        let second = provision_lock(&Uuid::new_v4());
        assert!(!Arc::ptr_eq(&first, &second));
    }
}
