use axum::http::StatusCode;
use diesel::RunQueryDsl;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = crate::db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_sharepoint_items (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            site_name TEXT NOT NULL,
            list_name TEXT NOT NULL,
            item_count BIGINT NOT NULL DEFAULT 0,
            last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_calendar_events (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            subject TEXT NOT NULL,
            start_time TIMESTAMPTZ NOT NULL,
            end_time TIMESTAMPTZ NOT NULL,
            location TEXT,
            attendees JSONB NOT NULL DEFAULT '[]'::jsonb,
            status VARCHAR(30) NOT NULL DEFAULT 'confirmed'
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_onedrive_files (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size_bytes BIGINT NOT NULL DEFAULT 0,
            last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            author TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS oauth_microsoft_settings (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id TEXT NOT NULL DEFAULT '',
            client_id TEXT NOT NULL DEFAULT '',
            client_secret_encrypted TEXT,
            scopes JSONB NOT NULL DEFAULT '[]'::jsonb,
            connected BOOLEAN NOT NULL DEFAULT false,
            last_sync_at TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;
    Ok(())
}
