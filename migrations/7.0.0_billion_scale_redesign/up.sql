-- Migration: 7.0.0 Billion Scale Redesign
-- Description: Complete database redesign for billion-user scale
-- Features:
--   - PostgreSQL ENUMs instead of VARCHAR for domain values
--   - Sharding support with shard_key (region/tenant based)
--   - Optimized indexes for high-throughput queries
--   - Partitioning-ready table structures
--   - No TEXT columns for domain values - all use SMALLINT enums
--
-- IMPORTANT: This is a DESTRUCTIVE migration - drops all existing tables
-- Only run on fresh installations or after full data export

-- ============================================================================
-- CLEANUP: Drop all existing objects
-- ============================================================================
DROP SCHEMA IF EXISTS gb CASCADE;
CREATE SCHEMA gb;
SET search_path TO gb, public;

-- ============================================================================
-- SHARDING INFRASTRUCTURE
-- ============================================================================

-- Shard configuration table (exists in each shard, contains global shard map)
CREATE TABLE gb.shard_config (
    shard_id SMALLINT PRIMARY KEY,
    region_code CHAR(3) NOT NULL,           -- ISO 3166-1 alpha-3: USA, BRA, DEU, etc.
    datacenter VARCHAR(32) NOT NULL,         -- e.g., 'us-east-1', 'eu-west-1'
    connection_string TEXT NOT NULL,         -- Encrypted connection string
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    min_tenant_id BIGINT NOT NULL,
    max_tenant_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tenant to shard mapping (replicated across all shards for routing)
CREATE TABLE gb.tenant_shard_map (
    tenant_id BIGINT PRIMARY KEY,
    shard_id SMALLINT NOT NULL REFERENCES gb.shard_config(shard_id),
    region_code CHAR(3) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tenant_shard_region ON gb.tenant_shard_map(region_code, shard_id);

-- ============================================================================
-- ENUM TYPES - All domain values as PostgreSQL ENUMs (stored as integers internally)
-- ============================================================================

-- Core enums
CREATE TYPE gb.channel_type AS ENUM (
    'web', 'whatsapp', 'telegram', 'msteams', 'slack', 'email', 'sms', 'voice', 'instagram', 'api'
);

CREATE TYPE gb.message_role AS ENUM (
    'user', 'assistant', 'system', 'tool', 'episodic', 'compact'
);

CREATE TYPE gb.message_type AS ENUM (
    'text', 'image', 'audio', 'video', 'document', 'location', 'contact', 'sticker', 'reaction'
);

CREATE TYPE gb.llm_provider AS ENUM (
    'openai', 'anthropic', 'azure_openai', 'azure_claude', 'google', 'local', 'ollama', 'groq', 'mistral', 'cohere'
);

CREATE TYPE gb.context_provider AS ENUM (
    'qdrant', 'pinecone', 'weaviate', 'milvus', 'pgvector', 'elasticsearch', 'none'
);

-- Task/workflow enums
CREATE TYPE gb.task_status AS ENUM (
    'pending', 'ready', 'running', 'paused', 'waiting_approval', 'completed', 'failed', 'cancelled'
);

CREATE TYPE gb.task_priority AS ENUM (
    'low', 'normal', 'high', 'urgent', 'critical'
);

CREATE TYPE gb.execution_mode AS ENUM (
    'autonomous', 'supervised', 'manual'
);

CREATE TYPE gb.risk_level AS ENUM (
    'none', 'low', 'medium', 'high', 'critical'
);

CREATE TYPE gb.approval_status AS ENUM (
    'pending', 'approved', 'rejected', 'expired', 'skipped'
);

CREATE TYPE gb.approval_decision AS ENUM (
    'approve', 'reject', 'skip'
);

-- Intent/AI enums
CREATE TYPE gb.intent_type AS ENUM (
    'app_create', 'todo', 'monitor', 'action', 'schedule', 'goal', 'tool', 'query', 'unknown'
);

CREATE TYPE gb.plan_status AS ENUM (
    'pending', 'approved', 'rejected', 'executing', 'completed', 'failed'
);

CREATE TYPE gb.safety_outcome AS ENUM (
    'allowed', 'blocked', 'warning', 'error'
);

CREATE TYPE gb.designer_change_type AS ENUM (
    'style', 'html', 'database', 'tool', 'scheduler', 'config', 'multiple', 'unknown'
);

-- Memory enums
CREATE TYPE gb.memory_type AS ENUM (
    'short', 'long', 'episodic', 'semantic', 'procedural'
);

-- Calendar/scheduling enums
CREATE TYPE gb.recurrence_pattern AS ENUM (
    'once', 'daily', 'weekly', 'biweekly', 'monthly', 'quarterly', 'yearly', 'custom'
);

CREATE TYPE gb.booking_status AS ENUM (
    'pending', 'confirmed', 'cancelled', 'completed', 'no_show'
);

CREATE TYPE gb.resource_type AS ENUM (
    'room', 'equipment', 'vehicle', 'person', 'virtual', 'other'
);

-- Permission enums
CREATE TYPE gb.permission_level AS ENUM (
    'none', 'read', 'write', 'admin', 'owner'
);

CREATE TYPE gb.sync_status AS ENUM (
    'synced', 'pending', 'conflict', 'error', 'deleted'
);

-- Email enums
CREATE TYPE gb.email_status AS ENUM (
    'draft', 'queued', 'sending', 'sent', 'delivered', 'bounced', 'failed', 'cancelled'
);

CREATE TYPE gb.responder_type AS ENUM (
    'out_of_office', 'vacation', 'custom', 'auto_reply'
);

-- Meeting enums
CREATE TYPE gb.participant_status AS ENUM (
    'invited', 'accepted', 'declined', 'tentative', 'waiting', 'admitted', 'left', 'kicked'
);

CREATE TYPE gb.background_type AS ENUM (
    'none', 'blur', 'image', 'video'
);

CREATE TYPE gb.poll_type AS ENUM (
    'single', 'multiple', 'ranked', 'open'
);

-- Test enums
CREATE TYPE gb.test_status AS ENUM (
    'pending', 'running', 'passed', 'failed', 'skipped', 'error', 'timeout'
);

CREATE TYPE gb.test_account_type AS ENUM (
    'sender', 'receiver', 'bot', 'admin', 'observer'
);

-- ============================================================================
-- CORE TABLES - Tenant-aware with shard_key
-- ============================================================================

-- Tenants (organizations/companies)
CREATE TABLE gb.tenants (
    id BIGSERIAL PRIMARY KEY,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid() UNIQUE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(128) NOT NULL UNIQUE,
    region_code CHAR(3) NOT NULL DEFAULT 'USA',
    plan_tier SMALLINT NOT NULL DEFAULT 0,  -- 0=free, 1=starter, 2=pro, 3=enterprise
    settings JSONB DEFAULT '{}'::jsonb,
    limits JSONB DEFAULT '{"users": 5, "bots": 1, "storage_gb": 1}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tenants_shard ON gb.tenants(shard_id);
CREATE INDEX idx_tenants_region ON gb.tenants(region_code);
CREATE INDEX idx_tenants_active ON gb.tenants(is_active) WHERE is_active;

-- Users
CREATE TABLE gb.users (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    username VARCHAR(128) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255),
    phone_number VARCHAR(32),
    display_name VARCHAR(255),
    avatar_url VARCHAR(512),
    locale CHAR(5) DEFAULT 'en-US',
    timezone VARCHAR(64) DEFAULT 'UTC',
    is_active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_users_tenant_email UNIQUE (tenant_id, email),
    CONSTRAINT uq_users_tenant_username UNIQUE (tenant_id, username)
);
CREATE INDEX idx_users_tenant ON gb.users(tenant_id);
CREATE INDEX idx_users_external ON gb.users(external_id);
CREATE INDEX idx_users_email ON gb.users(email);

-- Bots
CREATE TABLE gb.bots (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    llm_provider gb.llm_provider NOT NULL DEFAULT 'openai',
    llm_config JSONB DEFAULT '{}'::jsonb,
    context_provider gb.context_provider NOT NULL DEFAULT 'qdrant',
    context_config JSONB DEFAULT '{}'::jsonb,
    system_prompt TEXT,
    personality JSONB DEFAULT '{}'::jsonb,
    capabilities JSONB DEFAULT '[]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_bots_tenant_name UNIQUE (tenant_id, name)
);
CREATE INDEX idx_bots_tenant ON gb.bots(tenant_id);
CREATE INDEX idx_bots_external ON gb.bots(external_id);
CREATE INDEX idx_bots_active ON gb.bots(tenant_id, is_active) WHERE is_active;

-- Bot Channels
CREATE TABLE gb.bot_channels (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    channel_type gb.channel_type NOT NULL,
    channel_identifier VARCHAR(255),  -- phone number, email, webhook id, etc.
    config JSONB DEFAULT '{}'::jsonb,
    credentials_vault_path VARCHAR(512),  -- Reference to Vault secret
    is_active BOOLEAN DEFAULT true,
    last_activity_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_bot_channel UNIQUE (bot_id, channel_type, channel_identifier)
);
CREATE INDEX idx_bot_channels_bot ON gb.bot_channels(bot_id);
CREATE INDEX idx_bot_channels_type ON gb.bot_channels(channel_type);

-- ============================================================================
-- SESSION AND MESSAGE TABLES - High volume, partition-ready
-- ============================================================================

-- User Sessions (partitioned by created_at for time-series queries)
CREATE TABLE gb.sessions (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    bot_id BIGINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    channel_type gb.channel_type NOT NULL DEFAULT 'web',
    title VARCHAR(512) DEFAULT 'New Conversation',
    context_data JSONB DEFAULT '{}'::jsonb,
    current_tool VARCHAR(255),
    message_count INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id, created_at)
) PARTITION BY RANGE (created_at);

-- Create partitions for sessions (monthly)
CREATE TABLE gb.sessions_y2024m01 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE gb.sessions_y2024m02 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
CREATE TABLE gb.sessions_y2024m03 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');
CREATE TABLE gb.sessions_y2024m04 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-04-01') TO ('2024-05-01');
CREATE TABLE gb.sessions_y2024m05 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-05-01') TO ('2024-06-01');
CREATE TABLE gb.sessions_y2024m06 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-06-01') TO ('2024-07-01');
CREATE TABLE gb.sessions_y2024m07 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-07-01') TO ('2024-08-01');
CREATE TABLE gb.sessions_y2024m08 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-08-01') TO ('2024-09-01');
CREATE TABLE gb.sessions_y2024m09 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-09-01') TO ('2024-10-01');
CREATE TABLE gb.sessions_y2024m10 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-10-01') TO ('2024-11-01');
CREATE TABLE gb.sessions_y2024m11 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-11-01') TO ('2024-12-01');
CREATE TABLE gb.sessions_y2024m12 PARTITION OF gb.sessions
    FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');
CREATE TABLE gb.sessions_y2025m01 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
CREATE TABLE gb.sessions_y2025m02 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');
CREATE TABLE gb.sessions_y2025m03 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-03-01') TO ('2025-04-01');
CREATE TABLE gb.sessions_y2025m04 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-04-01') TO ('2025-05-01');
CREATE TABLE gb.sessions_y2025m05 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-05-01') TO ('2025-06-01');
CREATE TABLE gb.sessions_y2025m06 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-06-01') TO ('2025-07-01');
CREATE TABLE gb.sessions_y2025m07 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-07-01') TO ('2025-08-01');
CREATE TABLE gb.sessions_y2025m08 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-08-01') TO ('2025-09-01');
CREATE TABLE gb.sessions_y2025m09 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-09-01') TO ('2025-10-01');
CREATE TABLE gb.sessions_y2025m10 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');
CREATE TABLE gb.sessions_y2025m11 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');
CREATE TABLE gb.sessions_y2025m12 PARTITION OF gb.sessions
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
-- Default partition for future data
CREATE TABLE gb.sessions_default PARTITION OF gb.sessions DEFAULT;

CREATE INDEX idx_sessions_user ON gb.sessions(user_id, created_at DESC);
CREATE INDEX idx_sessions_bot ON gb.sessions(bot_id, created_at DESC);
CREATE INDEX idx_sessions_tenant ON gb.sessions(tenant_id, created_at DESC);
CREATE INDEX idx_sessions_external ON gb.sessions(external_id);

-- Message History (partitioned by created_at, highest volume table)
CREATE TABLE gb.messages (
    id BIGSERIAL,
    session_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    role gb.message_role NOT NULL,
    message_type gb.message_type NOT NULL DEFAULT 'text',
    content TEXT NOT NULL,                   -- Encrypted content
    content_hash CHAR(64),                   -- SHA-256 for deduplication
    media_url VARCHAR(1024),
    metadata JSONB DEFAULT '{}'::jsonb,
    token_count INT DEFAULT 0,
    processing_time_ms INT,
    llm_model VARCHAR(64),
    message_index INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id, created_at)
) PARTITION BY RANGE (created_at);

