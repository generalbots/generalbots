use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
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
