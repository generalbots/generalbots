CREATE TABLE IF NOT EXISTS sandbox_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID,
    user_id UUID,
    language VARCHAR(24) NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT 'queued',
    exit_code INTEGER,
    stdout_ref TEXT,
    stderr_ref TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sandbox_runs_org_created ON sandbox_runs(org_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_runs_user_created ON sandbox_runs(user_id, created_at DESC);
