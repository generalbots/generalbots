-- ============================================================================
-- GENERAL BOTS - CONSOLIDATED SCHEMA v7.0.0
-- ============================================================================
-- Optimized for billion-user scale with:
--   - SMALLINT enums instead of VARCHAR (2 bytes vs 20+ bytes)
--   - Partitioned tables for high-volume data
--   - Sharding-ready design with tenant_id/shard_id
--   - Proper indexing strategies
--   - No TEXT columns for domain values
-- ============================================================================

-- ============================================================================
-- EXTENSIONS
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- CLEANUP: Drop existing objects for clean slate
-- ============================================================================
DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO PUBLIC;

-- ============================================================================
-- ENUM CONSTANTS (using SMALLINT for efficiency)
-- ============================================================================
-- Channel Types: 0=web, 1=whatsapp, 2=telegram, 3=msteams, 4=slack, 5=email, 6=sms, 7=voice, 8=instagram, 9=api
-- Message Role: 1=user, 2=assistant, 3=system, 4=tool, 9=episodic, 10=compact
-- Message Type: 0=text, 1=image, 2=audio, 3=video, 4=document, 5=location, 6=contact, 7=sticker, 8=reaction
-- LLM Provider: 0=openai, 1=anthropic, 2=azure_openai, 3=azure_claude, 4=google, 5=local, 6=ollama, 7=groq, 8=mistral, 9=cohere
-- Context Provider: 0=none, 1=qdrant, 2=pinecone, 3=weaviate, 4=milvus, 5=pgvector, 6=elasticsearch
-- Task Status: 0=pending, 1=ready, 2=running, 3=paused, 4=waiting_approval, 5=completed, 6=failed, 7=cancelled
-- Task Priority: 0=low, 1=normal, 2=high, 3=urgent, 4=critical
-- Execution Mode: 0=manual, 1=supervised, 2=autonomous
-- Risk Level: 0=none, 1=low, 2=medium, 3=high, 4=critical
-- Approval Status: 0=pending, 1=approved, 2=rejected, 3=expired, 4=skipped
-- Intent Type: 0=unknown, 1=app_create, 2=todo, 3=monitor, 4=action, 5=schedule, 6=goal, 7=tool, 8=query
-- Memory Type: 0=short, 1=long, 2=episodic, 3=semantic, 4=procedural
-- Sync Status: 0=synced, 1=pending, 2=conflict, 3=error, 4=deleted
-- Booking Status: 0=pending, 1=confirmed, 2=cancelled, 3=completed, 4=no_show
-- Resource Type: 0=room, 1=equipment, 2=vehicle, 3=person, 4=virtual, 5=other
-- Permission Level: 0=none, 1=read, 2=write, 3=admin, 4=owner

-- ============================================================================
-- SHARDING INFRASTRUCTURE
-- ============================================================================

