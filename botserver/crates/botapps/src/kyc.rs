use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub status: String,
    pub documents: Vec<String>,
    pub reviewed_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: Uuid,
    pub document_id: Uuid,
    pub signer_name: String,
    pub signer_email: String,
    pub status: String,
    pub signed_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Uuid,
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub valid_from: chrono::DateTime<Utc>,
    pub valid_until: chrono::DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVerification {
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub documents: Option<Vec<String>>,
}

fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS identity_kyc_workflows (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            kind VARCHAR(50) NOT NULL DEFAULT 'identity',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            documents JSONB NOT NULL DEFAULT '[]'::jsonb,
            reviewed_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS identity_signatures (
            id UUID PRIMARY KEY,
            document_id UUID NOT NULL,
            signer_name TEXT NOT NULL DEFAULT '',
            signer_email TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            signed_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS identity_certificates (
            id UUID PRIMARY KEY,
            subject TEXT NOT NULL,
            issuer TEXT NOT NULL DEFAULT '',
            serial TEXT NOT NULL DEFAULT '',
            valid_from TIMESTAMPTZ NOT NULL,
            valid_until TIMESTAMPTZ NOT NULL,
            status VARCHAR(30) NOT NULL DEFAULT 'active'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

pub async fn list_verifications() -> Result<Json<Vec<Verification>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] user_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] documents: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] reviewed_by: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] completed_at: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, user_id, kind, status, documents, reviewed_by, created_at, completed_at
         FROM identity_kyc_workflows ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Verification {
        id: r.id, user_id: r.user_id, kind: r.kind, status: r.status,
        documents: r.documents.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        reviewed_by: r.reviewed_by, created_at: r.created_at, completed_at: r.completed_at,
    }).collect()))
}

pub async fn update_verification(
    Path(id): Path<String>,
    Json(req): Json<UpdateVerification>,
) -> Result<Json<Verification>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let completed_at = if req.status == "approved" || req.status == "rejected" {
        Some(Utc::now())
    } else {
        None
    };
    let docs_json = req.documents.as_ref()
        .map(|d| serde_json::to_value(d).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize: {e}"))))
        .transpose()?
        .unwrap_or(serde_json::json!([]));
    diesel::sql_query(
        "UPDATE identity_kyc_workflows SET status = $1, reviewed_by = $2, documents = $3, completed_at = COALESCE($4, completed_at) WHERE id = $5",
    )
    .bind::<diesel::sql_types::Text, _>(&req.status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(req.reviewed_by)
    .bind::<diesel::sql_types::Jsonb, _>(&docs_json)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(completed_at)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Verification {
        id: parsed, user_id: Uuid::nil(), kind: "identity".to_string(), status: req.status,
        documents: req.documents.unwrap_or_default(), reviewed_by: req.reviewed_by,
        created_at: Utc::now(), completed_at,
    }))
}

pub async fn list_signatures() -> Result<Json<Vec<Signature>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] document_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] signer_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] signer_email: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] signed_at: Option<chrono::DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, document_id, signer_name, signer_email, status, signed_at, created_at
         FROM identity_signatures ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Signature {
        id: r.id, document_id: r.document_id, signer_name: r.signer_name, signer_email: r.signer_email,
        status: r.status, signed_at: r.signed_at, created_at: r.created_at,
    }).collect()))
}

pub async fn sign_document(Path(id): Path<String>) -> Result<Json<Signature>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let now = Utc::now();
    let n = diesel::sql_query(
        "UPDATE identity_signatures SET status = 'signed', signed_at = $1 WHERE id = $2",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Signature {id} not found")));
    }
    Ok(Json(Signature {
        id: parsed, document_id: Uuid::nil(), signer_name: String::new(), signer_email: String::new(),
        status: "signed".to_string(), signed_at: Some(now), created_at: now,
    }))
}

pub async fn list_certificates() -> Result<Json<Vec<Certificate>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] subject: String,
        #[diesel(sql_type = diesel::sql_types::Text)] issuer: String,
        #[diesel(sql_type = diesel::sql_types::Text)] serial: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] valid_from: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] valid_until: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, subject, issuer, serial, valid_from, valid_until, status
         FROM identity_certificates ORDER BY subject ASC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Certificate {
        id: r.id, subject: r.subject, issuer: r.issuer, serial: r.serial,
        valid_from: r.valid_from, valid_until: r.valid_until, status: r.status,
    }).collect()))
}
