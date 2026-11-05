-- #1705 — long-horizon goal runs with automatic checkpoints.
CREATE TABLE IF NOT EXISTS goal_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    key_result_id UUID REFERENCES okr_key_results(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    step_value DECIMAL(15,2) NOT NULL DEFAULT 1,
    interval_seconds INTEGER NOT NULL DEFAULT 60 CHECK (interval_seconds >= 5),
    status TEXT NOT NULL DEFAULT 'running',
    checkpoints INTEGER NOT NULL DEFAULT 0 CHECK (checkpoints >= 0),
    last_value DECIMAL(15,2),
    next_checkpoint_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_goal_runs_status
    ON goal_runs (status, next_checkpoint_at);
