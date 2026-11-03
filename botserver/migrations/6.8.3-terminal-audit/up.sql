-- #1645 — durable audit trail for terminal actions. The in-memory action
-- registry is short-lived; this table answers "what ran, who approved it and
-- what happened" after a restart, and is retained for 90 days.
CREATE TABLE IF NOT EXISTS terminal_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    session_id TEXT,
    action_id UUID,
    command TEXT NOT NULL,
    cwd TEXT,
    container TEXT,
    decision TEXT NOT NULL,
    exit_code INTEGER,
    output_excerpt TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_terminal_audit_user_created
    ON terminal_audit (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_terminal_audit_created
    ON terminal_audit (created_at);
