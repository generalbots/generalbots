use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

/// Reconciles the crate's expected columns with the migration-owned tables.
///
/// The migrations (6.5.08-kyc-identity, 6.5.15.1-consolidated) create richer
/// enterprise tables (bot_id, profile_id, workflow_name, ...). This crate's
/// UI queries extra columns (user_id, kind, documents, reviewed_by), so we
/// add them idempotently instead of attempting to CREATE the table (which
/// no-ops once the migration table exists).
pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS identity_kyc_workflows (
            id UUID PRIMARY KEY,
            bot_id UUID,
            profile_id UUID,
            workflow_name VARCHAR(100) NOT NULL DEFAULT '',
            current_step VARCHAR(100) NOT NULL DEFAULT '',
            steps_completed JSONB NOT NULL DEFAULT '[]'::jsonb,
            total_steps INTEGER NOT NULL DEFAULT 1,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    // Migration-owned table may already exist; add the columns this crate's
    // handlers query.
    diesel::sql_query(
        "ALTER TABLE identity_kyc_workflows ADD COLUMN IF NOT EXISTS user_id UUID,
         ADD COLUMN IF NOT EXISTS kind VARCHAR(50) NOT NULL DEFAULT 'identity',
         ADD COLUMN IF NOT EXISTS documents JSONB NOT NULL DEFAULT '[]'::jsonb,
         ADD COLUMN IF NOT EXISTS reviewed_by UUID,
         ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
         ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS identity_signatures (
            id UUID PRIMARY KEY,
            bot_id UUID,
            profile_id UUID,
            document_id UUID NOT NULL,
            signature_data TEXT NOT NULL DEFAULT '',
            signature_image_url TEXT,
            ip_address VARCHAR(64),
            user_agent TEXT,
            signed_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    // Migration table may already exist; add this crate's display columns.
    diesel::sql_query(
        "ALTER TABLE identity_signatures ADD COLUMN IF NOT EXISTS signer_name TEXT NOT NULL DEFAULT '',
         ADD COLUMN IF NOT EXISTS signer_email TEXT NOT NULL DEFAULT '',
         ADD COLUMN IF NOT EXISTS status VARCHAR(30) NOT NULL DEFAULT 'pending',
         ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
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
            status VARCHAR(30) NOT NULL DEFAULT 'active',
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}
