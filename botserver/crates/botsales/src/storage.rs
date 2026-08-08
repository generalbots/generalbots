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
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}
