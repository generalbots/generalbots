-- Canonical tenant-scoped integration connection control plane (#939, slice 1).
--
-- `integration_connections` is the single registry of external provider
-- connections. Credentials NEVER live in this table: `vault_path` points at
-- the canonical Vault KV2 location (see botintegrations::secrets) and rows
-- carry only metadata, lifecycle status and audit timestamps. No foreign keys
-- to users are declared on purpose - the identity mapping is still unstable.
--
-- `integration_connection_events` is the append-only audit trail for every
-- write against a connection (created, rotated, tested, revoked, deleted).

CREATE TABLE IF NOT EXISTS integration_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    provider_slug VARCHAR(100) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    auth_kind VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    vault_path VARCHAR(700) NOT NULL UNIQUE,
    granted_scopes JSONB NOT NULL DEFAULT '[]',
    configuration JSONB NOT NULL DEFAULT '{}',
    provider_account_id VARCHAR(255),
    credential_version BIGINT NOT NULL DEFAULT 1,
    expires_at TIMESTAMPTZ,
    last_refreshed_at TIMESTAMPTZ,
    last_tested_at TIMESTAMPTZ,
    last_test_status VARCHAR(32),
    last_error_code VARCHAR(100),
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_integration_connections_scope
    ON integration_connections(org_id, branch_id, bot_id, owner_user_id);

CREATE INDEX IF NOT EXISTS idx_integration_connections_provider_status
    ON integration_connections(provider_slug, status);

CREATE INDEX IF NOT EXISTS idx_integration_connections_active_expiry
    ON integration_connections(expires_at) WHERE status = 'active';

CREATE TABLE IF NOT EXISTS integration_connection_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id UUID REFERENCES integration_connections(id) ON DELETE SET NULL,
    org_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    actor_user_id UUID NOT NULL,
    event_type VARCHAR(80) NOT NULL,
    outcome VARCHAR(32) NOT NULL,
    risk_level VARCHAR(20) NOT NULL DEFAULT 'low',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_integration_connection_events_scope
    ON integration_connection_events(org_id, branch_id, bot_id, created_at);
