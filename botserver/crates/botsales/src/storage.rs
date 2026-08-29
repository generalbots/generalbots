use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    for sql in [
        "CREATE TABLE IF NOT EXISTS sales_deals (
            id UUID PRIMARY KEY, title TEXT NOT NULL, contact_id UUID NOT NULL DEFAULT gen_random_uuid(),
            value NUMERIC(18,2) NOT NULL DEFAULT 0, stage VARCHAR(50) NOT NULL DEFAULT 'lead',
            status VARCHAR(30) NOT NULL DEFAULT 'open', probability NUMERIC(5,2) NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), closed_at TIMESTAMPTZ,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS sales_contacts (
            id UUID PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL DEFAULT '',
            phone TEXT NOT NULL DEFAULT '', company TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS sales_activities (
            id UUID PRIMARY KEY, deal_id UUID NOT NULL, kind VARCHAR(50) NOT NULL DEFAULT 'call',
            description TEXT NOT NULL DEFAULT '', activity_date DATE NOT NULL DEFAULT CURRENT_DATE,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS sales_leads (
            id UUID PRIMARY KEY, name TEXT NOT NULL DEFAULT '', company TEXT NOT NULL DEFAULT '',
            source TEXT NOT NULL DEFAULT '', score INTEGER NOT NULL DEFAULT 50,
            status VARCHAR(30) NOT NULL DEFAULT 'new', owner TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS sales_quotes (
            id UUID PRIMARY KEY, quote_number TEXT NOT NULL DEFAULT '', title TEXT NOT NULL DEFAULT '',
            customer TEXT NOT NULL DEFAULT '', amount NUMERIC(18,2) NOT NULL DEFAULT 0,
            valid_until DATE, status VARCHAR(30) NOT NULL DEFAULT 'draft',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS sales_orders (
            id UUID PRIMARY KEY, order_number TEXT NOT NULL DEFAULT '', customer TEXT NOT NULL DEFAULT '',
            items INTEGER NOT NULL DEFAULT 0, total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending', delivery TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}
