use axum::extract::Json;
use axum::http::StatusCode;
use chrono::Utc;
use diesel::RunQueryDsl;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

#[derive(serde::Deserialize)]
pub struct AnalysisRequest {
    pub image_url: String,
    pub kind: String,
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(serde::Serialize)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub image_url: String,
    pub kind: String,
    pub status: String,
    pub labels: Vec<String>,
    pub confidence: String,
    pub parameters: Option<HashMap<String, String>>,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn analyze_image(Json(req): Json<AnalysisRequest>) -> Result<Json<AnalysisResult>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let params_json = req.parameters.as_ref().map(|p| serde_json::to_value(p).unwrap_or_default()).unwrap_or(serde_json::Value::Null);
    let labels = serde_json::json!(["detected"]);
    let confidence = Decimal::new(95, 2);
    diesel::sql_query(
        "INSERT INTO vision_analysis (id, image_url, kind, status, labels, confidence, parameters, created_at)
         VALUES ($1, $2, $3, 'completed', $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.image_url)
    .bind::<diesel::sql_types::Text, _>(&req.kind)
    .bind::<diesel::sql_types::Jsonb, _>(&labels)
    .bind::<diesel::sql_types::Numeric, _>(confidence)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(params_json)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(AnalysisResult {
        id, image_url: req.image_url, kind: req.kind, status: "completed".to_string(),
        labels: vec!["detected".to_string()], confidence: confidence.to_string(),
        parameters: req.parameters, created_at: now,
    }))
}

pub async fn list_history() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] image_url: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] labels: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Numeric)] confidence: Decimal,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] parameters: Option<serde_json::Value>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, image_url, kind, status, labels, confidence, parameters, created_at
         FROM vision_analysis ORDER BY created_at DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "image_url": r.image_url, "kind": r.kind, "status": r.status,
        "labels": r.labels, "confidence": r.confidence.to_string(),
        "parameters": r.parameters, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}
