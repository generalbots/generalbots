//! Frame-quality and cost metrics for the admin view (#1679).

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use botcore::shared::state::AppState;
use diesel::prelude::*;

#[derive(Debug, diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

fn count(conn: &mut diesel::PgConnection, sql: &str) -> i64 {
    diesel::sql_query(sql)
        .get_result::<CountRow>(conn)
        .map(|row| row.count)
        .unwrap_or(0)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/runs/quality", get(run_quality))
}

async fn run_quality(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let _bot = params.get("bot_id").cloned().unwrap_or_default();
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            return Json(serde_json::json!({
                "error": "database unavailable",
                "detail": error.to_string(),
            }))
        }
    };

    let frames = count(&mut conn, "SELECT count(*)::bigint AS count FROM run_events");
    let ui_frames = count(
        &mut conn,
        "SELECT count(*)::bigint AS count FROM run_events WHERE event_type = 'CUSTOM' AND payload->>'name' LIKE 'gb.%'",
    );
    let questions = count(
        &mut conn,
        "SELECT count(*)::bigint AS count FROM run_events WHERE event_type = 'CUSTOM' AND payload->>'name' = 'gb.question'",
    );
    let apps = count(
        &mut conn,
        "SELECT count(*)::bigint AS count FROM run_events WHERE event_type = 'CUSTOM' AND payload->>'name' = 'gb.app'",
    );
    let consent_denials = count(
        &mut conn,
        "SELECT count(*)::bigint AS count FROM consent_audit WHERE outcome = 'denied' AND created_at > NOW() - INTERVAL '24 hours'",
    );
    let sandbox_errors = count(
        &mut conn,
        "SELECT count(*)::bigint AS count FROM run_events WHERE payload->>'code' LIKE '%sandbox%'",
    );
    let runs = count(
        &mut conn,
        "SELECT count(DISTINCT run_id)::bigint AS count FROM run_events",
    );
    let token_in = count(
        &mut conn,
        "SELECT COALESCE(sum(tokens_in), 0)::bigint AS count FROM agent_spans",
    );
    let token_out = count(
        &mut conn,
        "SELECT COALESCE(sum(tokens_out), 0)::bigint AS count FROM agent_spans",
    );
    let per_run = if runs > 0 {
        ui_frames as f64 / runs as f64
    } else {
        0.0
    };

    Json(serde_json::json!({
        "frames": frames,
        "ui_frames": ui_frames,
        "question_frames": questions,
        "app_frames": apps,
        "runs": runs,
        "ui_frames_per_run": per_run,
        "consent_denials_24h": consent_denials,
        "sandbox_errors": sandbox_errors,
        "tokens_in": token_in,
        "tokens_out": token_out,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}
