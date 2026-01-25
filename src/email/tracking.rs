use crate::shared::state::AppState;
use super::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{debug, info, warn};
use std::sync::Arc;
use uuid::Uuid;

const TRACKING_PIXEL: [u8; 43] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];

pub fn is_tracking_pixel_enabled(state: &Arc<AppState>, bot_id: Option<Uuid>) -> bool {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let bot_id = bot_id.unwrap_or(Uuid::nil());

    config_manager
        .get_config(&bot_id, "email-read-pixel", Some("false"))
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

pub fn inject_tracking_pixel(html_body: &str, tracking_id: &str, state: &Arc<AppState>) -> String {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
    let base_url = config_manager
        .get_config(&Uuid::nil(), "server-url", Some("http://localhost:8080"))
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    let pixel_url = format!("{}/api/email/tracking/pixel/{}", base_url, tracking_id);
    let pixel_html = format!(
        r#"<img src="{}" width="1" height="1" style="display:none;visibility:hidden;width:1px;height:1px;border:0;" alt="" />"#,
        pixel_url
    );

    if html_body.to_lowercase().contains("</body>") {
        html_body
            .replace("</body>", &format!("{}</body>", pixel_html))
            .replace("</BODY>", &format!("{}</BODY>", pixel_html))
    } else {
        format!("{}{}", html_body, pixel_html)
    }
}

pub fn save_email_tracking_record(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
    account_id: Uuid,
    bot_id: Uuid,
    from_email: &str,
    to_email: &str,
    cc: Option<&str>,
    bcc: Option<&str>,
    subject: &str,
) -> Result<(), String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        "INSERT INTO sent_email_tracking
           (id, tracking_id, bot_id, account_id, from_email, to_email, cc, bcc, subject, sent_at, read_count, is_read)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, false)"
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(account_id)
    .bind::<diesel::sql_types::Text, _>(from_email)
    .bind::<diesel::sql_types::Text, _>(to_email)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(cc)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(bcc)
    .bind::<diesel::sql_types::Text, _>(subject)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut db_conn)
    .map_err(|e| format!("Failed to save tracking record: {}", e))?;

    debug!("Saved email tracking record: tracking_id={}", tracking_id);
    Ok(())
}

pub async fn serve_tracking_pixel(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(_query): Query<TrackingPixelQuery>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Ok(tracking_uuid) = Uuid::parse_str(&tracking_id) {
        let conn = state.conn.clone();
        let ip_clone = client_ip.clone();
        let ua_clone = user_agent.clone();

        let _ = tokio::task::spawn_blocking(move || {
            update_email_read_status(conn, tracking_uuid, ip_clone, ua_clone)
        })
        .await;

        info!(
            "Email read tracked: tracking_id={}, ip={:?}",
            tracking_id, client_ip
        );
    } else {
        warn!("Invalid tracking ID received: {}", tracking_id);
    }

    (
        StatusCode::OK,
        [
            ("content-type", "image/gif"),
            (
                "cache-control",
                "no-store, no-cache, must-revalidate, max-age=0",
            ),
            ("pragma", "no-cache"),
            ("expires", "0"),
        ],
        TRACKING_PIXEL.to_vec(),
    )
}

fn update_email_read_status(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
    client_ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;
    let now = Utc::now();

    diesel::sql_query(
        r"UPDATE sent_email_tracking
           SET
               is_read = true,
               read_count = read_count + 1,
               read_at = COALESCE(read_at, $2),
               first_read_ip = COALESCE(first_read_ip, $3),
               last_read_ip = $3,
               user_agent = COALESCE(user_agent, $4),
               updated_at = $2
           WHERE tracking_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(client_ip.as_deref())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_agent.as_deref())
    .execute(&mut db_conn)
    .map_err(|e| format!("Failed to update tracking record: {}", e))?;

    debug!("Updated email read status: tracking_id={}", tracking_id);
    Ok(())
}

