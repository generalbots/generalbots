//! Durable app-log endpoints (#829): persist client/server log entries to the
//! `autotask_app_logs` table so logs survive restarts. The in-memory
//! `AppLogStore` remains as the live cache for the designer error context;
//! every write also lands in PostgreSQL.

use crate::api::AutoTaskApi;
use crate::app_logs::{
    AppLogEntry, ClientLogRequest, LogLevel, LogQueryParams, LogSource, LogStats, APP_LOGS,
};
use crate::types::DbPool;
use axum::{extract::Query, response::IntoResponse, Json};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Jsonb, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use diesel::sql_query;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(diesel::QueryableByName)]
struct LogRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Timestamptz)]
    timestamp: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = Text)]
    level: String,
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Text)]
    app_name: String,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    bot_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    message: String,
    #[diesel(sql_type = Nullable<Text>)]
    details: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    file_path: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    line_number: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    stack_trace: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct StatsRow {
    #[diesel(sql_type = BigInt)]
    total_logs: i64,
    #[diesel(sql_type = BigInt)]
    errors: i64,
    #[diesel(sql_type = BigInt)]
    warnings: i64,
    #[diesel(sql_type = Jsonb)]
    by_app: serde_json::Value,
}

fn row_to_entry(row: LogRow) -> AppLogEntry {
    AppLogEntry {
        id: row.id,
        timestamp: row.timestamp,
        level: parse_level(&row.level),
        source: parse_source(&row.source),
        app_name: row.app_name,
        bot_id: row.bot_id,
        user_id: row.user_id,
        message: row.message,
        details: row.details,
        file_path: row.file_path,
        line_number: row.line_number.map(|n| n as u32),
        stack_trace: row.stack_trace,
    }
}

fn parse_level(raw: &str) -> LogLevel {
    match raw.to_lowercase().as_str() {
        "debug" => LogLevel::Debug,
        "warn" | "warning" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "critical" => LogLevel::Critical,
        _ => LogLevel::Info,
    }
}

fn parse_source(raw: &str) -> LogSource {
    match raw.to_lowercase().as_str() {
        "client" => LogSource::Client,
        "generator" => LogSource::Generator,
        "designer" => LogSource::Designer,
        "validation" => LogSource::Validation,
        "runtime" => LogSource::Runtime,
        _ => LogSource::Server,
    }
}

fn level_str(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
        LogLevel::Critical => "critical",
    }
}

fn source_str(source: LogSource) -> &'static str {
    match source {
        LogSource::Server => "server",
        LogSource::Client => "client",
        LogSource::Generator => "generator",
        LogSource::Designer => "designer",
        LogSource::Validation => "validation",
        LogSource::Runtime => "runtime",
    }
}

fn persist_entry(pool: &DbPool, entry: &AppLogEntry) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
    sql_query(
        "INSERT INTO autotask_app_logs \
         (timestamp, level, source, app_name, bot_id, user_id, message, details, file_path, line_number, stack_trace) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind::<Timestamptz, _>(entry.timestamp)
    .bind::<Text, _>(level_str(entry.level))
    .bind::<Text, _>(source_str(entry.source))
    .bind::<Text, _>(&entry.app_name)
    .bind::<Nullable<DieselUuid>, _>(entry.bot_id)
    .bind::<Nullable<DieselUuid>, _>(entry.user_id)
    .bind::<Text, _>(&entry.message)
    .bind::<Nullable<Text>, _>(entry.details.as_deref())
    .bind::<Nullable<Text>, _>(entry.file_path.as_deref())
    .bind::<Nullable<BigInt>, _>(entry.line_number.map(i64::from))
    .bind::<Nullable<Text>, _>(entry.stack_trace.as_deref())
    .execute(&mut conn)
    .map_err(|e| format!("Failed to persist app log: {e}"))?;
    Ok(())
}

/// POST /api/app-logs/client — durable client log ingestion.
pub async fn client_log(
    axum::extract::State(api): axum::extract::State<Arc<AutoTaskApi>>,
    Json(request): Json<ClientLogRequest>,
) -> impl IntoResponse {
    let pool = api.state().db_pool().clone();
    let entry = AppLogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        level: parse_level(&request.level),
        source: LogSource::Client,
        app_name: request.app_name.clone(),
        bot_id: None,
        user_id: None,
        message: request.message.clone(),
        details: request.details.clone(),
        file_path: request.file_path.clone(),
        line_number: request.line_number,
        stack_trace: request.stack_trace.clone(),
    };
    // Keep the live cache in sync for the designer error context.
    APP_LOGS.log_client(request, None, None);
    let result = tokio::task::spawn_blocking(move || persist_entry(&pool, &entry)).await;
    match result {
        Ok(Ok(())) => Json(serde_json::json!({ "success": true })),
        Ok(Err(e)) => {
            log::error!("client_log persist failed: {e}");
            Json(serde_json::json!({ "success": false, "error": e }))
        }
        Err(e) => {
            log::error!("client_log task panicked: {e}");
            Json(serde_json::json!({ "success": false, "error": "log persistence failed" }))
        }
    }
}

