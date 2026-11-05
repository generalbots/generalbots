-- #1702 — browser-action ledger. One row per consent decision and result,
-- so the TS-07 trail covers browser steps the same way terminal_audit covers
-- shell commands (actor, tool, args, decision, result, screenshot path).
CREATE TABLE IF NOT EXISTS browser_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    session_id TEXT,
    action_id UUID,
    target TEXT,
    action TEXT NOT NULL,
    decision TEXT NOT NULL,
    result TEXT,
    screenshot_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_browser_audit_user_created
    ON browser_audit (user_id, created_at DESC);
