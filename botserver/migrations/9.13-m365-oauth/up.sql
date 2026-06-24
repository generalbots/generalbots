DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS oauth_microsoft_settings (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        tenant_id TEXT NOT NULL,
        client_id TEXT NOT NULL,
        client_secret_encrypted TEXT,
        redirect_uri TEXT,
        user_principal_name TEXT,
        connected_at TIMESTAMPTZ,
        last_sync TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating oauth_microsoft_settings table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS oauth_microsoft_tokens (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        user_id UUID NOT NULL,
        access_token TEXT NOT NULL,
        refresh_token TEXT,
        expires_at TIMESTAMPTZ NOT NULL,
        scope TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_m365_tokens_user ON oauth_microsoft_tokens(user_id);
    CREATE INDEX IF NOT EXISTS idx_m365_tokens_expiry ON oauth_microsoft_tokens(expires_at);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating oauth_microsoft_tokens table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS oauth_microsoft_states (
        token TEXT PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        consumed_at TIMESTAMPTZ
    );

    CREATE INDEX IF NOT EXISTS idx_m365_states_created ON oauth_microsoft_states(created_at);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating oauth_microsoft_states table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS m365_sharepoint_items (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        site_id TEXT NOT NULL,
        list_id TEXT,
        item_id TEXT,
        title TEXT,
        fields JSONB NOT NULL DEFAULT '{}',
        author TEXT,
        modified_at TIMESTAMPTZ,
        synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_sp_items_site ON m365_sharepoint_items(site_id);
    CREATE INDEX IF NOT EXISTS idx_sp_items_list ON m365_sharepoint_items(list_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating m365_sharepoint_items table: %', SQLERRM;
END $$;
