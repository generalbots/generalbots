use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    for sql in [
        "CREATE TABLE IF NOT EXISTS erp_financial (
            id UUID PRIMARY KEY, kind VARCHAR(50) NOT NULL, description TEXT NOT NULL DEFAULT '',
            amount NUMERIC(18,2) NOT NULL DEFAULT 0, category VARCHAR(100) NOT NULL DEFAULT 'general',
            entry_date DATE NOT NULL DEFAULT CURRENT_DATE, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS erp_inventory (
            id UUID PRIMARY KEY, sku VARCHAR(100) NOT NULL UNIQUE, name TEXT NOT NULL,
            quantity BIGINT NOT NULL DEFAULT 0, unit_cost NUMERIC(18,2) NOT NULL DEFAULT 0,
            location VARCHAR(200) NOT NULL DEFAULT '', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS erp_procurement (
            id UUID PRIMARY KEY, supplier TEXT NOT NULL, items JSONB NOT NULL DEFAULT '[]'::jsonb,
            total NUMERIC(18,2) NOT NULL DEFAULT 0, status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS erp_branches (
            id UUID PRIMARY KEY, name TEXT NOT NULL, address TEXT NOT NULL DEFAULT '',
            manager TEXT NOT NULL DEFAULT '', active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}
