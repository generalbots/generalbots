use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    for sql in [
        "CREATE TABLE IF NOT EXISTS hr_employees (
            id UUID PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL DEFAULT '',
            department VARCHAR(100) NOT NULL DEFAULT '', role VARCHAR(100) NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'active', hired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS hr_recruitment (
            id UUID PRIMARY KEY, position TEXT NOT NULL, department VARCHAR(100) NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'open', candidates BIGINT NOT NULL DEFAULT 0,
            opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
        "CREATE TABLE IF NOT EXISTS hr_attendance (
            id UUID PRIMARY KEY, employee_id UUID NOT NULL, date DATE NOT NULL,
            clock_in TIMESTAMPTZ NOT NULL, clock_out TIMESTAMPTZ,
            hours_worked NUMERIC(8,2) NOT NULL DEFAULT 0)",
        "CREATE TABLE IF NOT EXISTS hr_review_cycles (
            id UUID PRIMARY KEY, name TEXT NOT NULL, start_date DATE NOT NULL,
            end_date DATE NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'draft',
            completed BIGINT NOT NULL DEFAULT 0, total BIGINT NOT NULL DEFAULT 0)",
        "CREATE TABLE IF NOT EXISTS hr_goals (
            id UUID PRIMARY KEY, employee_id UUID NOT NULL, title TEXT NOT NULL,
            completion INTEGER NOT NULL DEFAULT 0, due_date DATE NOT NULL)",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}