pub async fn get_tracking_status(
    Path(tracking_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatusResponse>>, EmailError> {
    let tracking_uuid =
        Uuid::parse_str(&tracking_id).map_err(|_| EmailError("Invalid tracking ID".to_string()))?;

    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || get_tracking_record(conn, tracking_uuid))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn get_tracking_record(
    conn: crate::shared::utils::DbPool,
    tracking_id: Uuid,
) -> Result<TrackingStatusResponse, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    #[derive(QueryableByName)]
    struct TrackingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tracking_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        to_email: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        subject: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        sent_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_read: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        read_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        read_count: i32,
    }

    let row: TrackingRow = diesel::sql_query(
        r"SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
           FROM sent_email_tracking WHERE tracking_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(tracking_id)
    .get_result(&mut db_conn)
    .map_err(|e| format!("Tracking record not found: {}", e))?;

    Ok(TrackingStatusResponse {
        tracking_id: row.tracking_id.to_string(),
        to_email: row.to_email,
        subject: row.subject,
        sent_at: row.sent_at.to_rfc3339(),
        is_read: row.is_read,
        read_at: row.read_at.map(|dt| dt.to_rfc3339()),
        read_count: row.read_count,
    })
}

pub async fn list_sent_emails_tracking(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTrackingQuery>,
) -> Result<Json<ApiResponse<Vec<TrackingStatusResponse>>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || list_tracking_records(conn, query))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn list_tracking_records(
    conn: crate::shared::utils::DbPool,
    query: ListTrackingQuery,
) -> Result<Vec<TrackingStatusResponse>, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    #[derive(QueryableByName)]
    struct TrackingRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        tracking_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        to_email: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        subject: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        sent_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_read: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        read_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        read_count: i32,
    }

    let base_query = match query.filter.as_deref() {
        Some("read") => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1 AND is_read = true
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
        }
        Some("unread") => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1 AND is_read = false
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
        }
        _ => {
            "SELECT tracking_id, to_email, subject, sent_at, is_read, read_at, read_count
               FROM sent_email_tracking WHERE account_id = $1
               ORDER BY sent_at DESC LIMIT $2 OFFSET $3"
        }
    };

    let rows: Vec<TrackingRow> = diesel::sql_query(base_query)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .load(&mut db_conn)
        .map_err(|e| format!("Query failed: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| TrackingStatusResponse {
            tracking_id: row.tracking_id.to_string(),
            to_email: row.to_email,
            subject: row.subject,
            sent_at: row.sent_at.to_rfc3339(),
            is_read: row.is_read,
            read_at: row.read_at.map(|dt| dt.to_rfc3339()),
            read_count: row.read_count,
        })
        .collect())
}

pub async fn get_tracking_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<TrackingStatsResponse>>, EmailError> {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || calculate_tracking_stats(conn))
        .await
        .map_err(|e| EmailError(format!("Task join error: {}", e)))?
        .map_err(EmailError)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: None,
    }))
}

fn calculate_tracking_stats(
    conn: crate::shared::utils::DbPool,
) -> Result<TrackingStatsResponse, String> {
    let mut db_conn = conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    #[derive(QueryableByName)]
    struct StatsRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_sent: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_read: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
        avg_time_hours: Option<f64>,
    }

    let stats: StatsRow = diesel::sql_query(
        r"SELECT
               COUNT(*) as total_sent,
               COUNT(*) FILTER (WHERE is_read = true) as total_read,
               AVG(EXTRACT(EPOCH FROM (read_at - sent_at)) / 3600) FILTER (WHERE is_read = true) as avg_time_hours
           FROM sent_email_tracking",
    )
    .get_result(&mut db_conn)
    .map_err(|e| format!("Stats query failed: {}", e))?;

    let read_rate = if stats.total_sent > 0 {
        (stats.total_read as f64 / stats.total_sent as f64) * 100.0
    } else {
        0.0
    };

    Ok(TrackingStatsResponse {
        total_sent: stats.total_sent,
        total_read: stats.total_read,
        read_rate,
        avg_time_to_read_hours: stats.avg_time_hours,
    })
}

pub fn get_emails(Path(campaign_id): Path<String>, State(_state): State<Arc<AppState>>) -> String {
    info!("Get emails requested for campaign: {campaign_id}");
    "No emails tracked".to_string()
}

pub async fn track_click(
    Path((campaign_id, email)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, EmailError> {
    info!("Click tracked - Campaign: {}, Email: {}", campaign_id, email);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Click tracked successfully"
    })))
}

pub async fn get_latest_email(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, EmailError> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Please use the new /api/email/list endpoint with account_id"
    })))
}

pub async fn get_email(
    Path(campaign_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, EmailError> {
    Ok(Json(serde_json::json!({
        "success": false,
        "message": "Please use the new /api/email/list endpoint with account_id"
    })))
}
