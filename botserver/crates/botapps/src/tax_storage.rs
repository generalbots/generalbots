use axum::http::StatusCode;
use rust_decimal::Decimal;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_nfe (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            series TEXT NOT NULL DEFAULT '1',
            emitter_cnpj TEXT NOT NULL DEFAULT '',
            recipient_cnpj TEXT NOT NULL DEFAULT '',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            authorized_at TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_nfse (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            service_code TEXT NOT NULL DEFAULT '',
            provider_cnpj TEXT NOT NULL DEFAULT '',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_cte (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            sender_cnpj TEXT NOT NULL DEFAULT '',
            recipient_cnpj TEXT NOT NULL DEFAULT '',
            modality TEXT NOT NULL DEFAULT 'normal',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_sped (
            id UUID PRIMARY KEY,
            period TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'fiscal',
            status VARCHAR(30) NOT NULL DEFAULT 'open',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

pub fn parse_decimal(s: &str) -> Result<Decimal, (StatusCode, String)> {
    s.parse::<Decimal>()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid decimal '{s}': {e}")))
}
