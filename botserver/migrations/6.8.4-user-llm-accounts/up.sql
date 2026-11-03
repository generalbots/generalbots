-- Per-user AI accounts (#1654). Credentials are never stored here:
-- Claude stays inside the user's VM; ChatGPT tokens live in Vault at vault_path.
CREATE TABLE IF NOT EXISTS user_llm_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unlinked',
    vm_name TEXT,
    account_email TEXT,
    plan TEXT,
    vault_path TEXT,
    host_id TEXT,
    client_id TEXT,
    scopes TEXT,
    linked_at TIMESTAMPTZ,
    last_verified_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_llm_accounts_status_check
        CHECK (status IN ('unlinked','linked','expired','revoked','error','provisioning','connecting','reconnect_required','maintenance'))
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_llm_accounts_user_provider
    ON user_llm_accounts (user_id, provider);
CREATE INDEX IF NOT EXISTS idx_user_llm_accounts_provider
    ON user_llm_accounts (provider, status);
