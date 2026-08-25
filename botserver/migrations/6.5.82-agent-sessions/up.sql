CREATE TABLE IF NOT EXISTS agent_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(64) NOT NULL UNIQUE,
    user_id UUID NOT NULL,
    org_id UUID,
    branch_id UUID,
    bot_id UUID NOT NULL,
    vm_name VARCHAR(128) NOT NULL UNIQUE,
    status VARCHAR(24) NOT NULL DEFAULT 'provisioning',
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_sessions_user_status ON agent_sessions(user_id, status);

CREATE TABLE IF NOT EXISTS agent_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_session_id UUID NOT NULL REFERENCES agent_sessions(id) ON DELETE CASCADE,
    label VARCHAR(200),
    incus_snapshot VARCHAR(160) NOT NULL,
    size_bytes BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_snapshots_session ON agent_snapshots(agent_session_id);
