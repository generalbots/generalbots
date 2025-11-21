-- Migration 6.0.7: Session KB Tracking
-- Adds table to track which KBs are active in each conversation session

-- Table for tracking KBs active in a session (set by ADD_KB in .bas tools)
CREATE TABLE IF NOT EXISTS session_kb_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    kb_name TEXT NOT NULL,
    kb_folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(session_id, kb_name)
);

CREATE INDEX IF NOT EXISTS idx_session_kb_session_id ON session_kb_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_bot_id ON session_kb_associations(bot_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_name ON session_kb_associations(kb_name);
CREATE INDEX IF NOT EXISTS idx_session_kb_active ON session_kb_associations(is_active) WHERE is_active = true;

-- Comments
COMMENT ON TABLE session_kb_associations IS 'Tracks which Knowledge Base collections are active in each conversation session';
COMMENT ON COLUMN session_kb_associations.kb_name IS 'Name of the KB folder (e.g., "circular", "comunicado", "geral")';
COMMENT ON COLUMN session_kb_associations.kb_folder_path IS 'Full path to KB folder: work/{bot}/{bot}.gbkb/{kb_name}';
COMMENT ON COLUMN session_kb_associations.qdrant_collection IS 'Qdrant collection name for this KB';
COMMENT ON COLUMN session_kb_associations.added_by_tool IS 'Name of the .bas tool that added this KB (e.g., "change-subject.bas")';
COMMENT ON COLUMN session_kb_associations.is_active IS 'Whether this KB is currently active in the session';
