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
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_recruitment (
            id UUID PRIMARY KEY, position TEXT NOT NULL, department VARCHAR(100) NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'open', candidates BIGINT NOT NULL DEFAULT 0,
            opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_attendance (
            id UUID PRIMARY KEY, employee_id UUID NOT NULL, date DATE NOT NULL,
            clock_in TIMESTAMPTZ NOT NULL, clock_out TIMESTAMPTZ,
            hours_worked NUMERIC(8,2) NOT NULL DEFAULT 0,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_review_cycles (
            id UUID PRIMARY KEY, name TEXT NOT NULL, start_date DATE NOT NULL,
            end_date DATE NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'draft',
            completed BIGINT NOT NULL DEFAULT 0, total BIGINT NOT NULL DEFAULT 0,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_goals (
            id UUID PRIMARY KEY, employee_id UUID NOT NULL, title TEXT NOT NULL,
            completion INTEGER NOT NULL DEFAULT 0, due_date DATE NOT NULL,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "ALTER TABLE hr_employees ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
        "ALTER TABLE hr_recruitment ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
        "ALTER TABLE hr_attendance ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
        "ALTER TABLE hr_review_cycles ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
        "ALTER TABLE hr_goals ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'",
        "CREATE TABLE IF NOT EXISTS hr_payroll_runs (
            id UUID PRIMARY KEY, period_label TEXT NOT NULL,
            employee_count BIGINT NOT NULL DEFAULT 0,
            gross NUMERIC(14,2) NOT NULL DEFAULT 0, net NUMERIC(14,2) NOT NULL DEFAULT 0,
            taxes NUMERIC(14,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'completed', run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_benefits (
            id UUID PRIMARY KEY, plan TEXT NOT NULL, provider TEXT NOT NULL DEFAULT '',
            type VARCHAR(50) NOT NULL DEFAULT '', enrolled BIGINT NOT NULL DEFAULT 0,
            monthly_cost NUMERIC(12,2) NOT NULL DEFAULT 0, status VARCHAR(30) NOT NULL DEFAULT 'active',
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS hr_training_courses (
            id UUID PRIMARY KEY, course TEXT NOT NULL, provider TEXT NOT NULL DEFAULT '',
            duration TEXT NOT NULL DEFAULT '', assigned BIGINT NOT NULL DEFAULT 0,
            completed BIGINT NOT NULL DEFAULT 0, status VARCHAR(30) NOT NULL DEFAULT 'open',
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}