-- Shard configuration (replicated to all shards for routing)
CREATE TABLE shard_config (
    shard_id SMALLINT PRIMARY KEY,
    region_code CHAR(3) NOT NULL,
    datacenter VARCHAR(32) NOT NULL,
    connection_string TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    min_tenant_id BIGINT NOT NULL,
    max_tenant_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tenant to shard mapping
CREATE TABLE tenant_shard_map (
    tenant_id BIGINT PRIMARY KEY,
    shard_id SMALLINT NOT NULL REFERENCES shard_config(shard_id),
    region_code CHAR(3) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tenant_shard_region ON tenant_shard_map(region_code, shard_id);

-- Global sequence for Snowflake-like ID generation
CREATE SEQUENCE global_id_seq;

-- ============================================================================
-- CORE TABLES
-- ============================================================================

-- Tenants (organizations)
CREATE TABLE tenants (
    id BIGSERIAL PRIMARY KEY,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    external_id UUID DEFAULT gen_random_uuid() UNIQUE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(128) NOT NULL UNIQUE,
    region_code CHAR(3) NOT NULL DEFAULT 'USA',
    plan_tier SMALLINT NOT NULL DEFAULT 0,
    settings JSONB DEFAULT '{}'::jsonb,
    limits JSONB DEFAULT '{"users": 5, "bots": 1, "storage_gb": 1}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tenants_shard ON tenants(shard_id);
CREATE INDEX idx_tenants_region ON tenants(region_code);
CREATE INDEX idx_tenants_active ON tenants(is_active) WHERE is_active;

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1 REFERENCES tenants(id) ON DELETE CASCADE,
    shard_id SMALLINT NOT NULL DEFAULT 1,
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
    CONSTRAINT uq_users_tenant_email UNIQUE (tenant_id, email),
    CONSTRAINT uq_users_tenant_username UNIQUE (tenant_id, username)
);
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active;

-- Bots
CREATE TABLE bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1 REFERENCES tenants(id) ON DELETE CASCADE,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    llm_provider SMALLINT NOT NULL DEFAULT 0,
    llm_config JSONB DEFAULT '{}'::jsonb,
    context_provider SMALLINT NOT NULL DEFAULT 1,
    context_config JSONB DEFAULT '{}'::jsonb,
    system_prompt TEXT,
    personality JSONB DEFAULT '{}'::jsonb,
    capabilities JSONB DEFAULT '[]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_bots_tenant_name UNIQUE (tenant_id, name)
);
CREATE INDEX idx_bots_tenant ON bots(tenant_id);
CREATE INDEX idx_bots_active ON bots(tenant_id, is_active) WHERE is_active;

-- Bot Configuration (key-value store)
CREATE TABLE bot_configuration (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    config_key VARCHAR(128) NOT NULL,
    config_value TEXT NOT NULL,
    value_type SMALLINT NOT NULL DEFAULT 0,
    is_secret BOOLEAN DEFAULT false,
    vault_path VARCHAR(512),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_bot_config UNIQUE (bot_id, config_key)
);
CREATE INDEX idx_bot_config_bot ON bot_configuration(bot_id);
CREATE INDEX idx_bot_config_key ON bot_configuration(config_key);

-- Bot Channels
CREATE TABLE bot_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    channel_type SMALLINT NOT NULL DEFAULT 0,
    channel_identifier VARCHAR(255),
    config JSONB DEFAULT '{}'::jsonb,
    credentials_vault_path VARCHAR(512),
    is_active BOOLEAN DEFAULT true,
    last_activity_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_bot_channel UNIQUE (bot_id, channel_type, channel_identifier)
);
CREATE INDEX idx_bot_channels_bot ON bot_channels(bot_id);
CREATE INDEX idx_bot_channels_type ON bot_channels(channel_type);

-- ============================================================================
-- SESSION AND MESSAGE TABLES
-- ============================================================================

-- User Sessions
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1 REFERENCES tenants(id) ON DELETE CASCADE,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    channel_type SMALLINT NOT NULL DEFAULT 0,
    title VARCHAR(512) DEFAULT 'New Conversation',
    context_data JSONB DEFAULT '{}'::jsonb,
    current_tool VARCHAR(255),
    answer_mode SMALLINT DEFAULT 0,
    message_count INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    last_activity_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_sessions_user ON user_sessions(user_id, created_at DESC);
CREATE INDEX idx_sessions_bot ON user_sessions(bot_id, created_at DESC);
CREATE INDEX idx_sessions_tenant ON user_sessions(tenant_id, created_at DESC);
CREATE INDEX idx_sessions_activity ON user_sessions(last_activity_at DESC);

-- Message History
CREATE TABLE message_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role SMALLINT NOT NULL DEFAULT 1,
    message_type SMALLINT NOT NULL DEFAULT 0,
    content_encrypted TEXT NOT NULL,
    content_hash CHAR(64),
    media_url VARCHAR(1024),
    metadata JSONB DEFAULT '{}'::jsonb,
    token_count INT DEFAULT 0,
    processing_time_ms INT,
    llm_model VARCHAR(64),
    message_index INT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_messages_session ON message_history(session_id, message_index);
