-- Migration 6.0.4: Configuration Management System
-- Eliminates .env dependency by storing all configuration in database

-- ============================================================================
-- SERVER CONFIGURATION TABLE
-- Stores server-wide configuration (replaces .env variables)
-- ============================================================================
CREATE TABLE IF NOT EXISTS server_configuration (
    id TEXT PRIMARY KEY,
    config_key TEXT NOT NULL UNIQUE,
    config_value TEXT NOT NULL,
    config_type TEXT NOT NULL DEFAULT 'string', -- string, integer, boolean, encrypted
    description TEXT,
    is_encrypted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_server_config_key ON server_configuration(config_key);
CREATE INDEX IF NOT EXISTS idx_server_config_type ON server_configuration(config_type);

-- ============================================================================
-- TENANT CONFIGURATION TABLE
-- Stores tenant-level configuration (multi-tenancy support)
-- ============================================================================
CREATE TABLE IF NOT EXISTS tenant_configuration (
    id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    config_type TEXT NOT NULL DEFAULT 'string',
    is_encrypted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, config_key)
);

CREATE INDEX IF NOT EXISTS idx_tenant_config_tenant ON tenant_configuration(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_config_key ON tenant_configuration(config_key);

-- ============================================================================
-- BOT CONFIGURATION TABLE
-- Stores bot-specific configuration (replaces bot config JSON)
-- ============================================================================
CREATE TABLE IF NOT EXISTS bot_configuration (
    id TEXT PRIMARY KEY,
    bot_id UUID NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    config_type TEXT NOT NULL DEFAULT 'string',
    is_encrypted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, config_key)
);