/// GET /api/app-logs — durable list, filtered by app/level/source/since.
pub async fn list_logs(
    axum::extract::State(api): axum::extract::State<Arc<AutoTaskApi>>,
    Query(params): Query<LogQueryParams>,
) -> impl IntoResponse {
    let pool = api.state().db_pool().clone();
    let app_name = params.app_name.clone();
    let level = params.level.clone();
    let source = params.source.clone();
    let since = params.since;
    let limit = params.limit.unwrap_or(100).min(500) as i64;

    let rows = tokio::task::spawn_blocking(move || -> Result<Vec<LogRow>, String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        sql_query(
            "SELECT id::text AS id, timestamp, level, source, app_name, bot_id, user_id, \
                    message, details, file_path, line_number, stack_trace \
             FROM autotask_app_logs \
             WHERE ($1::text IS NULL OR app_name = $1) \
               AND ($2::text IS NULL OR level = $2) \
               AND ($3::text IS NULL OR source = $3) \
               AND ($4::timestamptz IS NULL OR timestamp >= $4) \
             ORDER BY timestamp DESC \
             LIMIT $5",
        )
        .bind::<Nullable<Text>, _>(app_name.as_deref())
        .bind::<Nullable<Text>, _>(level.as_deref())
        .bind::<Nullable<Text>, _>(source.as_deref())
        .bind::<Nullable<Timestamptz>, _>(since)
        .bind::<BigInt, _>(limit)
        .load::<LogRow>(&mut conn)
        .map_err(|e| format!("Failed to list app logs: {e}"))
    })
    .await;

    match rows {
        Ok(Ok(rows)) => {
            let entries: Vec<AppLogEntry> = rows.into_iter().map(row_to_entry).collect();
            Json(serde_json::json!({ "success": true, "logs": entries }))
        }
        Ok(Err(e)) => {
            log::error!("list_logs failed: {e}");
            Json(serde_json::json!({ "success": false, "error": e }))
        }
        Err(e) => {
            log::error!("list_logs task panicked: {e}");
            Json(serde_json::json!({ "success": false, "error": "log query failed" }))
        }
    }
}

/// GET /api/app-logs/stats — durable aggregate counts.
pub async fn log_stats(
    axum::extract::State(api): axum::extract::State<Arc<AutoTaskApi>>,
) -> impl IntoResponse {
    let pool = api.state().db_pool().clone();
    let result = tokio::task::spawn_blocking(move || -> Result<LogStats, String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        let row: StatsRow = sql_query(
            "SELECT \
               (SELECT COUNT(*) FROM autotask_app_logs) AS total_logs, \
               (SELECT COUNT(*) FROM autotask_app_logs WHERE level IN ('error', 'critical')) AS errors, \
               (SELECT COUNT(*) FROM autotask_app_logs WHERE level = 'warn') AS warnings, \
               COALESCE((SELECT jsonb_object_agg(app_name, cnt) FROM \
                 (SELECT app_name, COUNT(*) AS cnt FROM autotask_app_logs GROUP BY app_name) s), '{}'::jsonb) AS by_app",
        )
        .get_result::<StatsRow>(&mut conn)
        .map_err(|e| format!("Failed to load log stats: {e}"))?;
        let by_app: HashMap<String, usize> = row
            .by_app
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            v.as_i64().unwrap_or(0).max(0) as usize,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(LogStats {
            total_logs: row.total_logs.max(0) as usize,
            errors: row.errors.max(0) as usize,
            warnings: row.warnings.max(0) as usize,
            by_app,
        })
    })
    .await;

    match result {
        Ok(Ok(stats)) => Json(serde_json::json!({ "success": true, "stats": stats })),
        Ok(Err(e)) => {
            log::error!("log_stats failed: {e}");
            Json(serde_json::json!({ "success": false, "error": e }))
        }
        Err(e) => {
            log::error!("log_stats task panicked: {e}");
            Json(serde_json::json!({ "success": false, "error": "log stats failed" }))
        }
    }
}

/// DELETE /api/app-logs/:app — clear durable logs for one app.
pub async fn clear_app_logs(
    axum::extract::State(api): axum::extract::State<Arc<AutoTaskApi>>,
    axum::extract::Path(app_name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let pool = api.state().db_pool().clone();
    APP_LOGS.clear_app_logs(&app_name);
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        sql_query("DELETE FROM autotask_app_logs WHERE app_name = $1")
            .bind::<Text, _>(&app_name)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to clear app logs: {e}"))?;
        info!("Cleared durable logs for app: {app_name}");
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => Json(serde_json::json!({ "success": true })),
        Ok(Err(e)) => {
            log::error!("clear_app_logs failed: {e}");
            Json(serde_json::json!({ "success": false, "error": e }))
        }
        Err(e) => {
            log::error!("clear_app_logs task panicked: {e}");
            Json(serde_json::json!({ "success": false, "error": "clear failed" }))
        }
    }
}
