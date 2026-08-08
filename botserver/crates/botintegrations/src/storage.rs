use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS integrations_connectors (
            id UUID PRIMARY KEY, name TEXT NOT NULL, kind VARCHAR(50) NOT NULL DEFAULT 'api',
            endpoint TEXT NOT NULL DEFAULT '', status VARCHAR(30) NOT NULL DEFAULT 'disconnected',
            config JSONB, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS integrations_etl_jobs (
            id UUID PRIMARY KEY, name TEXT NOT NULL, source TEXT NOT NULL DEFAULT '',
            target TEXT NOT NULL DEFAULT '', schedule TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'inactive', last_run TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(())
}
