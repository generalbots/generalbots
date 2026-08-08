use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS pos_products (
            id UUID PRIMARY KEY,
            sku TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            price NUMERIC(18,2) NOT NULL DEFAULT 0,
            stock BIGINT NOT NULL DEFAULT 0,
            category VARCHAR(100) NOT NULL DEFAULT 'general',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS pos_orders (
            id UUID PRIMARY KEY,
            items JSONB NOT NULL,
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'created',
            payment_method VARCHAR(50) NOT NULL DEFAULT 'cash',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}