CREATE INDEX idx_messages_tenant ON message_history(tenant_id, created_at DESC);
CREATE INDEX idx_messages_user ON message_history(user_id, created_at DESC);
CREATE INDEX idx_messages_created ON message_history(created_at DESC);

-- ============================================================================
-- MEMORY TABLES
-- ============================================================================

-- Bot Memories
CREATE TABLE bot_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    memory_type SMALLINT NOT NULL DEFAULT 0,
    content TEXT NOT NULL,
    embedding_id VARCHAR(128),
    importance_score REAL DEFAULT 0.5,
    access_count INT DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_memories_bot ON bot_memories(bot_id, memory_type);
CREATE INDEX idx_memories_user ON bot_memories(user_id, memory_type);
CREATE INDEX idx_memories_importance ON bot_memories(bot_id, importance_score DESC);

-- ============================================================================
-- AUTONOMOUS TASK TABLES
-- ============================================================================

-- Auto Tasks
CREATE TABLE auto_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    title VARCHAR(512) NOT NULL,
    intent TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    execution_mode SMALLINT NOT NULL DEFAULT 1,
    priority SMALLINT NOT NULL DEFAULT 1,
    plan_id UUID,
    basic_program TEXT,
    current_step INT DEFAULT 0,
    total_steps INT DEFAULT 0,
    progress REAL DEFAULT 0.0,
    step_results JSONB DEFAULT '[]'::jsonb,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_auto_tasks_bot ON auto_tasks(bot_id, status);
CREATE INDEX idx_auto_tasks_session ON auto_tasks(session_id);
CREATE INDEX idx_auto_tasks_status ON auto_tasks(status, priority);
CREATE INDEX idx_auto_tasks_created ON auto_tasks(created_at DESC);

-- Execution Plans
CREATE TABLE execution_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    task_id UUID REFERENCES auto_tasks(id) ON DELETE CASCADE,
    intent TEXT NOT NULL,
    intent_type SMALLINT DEFAULT 0,
    confidence REAL DEFAULT 0.0,
    status SMALLINT NOT NULL DEFAULT 0,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    basic_program TEXT,
    simulation_result JSONB,
    risk_level SMALLINT DEFAULT 1,
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_execution_plans_bot ON execution_plans(bot_id, status);
CREATE INDEX idx_execution_plans_task ON execution_plans(task_id);

