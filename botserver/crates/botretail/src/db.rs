//! Database pool access for the retail mutation handlers.

use axum::http::StatusCode;
use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use std::sync::OnceLock;

static POOL: OnceLock<Result<DbPool, String>> = OnceLock::new();

/// Returns the process-wide database pool, initializing it on first use.
pub fn pool() -> Result<&'static DbPool, (StatusCode, String)> {
    POOL.get_or_init(|| match botcore::shared::utils::create_conn() {
        Ok(p) => Ok(p),
        Err(e) => Err(format!("DB connection init failed: {e}")),
    })
    .as_ref()
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.clone()))
}

/// Maps a Diesel error into an HTTP status / message pair.
pub fn map_diesel_err(e: diesel::result::Error) -> (StatusCode, String) {
    match e {
        diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
        other => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {other}")),
    }
}

/// Creates the tables backing the retail create/edit flows.
pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    for sql in [
        "CREATE TABLE IF NOT EXISTS retail_branches (
            id UUID PRIMARY KEY, branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            code TEXT NOT NULL DEFAULT '', name TEXT NOT NULL DEFAULT '', address TEXT NOT NULL DEFAULT '',
            manager TEXT NOT NULL DEFAULT '', stock_value NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'active', pricing_rules JSONB NOT NULL DEFAULT '[]'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS retail_promotions (
            id UUID PRIMARY KEY, branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            name TEXT NOT NULL DEFAULT '', type TEXT NOT NULL DEFAULT '', discount TEXT NOT NULL DEFAULT '',
            valid_from DATE, valid_to DATE, status VARCHAR(30) NOT NULL DEFAULT 'active',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS retail_suppliers (
            id UUID PRIMARY KEY, branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            cnpj TEXT NOT NULL DEFAULT '', name TEXT NOT NULL DEFAULT '', contact TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '', lead_time_days INTEGER NOT NULL DEFAULT 0, rating NUMERIC(3,2) NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(map_diesel_err)?;
    }
    Ok(())
}
