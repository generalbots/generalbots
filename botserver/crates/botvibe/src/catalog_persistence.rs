//! Write-through persistence for the Vibe catalog entities (#816).
//!
//! `CanvasStore`, `IssueStore`, `SessionStore` and `TeamStore` keep their
//! in-memory `RwLock<Vec<_>>` as the live authority (same pattern as
//! `run_store`), but every mutation is also upserted into the corresponding
//! `vibe_*` table so data survives a restart. When a pool is provided the
//! store hydrates itself from the database on construction.

use crate::canvases::VibeCanvas;
use crate::issues::VibeIssue;
use crate::sessions::VibeSession;
use crate::teams::VibeTeam;
use crate::types::DbPool;
use diesel::prelude::*;

/// Upserts a canvas row (JSONB content).
pub fn save_canvas(pool: &DbPool, canvas: &VibeCanvas) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("canvas persist: pool get: {e}"))?;
    diesel::sql_query(
        "INSERT INTO vibe_canvases \
         (canvas_id, title, project, content, share_token, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (canvas_id) DO UPDATE SET \
           title = EXCLUDED.title, project = EXCLUDED.project, \
           content = EXCLUDED.content, share_token = EXCLUDED.share_token, \
           updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(canvas.canvas_id)
    .bind::<diesel::sql_types::Text, _>(&canvas.title)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(canvas.project.as_deref())
    .bind::<diesel::sql_types::Jsonb, _>(&canvas.content)
    .bind::<diesel::sql_types::Text, _>(&canvas.share_token)
    .bind::<diesel::sql_types::Timestamptz, _>(canvas.created_at)
    .bind::<diesel::sql_types::Timestamptz, _>(canvas.updated_at)
    .execute(&mut conn)
    .map_err(|e| format!("canvas persist: {e}"))?;
    Ok(())
}

pub fn delete_canvas(pool: &DbPool, canvas_id: uuid::Uuid) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("canvas delete: pool get: {e}"))?;
    diesel::sql_query("DELETE FROM vibe_canvases WHERE canvas_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(canvas_id)
        .execute(&mut conn)
        .map_err(|e| format!("canvas delete: {e}"))?;
    Ok(())
}

/// Upserts an issue row (JSONB labels).
pub fn save_issue(pool: &DbPool, issue: &VibeIssue) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("issue persist: pool get: {e}"))?;
    let labels = serde_json::to_value(&issue.labels)
        .map_err(|e| format!("issue persist: labels serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO vibe_issues \
         (issue_id, title, body, labels, state, assignee, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (issue_id) DO UPDATE SET \
           title = EXCLUDED.title, body = EXCLUDED.body, labels = EXCLUDED.labels, \
           state = EXCLUDED.state, assignee = EXCLUDED.assignee, \
           updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(issue.issue_id)
    .bind::<diesel::sql_types::Text, _>(&issue.title)
    .bind::<diesel::sql_types::Text, _>(&issue.body)
    .bind::<diesel::sql_types::Jsonb, _>(labels)
    .bind::<diesel::sql_types::Text, _>(issue.state.as_str())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(issue.assignee.as_deref())
    .bind::<diesel::sql_types::Timestamptz, _>(issue.created_at)
    .bind::<diesel::sql_types::Timestamptz, _>(issue.updated_at)
    .execute(&mut conn)
    .map_err(|e| format!("issue persist: {e}"))?;
    Ok(())
}

/// Upserts a session row (JSONB run snapshot).
pub fn save_session(pool: &DbPool, session: &VibeSession) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("session persist: pool get: {e}"))?;
    let run_json = session
        .run
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|e| format!("session persist: run serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO vibe_sessions \
         (session_id, parent_session_id, bot_id, user_id, intent, use_case, \
          budget_cents, run, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (session_id) DO UPDATE SET \
           parent_session_id = EXCLUDED.parent_session_id, \
           intent = EXCLUDED.intent, use_case = EXCLUDED.use_case, \
           budget_cents = EXCLUDED.budget_cents, run = EXCLUDED.run, \
           updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(session.session_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(session.parent_session_id)
    .bind::<diesel::sql_types::Uuid, _>(session.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(session.user_id)
    .bind::<diesel::sql_types::Text, _>(&session.intent)
    .bind::<diesel::sql_types::Text, _>(&session.use_case.to_string())
    .bind::<diesel::sql_types::BigInt, _>(session.budget_cents as i64)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(run_json)
    .bind::<diesel::sql_types::Timestamptz, _>(session.created_at)
    .bind::<diesel::sql_types::Timestamptz, _>(session.updated_at)
    .execute(&mut conn)
    .map_err(|e| format!("session persist: {e}"))?;
    Ok(())
}

/// Upserts a team row (JSONB members + shared tasks).
pub fn save_team(pool: &DbPool, team: &VibeTeam) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("team persist: pool get: {e}"))?;
    let members = serde_json::to_value(&team.members)
        .map_err(|e| format!("team persist: members serialize: {e}"))?;
    let shared_tasks = serde_json::to_value(&team.shared_tasks)
        .map_err(|e| format!("team persist: shared_tasks serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO vibe_teams \
         (team_id, name, objective, members, shared_tasks, status, created_at, completed_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (team_id) DO UPDATE SET \
           name = EXCLUDED.name, objective = EXCLUDED.objective, \
           members = EXCLUDED.members, shared_tasks = EXCLUDED.shared_tasks, \
           status = EXCLUDED.status, completed_at = EXCLUDED.completed_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(team.team_id)
    .bind::<diesel::sql_types::Text, _>(&team.name)
    .bind::<diesel::sql_types::Text, _>(&team.objective)
    .bind::<diesel::sql_types::Jsonb, _>(members)
    .bind::<diesel::sql_types::Jsonb, _>(shared_tasks)
    .bind::<diesel::sql_types::Text, _>(&team.status)
    .bind::<diesel::sql_types::Timestamptz, _>(team.created_at)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(team.completed_at)
    .execute(&mut conn)
    .map_err(|e| format!("team persist: {e}"))?;
    Ok(())
}
