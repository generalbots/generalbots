use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS vision_analysis (
            id UUID PRIMARY KEY,
            image_url TEXT NOT NULL,
            kind VARCHAR(50) NOT NULL DEFAULT 'general',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            labels JSONB NOT NULL DEFAULT '[]'::jsonb,
            confidence NUMERIC(5,4) NOT NULL DEFAULT 0,
            parameters JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    ).execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(())
}
