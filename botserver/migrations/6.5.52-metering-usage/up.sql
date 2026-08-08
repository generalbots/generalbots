-- 6.5.52-metering-usage
-- #769: VM metering for Private Cloud. Metering only — no usage billing.
-- metering_usage: rows per (org, project, env, meter) accumulated per period.
-- metering_limits: protective caps per org+plan (soft/hard).
-- metering_overrides: audit trail of admin capacity overrides.

CREATE TABLE IF NOT EXISTS metering_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    project_id UUID,
    env VARCHAR(16) NOT NULL DEFAULT 'production',
    meter VARCHAR(32) NOT NULL,
    amount NUMERIC(18,6) NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_metering_usage_lookup
    ON metering_usage(org_id, meter, period_start);
CREATE INDEX IF NOT EXISTS idx_metering_usage_project
    ON metering_usage(project_id) WHERE project_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS metering_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    scope VARCHAR(16) NOT NULL DEFAULT 'free',
    meter VARCHAR(32) NOT NULL,
    hard_limit NUMERIC(18,6),
    soft_limit NUMERIC(18,6),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT metering_limits_org_meter UNIQUE (org_id, scope, meter)
);
CREATE INDEX IF NOT EXISTS idx_metering_limits_scope ON metering_limits(org_id, scope);

CREATE TABLE IF NOT EXISTS metering_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    actor_user_id UUID NOT NULL,
    reason VARCHAR(255) NOT NULL,
    until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_metering_overrides_org ON metering_overrides(org_id, until);