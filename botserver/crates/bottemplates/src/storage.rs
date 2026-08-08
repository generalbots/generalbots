use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS app_templates (
            id UUID PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
            kind VARCHAR(50) NOT NULL DEFAULT 'app', version TEXT NOT NULL DEFAULT '1.0',
            author TEXT NOT NULL DEFAULT '', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS app_template_deploys (
            id UUID PRIMARY KEY, template_id UUID NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'deployed',
            target TEXT NOT NULL DEFAULT 'production', deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(())
}