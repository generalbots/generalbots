-- Migration: 6.1.1 Multi-Agent Memory Support (DOWN)
-- Description: Rollback for user memory, session preferences, and A2A protocol messaging

-- Drop triggers first
DROP TRIGGER IF EXISTS update_user_memories_updated_at ON user_memories;
DROP TRIGGER IF EXISTS update_bot_memory_extended_updated_at ON bot_memory_extended;
DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;

-- Drop functions
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP FUNCTION IF EXISTS cleanup_expired_bot_memory();
DROP FUNCTION IF EXISTS cleanup_expired_a2a_messages();

-- Drop indexes (will be dropped with tables, but explicit for clarity)
DROP INDEX IF EXISTS idx_session_bots_active;
DROP INDEX IF EXISTS idx_session_bots_session;
DROP INDEX IF EXISTS idx_gen_api_tools_bot;
DROP INDEX IF EXISTS idx_conv_costs_time;
DROP INDEX IF EXISTS idx_conv_costs_bot;
DROP INDEX IF EXISTS idx_conv_costs_user;
DROP INDEX IF EXISTS idx_conv_costs_session;
DROP INDEX IF EXISTS idx_episodic_time;
DROP INDEX IF EXISTS idx_episodic_session;
DROP INDEX IF EXISTS idx_episodic_user;
DROP INDEX IF EXISTS idx_episodic_bot;
DROP INDEX IF EXISTS idx_kg_rel_type;
DROP INDEX IF EXISTS idx_kg_rel_to;
DROP INDEX IF EXISTS idx_kg_rel_from;
DROP INDEX IF EXISTS idx_kg_rel_bot;
DROP INDEX IF EXISTS idx_kg_entities_name;
DROP INDEX IF EXISTS idx_kg_entities_type;
DROP INDEX IF EXISTS idx_kg_entities_bot;
DROP INDEX IF EXISTS idx_bot_memory_ext_expires;
DROP INDEX IF EXISTS idx_bot_memory_ext_type;
DROP INDEX IF EXISTS idx_bot_memory_ext_session;
DROP INDEX IF EXISTS idx_bot_memory_ext_bot;
DROP INDEX IF EXISTS idx_a2a_messages_timestamp;
DROP INDEX IF EXISTS idx_a2a_messages_pending;
DROP INDEX IF EXISTS idx_a2a_messages_correlation;
DROP INDEX IF EXISTS idx_a2a_messages_to_agent;
DROP INDEX IF EXISTS idx_a2a_messages_session;
DROP INDEX IF EXISTS idx_session_preferences_session;
DROP INDEX IF EXISTS idx_user_memories_type;
DROP INDEX IF EXISTS idx_user_memories_user_id;
DROP INDEX IF EXISTS idx_bot_reflections_bot;
DROP INDEX IF EXISTS idx_bot_reflections_session;
DROP INDEX IF EXISTS idx_bot_reflections_time;
DROP INDEX IF EXISTS idx_conv_messages_session;
DROP INDEX IF EXISTS idx_conv_messages_time;
DROP INDEX IF EXISTS idx_conv_messages_bot;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS conversation_messages;
DROP TABLE IF EXISTS bot_reflections;
DROP TABLE IF EXISTS session_bots;
DROP TABLE IF EXISTS generated_api_tools;
DROP TABLE IF EXISTS conversation_costs;
DROP TABLE IF EXISTS episodic_memories;
DROP TABLE IF EXISTS kg_relationships;
DROP TABLE IF EXISTS kg_entities;
DROP TABLE IF EXISTS bot_memory_extended;
DROP TABLE IF EXISTS a2a_messages;
DROP TABLE IF EXISTS session_preferences;
DROP TABLE IF EXISTS user_memories;
