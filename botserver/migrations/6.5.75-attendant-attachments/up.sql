-- Attendant conversation attachments.
--
-- Files shared by agents (and, in future, customers) inside an attendant
-- conversation. Content is stored as BYTEA; metadata (name, type, size) is
-- mirrored into the message's `attachments` JSONB so the thread can render
-- previews without fetching every blob. Retention/cleanup is governed by the
-- session lifecycle (cascade on session delete).

CREATE TABLE IF NOT EXISTS attendant_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES attendant_sessions(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL DEFAULT 'application/octet-stream',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_attendant_attachments_session
    ON attendant_attachments(session_id, created_at DESC);
