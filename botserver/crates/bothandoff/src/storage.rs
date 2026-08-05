use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

/// Reconciles the crate's expected columns with the migration-owned tables.
///
/// The migrations (6.3.3-01-handoff, 6.5.11-chatbot-handoff) create richer
/// enterprise tables (bot_id, user_id, topic, ...). This crate's UI queries
/// extra display columns (user_name, channel, waiting_since), so we add them
/// idempotently instead of attempting to CREATE the table (which no-ops once
/// the migration table exists).
pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS handoff_queue (
            id UUID PRIMARY KEY,
            bot_id UUID,
            session_id UUID,
            user_id UUID,
            topic TEXT NOT NULL DEFAULT '',
            reason TEXT,
            priority TEXT NOT NULL DEFAULT 'normal',
            status TEXT NOT NULL DEFAULT 'queued',
            agent_id UUID,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    // Migration-owned table may already exist; add the display columns this
    // crate's handlers query.
    diesel::sql_query(
        "ALTER TABLE handoff_queue ADD COLUMN IF NOT EXISTS user_name TEXT NOT NULL DEFAULT '',
         ADD COLUMN IF NOT EXISTS channel VARCHAR(50) NOT NULL DEFAULT 'chat',
         ADD COLUMN IF NOT EXISTS waiting_since TIMESTAMPTZ NOT NULL DEFAULT NOW()",
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
