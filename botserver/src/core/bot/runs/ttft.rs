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