CREATE INDEX IF NOT EXISTS idx_bot_config_bot ON bot_configuration(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_config_key ON bot_configuration(config_key);

-- ============================================================================
-- MODEL CONFIGURATIONS TABLE
-- Stores LLM and Embedding model configurations
-- ============================================================================
CREATE TABLE IF NOT EXISTS model_configurations (
    id TEXT PRIMARY KEY,
    model_name TEXT NOT NULL UNIQUE, -- Friendly name: "deepseek-1.5b", "gpt-oss-20b"
    model_type TEXT NOT NULL, -- 'llm' or 'embed'
    provider TEXT NOT NULL, -- 'openai', 'groq', 'local', 'ollama', etc.
    endpoint TEXT NOT NULL,
    api_key TEXT, -- Encrypted
    model_id TEXT NOT NULL, -- Actual model identifier
    context_window INTEGER,
    max_tokens INTEGER,
    temperature REAL DEFAULT 0.7,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_model_config_type ON model_configurations(model_type);
CREATE INDEX IF NOT EXISTS idx_model_config_active ON model_configurations(is_active);
CREATE INDEX IF NOT EXISTS idx_model_config_default ON model_configurations(is_default);

-- ============================================================================
-- CONNECTION CONFIGURATIONS TABLE
-- Stores custom database connections (replaces CUSTOM_* env vars)
-- ============================================================================
CREATE TABLE IF NOT EXISTS connection_configurations (
    id TEXT PRIMARY KEY,
    bot_id UUID NOT NULL,
    connection_name TEXT NOT NULL, -- Used in BASIC: FIND "conn1.table"
    connection_type TEXT NOT NULL, -- 'postgres', 'mysql', 'mssql', 'mongodb', etc.
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    database_name TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL, -- Encrypted
    ssl_enabled BOOLEAN NOT NULL DEFAULT false,
    additional_params JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, connection_name)
);

CREATE INDEX IF NOT EXISTS idx_connection_config_bot ON connection_configurations(bot_id);
CREATE INDEX IF NOT EXISTS idx_connection_config_name ON connection_configurations(connection_name);
CREATE INDEX IF NOT EXISTS idx_connection_config_active ON connection_configurations(is_active);

-- ============================================================================
-- COMPONENT INSTALLATIONS TABLE
-- Tracks installed components (postgres, minio, qdrant, etc.)
-- ============================================================================
CREATE TABLE IF NOT EXISTS component_installations (
    id TEXT PRIMARY KEY,
    component_name TEXT NOT NULL UNIQUE, -- 'tables', 'drive', 'vectordb', 'cache', 'llm'
    component_type TEXT NOT NULL, -- 'database', 'storage', 'vector', 'cache', 'compute'
    version TEXT NOT NULL,
    install_path TEXT NOT NULL, -- Relative to botserver-stack
    binary_path TEXT, -- Path to executable
    data_path TEXT, -- Path to data directory
    config_path TEXT, -- Path to config file
    log_path TEXT, -- Path to log directory
    status TEXT NOT NULL DEFAULT 'stopped', -- 'running', 'stopped', 'error', 'installing'
    port INTEGER,
    pid INTEGER,
    auto_start BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_started_at TIMESTAMPTZ,
    last_stopped_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_component_name ON component_installations(component_name);
CREATE INDEX IF NOT EXISTS idx_component_status ON component_installations(status);

-- ============================================================================
-- TENANTS TABLE
-- Multi-tenancy support
-- ============================================================================
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_active ON tenants(is_active);

-- ============================================================================
-- BOT SESSIONS ENHANCEMENT
-- Add tenant_id to existing sessions if column doesn't exist
-- ============================================================================
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_sessions' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE user_sessions ADD COLUMN tenant_id UUID;
        CREATE INDEX idx_user_sessions_tenant ON user_sessions(tenant_id);
    END IF;
END $$;

-- ============================================================================
-- BOTS TABLE ENHANCEMENT
-- Add tenant_id if it doesn't exist
-- ============================================================================
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'bots' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE bots ADD COLUMN tenant_id UUID;
        CREATE INDEX idx_bots_tenant ON bots(tenant_id);
    END IF;
END $$;

INSERT INTO tenants (id, name, slug, is_active) VALUES
    (gen_random_uuid(), 'Default Tenant', 'default', true)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- DEFAULT MODELS
-- Add some default model configurations
-- ============================================================================
INSERT INTO model_configurations (id, model_name, model_type, provider, endpoint, model_id, context_window, max_tokens, is_default) VALUES
    (gen_random_uuid()::text, 'gpt-4', 'llm', 'openai', 'https://api.openai.com/v1', 'gpt-4', 8192, 4096, true),
    (gen_random_uuid()::text, 'gpt-3.5-turbo', 'llm', 'openai', 'https://api.openai.com/v1', 'gpt-3.5-turbo', 4096, 2048, false),
    (gen_random_uuid()::text, 'bge-large', 'embed', 'local', 'http://localhost:8081', 'BAAI/bge-large-en-v1.5', 512, 1024, true)
ON CONFLICT (model_name) DO NOTHING;

-- ============================================================================
-- COMPONENT LOGGING TABLE
-- Track component lifecycle events
-- ============================================================================
CREATE TABLE IF NOT EXISTS component_logs (
    id TEXT PRIMARY KEY,
    component_name TEXT NOT NULL,
    log_level TEXT NOT NULL, -- 'info', 'warning', 'error', 'debug'
    message TEXT NOT NULL,
    details JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_component_logs_component ON component_logs(component_name);
CREATE INDEX IF NOT EXISTS idx_component_logs_level ON component_logs(log_level);
CREATE INDEX IF NOT EXISTS idx_component_logs_created ON component_logs(created_at);

-- ============================================================================
-- GBOT CONFIG SYNC TABLE
-- Tracks .gbot/config.csv file changes and last sync
-- ============================================================================
CREATE TABLE IF NOT EXISTS gbot_config_sync (
    id TEXT PRIMARY KEY,
    bot_id UUID NOT NULL UNIQUE,
    config_file_path TEXT NOT NULL,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    file_hash TEXT NOT NULL,
    sync_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_gbot_sync_bot ON gbot_config_sync(bot_id);