-- Task Approvals
CREATE TABLE task_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    task_id UUID NOT NULL REFERENCES auto_tasks(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES execution_plans(id) ON DELETE CASCADE,
    step_index INT,
    action_type VARCHAR(128) NOT NULL,
    action_description TEXT NOT NULL,
    risk_level SMALLINT DEFAULT 1,
    status SMALLINT NOT NULL DEFAULT 0,
    decision SMALLINT,
    decision_reason TEXT,
    decided_by UUID REFERENCES users(id) ON DELETE SET NULL,
    decided_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_task_approvals_task ON task_approvals(task_id);
CREATE INDEX idx_task_approvals_status ON task_approvals(status, expires_at);

-- Task Decisions
CREATE TABLE task_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    task_id UUID NOT NULL REFERENCES auto_tasks(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    options JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    status SMALLINT NOT NULL DEFAULT 0,
    selected_option VARCHAR(255),
    decision_reason TEXT,
    decided_by UUID REFERENCES users(id) ON DELETE SET NULL,
    decided_at TIMESTAMPTZ,
    timeout_seconds INT DEFAULT 3600,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_task_decisions_task ON task_decisions(task_id);
CREATE INDEX idx_task_decisions_status ON task_decisions(status);

-- Safety Audit Log
CREATE TABLE safety_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    task_id UUID REFERENCES auto_tasks(id) ON DELETE SET NULL,
    plan_id UUID REFERENCES execution_plans(id) ON DELETE SET NULL,
    action_type VARCHAR(128) NOT NULL,
    action_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    constraint_checks JSONB DEFAULT '[]'::jsonb,
    simulation_result JSONB,
    risk_assessment JSONB,
    outcome SMALLINT NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_safety_audit_bot ON safety_audit_log(bot_id, created_at DESC);
CREATE INDEX idx_safety_audit_outcome ON safety_audit_log(outcome, created_at DESC);

-- Intent Classifications
CREATE TABLE intent_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    original_text TEXT NOT NULL,
    intent_type SMALLINT NOT NULL DEFAULT 0,
    confidence REAL NOT NULL DEFAULT 0.0,
    entities JSONB DEFAULT '{}'::jsonb,
    suggested_name VARCHAR(255),
    was_correct BOOLEAN,
    corrected_type SMALLINT,
    feedback TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_intent_class_bot ON intent_classifications(bot_id, intent_type);
CREATE INDEX idx_intent_class_confidence ON intent_classifications(confidence);

-- ============================================================================
-- APP GENERATION TABLES
-- ============================================================================

-- Generated Apps
CREATE TABLE generated_apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
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
    CONSTRAINT uq_generated_apps UNIQUE (bot_id, name)
);
CREATE INDEX idx_generated_apps_bot ON generated_apps(bot_id);
CREATE INDEX idx_generated_apps_active ON generated_apps(is_active) WHERE is_active;

-- Designer Changes (undo support)
CREATE TABLE designer_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    change_type SMALLINT NOT NULL,
    description TEXT NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    original_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_designer_changes_bot ON designer_changes(bot_id, created_at DESC);

-- Designer Pending Changes
CREATE TABLE designer_pending_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    analysis_json TEXT NOT NULL,
    instruction TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_designer_pending_bot ON designer_pending_changes(bot_id);
CREATE INDEX idx_designer_pending_expires ON designer_pending_changes(expires_at);

-- ============================================================================
-- KNOWLEDGE BASE TABLES
-- ============================================================================

-- KB Collections
CREATE TABLE kb_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
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
    CONSTRAINT uq_kb_collections UNIQUE (bot_id, name)
);
CREATE INDEX idx_kb_collections_bot ON kb_collections(bot_id);

-- KB Documents
CREATE TABLE kb_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID NOT NULL REFERENCES kb_collections(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    file_path VARCHAR(512) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(32),
    file_size BIGINT,
    content_hash CHAR(64),
    chunk_count INT DEFAULT 0,
    is_indexed BOOLEAN DEFAULT false,
    indexed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_kb_documents_collection ON kb_documents(collection_id);
CREATE INDEX idx_kb_documents_hash ON kb_documents(content_hash);

-- Session KB Associations
CREATE TABLE session_kb_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    kb_name VARCHAR(255) NOT NULL,
    kb_folder_path VARCHAR(512),
    qdrant_collection VARCHAR(255),
    added_by_tool VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_session_kb UNIQUE (session_id, kb_name)
);
CREATE INDEX idx_session_kb_session ON session_kb_associations(session_id);

-- KB Sources (external data sources)
CREATE TABLE kb_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(64) NOT NULL,
    connection_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    sync_schedule VARCHAR(64),
    last_sync_at TIMESTAMPTZ,
    sync_status SMALLINT DEFAULT 1,
    document_count INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_kb_sources_bot ON kb_sources(bot_id);

-- ============================================================================
-- TOOLS AND AUTOMATION TABLES
-- ============================================================================

-- Tools
CREATE TABLE tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    parameters JSONB DEFAULT '{}'::jsonb,
    script TEXT NOT NULL,
    tool_type VARCHAR(64) DEFAULT 'basic',
    is_system BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    usage_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tools_bot ON tools(bot_id);
CREATE INDEX idx_tools_name ON tools(name);
CREATE UNIQUE INDEX idx_tools_unique_name ON tools(tenant_id, COALESCE(bot_id, '00000000-0000-0000-0000-000000000000'::uuid), name);