-- Create partitions for messages (monthly - can be more granular for high volume)
CREATE TABLE gb.messages_y2024m01 PARTITION OF gb.messages FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE gb.messages_y2024m02 PARTITION OF gb.messages FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
CREATE TABLE gb.messages_y2024m03 PARTITION OF gb.messages FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');
CREATE TABLE gb.messages_y2024m04 PARTITION OF gb.messages FOR VALUES FROM ('2024-04-01') TO ('2024-05-01');
CREATE TABLE gb.messages_y2024m05 PARTITION OF gb.messages FOR VALUES FROM ('2024-05-01') TO ('2024-06-01');
CREATE TABLE gb.messages_y2024m06 PARTITION OF gb.messages FOR VALUES FROM ('2024-06-01') TO ('2024-07-01');
CREATE TABLE gb.messages_y2024m07 PARTITION OF gb.messages FOR VALUES FROM ('2024-07-01') TO ('2024-08-01');
CREATE TABLE gb.messages_y2024m08 PARTITION OF gb.messages FOR VALUES FROM ('2024-08-01') TO ('2024-09-01');
CREATE TABLE gb.messages_y2024m09 PARTITION OF gb.messages FOR VALUES FROM ('2024-09-01') TO ('2024-10-01');
CREATE TABLE gb.messages_y2024m10 PARTITION OF gb.messages FOR VALUES FROM ('2024-10-01') TO ('2024-11-01');
CREATE TABLE gb.messages_y2024m11 PARTITION OF gb.messages FOR VALUES FROM ('2024-11-01') TO ('2024-12-01');
CREATE TABLE gb.messages_y2024m12 PARTITION OF gb.messages FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');
CREATE TABLE gb.messages_y2025m01 PARTITION OF gb.messages FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
CREATE TABLE gb.messages_y2025m02 PARTITION OF gb.messages FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');
CREATE TABLE gb.messages_y2025m03 PARTITION OF gb.messages FOR VALUES FROM ('2025-03-01') TO ('2025-04-01');
CREATE TABLE gb.messages_y2025m04 PARTITION OF gb.messages FOR VALUES FROM ('2025-04-01') TO ('2025-05-01');
CREATE TABLE gb.messages_y2025m05 PARTITION OF gb.messages FOR VALUES FROM ('2025-05-01') TO ('2025-06-01');
CREATE TABLE gb.messages_y2025m06 PARTITION OF gb.messages FOR VALUES FROM ('2025-06-01') TO ('2025-07-01');
CREATE TABLE gb.messages_y2025m07 PARTITION OF gb.messages FOR VALUES FROM ('2025-07-01') TO ('2025-08-01');
CREATE TABLE gb.messages_y2025m08 PARTITION OF gb.messages FOR VALUES FROM ('2025-08-01') TO ('2025-09-01');
CREATE TABLE gb.messages_y2025m09 PARTITION OF gb.messages FOR VALUES FROM ('2025-09-01') TO ('2025-10-01');
CREATE TABLE gb.messages_y2025m10 PARTITION OF gb.messages FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');
CREATE TABLE gb.messages_y2025m11 PARTITION OF gb.messages FOR VALUES FROM ('2025-11-01') TO ('2025-12-01');
CREATE TABLE gb.messages_y2025m12 PARTITION OF gb.messages FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
CREATE TABLE gb.messages_default PARTITION OF gb.messages DEFAULT;

