use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS handoff_queue (
            id UUID PRIMARY KEY,
            session_id UUID NOT NULL,
            user_name TEXT NOT NULL DEFAULT '',
            channel VARCHAR(50) NOT NULL DEFAULT 'chat',
            priority VARCHAR(20) NOT NULL DEFAULT 'normal',
            waiting_since TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS conversation_analytics (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            period TEXT NOT NULL,
            total_transfers BIGINT NOT NULL DEFAULT 0,
            avg_wait_seconds BIGINT NOT NULL DEFAULT 0,
            avg_handle_seconds BIGINT NOT NULL DEFAULT 0,
            satisfaction_avg NUMERIC(4,2) NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS handoff_channels (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind VARCHAR(50) NOT NULL DEFAULT 'chat',
            status VARCHAR(30) NOT NULL DEFAULT 'active',
            active_agents BIGINT NOT NULL DEFAULT 0
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS conversation_ratings (
            id UUID PRIMARY KEY,
            session_id UUID NOT NULL,
            rating INTEGER NOT NULL,
            comment TEXT,
            submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}
