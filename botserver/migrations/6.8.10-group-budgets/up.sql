-- #1697 — pooled group budgets and per-member usage attribution.
CREATE TABLE IF NOT EXISTS group_budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL,
    group_name TEXT NOT NULL,
    budget_milli BIGINT NOT NULL DEFAULT 0 CHECK (budget_milli >= 0),
    spent_milli BIGINT NOT NULL DEFAULT 0 CHECK (spent_milli >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (group_name)
);

CREATE TABLE IF NOT EXISTS group_spend (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES group_budgets(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    run_id TEXT,
    kind TEXT NOT NULL DEFAULT 'run',
    amount_milli BIGINT NOT NULL DEFAULT 0 CHECK (amount_milli >= 0),
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_group_spend_budget_created
    ON group_spend (budget_id, created_at DESC);
