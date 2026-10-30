-- #1580: VM is per user (not per session), chat is always agent on.
-- One running VM per logged-in user; each conversation gets a folder inside it.

ALTER TABLE agent_sessions ADD COLUMN IF NOT EXISTS active_conversation_id VARCHAR(64);

-- Drop the old per-session unique constraint and add per-user unique.
-- Only one RUNNING/PROVISIONING VM per user at a time.
DROP INDEX IF EXISTS agent_sessions_session_id_key;
ALTER TABLE agent_sessions DROP CONSTRAINT IF EXISTS agent_sessions_session_id_key;
CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_sessions_user_active
    ON agent_sessions(user_id) WHERE status IN ('running', 'provisioning');

-- Track which conversations have folders provisioned inside the VM.
CREATE TABLE IF NOT EXISTS agent_conversation_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_session_id UUID NOT NULL REFERENCES agent_sessions(id) ON DELETE CASCADE,
    conversation_id VARCHAR(64) NOT NULL,
    folder_path VARCHAR(256) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_session_id, conversation_id)
);

CREATE INDEX IF NOT EXISTS idx_agent_conv_folders_session
    ON agent_conversation_folders(agent_session_id);
