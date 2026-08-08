use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS timeclock_events (
            id UUID PRIMARY KEY,
            employee_id UUID NOT NULL,
            kind VARCHAR(20) NOT NULL,
            ts TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            notes TEXT,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS timeclock_records (
            id UUID PRIMARY KEY,
            employee_id UUID NOT NULL,
            date DATE NOT NULL,
            clock_in TIMESTAMPTZ NOT NULL,
            clock_out TIMESTAMPTZ,
            hours_worked NUMERIC(8,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'open',
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS timeclock_overtime (
            id UUID PRIMARY KEY,
            employee_id UUID NOT NULL,
            date DATE NOT NULL,
            hours NUMERIC(8,2) NOT NULL DEFAULT 0,
            reason TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            approved_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS timeclock_reports (
            id UUID PRIMARY KEY,
            period TEXT NOT NULL,
            total_hours NUMERIC(10,2) NOT NULL DEFAULT 0,
            overtime_hours NUMERIC(10,2) NOT NULL DEFAULT 0,
            employees BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}
