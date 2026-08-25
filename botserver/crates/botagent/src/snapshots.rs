//! Snapshot management for agent VMs (#1167): create/list/restore/delete with
//! per-session FIFO eviction enforced against `max_snapshots_per_session`.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::authorize_session_access;
use crate::models::{AgentSessionRow, AgentSnapshotRow, SnapshotCreateBody};
use crate::schema::{agent_sessions, agent_snapshots};
use crate::vm;
use crate::AgentService;

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Snapshot operation failed".to_string(),
    )
}

async fn run_incus(
    state: &AgentService,
    args: Vec<String>,
) -> Result<std::process::Output, (StatusCode, String)> {
    let mut cmd = botlib::security::command_guard::SafeCommand::new(state.incus_bin())
        .map_err(|e| internal(format!("incus guard: {e}")))?;
    for arg in &args {
        cmd = cmd.arg(arg).map_err(|e| internal(format!("incus arg: {e}")))?;
    }
    tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| internal(format!("incus execution: {e}")))
}

fn load_session_by_id(
    conn: &mut diesel::PgConnection,
    session_pk: Uuid,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    use crate::schema::agent_sessions::dsl::{agent_sessions, id};
    agent_sessions
        .filter(id.eq(session_pk))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent session not found".to_string()))
}

/// `GET /api/agent/sessions/{id}/snapshots`
pub async fn list_snapshots(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let clean = vm::sanitize_session_id(&session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let session = {
        use crate::schema::agent_sessions::dsl::{agent_sessions, session_id as sid};
        agent_sessions
            .filter(sid.eq(&clean))
            .first::<AgentSessionRow>(&mut conn)
            .optional()
            .map_err(|e| internal(format!("agent_sessions select: {e}")))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent session not found".to_string()))?
    };
    authorize_session_access(&headers, &session.user_id)?;

    use crate::schema::agent_snapshots::dsl::{agent_session_id, agent_snapshots, created_at};
    let items = agent_snapshots
        .filter(agent_session_id.eq(session.id))
        .order(created_at.asc())
        .load::<AgentSnapshotRow>(&mut conn)
        .map_err(|e| internal(format!("agent_snapshots select: {e}")))?;

    Ok(Json(serde_json::json!({ "items": items })))
}

/// `POST /api/agent/sessions/{id}/snapshots` — create an Incus snapshot.
pub async fn create_snapshot(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(body): Json<SnapshotCreateBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let clean = vm::sanitize_session_id(&session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let session = {
        use crate::schema::agent_sessions::dsl::{agent_sessions, session_id as sid};
        agent_sessions
            .filter(sid.eq(&clean))
            .first::<AgentSessionRow>(&mut conn)
            .optional()
            .map_err(|e| internal(format!("agent_sessions select: {e}")))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent session not found".to_string()))?
    };
    authorize_session_access(&headers, &session.user_id)?;

    let tag = format!("snap-{}", Utc::now().timestamp());
    run_incus(
        &state,
        vec![
            "snapshot".to_string(),
            "create".to_string(),
            format!("{}/{}", session.vm_name, tag),
        ],
    )
    .await?;

    let row = AgentSnapshotRow {
        id: Uuid::new_v4(),
        agent_session_id: session.id,
        label: body.label.filter(|l| !l.trim().is_empty()),
        incus_snapshot: tag.clone(),
        size_bytes: None,
        created_at: Utc::now(),
    };
    diesel::insert_into(agent_snapshots::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_snapshots insert: {e}")))?;

    evict_oldest(&state, &session, &mut conn).await?;

    Ok(Json(serde_json::json!({ "status": "created", "item": row })))
}

/// FIFO eviction: drop the oldest snapshots beyond the configured cap.
async fn evict_oldest(
    state: &Arc<AgentService>,
    session: &AgentSessionRow,
    conn: &mut diesel::PgConnection,
) -> Result<(), (StatusCode, String)> {
    use crate::schema::agent_snapshots::dsl::{agent_session_id, agent_snapshots, created_at, id};

    let cap = state.max_snapshots_per_session();
    if cap <= 0 {
        return Ok(());
    }
    let overage = agent_snapshots
        .filter(agent_session_id.eq(session.id))
        .order(created_at.asc())
        .load::<AgentSnapshotRow>(conn)
        .map_err(|e| internal(format!("agent_snapshots list: {e}")))?
        .len() as i64
        - cap;

    if overage <= 0 {
        return Ok(());
    }

    let stale: Vec<AgentSnapshotRow> = agent_snapshots
        .filter(agent_session_id.eq(session.id))
        .order(created_at.asc())
        .limit(overage)
        .load(conn)
        .map_err(|e| internal(format!("agent_snapshots eviction select: {e}")))?;

    for snap in stale {
        if let Err(e) = run_incus(
            state,
            vec![
                "delete".to_string(),
                "--force".to_string(),
                format!("{}/{}", session.vm_name, snap.incus_snapshot),
            ],
        )
        .await
        {
            tracing::warn!(
                "best-effort incus snapshot delete {} failed: {}",
                snap.incus_snapshot,
                e.1
            );
        }
        diesel::delete(agent_snapshots.filter(id.eq(snap.id)))
            .execute(&mut *conn)
            .map_err(|e| internal(format!("agent_snapshots delete: {e}")))?;
    }
    Ok(())
}

fn load_snapshot_with_session(
    conn: &mut diesel::PgConnection,
    snapshot_id: Uuid,
) -> Result<(AgentSnapshotRow, AgentSessionRow), (StatusCode, String)> {
    use crate::schema::agent_snapshots::dsl::{agent_snapshots, id};
    let snap = agent_snapshots
        .filter(id.eq(snapshot_id))
        .first::<AgentSnapshotRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_snapshots select: {e}")))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Snapshot not found".to_string()))?;
    let session = load_session_by_id(conn, snap.agent_session_id)?;
    Ok((snap, session))
}

/// `POST /api/agent/snapshots/{id}/restore` — stop → restore → start.
pub async fn restore_snapshot(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let (snap, session) = load_snapshot_with_session(&mut conn, snapshot_id)?;
    authorize_session_access(&headers, &session.user_id)?;
    {
        use crate::schema::agent_sessions::dsl::{id, status};
        diesel::update(agent_sessions::table.filter(id.eq(session.id)))
            .set(status.eq(vm::STATUS_PROVISIONING))
            .execute(&mut conn)
            .map_err(|e| internal(format!("agent_sessions status: {e}")))?;
    }
    drop(conn);

    let result = async {
        if vm::is_running(&state, &session.vm_name).await? {
            run_incus(
                &state,
                vec!["stop".to_string(), "--force".to_string(), session.vm_name.clone()],
            )
            .await?;
        }
        run_incus(
            &state,
            vec![
                "restore".to_string(),
                session.vm_name.clone(),
                snap.incus_snapshot.clone(),
            ],
        )
        .await?;
        run_incus(
            &state,
            vec!["start".to_string(), session.vm_name.clone()],
        )
        .await?;
        vm::wait_for_running(&state, &session.vm_name).await
    }
    .await;

    let final_status = match result {
        Ok(()) => vm::STATUS_RUNNING,
        Err(ref e) => {
            tracing::error!("restore of {snapshot_id} failed: {}", e.1);
            vm::STATUS_ERROR
        }
    };
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    {
        use crate::schema::agent_sessions::dsl::{id, last_active_at, status};
        diesel::update(agent_sessions::table.filter(id.eq(session.id)))
            .set((status.eq(final_status), last_active_at.eq(Utc::now())))
            .execute(&mut conn)
            .map_err(|e| internal(format!("agent_sessions status: {e}")))?;
    }
    result?;

    Ok(Json(serde_json::json!({
        "status": "restored",
        "session_id": session.session_id,
        "snapshot": snap.incus_snapshot,
    })))
}

/// `DELETE /api/agent/snapshots/{id}`
pub async fn delete_snapshot(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Path(snapshot_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let (snap, session) = load_snapshot_with_session(&mut conn, snapshot_id)?;
    authorize_session_access(&headers, &session.user_id)?;

    run_incus(
        &state,
        vec![
            "delete".to_string(),
            "--force".to_string(),
            format!("{}/{}", session.vm_name, snap.incus_snapshot),
        ],
    )
    .await?;

    use crate::schema::agent_snapshots::dsl::{agent_snapshots, id};
    diesel::delete(agent_snapshots.filter(id.eq(snap.id)))
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_snapshots delete: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}
