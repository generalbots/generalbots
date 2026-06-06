use axum::http::StatusCode;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS itsm_incidents (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            severity VARCHAR(20) NOT NULL DEFAULT 'medium',
            status VARCHAR(30) NOT NULL DEFAULT 'open',
            assignee TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            resolved_at TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS itsm_service_requests (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            category VARCHAR(50) NOT NULL DEFAULT 'general',
            status VARCHAR(30) NOT NULL DEFAULT 'open',
            requester TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS itsm_cmdb (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind VARCHAR(50) NOT NULL DEFAULT 'service',
            owner TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'active',
            dependencies JSONB NOT NULL DEFAULT '[]'::jsonb
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS itsm_kb (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            tags JSONB NOT NULL DEFAULT '[]'::jsonb,
            author TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;

    Ok(())
}
