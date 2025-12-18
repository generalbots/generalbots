CREATE TABLE IF NOT EXISTS connected_accounts (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id UUID,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    account_type TEXT NOT NULL DEFAULT 'email',
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    scopes TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 300,
    last_sync_at TIMESTAMPTZ,
    last_sync_status TEXT,
    last_sync_error TEXT,
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connected_accounts_bot_id ON connected_accounts(bot_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_user_id ON connected_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_email ON connected_accounts(email);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_provider ON connected_accounts(provider);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_status ON connected_accounts(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_connected_accounts_bot_email ON connected_accounts(bot_id, email);

CREATE TABLE IF NOT EXISTS session_account_associations (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT
);

CREATE INDEX IF NOT EXISTS idx_session_account_assoc_session ON session_account_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_account ON session_account_associations(account_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_active ON session_account_associations(session_id, is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_session_account_assoc_unique ON session_account_associations(session_id, account_id);

CREATE TABLE IF NOT EXISTS account_sync_items (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    subject TEXT,
    content_preview TEXT,
    sender TEXT,
    recipients TEXT,
    item_date TIMESTAMPTZ,
    folder TEXT,
    labels TEXT,
    has_attachments BOOLEAN DEFAULT false,
    qdrant_point_id TEXT,
    embedding_status TEXT DEFAULT 'pending',
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_account_sync_items_account ON account_sync_items(account_id);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_type ON account_sync_items(item_type);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_date ON account_sync_items(item_date);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_embedding ON account_sync_items(embedding_status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_account_sync_items_unique ON account_sync_items(account_id, item_type, item_id);
