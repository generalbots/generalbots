-- Migration: 6.1.1 Multi-Agent Memory Support
-- Description: Adds tables for user memory, session preferences, and A2A protocol messaging

-- ============================================================================
-- User Memories Table
-- Cross-session memory that persists for users across all sessions and bots
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_memories (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    memory_type VARCHAR(50) NOT NULL DEFAULT 'preference',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_memories_unique_key UNIQUE (user_id, key)
);

CREATE INDEX IF NOT EXISTS idx_user_memories_user_id ON user_memories(user_id);
CREATE INDEX IF NOT EXISTS idx_user_memories_type ON user_memories(user_id, memory_type);

-- ============================================================================
-- Session Preferences Table
-- Stores per-session configuration like current model, routing strategy, etc.
-- ============================================================================
CREATE TABLE IF NOT EXISTS session_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    preference_key VARCHAR(255) NOT NULL,
    preference_value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT session_preferences_unique UNIQUE (session_id, preference_key)
);

CREATE INDEX IF NOT EXISTS idx_session_preferences_session ON session_preferences(session_id);

-- ============================================================================
-- A2A Messages Table
-- Agent-to-Agent protocol messages for multi-agent orchestration
-- Based on https://a2a-protocol.org/latest/
-- ============================================================================
CREATE TABLE IF NOT EXISTS a2a_messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    from_agent VARCHAR(255) NOT NULL,
    to_agent VARCHAR(255),  -- NULL for broadcast messages
    message_type VARCHAR(50) NOT NULL,
    payload TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata TEXT DEFAULT '{}',
    ttl_seconds INTEGER NOT NULL DEFAULT 30,
    hop_count INTEGER NOT NULL DEFAULT 0,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_a2a_messages_session ON a2a_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_to_agent ON a2a_messages(session_id, to_agent);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_correlation ON a2a_messages(correlation_id);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_pending ON a2a_messages(session_id, to_agent, processed) WHERE processed = FALSE;
CREATE INDEX IF NOT EXISTS idx_a2a_messages_timestamp ON a2a_messages(timestamp);

-- ============================================================================
-- Extended Bot Memory Table
-- Enhanced memory with TTL and different memory types
-- ============================================================================
CREATE TABLE IF NOT EXISTS bot_memory_extended (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID,  -- NULL for long-term memory
    memory_type VARCHAR(20) NOT NULL CHECK (memory_type IN ('short', 'long', 'episodic')),
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    ttl_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    CONSTRAINT bot_memory_extended_unique UNIQUE (bot_id, session_id, key)
);

CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_bot ON bot_memory_extended(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_session ON bot_memory_extended(bot_id, session_id);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_type ON bot_memory_extended(bot_id, memory_type);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_expires ON bot_memory_extended(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- Knowledge Graph Entities Table
-- For graph-based memory and entity relationships
-- ============================================================================
CREATE TABLE IF NOT EXISTS kg_entities (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_name VARCHAR(500) NOT NULL,
    properties JSONB DEFAULT '{}',
    embedding_vector BYTEA,  -- For vector similarity search
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT kg_entities_unique UNIQUE (bot_id, entity_type, entity_name)
);

CREATE INDEX IF NOT EXISTS idx_kg_entities_bot ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(bot_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);

-- ============================================================================
-- Knowledge Graph Relationships Table
-- For storing relationships between entities
-- ============================================================================
CREATE TABLE IF NOT EXISTS kg_relationships (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    from_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    weight FLOAT DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT kg_relationships_unique UNIQUE (from_entity_id, to_entity_id, relationship_type)
);

CREATE INDEX IF NOT EXISTS idx_kg_rel_bot ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_type ON kg_relationships(bot_id, relationship_type);

-- ============================================================================
-- Episodic Memory Table
-- For storing conversation summaries and episodes
-- ============================================================================
CREATE TABLE IF NOT EXISTS episodic_memories (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    session_id UUID,
    summary TEXT NOT NULL,
    key_topics JSONB DEFAULT '[]',
    decisions JSONB DEFAULT '[]',
    action_items JSONB DEFAULT '[]',
    message_count INTEGER NOT NULL DEFAULT 0,
    start_timestamp TIMESTAMPTZ NOT NULL,
    end_timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_episodic_bot ON episodic_memories(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodic_user ON episodic_memories(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_session ON episodic_memories(session_id);
CREATE INDEX IF NOT EXISTS idx_episodic_time ON episodic_memories(bot_id, user_id, created_at);

-- ============================================================================
-- Conversation Cost Tracking Table
-- For monitoring LLM usage and costs
-- ============================================================================
CREATE TABLE IF NOT EXISTS conversation_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model_used VARCHAR(100),
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd DECIMAL(10, 6) NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conv_costs_session ON conversation_costs(session_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_user ON conversation_costs(user_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_bot ON conversation_costs(bot_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_time ON conversation_costs(timestamp);

-- ============================================================================
-- Generated API Tools Table
-- For tracking tools generated from OpenAPI specs
-- ============================================================================
CREATE TABLE IF NOT EXISTS generated_api_tools (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    api_name VARCHAR(255) NOT NULL,
    spec_url TEXT NOT NULL,
    spec_hash VARCHAR(64) NOT NULL,
    tool_count INTEGER NOT NULL DEFAULT 0,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT generated_api_tools_unique UNIQUE (bot_id, api_name)
);

CREATE INDEX IF NOT EXISTS idx_gen_api_tools_bot ON generated_api_tools(bot_id);

-- ============================================================================
-- Session Bots Junction Table (if not exists)
-- For multi-agent sessions
-- ============================================================================
CREATE TABLE IF NOT EXISTS session_bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    bot_name VARCHAR(255) NOT NULL,
    trigger_config JSONB DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT session_bots_unique UNIQUE (session_id, bot_name)
);

CREATE INDEX IF NOT EXISTS idx_session_bots_session ON session_bots(session_id);
CREATE INDEX IF NOT EXISTS idx_session_bots_active ON session_bots(session_id, is_active);

-- ============================================================================
-- Cleanup function for expired A2A messages
-- ============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_a2a_messages()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM a2a_messages
    WHERE ttl_seconds > 0
    AND timestamp + (ttl_seconds || ' seconds')::INTERVAL < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Cleanup function for expired bot memory
-- ============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_bot_memory()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM bot_memory_extended
    WHERE expires_at IS NOT NULL AND expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Trigger to update updated_at timestamp
-- ============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables with updated_at
DROP TRIGGER IF EXISTS update_user_memories_updated_at ON user_memories;
CREATE TRIGGER update_user_memories_updated_at
    BEFORE UPDATE ON user_memories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_bot_memory_extended_updated_at ON bot_memory_extended;
CREATE TRIGGER update_bot_memory_extended_updated_at
    BEFORE UPDATE ON bot_memory_extended
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Bot Reflections Table
-- For storing agent self-reflection analysis results
-- ============================================================================
CREATE TABLE IF NOT EXISTS bot_reflections (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    reflection_type TEXT NOT NULL,
    score FLOAT NOT NULL DEFAULT 0.0,
    insights TEXT NOT NULL DEFAULT '[]',
    improvements TEXT NOT NULL DEFAULT '[]',
    positive_patterns TEXT NOT NULL DEFAULT '[]',
    concerns TEXT NOT NULL DEFAULT '[]',
    raw_response TEXT NOT NULL DEFAULT '',
    messages_analyzed INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bot_reflections_bot ON bot_reflections(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_session ON bot_reflections(session_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_time ON bot_reflections(bot_id, created_at);

-- ============================================================================
-- Conversation Messages Table
-- For storing conversation history (if not already exists)
-- ============================================================================
CREATE TABLE IF NOT EXISTS conversation_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID,
    role VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    token_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conv_messages_session ON conversation_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_conv_messages_time ON conversation_messages(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_conv_messages_bot ON conversation_messages(bot_id);
