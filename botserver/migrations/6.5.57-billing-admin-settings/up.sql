-- Persist per-branch billing admin settings so the admin dashboard's
-- alert configuration and quota limits survive restarts instead of
-- echoing a fake success.
CREATE TABLE IF NOT EXISTS billing_alert_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (branch_id)
);

CREATE INDEX IF NOT EXISTS idx_billing_alert_settings_branch_id
    ON billing_alert_settings(branch_id);

CREATE TABLE IF NOT EXISTS billing_quota_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    quota_key VARCHAR(64) NOT NULL,
    quota_limit BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (branch_id, quota_key)
);

CREATE INDEX IF NOT EXISTS idx_billing_quota_settings_branch_id
    ON billing_quota_settings(branch_id);
