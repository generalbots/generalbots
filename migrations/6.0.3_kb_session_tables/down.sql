-- Drop indexes
DROP INDEX IF EXISTS idx_session_tool_name;
DROP INDEX IF EXISTS idx_session_tool_session;
DROP INDEX IF EXISTS idx_user_kb_website;
DROP INDEX IF EXISTS idx_user_kb_name;
DROP INDEX IF EXISTS idx_user_kb_bot_id;
DROP INDEX IF EXISTS idx_user_kb_user_id;

-- Drop tables
DROP TABLE IF EXISTS session_tool_associations;
DROP TABLE IF EXISTS user_kb_associations;
