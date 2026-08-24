CREATE TABLE IF NOT EXISTS app_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    org_id UUID,
    branch_id UUID,
    app_id VARCHAR(64) NOT NULL,
    action_class VARCHAR(32) NOT NULL,
    scope JSONB NOT NULL DEFAULT '{}'::jsonb,
    granted BOOLEAN NOT NULL DEFAULT TRUE,
    granted_via VARCHAR(16) NOT NULL DEFAULT 'prompt',
    expires_at TIMESTAMPTZ,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, app_id, action_class)
);

CREATE TABLE IF NOT EXISTS consent_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_id UUID,
    user_id UUID,
    request JSONB NOT NULL,
    outcome VARCHAR(24) NOT NULL,
    decided_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_consent_audit_user ON consent_audit(user_id, created_at DESC);
