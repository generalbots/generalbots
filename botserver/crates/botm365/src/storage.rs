use axum::http::StatusCode;
use diesel::RunQueryDsl;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = crate::db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_sharepoint_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            bot_id UUID NOT NULL,
            organization_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            site_id TEXT NOT NULL,
            list_id TEXT,
            item_id TEXT,
            title TEXT,
            fields JSONB NOT NULL DEFAULT '{}'::jsonb,
            author TEXT,
            modified_at TIMESTAMPTZ,
            synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_calendar_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            bot_id UUID NOT NULL,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
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
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            bot_id UUID NOT NULL,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
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
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            tenant_id TEXT NOT NULL DEFAULT '',
            client_id TEXT NOT NULL DEFAULT '',
            client_secret_encrypted TEXT,
            redirect_uri TEXT,
            user_principal_name TEXT,
            connected_at TIMESTAMPTZ,
            last_sync TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(crate::db::map_diesel_err)?;

    Ok(())
}
