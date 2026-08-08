use axum::http::StatusCode;
use diesel::RunQueryDsl;
use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    for sql in [
        "CREATE TABLE IF NOT EXISTS minutes_meetings (
            id UUID PRIMARY KEY, title TEXT NOT NULL, meeting_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            duration_minutes BIGINT NOT NULL DEFAULT 0, participants JSONB NOT NULL DEFAULT '[]'::jsonb,
            status VARCHAR(30) NOT NULL DEFAULT 'scheduled', transcript_id UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS minutes_transcripts (
            id UUID PRIMARY KEY, meeting_id UUID NOT NULL, content TEXT NOT NULL DEFAULT '',
            language VARCHAR(10) NOT NULL DEFAULT 'pt-BR', word_count BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
        "CREATE TABLE IF NOT EXISTS minutes_documents (
            id UUID PRIMARY KEY, meeting_id UUID NOT NULL, title TEXT NOT NULL,
            kind VARCHAR(50) NOT NULL DEFAULT 'minutes', content TEXT NOT NULL DEFAULT '',
            version BIGINT NOT NULL DEFAULT 1, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    ] {
        diesel::sql_query(sql).execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(())
}