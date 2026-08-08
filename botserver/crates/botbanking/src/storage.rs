use axum::http::StatusCode;
use diesel::RunQueryDsl;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = crate::db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_transactions (
            id UUID PRIMARY KEY,
            account_id UUID NOT NULL,
            kind VARCHAR(30) NOT NULL DEFAULT 'credit',
            amount NUMERIC(18,4) NOT NULL DEFAULT 0,
            currency VARCHAR(8) NOT NULL DEFAULT 'BRL',
            description TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_platforms (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'active',
            last_sync TIMESTAMPTZ,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_reconcile_results (
            id UUID PRIMARY KEY,
            period TEXT NOT NULL,
            matched BIGINT NOT NULL DEFAULT 0,
            unmatched BIGINT NOT NULL DEFAULT 0,
            total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'completed',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_reports (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT '',
            period TEXT NOT NULL DEFAULT '',
            url TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    Ok(())
}
