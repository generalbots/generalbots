-- #1696 — group-level per-run cap policy. A user inherits the most
-- restrictive non-zero limits from the groups they belong to; per-bot config
-- and GB_RUN_MAX_* defaults remain the fallbacks.
CREATE TABLE IF NOT EXISTS group_cap_policies (
    group_name TEXT PRIMARY KEY,
    max_tokens BIGINT NOT NULL DEFAULT 0 CHECK (max_tokens >= 0),
    max_seconds BIGINT NOT NULL DEFAULT 0 CHECK (max_seconds >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