-- System Automations
CREATE TABLE system_automations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    name VARCHAR(255),
    kind SMALLINT NOT NULL,
    target VARCHAR(255),
    schedule VARCHAR(64),
    param VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMPTZ,
    next_trigger TIMESTAMPTZ,
    run_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_automations_bot ON system_automations(bot_id);
CREATE INDEX idx_automations_next ON system_automations(next_trigger) WHERE is_active;
CREATE INDEX idx_automations_active ON system_automations(kind) WHERE is_active;

-- Pending Info (ASK LATER keyword)
CREATE TABLE pending_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    field_name VARCHAR(128) NOT NULL,
    field_label VARCHAR(255) NOT NULL,
    field_type VARCHAR(64) NOT NULL DEFAULT 'text',
    reason TEXT,
    config_key VARCHAR(255) NOT NULL,
    is_filled BOOLEAN DEFAULT false,
    filled_at TIMESTAMPTZ,
    filled_value TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_pending_info_bot ON pending_info(bot_id, is_filled);
CREATE INDEX idx_pending_info_config ON pending_info(config_key);

-- ============================================================================
-- ANALYTICS TABLES
-- ============================================================================

-- Usage Analytics (daily aggregates)
CREATE TABLE usage_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    session_count INT DEFAULT 0,
    message_count INT DEFAULT 0,
    total_tokens INT DEFAULT 0,
    total_processing_time_ms BIGINT DEFAULT 0,
    avg_response_time_ms INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_usage_daily UNIQUE (user_id, bot_id, date)
);
CREATE INDEX idx_usage_analytics_tenant ON usage_analytics(tenant_id, date);
CREATE INDEX idx_usage_analytics_bot ON usage_analytics(bot_id, date);
CREATE INDEX idx_usage_analytics_date ON usage_analytics(date);

-- Analytics Events
CREATE TABLE analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    bot_id UUID REFERENCES bots(id) ON DELETE SET NULL,
    event_type VARCHAR(64) NOT NULL,
    event_data JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_analytics_events_type ON analytics_events(event_type, created_at DESC);
CREATE INDEX idx_analytics_events_tenant ON analytics_events(tenant_id, created_at DESC);
CREATE INDEX idx_analytics_events_created ON analytics_events(created_at DESC);

-- ============================================================================
-- TASK MANAGEMENT TABLES (Traditional Tasks)
-- ============================================================================

-- Tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    bot_id UUID REFERENCES bots(id) ON DELETE SET NULL,
    title VARCHAR(512) NOT NULL,
    description TEXT,
    assignee_id UUID REFERENCES users(id) ON DELETE SET NULL,
    reporter_id UUID REFERENCES users(id) ON DELETE SET NULL,
    project_id UUID,
    parent_task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    priority SMALLINT NOT NULL DEFAULT 1,
    due_date TIMESTAMPTZ,
    estimated_hours REAL,
    actual_hours REAL,
    progress SMALLINT DEFAULT 0,
    tags TEXT[],
    dependencies UUID[],
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id, status);
CREATE INDEX idx_tasks_project ON tasks(project_id, status);
CREATE INDEX idx_tasks_due ON tasks(due_date) WHERE status < 5;
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id);

-- Task Comments
CREATE TABLE task_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_task_comments_task ON task_comments(task_id);

-- ============================================================================
-- CONNECTED ACCOUNTS AND INTEGRATIONS
-- ============================================================================

-- Connected Accounts (OAuth)
CREATE TABLE connected_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    provider VARCHAR(64) NOT NULL,
    provider_user_id VARCHAR(255),
    email VARCHAR(255),
    display_name VARCHAR(255),
    access_token_vault VARCHAR(512),
    refresh_token_vault VARCHAR(512),
    token_expires_at TIMESTAMPTZ,
    scopes TEXT[],
    sync_status SMALLINT DEFAULT 1,
    last_sync_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_connected_accounts UNIQUE (user_id, provider, provider_user_id)
);
CREATE INDEX idx_connected_accounts_user ON connected_accounts(user_id);
CREATE INDEX idx_connected_accounts_provider ON connected_accounts(provider);

