CREATE TABLE IF NOT EXISTS integration_automations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    provider_slug VARCHAR(100) NOT NULL,
    action_key VARCHAR(200) NOT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    schedule VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_run_at TIMESTAMPTZ,
    last_outcome TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_integration_automations_due
    ON integration_automations (enabled, last_run_at);
