-- Migration 6.0.3: Additional KB and session tables
-- This migration adds user_kb_associations and session_tool_associations tables
-- Note: kb_documents, kb_collections, and basic_tools are already created in 6.0.2

-- Table for user KB associations (which KBs are active for a user)
CREATE TABLE IF NOT EXISTS user_kb_associations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    bot_id TEXT NOT NULL,
    kb_name TEXT NOT NULL,
    is_website INTEGER NOT NULL DEFAULT 0,
    website_url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(user_id, bot_id, kb_name)
);

CREATE INDEX IF NOT EXISTS idx_user_kb_user_id ON user_kb_associations(user_id);
CREATE INDEX IF NOT EXISTS idx_user_kb_bot_id ON user_kb_associations(bot_id);
CREATE INDEX IF NOT EXISTS idx_user_kb_name ON user_kb_associations(kb_name);
CREATE INDEX IF NOT EXISTS idx_user_kb_website ON user_kb_associations(is_website);

-- Table for session tool associations (which tools are available in a session)
CREATE TABLE IF NOT EXISTS session_tool_associations (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    added_at TEXT NOT NULL,
    UNIQUE(session_id, tool_name)
);

CREATE INDEX IF NOT EXISTS idx_session_tool_session ON session_tool_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_tool_name ON session_tool_associations(tool_name);
