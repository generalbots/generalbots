-- Rollback Multi-Agent Bots Migration

-- Drop triggers first
DROP TRIGGER IF EXISTS update_bots_updated_at ON bots;
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop tables in reverse order of creation (respecting foreign key dependencies)
DROP TABLE IF EXISTS play_content;
DROP TABLE IF EXISTS hear_wait_states;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS conversation_branches;
DROP TABLE IF EXISTS bot_messages;
DROP TABLE IF EXISTS session_bots;
DROP TABLE IF EXISTS bot_triggers;
DROP TABLE IF EXISTS bots;
