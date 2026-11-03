-- #1665 — ordered, append-only log for one conversation thread.
-- The run summary stays in agent_runs; this table only carries the frames a
-- client needs to replay a run after a reconnect or a restart.
CREATE TABLE IF NOT EXISTS run_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    v SMALLINT NOT NULL DEFAULT 1,
    thread_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    parent_run_id TEXT,
    seq BIGINT NOT NULL,
    event_type VARCHAR(48) NOT NULL,
    event_id VARCHAR(64) NOT NULL,
    source VARCHAR(24) NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    bot_id UUID NOT NULL,
    user_id UUID,
    org_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT run_events_thread_seq_unique UNIQUE (thread_id, seq),
    CONSTRAINT run_events_event_id_unique UNIQUE (event_id),
    CONSTRAINT run_events_seq_positive CHECK (seq >= 1)
);

CREATE INDEX IF NOT EXISTS idx_run_events_thread_seq
    ON run_events (thread_id, seq);
CREATE INDEX IF NOT EXISTS idx_run_events_bot_thread
    ON run_events (bot_id, thread_id, seq);
CREATE INDEX IF NOT EXISTS idx_run_events_run
    ON run_events (run_id, seq);
CREATE INDEX IF NOT EXISTS idx_run_events_created
    ON run_events (created_at);

-- A run summary is the long-lived record. The chat columns let a chat turn
-- write its summary into the existing table instead of a parallel one.
ALTER TABLE agent_runs ADD COLUMN IF NOT EXISTS session_id UUID;
ALTER TABLE agent_runs ADD COLUMN IF NOT EXISTS thread_id TEXT;
ALTER TABLE agent_runs ADD COLUMN IF NOT EXISTS user_id UUID;
ALTER TABLE agent_runs ADD COLUMN IF NOT EXISTS parent_run_id UUID;

CREATE INDEX IF NOT EXISTS idx_agent_runs_thread
    ON agent_runs (thread_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_runs_session
    ON agent_runs (session_id, created_at DESC);

-- Pointer only: the summary lives in agent_runs and the frames in run_events.
-- The encrypted content column stays the single storage for message text.
ALTER TABLE message_history ADD COLUMN IF NOT EXISTS run_id UUID;
ALTER TABLE message_history ADD COLUMN IF NOT EXISTS thread_id TEXT;

CREATE INDEX IF NOT EXISTS idx_message_history_run
    ON message_history (run_id)
    WHERE run_id IS NOT NULL;