CREATE INDEX idx_messages_session ON gb.messages(session_id, message_index);
CREATE INDEX idx_messages_tenant ON gb.messages(tenant_id, created_at DESC);
CREATE INDEX idx_messages_user ON gb.messages(user_id, created_at DESC);

-- ============================================================================
-- CONFIGURATION TABLES
-- ============================================================================

-- Bot Configuration (key-value with proper typing)
CREATE TABLE gb.bot_config (
    id BIGSERIAL PRIMARY KEY,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    config_key VARCHAR(128) NOT NULL,
    config_value TEXT NOT NULL,
    value_type SMALLINT NOT NULL DEFAULT 0,  -- 0=string, 1=int, 2=float, 3=bool, 4=json
    is_secret BOOLEAN DEFAULT false,
    vault_path VARCHAR(512),  -- If is_secret, reference to Vault
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_bot_config UNIQUE (bot_id, config_key)
);
CREATE INDEX idx_bot_config_bot ON gb.bot_config(bot_id);
CREATE INDEX idx_bot_config_key ON gb.bot_config(config_key);

-- ============================================================================
-- MEMORY TABLES
-- ============================================================================

-- Bot Memories (for long-term context)
CREATE TABLE gb.memories (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT,
    session_id BIGINT,
    memory_type gb.memory_type NOT NULL,
    content TEXT NOT NULL,
    embedding_id VARCHAR(128),  -- Reference to vector DB
    importance_score REAL DEFAULT 0.5,
    access_count INT DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_memories_bot ON gb.memories(bot_id, memory_type);
CREATE INDEX idx_memories_user ON gb.memories(user_id, memory_type);
CREATE INDEX idx_memories_importance ON gb.memories(bot_id, importance_score DESC);

-- ============================================================================
-- AUTONOMOUS TASK TABLES
-- ============================================================================

-- Auto Tasks
CREATE TABLE gb.auto_tasks (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    session_id BIGINT,
    external_id UUID DEFAULT gen_random_uuid(),
    title VARCHAR(512) NOT NULL,
    intent TEXT NOT NULL,
    status gb.task_status NOT NULL DEFAULT 'pending',
    execution_mode gb.execution_mode NOT NULL DEFAULT 'supervised',
    priority gb.task_priority NOT NULL DEFAULT 'normal',
    plan_id BIGINT,
    basic_program TEXT,
    current_step INT DEFAULT 0,
    total_steps INT DEFAULT 0,
    progress REAL DEFAULT 0.0,
    step_results JSONB DEFAULT '[]'::jsonb,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_auto_tasks_bot ON gb.auto_tasks(bot_id, status);
CREATE INDEX idx_auto_tasks_session ON gb.auto_tasks(session_id);
CREATE INDEX idx_auto_tasks_status ON gb.auto_tasks(status, priority);
CREATE INDEX idx_auto_tasks_external ON gb.auto_tasks(external_id);

-- Execution Plans
CREATE TABLE gb.execution_plans (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    task_id BIGINT,
    external_id UUID DEFAULT gen_random_uuid(),
    intent TEXT NOT NULL,
    intent_type gb.intent_type,
    confidence REAL DEFAULT 0.0,
    status gb.plan_status NOT NULL DEFAULT 'pending',
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    basic_program TEXT,
    simulation_result JSONB,
    risk_level gb.risk_level DEFAULT 'low',
    approved_by BIGINT,
    approved_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_execution_plans_bot ON gb.execution_plans(bot_id, status);
CREATE INDEX idx_execution_plans_task ON gb.execution_plans(task_id);
CREATE INDEX idx_execution_plans_external ON gb.execution_plans(external_id);

-- Task Approvals
CREATE TABLE gb.task_approvals (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    task_id BIGINT NOT NULL,
    plan_id BIGINT,
    step_index INT,
    action_type VARCHAR(128) NOT NULL,
    action_description TEXT NOT NULL,
    risk_level gb.risk_level DEFAULT 'low',
    status gb.approval_status NOT NULL DEFAULT 'pending',
    decision gb.approval_decision,
    decision_reason TEXT,
    decided_by BIGINT,
    decided_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_task_approvals_task ON gb.task_approvals(task_id);
CREATE INDEX idx_task_approvals_status ON gb.task_approvals(status, expires_at);

-- Task Decisions
CREATE TABLE gb.task_decisions (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    task_id BIGINT NOT NULL,
    question TEXT NOT NULL,
    options JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    status gb.approval_status NOT NULL DEFAULT 'pending',
    selected_option VARCHAR(255),
    decision_reason TEXT,
    decided_by BIGINT,
    decided_at TIMESTAMPTZ,
    timeout_seconds INT DEFAULT 3600,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_task_decisions_task ON gb.task_decisions(task_id);
CREATE INDEX idx_task_decisions_status ON gb.task_decisions(status);

-- Safety Audit Log
CREATE TABLE gb.safety_audit_log (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    task_id BIGINT,
    plan_id BIGINT,
    action_type VARCHAR(128) NOT NULL,
    action_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    constraint_checks JSONB DEFAULT '[]'::jsonb,
    simulation_result JSONB,
    risk_assessment JSONB,
    outcome gb.safety_outcome NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_safety_audit_bot ON gb.safety_audit_log(bot_id, created_at DESC);
CREATE INDEX idx_safety_audit_outcome ON gb.safety_audit_log(outcome, created_at DESC);

-- Intent Classifications (for analytics and ML)
CREATE TABLE gb.intent_classifications (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    session_id BIGINT,
    original_text TEXT NOT NULL,
    intent_type gb.intent_type NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.0,
    entities JSONB DEFAULT '{}'::jsonb,
    suggested_name VARCHAR(255),
    was_correct BOOLEAN,
    corrected_type gb.intent_type,
    feedback TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_intent_class_bot ON gb.intent_classifications(bot_id, intent_type);
CREATE INDEX idx_intent_class_confidence ON gb.intent_classifications(confidence);

-- ============================================================================
-- APP GENERATION TABLES
-- ============================================================================

-- Generated Apps
CREATE TABLE gb.generated_apps (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    domain VARCHAR(128),
    intent_source TEXT,
    pages JSONB DEFAULT '[]'::jsonb,
    tables_created JSONB DEFAULT '[]'::jsonb,
    tools JSONB DEFAULT '[]'::jsonb,
    schedulers JSONB DEFAULT '[]'::jsonb,
    app_path VARCHAR(512),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_generated_apps UNIQUE (bot_id, name)
);
CREATE INDEX idx_generated_apps_bot ON gb.generated_apps(bot_id);
CREATE INDEX idx_generated_apps_external ON gb.generated_apps(external_id);

-- Designer Changes (for undo support)
CREATE TABLE gb.designer_changes (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    session_id BIGINT,
    change_type gb.designer_change_type NOT NULL,
    description TEXT NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    original_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_designer_changes_bot ON gb.designer_changes(bot_id, created_at DESC);

-- ============================================================================
-- KNOWLEDGE BASE TABLES
-- ============================================================================

-- KB Collections
CREATE TABLE gb.kb_collections (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    folder_path VARCHAR(512),
    qdrant_collection VARCHAR(255),
    document_count INT DEFAULT 0,
    chunk_count INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    last_indexed_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_kb_collections UNIQUE (bot_id, name)
);
CREATE INDEX idx_kb_collections_bot ON gb.kb_collections(bot_id);
CREATE INDEX idx_kb_collections_external ON gb.kb_collections(external_id);

-- KB Documents
CREATE TABLE gb.kb_documents (
    id BIGSERIAL,
    collection_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    file_path VARCHAR(512) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(32),
    file_size BIGINT,
    content_hash CHAR(64),
    chunk_count INT DEFAULT 0,
    is_indexed BOOLEAN DEFAULT false,
    indexed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_kb_documents_collection ON gb.kb_documents(collection_id);
CREATE INDEX idx_kb_documents_hash ON gb.kb_documents(content_hash);

-- Session KB Associations
CREATE TABLE gb.session_kb_associations (
    id BIGSERIAL,
    session_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    kb_name VARCHAR(255) NOT NULL,
    kb_folder_path VARCHAR(512),
    qdrant_collection VARCHAR(255),
    added_by_tool VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_session_kb UNIQUE (session_id, kb_name)
);
CREATE INDEX idx_session_kb_session ON gb.session_kb_associations(session_id);

-- ============================================================================
-- ANALYTICS TABLES (partitioned for high volume)
-- ============================================================================

-- Usage Analytics (daily aggregates per user/bot)
CREATE TABLE gb.usage_analytics (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    bot_id BIGINT NOT NULL,
    date DATE NOT NULL,
    session_count INT DEFAULT 0,
    message_count INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    total_processing_time_ms BIGINT DEFAULT 0,
    avg_response_time_ms INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id, date),
    CONSTRAINT uq_usage_daily UNIQUE (user_id, bot_id, date)
) PARTITION BY RANGE (date);

-- Create partitions for analytics (monthly)
CREATE TABLE gb.usage_analytics_y2024m01 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE gb.usage_analytics_y2024m02 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
CREATE TABLE gb.usage_analytics_y2024m03 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');
CREATE TABLE gb.usage_analytics_y2024m04 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-04-01') TO ('2024-05-01');
CREATE TABLE gb.usage_analytics_y2024m05 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-05-01') TO ('2024-06-01');
CREATE TABLE gb.usage_analytics_y2024m06 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-06-01') TO ('2024-07-01');
CREATE TABLE gb.usage_analytics_y2024m07 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-07-01') TO ('2024-08-01');
CREATE TABLE gb.usage_analytics_y2024m08 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-08-01') TO ('2024-09-01');
CREATE TABLE gb.usage_analytics_y2024m09 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-09-01') TO ('2024-10-01');
CREATE TABLE gb.usage_analytics_y2024m10 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-10-01') TO ('2024-11-01');
CREATE TABLE gb.usage_analytics_y2024m11 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-11-01') TO ('2024-12-01');
CREATE TABLE gb.usage_analytics_y2024m12 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');
CREATE TABLE gb.usage_analytics_y2025 PARTITION OF gb.usage_analytics FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');
CREATE TABLE gb.usage_analytics_default PARTITION OF gb.usage_analytics DEFAULT;

CREATE INDEX idx_usage_analytics_tenant ON gb.usage_analytics(tenant_id, date);
CREATE INDEX idx_usage_analytics_bot ON gb.usage_analytics(bot_id, date);

-- Analytics Events (for detailed tracking)
CREATE TABLE gb.analytics_events (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT,
    session_id BIGINT,
    bot_id BIGINT,
    event_type VARCHAR(64) NOT NULL,
    event_data JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id, created_at)
) PARTITION BY RANGE (created_at);

CREATE TABLE gb.analytics_events_y2024 PARTITION OF gb.analytics_events FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');
CREATE TABLE gb.analytics_events_y2025 PARTITION OF gb.analytics_events FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');
CREATE TABLE gb.analytics_events_default PARTITION OF gb.analytics_events DEFAULT;

CREATE INDEX idx_analytics_events_type ON gb.analytics_events(event_type, created_at DESC);
CREATE INDEX idx_analytics_events_tenant ON gb.analytics_events(tenant_id, created_at DESC);

-- ============================================================================
-- TOOLS AND AUTOMATION TABLES
-- ============================================================================

-- Tools Definition
CREATE TABLE gb.tools (
    id BIGSERIAL,
    bot_id BIGINT,  -- NULL for system-wide tools
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    parameters JSONB DEFAULT '{}'::jsonb,
    script TEXT NOT NULL,
    tool_type VARCHAR(64) DEFAULT 'basic',
    is_system BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    usage_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_tools_bot ON gb.tools(bot_id);
CREATE INDEX idx_tools_name ON gb.tools(name);
CREATE UNIQUE INDEX idx_tools_unique_name ON gb.tools(tenant_id, COALESCE(bot_id, 0), name);

-- System Automations
CREATE TABLE gb.automations (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    kind SMALLINT NOT NULL,  -- 1=scheduler, 2=monitor, 3=trigger
    target VARCHAR(255),
    schedule VARCHAR(64),  -- Cron expression
    param VARCHAR(255),
    recurrence gb.recurrence_pattern,
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMPTZ,
    next_trigger TIMESTAMPTZ,
    run_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_automations_bot ON gb.automations(bot_id);
CREATE INDEX idx_automations_next ON gb.automations(next_trigger) WHERE is_active;

-- ============================================================================
-- CALENDAR AND SCHEDULING TABLES
-- ============================================================================

-- Calendar Events
CREATE TABLE gb.calendar_events (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    title VARCHAR(512) NOT NULL,
    description TEXT,
    location VARCHAR(512),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    all_day BOOLEAN DEFAULT false,
    recurrence gb.recurrence_pattern,
    recurrence_rule TEXT,
    reminder_minutes INT[],
    status gb.booking_status DEFAULT 'confirmed',
    is_private BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_calendar_events_user ON gb.calendar_events(user_id, start_time);
CREATE INDEX idx_calendar_events_time ON gb.calendar_events(start_time, end_time);

-- Resources (rooms, equipment, etc.)
CREATE TABLE gb.resources (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type gb.resource_type NOT NULL,
    capacity INT,
    location VARCHAR(512),
    amenities JSONB DEFAULT '[]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_resources_tenant ON gb.resources(tenant_id, resource_type);

-- Resource Bookings
CREATE TABLE gb.resource_bookings (
    id BIGSERIAL,
    resource_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    event_id BIGINT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    status gb.booking_status NOT NULL DEFAULT 'pending',
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_resource_bookings_resource ON gb.resource_bookings(resource_id, start_time);
CREATE INDEX idx_resource_bookings_user ON gb.resource_bookings(user_id);

-- ============================================================================
-- EMAIL TABLES
-- ============================================================================

-- Email Messages
CREATE TABLE gb.email_messages (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    folder VARCHAR(64) DEFAULT 'inbox',
    from_address VARCHAR(255) NOT NULL,
    to_addresses TEXT[] NOT NULL,
    cc_addresses TEXT[],
    bcc_addresses TEXT[],
    subject VARCHAR(998),
    body_text TEXT,
    body_html TEXT,
    headers JSONB DEFAULT '{}'::jsonb,
    attachments JSONB DEFAULT '[]'::jsonb,
    status gb.email_status NOT NULL DEFAULT 'draft',
    is_read BOOLEAN DEFAULT false,
    is_starred BOOLEAN DEFAULT false,
    labels TEXT[],
    sent_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_email_messages_user ON gb.email_messages(user_id, folder, received_at DESC);
CREATE INDEX idx_email_messages_status ON gb.email_messages(status);

-- ============================================================================
-- MEETING TABLES
-- ============================================================================

-- Meetings
CREATE TABLE gb.meetings (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    bot_id BIGINT,
    external_id UUID DEFAULT gen_random_uuid(),
    title VARCHAR(512) NOT NULL,
    description TEXT,
    host_id BIGINT NOT NULL,
    room_code VARCHAR(32) UNIQUE,
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    max_participants INT DEFAULT 100,
    settings JSONB DEFAULT '{}'::jsonb,
    recording_url VARCHAR(1024),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_meetings_host ON gb.meetings(host_id);
CREATE INDEX idx_meetings_room ON gb.meetings(room_code);

-- Meeting Participants
CREATE TABLE gb.meeting_participants (
    id BIGSERIAL,
    meeting_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    user_id BIGINT,
    external_email VARCHAR(255),
    display_name VARCHAR(255),
    status gb.participant_status NOT NULL DEFAULT 'invited',
    role VARCHAR(32) DEFAULT 'participant',
    joined_at TIMESTAMPTZ,
    left_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_meeting_participants_meeting ON gb.meeting_participants(meeting_id);
CREATE INDEX idx_meeting_participants_user ON gb.meeting_participants(user_id);

-- ============================================================================
-- TASK MANAGEMENT TABLES
-- ============================================================================

-- Tasks (traditional task management, not auto_tasks)
CREATE TABLE gb.tasks (
    id BIGSERIAL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    title VARCHAR(512) NOT NULL,
    description TEXT,
    assignee_id BIGINT,
    reporter_id BIGINT,
    project_id BIGINT,
    parent_task_id BIGINT,
    status gb.task_status NOT NULL DEFAULT 'pending',
    priority gb.task_priority NOT NULL DEFAULT 'normal',
    due_date TIMESTAMPTZ,
    estimated_hours REAL,
    actual_hours REAL,
    progress SMALLINT DEFAULT 0,
    tags TEXT[],
    dependencies BIGINT[],
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_tasks_assignee ON gb.tasks(assignee_id, status);
CREATE INDEX idx_tasks_project ON gb.tasks(project_id, status);
CREATE INDEX idx_tasks_due ON gb.tasks(due_date) WHERE status NOT IN ('completed', 'cancelled');

-- Task Comments
CREATE TABLE gb.task_comments (
    id BIGSERIAL,
    task_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    author_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_task_comments_task ON gb.task_comments(task_id);

-- ============================================================================
-- CONNECTED ACCOUNTS AND INTEGRATIONS
-- ============================================================================

-- Connected Accounts (OAuth integrations)
CREATE TABLE gb.connected_accounts (
    id BIGSERIAL,
    user_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    external_id UUID DEFAULT gen_random_uuid(),
    provider VARCHAR(64) NOT NULL,
    provider_user_id VARCHAR(255),
    email VARCHAR(255),
    display_name VARCHAR(255),
    access_token_vault VARCHAR(512),  -- Vault path for encrypted token
    refresh_token_vault VARCHAR(512),
    token_expires_at TIMESTAMPTZ,
    scopes TEXT[],
    sync_status gb.sync_status DEFAULT 'pending',
    last_sync_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id),
    CONSTRAINT uq_connected_accounts UNIQUE (user_id, provider, provider_user_id)
);
CREATE INDEX idx_connected_accounts_user ON gb.connected_accounts(user_id);
CREATE INDEX idx_connected_accounts_provider ON gb.connected_accounts(provider);

-- ============================================================================
-- PENDING INFO (for ASK LATER keyword)
-- ============================================================================

CREATE TABLE gb.pending_info (
    id BIGSERIAL,
    bot_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    shard_id SMALLINT NOT NULL,
    field_name VARCHAR(128) NOT NULL,
    field_label VARCHAR(255) NOT NULL,
    field_type VARCHAR(64) NOT NULL DEFAULT 'text',
    reason TEXT,
    config_key VARCHAR(255) NOT NULL,
    is_filled BOOLEAN DEFAULT false,
    filled_at TIMESTAMPTZ,
    filled_value TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, shard_id)
);
CREATE INDEX idx_pending_info_bot ON gb.pending_info(bot_id, is_filled);
CREATE INDEX idx_pending_info_config ON gb.pending_info(config_key);

-- ============================================================================
-- HELPER FUNCTIONS FOR SHARDING
-- ============================================================================

-- Function to get shard_id for a tenant
CREATE OR REPLACE FUNCTION gb.get_shard_id(p_tenant_id BIGINT)
RETURNS SMALLINT AS $$
BEGIN
    RETURN (SELECT shard_id FROM gb.tenant_shard_map WHERE tenant_id = p_tenant_id);
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to generate next ID with shard awareness
CREATE OR REPLACE FUNCTION gb.generate_sharded_id(p_shard_id SMALLINT)
RETURNS BIGINT AS $$
DECLARE
    v_time_part BIGINT;
    v_shard_part BIGINT;
    v_seq_part BIGINT;
BEGIN
    -- Snowflake-like ID: timestamp (41 bits) + shard (10 bits) + sequence (12 bits)
    v_time_part := (EXTRACT(EPOCH FROM NOW())::BIGINT - 1704067200) << 22;  -- Since 2024-01-01
    v_shard_part := (p_shard_id::BIGINT & 1023) << 12;
    v_seq_part := (nextval('gb.global_seq') & 4095);
    RETURN v_time_part | v_shard_part | v_seq_part;
END;
$$ LANGUAGE plpgsql;

-- Global sequence for ID generation
CREATE SEQUENCE IF NOT EXISTS gb.global_seq;

-- ============================================================================
-- GRANTS AND COMMENTS
-- ============================================================================

COMMENT ON SCHEMA gb IS 'General Bots billion-scale schema v7.0.0';
COMMENT ON TABLE gb.shard_config IS 'Shard configuration for horizontal scaling';
COMMENT ON TABLE gb.tenant_shard_map IS 'Maps tenants to their respective shards';
COMMENT ON TABLE gb.tenants IS 'Multi-tenant organizations';
COMMENT ON TABLE gb.users IS 'User accounts with tenant isolation';
COMMENT ON TABLE gb.bots IS 'Bot configurations';
COMMENT ON TABLE gb.sessions IS 'Conversation sessions (partitioned by month)';
COMMENT ON TABLE gb.messages IS 'Message history (partitioned by month, highest volume)';
COMMENT ON TABLE gb.auto_tasks IS 'Autonomous task execution';
COMMENT ON TABLE gb.execution_plans IS 'LLM-compiled execution plans';

-- Default shard for single-node deployment
INSERT INTO gb.shard_config (shard_id, region_code, datacenter, connection_string, is_primary, min_tenant_id, max_tenant_id)
VALUES (1, 'USA', 'local', 'postgresql://localhost:5432/botserver', true, 1, 9223372036854775807);

-- Default tenant for backwards compatibility
INSERT INTO gb.tenants (id, shard_id, name, slug, region_code, plan_tier)
VALUES (1, 1, 'Default', 'default', 'USA', 0);

INSERT INTO gb.tenant_shard_map (tenant_id, shard_id, region_code)
VALUES (1, 1, 'USA');