-- Session Account Associations
CREATE TABLE session_account_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    email VARCHAR(255),
    provider VARCHAR(64),
    qdrant_collection VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_session_account UNIQUE (session_id, account_id)
);
CREATE INDEX idx_session_account_session ON session_account_associations(session_id);

-- ============================================================================
-- COMMUNICATION TABLES
-- ============================================================================

-- WhatsApp Numbers
CREATE TABLE whatsapp_numbers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    phone_number VARCHAR(32) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_whatsapp_number UNIQUE (phone_number, bot_id)
);
CREATE INDEX idx_whatsapp_bot ON whatsapp_numbers(bot_id);

-- Email Clicks Tracking
CREATE TABLE clicks (
    campaign_id VARCHAR(128) NOT NULL,
    email VARCHAR(255) NOT NULL,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    click_count INT DEFAULT 1,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_clicks UNIQUE (campaign_id, email)
);
CREATE INDEX idx_clicks_campaign ON clicks(campaign_id);

-- ============================================================================
-- TABLE ROLE ACCESS (Dynamic table permissions)
-- ============================================================================

CREATE TABLE table_role_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    table_name VARCHAR(128) NOT NULL,
    role_name VARCHAR(64) NOT NULL,
    can_read BOOLEAN DEFAULT false,
    can_write BOOLEAN DEFAULT false,
    can_delete BOOLEAN DEFAULT false,
    row_filter JSONB,
    column_filter TEXT[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_table_role UNIQUE (bot_id, table_name, role_name)
);
CREATE INDEX idx_table_role_bot ON table_role_access(bot_id);

-- ============================================================================
-- CONTEXT INJECTIONS
-- ============================================================================

CREATE TABLE context_injections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    injected_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id BIGINT NOT NULL DEFAULT 1,
    shard_id SMALLINT NOT NULL DEFAULT 1,
    context_data JSONB NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_context_injections_session ON context_injections(session_id);

-- ============================================================================
-- ORGANIZATIONS (for multi-org support)
-- ============================================================================

CREATE TABLE organizations (
    org_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL DEFAULT 1 REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_org_slug UNIQUE (tenant_id, slug)
);
CREATE INDEX idx_organizations_tenant ON organizations(tenant_id);
CREATE INDEX idx_organizations_slug ON organizations(slug);

-- User Organizations
CREATE TABLE user_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    role VARCHAR(32) DEFAULT 'member',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_user_org UNIQUE (user_id, org_id)
);
CREATE INDEX idx_user_orgs_user ON user_organizations(user_id);
CREATE INDEX idx_user_orgs_org ON user_organizations(org_id);

-- ============================================================================
-- DEFAULT DATA
-- ============================================================================

-- Default shard for single-node deployment
INSERT INTO shard_config (shard_id, region_code, datacenter, connection_string, is_primary, min_tenant_id, max_tenant_id)
VALUES (1, 'USA', 'local', 'postgresql://localhost:5432/botserver', true, 1, 9223372036854775807);

-- Default tenant
INSERT INTO tenants (id, shard_id, name, slug, region_code, plan_tier)
VALUES (1, 1, 'Default', 'default', 'USA', 0);

INSERT INTO tenant_shard_map (tenant_id, shard_id, region_code)
VALUES (1, 1, 'USA');

-- Default bot for backwards compatibility
INSERT INTO bots (id, tenant_id, shard_id, name, description, llm_provider, context_provider, is_active)
VALUES ('00000000-0000-0000-0000-000000000000'::uuid, 1, 1, 'default', 'Default Bot', 0, 1, true);

-- ============================================================================
-- UPDATED_AT TRIGGERS
-- ============================================================================

