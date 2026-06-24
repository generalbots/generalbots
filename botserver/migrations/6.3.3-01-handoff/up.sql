-- ============================================
-- Chatbot Handoff Module
-- Version: 6.3.3
-- Issue: #621 - Chatbot Gaps / Handoff / Analytics
-- ============================================

CREATE TABLE IF NOT EXISTS handoff_queue (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID,
    user_id UUID,
    topic TEXT NOT NULL,
    reason TEXT,
    priority TEXT NOT NULL DEFAULT 'normal',
    status TEXT NOT NULL DEFAULT 'queued',
    agent_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_handoff_queue_bot_status
    ON handoff_queue (bot_id, status);

CREATE INDEX IF NOT EXISTS idx_handoff_queue_session
    ON handoff_queue (session_id);

CREATE INDEX IF NOT EXISTS idx_handoff_queue_agent
    ON handoff_queue (agent_id);

CREATE TABLE IF NOT EXISTS handoff_events (
    id UUID PRIMARY KEY,
    handoff_id UUID NOT NULL,
    kind TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    actor_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_handoff_events_handoff_created
    ON handoff_events (handoff_id, created_at DESC);
