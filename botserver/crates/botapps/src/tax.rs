use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;

use crate::db;
use crate::tax_storage::{ensure_schema_sync, parse_decimal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFe {
    pub id: Uuid,
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub authorized_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFSe {
    pub id: Uuid,
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTe {
    pub id: Uuid,
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sped {
    pub id: Uuid,
    pub period: String,
    pub kind: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewNFe {
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct NewNFSe {
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCTe {
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: String,
}

pub async fn list_nfe() -> Result<Json<Vec<NFe>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] number: String,
        #[diesel(sql_type = diesel::sql_types::Text)] series: String,
        #[diesel(sql_type = diesel::sql_types::Text)] emitter_cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Text)] recipient_cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] authorized_at: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, number, series, emitter_cnpj, recipient_cnpj, total, status, created_at, authorized_at
         FROM brazil_nfe ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| NFe {
        id: r.id, number: r.number, series: r.series, emitter_cnpj: r.emitter_cnpj,
        recipient_cnpj: r.recipient_cnpj, total: r.total.to_string(), status: r.status,
        created_at: r.created_at, authorized_at: r.authorized_at,
    }).collect()))
}

pub async fn create_nfe(Json(req): Json<NewNFe>) -> Result<Json<NFe>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let total = parse_decimal(&req.total)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO brazil_nfe (id, number, series, emitter_cnpj, recipient_cnpj, total, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.number)
    .bind::<diesel::sql_types::Text, _>(&req.series)
    .bind::<diesel::sql_types::Text, _>(&req.emitter_cnpj)
    .bind::<diesel::sql_types::Text, _>(&req.recipient_cnpj)
    .bind::<diesel::sql_types::Numeric, _>(total)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(NFe {
        id, number: req.number, series: req.series, emitter_cnpj: req.emitter_cnpj,
        recipient_cnpj: req.recipient_cnpj, total: total.to_string(),
        status: "pending".to_string(), created_at: now, authorized_at: None,
    }))
}

pub async fn authorize_nfe(Path(id): Path<String>) -> Result<Json<NFe>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let now = Utc::now();
    let n = diesel::sql_query(
        "UPDATE brazil_nfe SET status = 'authorized', authorized_at = $1 WHERE id = $2",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, format!("NFe {id} not found")));
    }
    Ok(Json(NFe {
        id: parsed, number: String::new(), series: String::new(), emitter_cnpj: String::new(),
        recipient_cnpj: String::new(), total: "0".to_string(),
        status: "authorized".to_string(), created_at: now, authorized_at: Some(now),
    }))
}

pub async fn list_nfse() -> Result<Json<Vec<NFSe>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] number: String,
        #[diesel(sql_type = diesel::sql_types::Text)] service_code: String,
        #[diesel(sql_type = diesel::sql_types::Text)] provider_cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, number, service_code, provider_cnpj, total, status, created_at
         FROM brazil_nfse ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| NFSe {
        id: r.id, number: r.number, service_code: r.service_code, provider_cnpj: r.provider_cnpj,
        total: r.total.to_string(), status: r.status, created_at: r.created_at,
    }).collect()))
}

pub async fn create_nfse(Json(req): Json<NewNFSe>) -> Result<Json<NFSe>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let total = parse_decimal(&req.total)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO brazil_nfse (id, number, service_code, provider_cnpj, total, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.number)
    .bind::<diesel::sql_types::Text, _>(&req.service_code)
    .bind::<diesel::sql_types::Text, _>(&req.provider_cnpj)
    .bind::<diesel::sql_types::Numeric, _>(total)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(NFSe {
        id, number: req.number, service_code: req.service_code, provider_cnpj: req.provider_cnpj,
        total: total.to_string(), status: "pending".to_string(), created_at: now,
    }))
}

pub async fn list_cte() -> Result<Json<Vec<CTe>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] number: String,
        #[diesel(sql_type = diesel::sql_types::Text)] sender_cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Text)] recipient_cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Text)] modality: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, number, sender_cnpj, recipient_cnpj, modality, total, status, created_at
         FROM brazil_cte ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| CTe {
        id: r.id, number: r.number, sender_cnpj: r.sender_cnpj, recipient_cnpj: r.recipient_cnpj,
        modality: r.modality, total: r.total.to_string(), status: r.status, created_at: r.created_at,
    }).collect()))
}

pub async fn create_cte(Json(req): Json<NewCTe>) -> Result<Json<CTe>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let total = parse_decimal(&req.total)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO brazil_cte (id, number, sender_cnpj, recipient_cnpj, modality, total, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.number)
    .bind::<diesel::sql_types::Text, _>(&req.sender_cnpj)
    .bind::<diesel::sql_types::Text, _>(&req.recipient_cnpj)
    .bind::<diesel::sql_types::Text, _>(&req.modality)
    .bind::<diesel::sql_types::Numeric, _>(total)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(CTe {
        id, number: req.number, sender_cnpj: req.sender_cnpj, recipient_cnpj: req.recipient_cnpj,
        modality: req.modality, total: total.to_string(), status: "pending".to_string(), created_at: now,
    }))
}

pub async fn list_sped() -> Result<Json<Vec<Sped>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] period: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, period, kind, status, created_at FROM brazil_sped ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Sped {
        id: r.id, period: r.period, kind: r.kind, status: r.status, created_at: r.created_at,
    }).collect()))
}
