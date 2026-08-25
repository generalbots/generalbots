CREATE TABLE IF NOT EXISTS agent_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID,
    branch_id UUID,
    bot_id UUID NOT NULL,
    title TEXT NOT NULL,
    goal TEXT NOT NULL,
    cron_expr VARCHAR(64) NOT NULL,
    timezone VARCHAR(64) NOT NULL DEFAULT 'UTC',
    owner_user_id UUID NOT NULL,
    delivery JSONB NOT NULL DEFAULT '{"email":true,"sms":false,"channels":[]}'::jsonb,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    max_runtime_secs INTEGER NOT NULL DEFAULT 900,
    tool_allowlist JSONB,
    next_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_schedules_bot ON agent_schedules(bot_id);
CREATE INDEX IF NOT EXISTS idx_agent_schedules_next_run ON agent_schedules(enabled, next_run_at);

CREATE TABLE IF NOT EXISTS agent_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID REFERENCES agent_schedules(id) ON DELETE SET NULL,
    bot_id UUID NOT NULL,
    trigger_kind VARCHAR(16) NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT 'queued',
    plan JSONB,
    steps JSONB,
    result_summary TEXT,
    artifacts JSONB,
    verdict JSONB,
    delivery_status JSONB,
    error TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agent_runs_schedule_created ON agent_runs(schedule_id, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_spans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
    parent_id UUID,
    kind VARCHAR(24) NOT NULL,
    name TEXT NOT NULL,
    input_ref TEXT,
    output_ref TEXT,
    tokens_in INTEGER,
    tokens_out INTEGER,
    vm_seconds INTEGER,
    status VARCHAR(24) NOT NULL DEFAULT 'ok',
    error TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_agent_spans_run ON agent_spans(run_id);

CREATE TABLE IF NOT EXISTS compute_usage_hourly (
    org_id UUID NOT NULL,
    hour TIMESTAMP NOT NULL,
    resource VARCHAR(32) NOT NULL,
    quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (org_id, hour, resource)
);
