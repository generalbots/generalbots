//! TTFT read endpoint (#1700).
//!
//! `GET /api/runs/ttft?hours=24` reads the `ttft_ms` samples the chat stream
//! writes into `llm_metrics` and returns the summary (count, min/max, mean,
//! p50, p95) plus an hourly trend. Statistics live in the tested pure module
//! `ttft_stats`, so the numbers the dashboard shows are covered by unit tests.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use botcore::shared::state::AppState;
use diesel::prelude::*;

use super::ttft_stats::{summarize, trend};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/runs/ttft", get(run_ttft))
}

/// Records one first-token sample for a chat turn (#1700).
///
/// Both streaming implementations must feed `llm_metrics`: the legacy
/// websocket loop (`ws/stream.rs`) and the pipeline (`pipeline/llm.rs`). This
/// helper keeps the row shape identical so the dashboard sees one series.
pub fn record_first_token(
    pool: &botcore::shared::utils::DbPool,
    session_id: uuid::Uuid,
    bot_uuid: uuid::Uuid,
    model: &str,
    ttft_ms: i64,
) {
    let pool = pool.clone();
    let model = model.to_string();
    tokio::task::spawn_blocking(move || {
        let Ok(mut conn) = pool.get() else { return };
        let _ = diesel::sql_query(
            "INSERT INTO llm_metrics (id, request_id, session_id, bot_id, model, request_type, input_tokens, output_tokens, total_tokens, latency_ms, ttft_ms, cached, success, estimated_cost, timestamp, metadata) \
             VALUES (gen_random_uuid(), gen_random_uuid(), $1, $2, $3, 'chat', 0, 0, 0, $4, $4, false, true, 0, NOW(), '{}'::jsonb)",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .bind::<diesel::sql_types::Text, _>(&model)
        .bind::<diesel::sql_types::BigInt, _>(ttft_ms)
        .execute(&mut conn);
    });
}

#[derive(Debug, diesel::QueryableByName)]
struct SampleRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    ttft_ms: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    bucket: String,
}

async fn run_ttft(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let hours = params
        .get("hours")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(24)
        .clamp(1, 24 * 30);
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "database unavailable",
                    "detail": error.to_string(),
                })),
            )
        }
    };
    let rows = diesel::sql_query(
        "SELECT ttft_ms, to_char(date_trunc('hour', timestamp), 'YYYY-MM-DD HH24:00') AS bucket \
         FROM llm_metrics \
         WHERE ttft_ms IS NOT NULL AND timestamp > NOW() - ($1 * INTERVAL '1 hour') \
         ORDER BY timestamp DESC LIMIT 5000",
    )
    .bind::<diesel::sql_types::Integer, _>(hours)
    .load::<SampleRow>(&mut conn)
    .unwrap_or_default();

    let samples: Vec<u64> = rows
        .iter()
        .filter_map(|row| row.ttft_ms)
        .filter_map(|value| u64::try_from(value).ok())
        .collect();
    let summary = summarize(&samples);
    let trend_rows: Vec<(String, u64)> = rows
        .iter()
        .filter_map(|row| {
            row.ttft_ms
                .and_then(|value| u64::try_from(value).ok())
                .map(|value| (row.bucket.clone(), value))
        })
        .collect();
    let buckets = trend(&trend_rows);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "hours": hours,
            "count": summary.count,
            "summary": {
                "count": summary.count,
                "min_ms": summary.min_ms,
                "max_ms": summary.max_ms,
                "avg_ms": summary.avg_ms,
                "p50_ms": summary.p50_ms,
                "p95_ms": summary.p95_ms,
            },
            "trend": buckets.iter().map(|bucket| serde_json::json!({
                "bucket": bucket.bucket,
                "count": bucket.count,
                "p50_ms": bucket.p50_ms,
                "p95_ms": bucket.p95_ms,
            })).collect::<Vec<_>>(),
        })),
    )
}
