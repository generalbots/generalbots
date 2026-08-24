CREATE TABLE IF NOT EXISTS connector_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    kind VARCHAR(32) NOT NULL,
    display_name VARCHAR(160),
    vault_token_ref VARCHAR(200) NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT 'connected',
    cursors JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connector_connections_org ON connector_connections(org_id);

CREATE TABLE IF NOT EXISTS indexed_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id UUID NOT NULL REFERENCES connector_connections(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    title TEXT NOT NULL,
    body_tsv TEXT,
    vector_ref TEXT,
    acl JSONB NOT NULL DEFAULT '[]'::jsonb,
    container TEXT,
    external_url TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (connection_id, external_id)
);
