-- #1613 — long-running work started from a conversation is first-class state.
-- Each row mirrors a live cron entry or systemd unit inside the user's agent
-- VM, so the conversation can report (and cancel) work that outlives a turn.

CREATE TABLE IF NOT EXISTS conversation_autotasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    title TEXT NOT NULL,
    kind VARCHAR(24) NOT NULL DEFAULT 'cron',
    schedule VARCHAR(128) NOT NULL,
    schedule_human VARCHAR(160),
    command TEXT NOT NULL,
    unit_name VARCHAR(128),
    status VARCHAR(24) NOT NULL DEFAULT 'pending',
    output_path TEXT,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conversation_autotasks_conversation
    ON conversation_autotasks(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_autotasks_status
    ON conversation_autotasks(status);
CREATE INDEX IF NOT EXISTS idx_conversation_autotasks_next_run
    ON conversation_autotasks(status, next_run_at);
