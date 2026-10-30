DROP TABLE IF EXISTS agent_conversation_folders;
DROP INDEX IF EXISTS idx_agent_sessions_user_active;
ALTER TABLE agent_sessions DROP COLUMN IF EXISTS active_conversation_id;
-- Restore the original per-session unique constraint.
CREATE UNIQUE INDEX IF NOT EXISTS agent_sessions_session_id_key ON agent_sessions(session_id);