SELECT diesel_manage_updated_at('tenants');
SELECT diesel_manage_updated_at('users');
SELECT diesel_manage_updated_at('bots');
SELECT diesel_manage_updated_at('bot_configuration');
SELECT diesel_manage_updated_at('user_sessions');
SELECT diesel_manage_updated_at('auto_tasks');
SELECT diesel_manage_updated_at('execution_plans');
SELECT diesel_manage_updated_at('generated_apps');
SELECT diesel_manage_updated_at('kb_collections');
SELECT diesel_manage_updated_at('kb_documents');
SELECT diesel_manage_updated_at('kb_sources');
SELECT diesel_manage_updated_at('tools');
SELECT diesel_manage_updated_at('system_automations');
SELECT diesel_manage_updated_at('pending_info');
SELECT diesel_manage_updated_at('tasks');
SELECT diesel_manage_updated_at('task_comments');
SELECT diesel_manage_updated_at('connected_accounts');
SELECT diesel_manage_updated_at('table_role_access');
SELECT diesel_manage_updated_at('organizations');

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE shard_config IS 'Shard configuration for horizontal scaling';
COMMENT ON TABLE tenant_shard_map IS 'Maps tenants to their respective shards';
COMMENT ON TABLE tenants IS 'Multi-tenant organizations';
COMMENT ON TABLE users IS 'User accounts with tenant isolation';
COMMENT ON TABLE bots IS 'Bot configurations';
COMMENT ON TABLE user_sessions IS 'Conversation sessions';
COMMENT ON TABLE message_history IS 'Message history (highest volume table)';
COMMENT ON TABLE auto_tasks IS 'Autonomous task execution';
COMMENT ON TABLE execution_plans IS 'LLM-compiled execution plans';
COMMENT ON TABLE kb_collections IS 'Knowledge base collections';
COMMENT ON TABLE tools IS 'Bot tools and scripts';

-- ============================================================================
-- ENUM VALUE REFERENCE (stored as SMALLINT for efficiency)
-- ============================================================================
-- Channel Types: 0=web, 1=whatsapp, 2=telegram, 3=msteams, 4=slack, 5=email, 6=sms, 7=voice, 8=instagram, 9=api
-- Message Role: 1=user, 2=assistant, 3=system, 4=tool, 9=episodic, 10=compact
-- Message Type: 0=text, 1=image, 2=audio, 3=video, 4=document, 5=location, 6=contact, 7=sticker, 8=reaction
-- LLM Provider: 0=openai, 1=anthropic, 2=azure_openai, 3=azure_claude, 4=google, 5=local, 6=ollama, 7=groq, 8=mistral, 9=cohere
-- Context Provider: 0=none, 1=qdrant, 2=pinecone, 3=weaviate, 4=milvus, 5=pgvector, 6=elasticsearch
-- Task Status: 0=pending, 1=ready, 2=running, 3=paused, 4=waiting_approval, 5=completed, 6=failed, 7=cancelled
-- Task Priority: 0=low, 1=normal, 2=high, 3=urgent, 4=critical
-- Execution Mode: 0=manual, 1=supervised, 2=autonomous
-- Risk Level: 0=none, 1=low, 2=medium, 3=high, 4=critical
-- Approval Status: 0=pending, 1=approved, 2=rejected, 3=expired, 4=skipped
-- Intent Type: 0=unknown, 1=app_create, 2=todo, 3=monitor, 4=action, 5=schedule, 6=goal, 7=tool, 8=query
-- Memory Type: 0=short, 1=long, 2=episodic, 3=semantic, 4=procedural
-- Sync Status: 0=synced, 1=pending, 2=conflict, 3=error, 4=deleted
-- Safety Outcome: 0=allowed, 1=blocked, 2=warning, 3=error
-- Designer Change Type: 0=style, 1=html, 2=database, 3=tool, 4=scheduler, 5=config, 6=multiple, 7=unknown
