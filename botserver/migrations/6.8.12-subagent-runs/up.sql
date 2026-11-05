-- #1694 — parallel sub-agent fan-out bookkeeping.
CREATE TABLE IF NOT EXISTS subagent_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_run_id TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    title TEXT NOT NULL,
    prompt TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    result TEXT,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subagent_runs_parent
    ON subagent_runs (parent_run_id, created_at);
