use axum::http::StatusCode;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS bot_database_queries (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL,
            query TEXT NOT NULL,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}
