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
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_accounts (
            id UUID PRIMARY KEY,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            bank TEXT NOT NULL,
            agency TEXT NOT NULL DEFAULT '',
            account_number TEXT NOT NULL DEFAULT '',
            account_type TEXT NOT NULL DEFAULT 'checking',
            balance NUMERIC(18,4) NOT NULL DEFAULT 0,
            currency VARCHAR(8) NOT NULL DEFAULT 'BRL',
            last_sync TIMESTAMPTZ,
            status VARCHAR(30) NOT NULL DEFAULT 'active'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_pix_transfers (
            id UUID PRIMARY KEY,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            direction VARCHAR(8) NOT NULL DEFAULT 'out',
            key_type TEXT NOT NULL DEFAULT 'cpf',
            key_value TEXT NOT NULL DEFAULT '',
            counterparty TEXT NOT NULL DEFAULT '',
            amount NUMERIC(18,4) NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'completed',
            end_to_end_id TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_statements (
            id UUID PRIMARY KEY,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            account_label TEXT NOT NULL DEFAULT '',
            period TEXT NOT NULL DEFAULT '',
            opening NUMERIC(18,4) NOT NULL DEFAULT 0,
            closing NUMERIC(18,4) NOT NULL DEFAULT 0,
            generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            format VARCHAR(12) NOT NULL DEFAULT 'pdf'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_settings (
            branch_id UUID PRIMARY KEY,
            tolerance_cents INTEGER NOT NULL DEFAULT 1,
            date_window_days INTEGER NOT NULL DEFAULT 3,
            auto_approve_under NUMERIC(18,4) NOT NULL DEFAULT 500,
            notify_on_unmatched BOOLEAN NOT NULL DEFAULT true,
            webhook TEXT NOT NULL DEFAULT '',
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    Ok(())
}
