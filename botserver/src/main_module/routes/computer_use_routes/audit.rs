//! Durable terminal audit (#1645): rows, read endpoint and 90-day purge.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use diesel::sql_types::{Integer, Nullable, Text, Uuid as DieselUuid};
use diesel::RunQueryDsl;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::{api_error, resolve_user, ApiError};

pub(super) struct TerminalAuditRow<'a> {
    pub(super) user_id: Uuid,
    pub(super) session_id: &'a str,
    pub(super) action_id: Option<Uuid>,
    pub(super) command: &'a str,
    pub(super) cwd: &'a str,
    pub(super) container: &'a str,
    pub(super) decision: &'a str,
    pub(super) exit_code: Option<i32>,
    pub(super) output: &'a str,
}

pub(super) fn audit_terminal(pool: &botcore::shared::utils::DbPool, row: TerminalAuditRow<'_>) {
    let Ok(mut conn) = pool.get() else {
        log::warn!("terminal audit: database unavailable, row not written");
        return;
    };
    let excerpt: String = row.output.chars().take(2000).collect();
    if let Err(error) = diesel::sql_query(
        "INSERT INTO terminal_audit
            (user_id, session_id, action_id, command, cwd, container, decision, exit_code, output_excerpt)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind::<DieselUuid, _>(row.user_id)
    .bind::<Nullable<Text>, _>(Some(row.session_id))
    .bind::<Nullable<DieselUuid>, _>(row.action_id)
    .bind::<Text, _>(row.command.chars().take(16_000).collect::<String>())
    .bind::<Nullable<Text>, _>(Some(row.cwd))
    .bind::<Nullable<Text>, _>(Some(row.container))
    .bind::<Text, _>(row.decision)
    .bind::<Nullable<Integer>, _>(row.exit_code)
    .bind::<Nullable<Text>, _>(Some(excerpt))
    .execute(&mut conn)
    {
        log::warn!("terminal audit insert failed: {error}");
    }
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub limit: Option<i64>,
}

/// GET /api/terminal/audit — the visible trail: command, decision, exit code
/// and timestamp, independent of the 30-minute in-memory registry.
pub async fn terminal_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        command: String,
        #[diesel(sql_type = Nullable<Text>)]
        cwd: Option<String>,
        #[diesel(sql_type = Text)]
        decision: String,
        #[diesel(sql_type = Nullable<Integer>)]
        exit_code: Option<i32>,
        #[diesel(sql_type = Nullable<Text>)]
        output_excerpt: Option<String>,
        #[diesel(sql_type = Text)]
        created_at: String,
    }
    let rows = diesel::sql_query(
        "SELECT id::text, command, cwd, decision, exit_code, output_excerpt,
                created_at::text AS created_at
         FROM terminal_audit WHERE user_id = $1
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<Row>(&mut conn)
    .unwrap_or_default();
    Ok(Json(json!({
        "entries": rows.into_iter().map(|row| json!({
            "id": row.id,
            "command": row.command,
            "cwd": row.cwd,
            "decision": row.decision,
            "exit_code": row.exit_code,
            "output_excerpt": row.output_excerpt,
            "created_at": row.created_at,
        })).collect::<Vec<_>>(),
    })))
}

pub(super) struct BrowserAuditRow<'a> {
    pub(super) user_id: Uuid,
    pub(super) session_id: Option<&'a str>,
    pub(super) action_id: Option<Uuid>,
    pub(super) target: &'a str,
    pub(super) action: &'a str,
    pub(super) decision: &'a str,
    pub(super) result: Option<&'a str>,
    pub(super) screenshot_path: Option<&'a str>,
}

/// #1702 — one ledger row per browser consent decision / result.
pub(super) fn audit_browser(pool: &botcore::shared::utils::DbPool, row: BrowserAuditRow<'_>) {
    let Ok(mut conn) = pool.get() else {
        log::warn!("browser audit: database unavailable, row not written");
        return;
    };
    let result: Option<String> = row.result.map(|value| value.chars().take(2000).collect());
    if let Err(error) = diesel::sql_query(
        "INSERT INTO browser_audit
            (user_id, session_id, action_id, target, action, decision, result, screenshot_path)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind::<DieselUuid, _>(row.user_id)
    .bind::<Nullable<Text>, _>(row.session_id)
    .bind::<Nullable<DieselUuid>, _>(row.action_id)
    .bind::<Nullable<Text>, _>(Some(row.target.chars().take(4000).collect::<String>()))
    .bind::<Text, _>(row.action.chars().take(200).collect::<String>())
    .bind::<Text, _>(row.decision.chars().take(40).collect::<String>())
    .bind::<Nullable<Text>, _>(result)
    .bind::<Nullable<Text>, _>(row.screenshot_path.map(str::to_string))
    .execute(&mut conn)
    {
        log::warn!("browser audit insert failed: {error}");
    }
}

/// GET /api/browser/audit — the visible browser trail: target, action,
/// decision, result excerpt, screenshot path and timestamp.
pub async fn browser_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Nullable<Text>)]
        target: Option<String>,
        #[diesel(sql_type = Text)]
        action: String,
        #[diesel(sql_type = Text)]
        decision: String,
        #[diesel(sql_type = Nullable<Text>)]
        result: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        screenshot_path: Option<String>,
        #[diesel(sql_type = Text)]
        created_at: String,
    }
    let rows = diesel::sql_query(
        "SELECT id::text, target, action, decision, result, screenshot_path,
                created_at::text AS created_at
         FROM browser_audit WHERE user_id = $1
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<Row>(&mut conn)
    .unwrap_or_default();
    Ok(Json(json!({
        "entries": rows.into_iter().map(|row| json!({
            "id": row.id,
            "target": row.target,
            "action": row.action,
            "decision": row.decision,
            "result": row.result,
            "screenshot_path": row.screenshot_path,
            "created_at": row.created_at,
        })).collect::<Vec<_>>(),
    })))
}

/// #1645 — 90-day retention. Started from the background boot sequence.
pub fn spawn_terminal_audit_purge(state: Arc<AppState>) {
    tokio::spawn(async move {
        let secs: u64 = std::env::var("GB_TERMINAL_AUDIT_PURGE_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value >= 60)
            .unwrap_or(3600);
        loop {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            match state.conn.get() {
                Ok(mut conn) => {
                    let deleted = diesel::sql_query(
                        "DELETE FROM terminal_audit WHERE created_at < NOW() - INTERVAL '90 days'",
                    )
                    .execute(&mut conn)
                    .unwrap_or(0);
                    if deleted > 0 {
                        log::info!("terminal audit: purged {deleted} row(s) older than 90 days");
                    }
                    let browser_deleted = diesel::sql_query(
                        "DELETE FROM browser_audit WHERE created_at < NOW() - INTERVAL '90 days'",
                    )
                    .execute(&mut conn)
                    .unwrap_or(0);
                    if browser_deleted > 0 {
                        log::info!("browser audit: purged {browser_deleted} row(s) older than 90 days");
                    }
                }
                Err(error) => log::warn!("terminal audit purge: db pool: {error}"),
            }
        }
    });
}
