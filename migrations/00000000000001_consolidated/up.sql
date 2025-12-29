
CREATE TABLE public.bots (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	"name" varchar(255) NOT NULL,
	description text NULL,
	llm_provider varchar(100) NOT NULL,
	llm_config jsonb DEFAULT '{}'::jsonb NOT NULL,
	context_provider varchar(100) NOT NULL,
	context_config jsonb DEFAULT '{}'::jsonb NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	is_active bool DEFAULT true NULL,
	CONSTRAINT bots_pkey PRIMARY KEY (id)
);


-- public.clicks definition

-- Drop table

-- DROP TABLE public.clicks;

CREATE TABLE public.clicks (
	campaign_id text NOT NULL,
	email text NOT NULL,
	updated_at timestamptz DEFAULT now() NULL,
	CONSTRAINT clicks_campaign_id_email_key UNIQUE (campaign_id, email)
);


-- public.organizations definition

-- Drop table

-- DROP TABLE public.organizations;

CREATE TABLE public.organizations (
	org_id uuid DEFAULT gen_random_uuid() NOT NULL,
	"name" varchar(255) NOT NULL,
	slug varchar(255) NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT organizations_pkey PRIMARY KEY (org_id),
	CONSTRAINT organizations_slug_key UNIQUE (slug)
);
CREATE INDEX idx_organizations_created_at ON public.organizations USING btree (created_at);
CREATE INDEX idx_organizations_slug ON public.organizations USING btree (slug);


-- public.system_automations definition

-- Drop table

-- DROP TABLE public.system_automations;

CREATE TABLE public.system_automations (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	bot_id uuid NOT NULL,
	kind int4 NOT NULL,
	"target" varchar(32) NULL,
	schedule bpchar(20) NULL,
	param varchar(32) NOT NULL,
	is_active bool DEFAULT true NOT NULL,
	last_triggered timestamptz NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT system_automations_pkey PRIMARY KEY (id)
);
CREATE INDEX idx_system_automations_active ON public.system_automations USING btree (kind) WHERE is_active;


-- public.tools definition

-- Drop table

-- DROP TABLE public.tools;

CREATE TABLE public.tools (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	"name" varchar(255) NOT NULL,
	description text NOT NULL,
	parameters jsonb DEFAULT '{}'::jsonb NOT NULL,
	script text NOT NULL,
	is_active bool DEFAULT true NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT tools_name_key UNIQUE (name),
	CONSTRAINT tools_pkey PRIMARY KEY (id)
);


-- public.users definition

-- Drop table

-- DROP TABLE public.users;

CREATE TABLE public.users (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	username varchar(255) NOT NULL,
	email varchar(255) NOT NULL,
	password_hash varchar(255) NOT NULL,
	phone_number varchar(50) NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	is_active bool DEFAULT true NULL,
	CONSTRAINT users_email_key UNIQUE (email),
	CONSTRAINT users_pkey PRIMARY KEY (id),
	CONSTRAINT users_username_key UNIQUE (username)
);


-- public.bot_channels definition

-- Drop table

-- DROP TABLE public.bot_channels;

CREATE TABLE public.bot_channels (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	bot_id uuid NOT NULL,
	channel_type int4 NOT NULL,
	config jsonb DEFAULT '{}'::jsonb NOT NULL,
	is_active bool DEFAULT true NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT bot_channels_bot_id_channel_type_key UNIQUE (bot_id, channel_type),
	CONSTRAINT bot_channels_pkey PRIMARY KEY (id),
	CONSTRAINT bot_channels_bot_id_fkey FOREIGN KEY (bot_id) REFERENCES public.bots(id) ON DELETE CASCADE
);
CREATE INDEX idx_bot_channels_type ON public.bot_channels USING btree (channel_type) WHERE is_active;


-- public.user_sessions definition

-- Drop table

-- DROP TABLE public.user_sessions;

CREATE TABLE public.user_sessions (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	user_id uuid NOT NULL,
	bot_id uuid NOT NULL,
	title varchar(500) DEFAULT 'New Conversation'::character varying NOT NULL,
	answer_mode int4 DEFAULT 0 NOT NULL,
	context_data jsonb DEFAULT '{}'::jsonb NOT NULL,
	current_tool varchar(255) NULL,
	message_count int4 DEFAULT 0 NOT NULL,
	total_tokens int4 DEFAULT 0 NOT NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	updated_at timestamptz DEFAULT now() NOT NULL,
	last_activity timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT user_sessions_pkey PRIMARY KEY (id),
	CONSTRAINT user_sessions_bot_id_fkey FOREIGN KEY (bot_id) REFERENCES public.bots(id) ON DELETE CASCADE,
	CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);
CREATE INDEX idx_user_sessions_updated_at ON public.user_sessions USING btree (updated_at);
CREATE INDEX idx_user_sessions_user_bot ON public.user_sessions USING btree (user_id, bot_id);


-- public.whatsapp_numbers definition

-- Drop table

-- DROP TABLE public.whatsapp_numbers;

CREATE TABLE public.whatsapp_numbers (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	bot_id uuid NOT NULL,
	phone_number varchar(50) NOT NULL,
	is_active bool DEFAULT true NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT whatsapp_numbers_phone_number_bot_id_key UNIQUE (phone_number, bot_id),
	CONSTRAINT whatsapp_numbers_pkey PRIMARY KEY (id),
	CONSTRAINT whatsapp_numbers_bot_id_fkey FOREIGN KEY (bot_id) REFERENCES public.bots(id) ON DELETE CASCADE
);


-- public.context_injections definition

-- Drop table

-- DROP TABLE public.context_injections;

CREATE TABLE public.context_injections (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	session_id uuid NOT NULL,
	injected_by uuid NOT NULL,
	context_data jsonb NOT NULL,
	reason text NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	CONSTRAINT context_injections_pkey PRIMARY KEY (id),
	CONSTRAINT context_injections_injected_by_fkey FOREIGN KEY (injected_by) REFERENCES public.users(id) ON DELETE CASCADE,
	CONSTRAINT context_injections_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(id) ON DELETE CASCADE
);


-- public.message_history definition

-- Drop table

-- DROP TABLE public.message_history;

CREATE TABLE public.message_history (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	session_id uuid NOT NULL,
	user_id uuid NOT NULL,
	"role" int4 NOT NULL,
	content_encrypted text NOT NULL,
	message_type int4 DEFAULT 0 NOT NULL,
	media_url text NULL,
	token_count int4 DEFAULT 0 NOT NULL,
	processing_time_ms int4 NULL,
	llm_model varchar(100) NULL,
	created_at timestamptz DEFAULT now() NOT NULL,
	message_index int4 NOT NULL,
	CONSTRAINT message_history_pkey PRIMARY KEY (id),
	CONSTRAINT message_history_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(id) ON DELETE CASCADE,
	CONSTRAINT message_history_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);
CREATE INDEX idx_message_history_created_at ON public.message_history USING btree (created_at);
CREATE INDEX idx_message_history_session_id ON public.message_history USING btree (session_id);


-- public.usage_analytics definition

-- Drop table

-- DROP TABLE public.usage_analytics;

CREATE TABLE public.usage_analytics (
	id uuid DEFAULT gen_random_uuid() NOT NULL,
	user_id uuid NOT NULL,
	bot_id uuid NOT NULL,
	session_id uuid NOT NULL,
	"date" date DEFAULT CURRENT_DATE NOT NULL,
	message_count int4 DEFAULT 0 NOT NULL,
	total_tokens int4 DEFAULT 0 NOT NULL,
	total_processing_time_ms int4 DEFAULT 0 NOT NULL,
	CONSTRAINT usage_analytics_pkey PRIMARY KEY (id),
	CONSTRAINT usage_analytics_bot_id_fkey FOREIGN KEY (bot_id) REFERENCES public.bots(id) ON DELETE CASCADE,
	CONSTRAINT usage_analytics_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(id) ON DELETE CASCADE,
	CONSTRAINT usage_analytics_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE
);
CREATE INDEX idx_usage_analytics_date ON public.usage_analytics USING btree (date);

CREATE TABLE bot_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, key)
);

CREATE INDEX idx_bot_memories_bot_id ON bot_memories(bot_id);
CREATE INDEX idx_bot_memories_key ON bot_memories(key);
-- Migration: Create KB and Tools tables
-- Description: Tables for Knowledge Base management and BASIC tools compilation

-- Table for KB documents metadata
CREATE TABLE IF NOT EXISTS kb_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    collection_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL DEFAULT 0,
    file_hash TEXT NOT NULL,
    first_published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, collection_name, file_path)
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_kb_documents_bot_id ON kb_documents(bot_id);
CREATE INDEX IF NOT EXISTS idx_kb_documents_collection ON kb_documents(collection_name);
CREATE INDEX IF NOT EXISTS idx_kb_documents_hash ON kb_documents(file_hash);
CREATE INDEX IF NOT EXISTS idx_kb_documents_indexed_at ON kb_documents(indexed_at);

-- Table for KB collections
CREATE TABLE IF NOT EXISTS kb_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name TEXT NOT NULL,
    folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    document_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, name)
);

-- Index for KB collections
CREATE INDEX IF NOT EXISTS idx_kb_collections_bot_id ON kb_collections(bot_id);
CREATE INDEX IF NOT EXISTS idx_kb_collections_name ON kb_collections(name);

-- Table for compiled BASIC tools
CREATE TABLE IF NOT EXISTS basic_tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    tool_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    ast_path TEXT NOT NULL,
    mcp_json JSONB,
    tool_json JSONB,
    compiled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, tool_name)
);

-- Index for BASIC tools
CREATE INDEX IF NOT EXISTS idx_basic_tools_bot_id ON basic_tools(bot_id);
CREATE INDEX IF NOT EXISTS idx_basic_tools_name ON basic_tools(tool_name);
CREATE INDEX IF NOT EXISTS idx_basic_tools_active ON basic_tools(is_active);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updating updated_at
DROP TRIGGER IF EXISTS update_kb_documents_updated_at ON kb_documents;
CREATE TRIGGER update_kb_documents_updated_at
    BEFORE UPDATE ON kb_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_kb_collections_updated_at ON kb_collections;
CREATE TRIGGER update_kb_collections_updated_at
    BEFORE UPDATE ON kb_collections
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_basic_tools_updated_at ON basic_tools;
CREATE TRIGGER update_basic_tools_updated_at
    BEFORE UPDATE ON basic_tools
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE kb_documents IS 'Stores metadata about documents in Knowledge Base collections';
COMMENT ON TABLE kb_collections IS 'Stores information about KB collections and their Qdrant mappings';
COMMENT ON TABLE basic_tools IS 'Stores compiled BASIC tools with their MCP and OpenAI tool definitions';

COMMENT ON COLUMN kb_documents.file_hash IS 'SHA256 hash of file content for change detection';
COMMENT ON COLUMN kb_documents.indexed_at IS 'Timestamp when document was last indexed in Qdrant';
COMMENT ON COLUMN kb_collections.qdrant_collection IS 'Name of corresponding Qdrant collection';
COMMENT ON COLUMN basic_tools.mcp_json IS 'Model Context Protocol tool definition';
COMMENT ON COLUMN basic_tools.tool_json IS 'OpenAI-compatible tool definition';
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
    (gen_random_uuid()::text, 'gpt-4', 'llm', 'openai', 'http://localhost:8081/v1', 'gpt-4', 8192, 4096, true),
    (gen_random_uuid()::text, 'gpt-3.5-turbo', 'llm', 'openai', 'http://localhost:8081/v1', 'gpt-3.5-turbo', 4096, 2048, false),
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

-- Migration 6.0.5: Add update-summary.bas scheduled automation
-- Description: Creates a scheduled automation that runs every minute to update summaries
-- This replaces the announcements system in legacy mode
-- Note: Bots are now created dynamically during bootstrap based on template folders

-- Add name column to system_automations if it doesn't exist
ALTER TABLE public.system_automations ADD COLUMN IF NOT EXISTS name VARCHAR(255);

-- Create index on name column for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_name ON public.system_automations(name);

-- Note: bot_configuration already has UNIQUE(bot_id, config_key) from migration 6.0.4
-- Do NOT add a global unique constraint on config_key alone as that breaks multi-bot configs

-- Migration 6.0.9: Add bot_id column to system_automations
-- Description: Introduces a bot_id column to associate automations with a specific bot.
-- The column is added as UUID and indexed for efficient queries.

-- Add bot_id column if it does not exist
ALTER TABLE public.system_automations
ADD COLUMN IF NOT EXISTS bot_id UUID NOT NULL;

-- Create an index on bot_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_id
ON public.system_automations (bot_id);


ALTER TABLE public.system_automations
ADD CONSTRAINT system_automations_bot_kind_param_unique
UNIQUE (bot_id, kind, param);

-- Add index for the new constraint
CREATE INDEX IF NOT EXISTS idx_system_automations_bot_kind_param
ON public.system_automations (bot_id, kind, param);


-- Migration 6.0.7: Fix clicks table primary key
-- Required by Diesel before we can run other migrations

-- Create new table with proper structure
CREATE TABLE IF NOT EXISTS public.new_clicks (
    id SERIAL PRIMARY KEY,
    campaign_id text NOT NULL,
    email text NOT NULL,
    updated_at timestamptz DEFAULT now() NULL,
    CONSTRAINT new_clicks_campaign_id_email_key UNIQUE (campaign_id, email)
);

-- Copy data from old table
INSERT INTO public.new_clicks (campaign_id, email, updated_at)
SELECT campaign_id, email, updated_at FROM public.clicks;

-- Drop old table and rename new one
DROP TABLE public.clicks;
ALTER TABLE public.new_clicks RENAME TO clicks;
-- Add user_email_accounts table for storing user email credentials
CREATE TABLE public.user_email_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    email varchar(255) NOT NULL,
    display_name varchar(255) NULL,
    imap_server varchar(255) NOT NULL,
    imap_port int4 DEFAULT 993 NOT NULL,
    smtp_server varchar(255) NOT NULL,
    smtp_port int4 DEFAULT 587 NOT NULL,
    username varchar(255) NOT NULL,
    password_encrypted text NOT NULL,
    is_primary bool DEFAULT false NOT NULL,
    is_active bool DEFAULT true NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT user_email_accounts_pkey PRIMARY KEY (id),
    CONSTRAINT user_email_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_email_accounts_user_email_key UNIQUE (user_id, email)
);

CREATE INDEX idx_user_email_accounts_user_id ON public.user_email_accounts USING btree (user_id);
CREATE INDEX idx_user_email_accounts_active ON public.user_email_accounts USING btree (is_active) WHERE is_active;

-- Add email drafts table
CREATE TABLE public.email_drafts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    account_id uuid NOT NULL,
    to_address text NOT NULL,
    cc_address text NULL,
    bcc_address text NULL,
    subject varchar(500) NULL,
    body text NULL,
    attachments jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_drafts_pkey PRIMARY KEY (id),
    CONSTRAINT email_drafts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT email_drafts_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.user_email_accounts(id) ON DELETE CASCADE
);

CREATE INDEX idx_email_drafts_user_id ON public.email_drafts USING btree (user_id);
CREATE INDEX idx_email_drafts_account_id ON public.email_drafts USING btree (account_id);

-- Add email folders metadata table (for caching and custom folders)
CREATE TABLE public.email_folders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    account_id uuid NOT NULL,
    folder_name varchar(255) NOT NULL,
    folder_path varchar(500) NOT NULL,
    unread_count int4 DEFAULT 0 NOT NULL,
    total_count int4 DEFAULT 0 NOT NULL,
    last_synced timestamptz NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_folders_pkey PRIMARY KEY (id),
    CONSTRAINT email_folders_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.user_email_accounts(id) ON DELETE CASCADE,
    CONSTRAINT email_folders_account_path_key UNIQUE (account_id, folder_path)
);

CREATE INDEX idx_email_folders_account_id ON public.email_folders USING btree (account_id);

-- Add sessions table enhancement for storing current email account
ALTER TABLE public.user_sessions
ADD COLUMN IF NOT EXISTS active_email_account_id uuid NULL,
ADD CONSTRAINT user_sessions_email_account_id_fkey
FOREIGN KEY (active_email_account_id) REFERENCES public.user_email_accounts(id) ON DELETE SET NULL;

-- Add user preferences table
CREATE TABLE public.user_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    preference_key varchar(100) NOT NULL,
    preference_value jsonb NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT user_preferences_pkey PRIMARY KEY (id),
    CONSTRAINT user_preferences_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_preferences_user_key_unique UNIQUE (user_id, preference_key)
);

CREATE INDEX idx_user_preferences_user_id ON public.user_preferences USING btree (user_id);

-- Add login tokens table for session management
CREATE TABLE public.user_login_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token_hash varchar(255) NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    last_used timestamptz DEFAULT now() NOT NULL,
    user_agent text NULL,
    ip_address varchar(50) NULL,
    is_active bool DEFAULT true NOT NULL,
    CONSTRAINT user_login_tokens_pkey PRIMARY KEY (id),
    CONSTRAINT user_login_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE,
    CONSTRAINT user_login_tokens_token_hash_key UNIQUE (token_hash)
);

CREATE INDEX idx_user_login_tokens_user_id ON public.user_login_tokens USING btree (user_id);
CREATE INDEX idx_user_login_tokens_expires ON public.user_login_tokens USING btree (expires_at) WHERE is_active;
-- Migration 6.0.7: Session KB Tracking
-- Adds table to track which KBs are active in each conversation session

-- Table for tracking KBs active in a session (set by ADD_KB in .bas tools)
CREATE TABLE IF NOT EXISTS session_kb_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    kb_name TEXT NOT NULL,
    kb_folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(session_id, kb_name)
);

CREATE INDEX IF NOT EXISTS idx_session_kb_session_id ON session_kb_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_bot_id ON session_kb_associations(bot_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_name ON session_kb_associations(kb_name);
CREATE INDEX IF NOT EXISTS idx_session_kb_active ON session_kb_associations(is_active) WHERE is_active = true;

-- Comments
COMMENT ON TABLE session_kb_associations IS 'Tracks which Knowledge Base collections are active in each conversation session';
COMMENT ON COLUMN session_kb_associations.kb_name IS 'Name of the KB folder (e.g., "circular", "comunicado", "geral")';
COMMENT ON COLUMN session_kb_associations.kb_folder_path IS 'Full path to KB folder: work/{bot}/{bot}.gbkb/{kb_name}';
COMMENT ON COLUMN session_kb_associations.qdrant_collection IS 'Qdrant collection name for this KB';
COMMENT ON COLUMN session_kb_associations.added_by_tool IS 'Name of the .bas tool that added this KB (e.g., "change-subject.bas")';
COMMENT ON COLUMN session_kb_associations.is_active IS 'Whether this KB is currently active in the session';
-- Add organization relationship to bots
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS org_id UUID,
ADD COLUMN IF NOT EXISTS is_default BOOLEAN DEFAULT false;

-- Add foreign key constraint to organizations
ALTER TABLE public.bots
ADD CONSTRAINT bots_org_id_fkey
FOREIGN KEY (org_id) REFERENCES public.organizations(org_id) ON DELETE CASCADE;

-- Create index for org_id lookups
CREATE INDEX IF NOT EXISTS idx_bots_org_id ON public.bots(org_id);

-- Create directory_users table to map directory (Zitadel) users to our system
CREATE TABLE IF NOT EXISTS public.directory_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    directory_id VARCHAR(255) NOT NULL UNIQUE, -- Zitadel user ID
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    bot_id UUID REFERENCES public.bots(id) ON DELETE SET NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    is_admin BOOLEAN DEFAULT false,
    is_bot_user BOOLEAN DEFAULT false, -- true for bot service accounts
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create indexes for directory_users
CREATE INDEX IF NOT EXISTS idx_directory_users_org_id ON public.directory_users(org_id);
CREATE INDEX IF NOT EXISTS idx_directory_users_bot_id ON public.directory_users(bot_id);
CREATE INDEX IF NOT EXISTS idx_directory_users_email ON public.directory_users(email);
CREATE INDEX IF NOT EXISTS idx_directory_users_directory_id ON public.directory_users(directory_id);

-- Create bot_access table to manage which users can access which bots
CREATE TABLE IF NOT EXISTS public.bot_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES public.directory_users(id) ON DELETE CASCADE,
    access_level VARCHAR(50) NOT NULL DEFAULT 'user', -- 'owner', 'admin', 'user', 'viewer'
    granted_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    granted_by UUID REFERENCES public.directory_users(id),
    UNIQUE(bot_id, user_id)
);

-- Create indexes for bot_access
CREATE INDEX IF NOT EXISTS idx_bot_access_bot_id ON public.bot_access(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_access_user_id ON public.bot_access(user_id);

-- Create OAuth application registry for directory integrations
CREATE TABLE IF NOT EXISTS public.oauth_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    project_id VARCHAR(255),
    client_id VARCHAR(255) NOT NULL UNIQUE,
    client_secret_encrypted TEXT NOT NULL, -- Store encrypted
    redirect_uris TEXT[] NOT NULL DEFAULT '{}',
    application_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create index for OAuth applications
CREATE INDEX IF NOT EXISTS idx_oauth_applications_org_id ON public.oauth_applications(org_id);
CREATE INDEX IF NOT EXISTS idx_oauth_applications_client_id ON public.oauth_applications(client_id);

-- Insert default organization if it doesn't exist
INSERT INTO public.organizations (org_id, name, slug, created_at, updated_at)
VALUES (
    'f47ac10b-58cc-4372-a567-0e02b2c3d479'::uuid, -- Fixed UUID for default org
    'Default Organization',
    'default',
    NOW(),
    NOW()
) ON CONFLICT (slug) DO NOTHING;

-- Insert default bot for the default organization
DO $$
DECLARE
    v_org_id UUID;
    v_bot_id UUID;
BEGIN
    -- Get the default organization ID
    SELECT org_id INTO v_org_id FROM public.organizations WHERE slug = 'default';

    -- Generate or use fixed UUID for default bot
    v_bot_id := 'f47ac10b-58cc-4372-a567-0e02b2c3d480'::uuid;

    -- Insert default bot if it doesn't exist
    INSERT INTO public.bots (
        id,
        org_id,
        name,
        description,
        llm_provider,
        llm_config,
        context_provider,
        context_config,
        is_default,
        is_active,
        created_at,
        updated_at
    )
    VALUES (
        v_bot_id,
        v_org_id,
        'Default Bot',
        'Default bot for the default organization',
        'openai',
        '{"model": "gpt-4", "temperature": 0.7}'::jsonb,
        'none',
        '{}'::jsonb,
        true,
        true,
        NOW(),
        NOW()
    ) ON CONFLICT (id) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        is_default = true,
        updated_at = NOW();

    -- Insert default admin user (admin@default)
    INSERT INTO public.directory_users (
        directory_id,
        username,
        email,
        org_id,
        bot_id,
        first_name,
        last_name,
        is_admin,
        is_bot_user,
        created_at,
        updated_at
    )
    VALUES (
        'admin-default-001', -- Will be replaced with actual Zitadel ID
        'admin',
        'admin@default',
        v_org_id,
        v_bot_id,
        'Admin',
        'Default',
        true,
        false,
        NOW(),
        NOW()
    ) ON CONFLICT (email) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        bot_id = EXCLUDED.bot_id,
        is_admin = true,
        updated_at = NOW();

    -- Insert default regular user (user@default)
    INSERT INTO public.directory_users (
        directory_id,
        username,
        email,
        org_id,
        bot_id,
        first_name,
        last_name,
        is_admin,
        is_bot_user,
        created_at,
        updated_at
    )
    VALUES (
        'user-default-001', -- Will be replaced with actual Zitadel ID
        'user',
        'user@default',
        v_org_id,
        v_bot_id,
        'User',
        'Default',
        false,
        false,
        NOW(),
        NOW()
    ) ON CONFLICT (email) DO UPDATE
    SET org_id = EXCLUDED.org_id,
        bot_id = EXCLUDED.bot_id,
        is_admin = false,
        updated_at = NOW();

    -- Grant bot access to admin user
    INSERT INTO public.bot_access (bot_id, user_id, access_level, granted_at)
    SELECT
        v_bot_id,
        id,
        'owner',
        NOW()
    FROM public.directory_users
    WHERE email = 'admin@default'
    ON CONFLICT (bot_id, user_id) DO UPDATE
    SET access_level = 'owner',
        granted_at = NOW();

    -- Grant bot access to regular user
    INSERT INTO public.bot_access (bot_id, user_id, access_level, granted_at)
    SELECT
        v_bot_id,
        id,
        'user',
        NOW()
    FROM public.directory_users
    WHERE email = 'user@default'
    ON CONFLICT (bot_id, user_id) DO UPDATE
    SET access_level = 'user',
        granted_at = NOW();

END $$;

-- Create function to update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at columns if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_directory_users_updated_at') THEN
        CREATE TRIGGER update_directory_users_updated_at
        BEFORE UPDATE ON public.directory_users
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_oauth_applications_updated_at') THEN
        CREATE TRIGGER update_oauth_applications_updated_at
        BEFORE UPDATE ON public.oauth_applications
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    END IF;
END $$;

-- Add comment documentation
COMMENT ON TABLE public.directory_users IS 'Maps directory (Zitadel) users to the system and their associated bots';
COMMENT ON TABLE public.bot_access IS 'Controls which users have access to which bots and their permission levels';
COMMENT ON TABLE public.oauth_applications IS 'OAuth application configurations for directory integration';
COMMENT ON COLUMN public.bots.is_default IS 'Indicates if this is the default bot for an organization';
COMMENT ON COLUMN public.directory_users.is_bot_user IS 'True if this user is a service account for bot operations';
COMMENT ON COLUMN public.bot_access.access_level IS 'Access level: owner (full control), admin (manage), user (use), viewer (read-only)';
-- Create website_crawls table for tracking crawled websites
CREATE TABLE IF NOT EXISTS website_crawls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    url TEXT NOT NULL,
    last_crawled TIMESTAMPTZ,
    next_crawl TIMESTAMPTZ,
    expires_policy VARCHAR(20) NOT NULL DEFAULT '1d',
    max_depth INTEGER DEFAULT 3,
    max_pages INTEGER DEFAULT 100,
    crawl_status SMALLINT DEFAULT 0, -- 0=pending, 1=success, 2=processing, 3=error
    pages_crawled INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure unique URL per bot
    CONSTRAINT unique_bot_url UNIQUE (bot_id, url),

    -- Foreign key to bots table
    CONSTRAINT fk_website_crawls_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_website_crawls_bot_id ON website_crawls(bot_id);
CREATE INDEX IF NOT EXISTS idx_website_crawls_next_crawl ON website_crawls(next_crawl);
CREATE INDEX IF NOT EXISTS idx_website_crawls_url ON website_crawls(url);
CREATE INDEX IF NOT EXISTS idx_website_crawls_status ON website_crawls(crawl_status);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_website_crawls_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER website_crawls_updated_at_trigger
    BEFORE UPDATE ON website_crawls
    FOR EACH ROW
    EXECUTE FUNCTION update_website_crawls_updated_at();

-- Create session_website_associations table for tracking websites added to sessions
-- Similar to session_kb_associations but for websites
CREATE TABLE IF NOT EXISTS session_website_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    website_url TEXT NOT NULL,
    collection_name TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    added_by_tool VARCHAR(255),

    -- Ensure unique website per session
    CONSTRAINT unique_session_website UNIQUE (session_id, website_url),

    -- Foreign key to sessions table
    CONSTRAINT fk_session_website_session
        FOREIGN KEY (session_id)
        REFERENCES user_sessions(id)
        ON DELETE CASCADE,

    -- Foreign key to bots table
    CONSTRAINT fk_session_website_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_session_website_associations_session_id
    ON session_website_associations(session_id) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_session_website_associations_bot_id
    ON session_website_associations(bot_id);

CREATE INDEX IF NOT EXISTS idx_session_website_associations_url
    ON session_website_associations(website_url);

CREATE INDEX IF NOT EXISTS idx_session_website_associations_collection
    ON session_website_associations(collection_name);
-- Migration: 6.1.0 Enterprise Features
-- Description: MUST-HAVE features to compete with Microsoft 365 and Google Workspace
-- NOTE: TABLES AND INDEXES ONLY - No views, triggers, or functions per project standards

-- ============================================================================
-- DROP EXISTING TABLES (clean state for re-run)
-- ============================================================================
DROP TABLE IF EXISTS public.user_organizations CASCADE;
DROP TABLE IF EXISTS public.email_received_events CASCADE;
DROP TABLE IF EXISTS public.folder_change_events CASCADE;
DROP TABLE IF EXISTS public.folder_monitors CASCADE;
DROP TABLE IF EXISTS public.email_monitors CASCADE;
DROP TABLE IF EXISTS account_sync_items CASCADE;
DROP TABLE IF EXISTS session_account_associations CASCADE;
DROP TABLE IF EXISTS connected_accounts CASCADE;
DROP TABLE IF EXISTS analytics_events CASCADE;
DROP TABLE IF EXISTS research_search_history CASCADE;
DROP TABLE IF EXISTS test_execution_logs CASCADE;
DROP TABLE IF EXISTS test_accounts CASCADE;
DROP TABLE IF EXISTS calendar_shares CASCADE;
DROP TABLE IF EXISTS calendar_resource_bookings CASCADE;
DROP TABLE IF EXISTS calendar_resources CASCADE;
DROP TABLE IF EXISTS task_recurrence CASCADE;
DROP TABLE IF EXISTS task_time_entries CASCADE;
DROP TABLE IF EXISTS task_dependencies CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;
DROP TABLE IF EXISTS document_presence CASCADE;
DROP TABLE IF EXISTS storage_quotas CASCADE;
DROP TABLE IF EXISTS file_sync_status CASCADE;
DROP TABLE IF EXISTS file_trash CASCADE;
DROP TABLE IF EXISTS file_activities CASCADE;
DROP TABLE IF EXISTS file_shares CASCADE;
DROP TABLE IF EXISTS file_comments CASCADE;
DROP TABLE IF EXISTS file_versions CASCADE;
DROP TABLE IF EXISTS user_virtual_backgrounds CASCADE;
DROP TABLE IF EXISTS meeting_captions CASCADE;
DROP TABLE IF EXISTS meeting_waiting_room CASCADE;
DROP TABLE IF EXISTS meeting_questions CASCADE;
DROP TABLE IF EXISTS meeting_polls CASCADE;
DROP TABLE IF EXISTS meeting_breakout_rooms CASCADE;
DROP TABLE IF EXISTS meeting_recordings CASCADE;
DROP TABLE IF EXISTS shared_mailbox_members CASCADE;
DROP TABLE IF EXISTS shared_mailboxes CASCADE;
DROP TABLE IF EXISTS distribution_lists CASCADE;
DROP TABLE IF EXISTS email_label_assignments CASCADE;
DROP TABLE IF EXISTS email_labels CASCADE;
DROP TABLE IF EXISTS email_rules CASCADE;
DROP TABLE IF EXISTS email_auto_responders CASCADE;
DROP TABLE IF EXISTS email_templates CASCADE;
DROP TABLE IF EXISTS scheduled_emails CASCADE;
DROP TABLE IF EXISTS email_signatures CASCADE;
DROP TABLE IF EXISTS global_email_signatures CASCADE;
DROP TABLE IF EXISTS external_connections CASCADE;
DROP TABLE IF EXISTS dynamic_table_fields CASCADE;
DROP TABLE IF EXISTS dynamic_table_definitions CASCADE;
DROP TABLE IF EXISTS workflow_step_executions CASCADE;
DROP TABLE IF EXISTS workflow_executions CASCADE;
DROP TABLE IF EXISTS workflow_steps CASCADE;
DROP TABLE IF EXISTS workflow_definitions CASCADE;
DROP TABLE IF EXISTS approval_history CASCADE;
DROP TABLE IF EXISTS approval_steps CASCADE;
DROP TABLE IF EXISTS approval_requests CASCADE;
DROP TABLE IF EXISTS kg_relationships CASCADE;
DROP TABLE IF EXISTS kg_entities CASCADE;
DROP TABLE IF EXISTS user_memories CASCADE;
DROP TABLE IF EXISTS episodic_memories CASCADE;
DROP TABLE IF EXISTS conversation_costs CASCADE;
DROP TABLE IF EXISTS conversation_episodes CASCADE;

-- ============================================================================
-- GLOBAL CONFIGURATION
-- ============================================================================

-- Global email signature (applied to all emails from this bot)
CREATE TABLE IF NOT EXISTS global_email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL DEFAULT 'Default',
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    position VARCHAR(20) DEFAULT 'bottom',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_bot_global_signature UNIQUE (bot_id, name),
    CONSTRAINT check_signature_position CHECK (position IN ('top', 'bottom'))
);

CREATE INDEX IF NOT EXISTS idx_global_signatures_bot ON global_email_signatures(bot_id) WHERE is_active = true;

-- ============================================================================
-- EMAIL ENTERPRISE FEATURES (Outlook/Gmail parity)
-- Note: Many features controlled via Stalwart IMAP/JMAP API
-- ============================================================================

-- User email signatures (in addition to global)
CREATE TABLE IF NOT EXISTS email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL DEFAULT 'Default',
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_signature_name UNIQUE (user_id, bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_email_signatures_user ON email_signatures(user_id);
CREATE INDEX IF NOT EXISTS idx_email_signatures_default ON email_signatures(user_id, bot_id) WHERE is_default = true;

-- Scheduled emails (send later)
CREATE TABLE IF NOT EXISTS scheduled_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    to_addresses TEXT NOT NULL,
    cc_addresses TEXT,
    bcc_addresses TEXT,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    attachments_json TEXT DEFAULT '[]',
    scheduled_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_scheduled_status CHECK (status IN ('pending', 'sent', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_scheduled_emails_pending ON scheduled_emails(scheduled_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_user ON scheduled_emails(user_id, bot_id);

-- Email templates
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    subject_template TEXT NOT NULL,
    body_html_template TEXT NOT NULL,
    body_plain_template TEXT,
    variables_json TEXT DEFAULT '[]',
    category VARCHAR(100),
    is_shared BOOLEAN DEFAULT false,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_templates_bot ON email_templates(bot_id);
CREATE INDEX IF NOT EXISTS idx_email_templates_category ON email_templates(category);
CREATE INDEX IF NOT EXISTS idx_email_templates_shared ON email_templates(bot_id) WHERE is_shared = true;

-- Auto-responders (Out of Office) - works with Stalwart Sieve
CREATE TABLE IF NOT EXISTS email_auto_responders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    responder_type VARCHAR(50) NOT NULL DEFAULT 'out_of_office',
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    send_to_internal_only BOOLEAN DEFAULT false,
    exclude_addresses TEXT,
    is_active BOOLEAN DEFAULT false,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_responder_type CHECK (responder_type IN ('out_of_office', 'vacation', 'custom')),
    CONSTRAINT unique_user_responder UNIQUE (user_id, bot_id, responder_type)
);

CREATE INDEX IF NOT EXISTS idx_auto_responders_active ON email_auto_responders(user_id, bot_id) WHERE is_active = true;

-- Email rules/filters - synced with Stalwart Sieve
CREATE TABLE IF NOT EXISTS email_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 0,
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    stop_processing BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_rules_user ON email_rules(user_id, bot_id);
CREATE INDEX IF NOT EXISTS idx_email_rules_priority ON email_rules(user_id, bot_id, priority);

-- Email labels/categories
CREATE TABLE IF NOT EXISTS email_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7) DEFAULT '#3b82f6',
    parent_id UUID REFERENCES email_labels(id) ON DELETE CASCADE,
    is_system BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_label UNIQUE (user_id, bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_email_labels_user ON email_labels(user_id, bot_id);

-- Email-label associations
CREATE TABLE IF NOT EXISTS email_label_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_message_id VARCHAR(255) NOT NULL,
    label_id UUID NOT NULL REFERENCES email_labels(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_email_label UNIQUE (email_message_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_label_assignments_email ON email_label_assignments(email_message_id);
CREATE INDEX IF NOT EXISTS idx_label_assignments_label ON email_label_assignments(label_id);

-- Distribution lists
CREATE TABLE IF NOT EXISTS distribution_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email_alias VARCHAR(255),
    description TEXT,
    members_json TEXT NOT NULL DEFAULT '[]',
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_distribution_lists_bot ON distribution_lists(bot_id);
CREATE INDEX IF NOT EXISTS idx_distribution_lists_owner ON distribution_lists(owner_id);

-- Shared mailboxes - managed via Stalwart
CREATE TABLE IF NOT EXISTS shared_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    email_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    settings_json TEXT DEFAULT '{}',
    stalwart_account_id VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_shared_mailbox_email UNIQUE (bot_id, email_address)
);

CREATE INDEX IF NOT EXISTS idx_shared_mailboxes_bot ON shared_mailboxes(bot_id);

CREATE TABLE IF NOT EXISTS shared_mailbox_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID NOT NULL REFERENCES shared_mailboxes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_level VARCHAR(20) DEFAULT 'read',
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_mailbox_member UNIQUE (mailbox_id, user_id),
    CONSTRAINT check_permission CHECK (permission_level IN ('read', 'write', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_shared_mailbox_members ON shared_mailbox_members(mailbox_id);
CREATE INDEX IF NOT EXISTS idx_shared_mailbox_user ON shared_mailbox_members(user_id);

-- ============================================================================
-- VIDEO MEETING FEATURES (Google Meet/Zoom parity)
-- ============================================================================

-- Meeting recordings
CREATE TABLE IF NOT EXISTS meeting_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    recorded_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    format VARCHAR(20) DEFAULT 'mp4',
    thumbnail_path TEXT,
    transcription_path TEXT,
    transcription_status VARCHAR(20) DEFAULT 'pending',
    is_shared BOOLEAN DEFAULT false,
    shared_with_json TEXT DEFAULT '[]',
    retention_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_transcription_status CHECK (transcription_status IN ('pending', 'processing', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_meeting_recordings_meeting ON meeting_recordings(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_recordings_bot ON meeting_recordings(bot_id);

-- Breakout rooms
CREATE TABLE IF NOT EXISTS meeting_breakout_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    room_number INTEGER NOT NULL,
    participants_json TEXT DEFAULT '[]',
    duration_minutes INTEGER,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_breakout_rooms_meeting ON meeting_breakout_rooms(meeting_id);

-- Meeting polls
CREATE TABLE IF NOT EXISTS meeting_polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    poll_type VARCHAR(20) DEFAULT 'single',
    options_json TEXT NOT NULL,
    is_anonymous BOOLEAN DEFAULT false,
    allow_multiple BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT false,
    results_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    CONSTRAINT check_poll_type CHECK (poll_type IN ('single', 'multiple', 'open'))
);

CREATE INDEX IF NOT EXISTS idx_meeting_polls_meeting ON meeting_polls(meeting_id);

-- Meeting Q&A
CREATE TABLE IF NOT EXISTS meeting_questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    asked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    question TEXT NOT NULL,
    is_anonymous BOOLEAN DEFAULT false,
    upvotes INTEGER DEFAULT 0,
    is_answered BOOLEAN DEFAULT false,
    answer TEXT,
    answered_by UUID REFERENCES users(id) ON DELETE SET NULL,
    answered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meeting_questions_meeting ON meeting_questions(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_questions_unanswered ON meeting_questions(meeting_id) WHERE is_answered = false;

-- Meeting waiting room
CREATE TABLE IF NOT EXISTS meeting_waiting_room (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    guest_name VARCHAR(255),
    guest_email VARCHAR(255),
    device_info_json TEXT DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'waiting',
    admitted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    admitted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_waiting_status CHECK (status IN ('waiting', 'admitted', 'rejected', 'left'))
);

CREATE INDEX IF NOT EXISTS idx_waiting_room_meeting ON meeting_waiting_room(meeting_id);
CREATE INDEX IF NOT EXISTS idx_waiting_room_status ON meeting_waiting_room(meeting_id, status);

-- Meeting live captions
CREATE TABLE IF NOT EXISTS meeting_captions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    speaker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    speaker_name VARCHAR(255),
    caption_text TEXT NOT NULL,
    language VARCHAR(10) DEFAULT 'en',
    confidence REAL,
    timestamp_ms BIGINT NOT NULL,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meeting_captions_meeting ON meeting_captions(meeting_id, timestamp_ms);

-- Virtual backgrounds
CREATE TABLE IF NOT EXISTS user_virtual_backgrounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100),
    background_type VARCHAR(20) DEFAULT 'image',
    file_path TEXT,
    blur_intensity INTEGER,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_bg_type CHECK (background_type IN ('image', 'blur', 'none'))
);

CREATE INDEX IF NOT EXISTS idx_virtual_backgrounds_user ON user_virtual_backgrounds(user_id);

-- ============================================================================
-- DRIVE ENTERPRISE FEATURES (Google Drive/OneDrive parity)
-- ============================================================================

-- File version history
CREATE TABLE IF NOT EXISTS file_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    version_number INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    modified_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    change_summary TEXT,
    is_current BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_file_version UNIQUE (file_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_file_versions_file ON file_versions(file_id);
CREATE INDEX IF NOT EXISTS idx_file_versions_current ON file_versions(file_id) WHERE is_current = true;

-- File comments
CREATE TABLE IF NOT EXISTS file_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES file_comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    anchor_data_json TEXT,
    is_resolved BOOLEAN DEFAULT false,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_file_comments_file ON file_comments(file_id);
CREATE INDEX IF NOT EXISTS idx_file_comments_unresolved ON file_comments(file_id) WHERE is_resolved = false;

-- File sharing permissions
CREATE TABLE IF NOT EXISTS file_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    shared_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_with_user UUID REFERENCES users(id) ON DELETE CASCADE,
    shared_with_email VARCHAR(255),
    shared_with_group UUID,
    permission_level VARCHAR(20) NOT NULL DEFAULT 'view',
    can_reshare BOOLEAN DEFAULT false,
    password_hash VARCHAR(255),
    expires_at TIMESTAMPTZ,
    link_token VARCHAR(64) UNIQUE,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_share_permission CHECK (permission_level IN ('view', 'comment', 'edit', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_file_shares_file ON file_shares(file_id);
CREATE INDEX IF NOT EXISTS idx_file_shares_user ON file_shares(shared_with_user);
CREATE INDEX IF NOT EXISTS idx_file_shares_token ON file_shares(link_token) WHERE link_token IS NOT NULL;

-- File activity log
CREATE TABLE IF NOT EXISTS file_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    activity_type VARCHAR(50) NOT NULL,
    details_json TEXT DEFAULT '{}',
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_file_activities_file ON file_activities(file_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_file_activities_user ON file_activities(user_id, created_at DESC);

-- Trash bin (soft delete with restore)
CREATE TABLE IF NOT EXISTS file_trash (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_file_id UUID NOT NULL,
    original_path TEXT NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    file_metadata_json TEXT NOT NULL,
    deleted_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    deleted_at TIMESTAMPTZ DEFAULT NOW(),
    permanent_delete_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_file_trash_owner ON file_trash(owner_id);
CREATE INDEX IF NOT EXISTS idx_file_trash_expiry ON file_trash(permanent_delete_at);

-- Offline sync tracking
CREATE TABLE IF NOT EXISTS file_sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(255) NOT NULL,
    file_id UUID NOT NULL,
    local_path TEXT,
    sync_status VARCHAR(20) DEFAULT 'synced',
    local_version INTEGER,
    remote_version INTEGER,
    conflict_data_json TEXT,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_sync_status CHECK (sync_status IN ('synced', 'pending', 'conflict', 'error')),
    CONSTRAINT unique_sync_entry UNIQUE (user_id, device_id, file_id)
);

CREATE INDEX IF NOT EXISTS idx_file_sync_user ON file_sync_status(user_id, device_id);
CREATE INDEX IF NOT EXISTS idx_file_sync_pending ON file_sync_status(user_id) WHERE sync_status = 'pending';

-- Storage quotas
CREATE TABLE IF NOT EXISTS storage_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID REFERENCES bots(id) ON DELETE CASCADE,
    quota_bytes BIGINT NOT NULL DEFAULT 5368709120,
    used_bytes BIGINT NOT NULL DEFAULT 0,
    warning_threshold_percent INTEGER DEFAULT 90,
    last_calculated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_quota UNIQUE (user_id),
    CONSTRAINT unique_bot_quota UNIQUE (bot_id)
);

CREATE INDEX IF NOT EXISTS idx_storage_quotas_user ON storage_quotas(user_id);

-- ============================================================================
-- COLLABORATION FEATURES
-- ============================================================================

-- Document presence (who's viewing/editing)
CREATE TABLE IF NOT EXISTS document_presence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cursor_position_json TEXT,
    selection_range_json TEXT,
    color VARCHAR(7),
    is_editing BOOLEAN DEFAULT false,
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_doc_user_presence UNIQUE (document_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_document_presence_doc ON document_presence(document_id);

-- ============================================================================
-- TASK ENTERPRISE FEATURES
-- ============================================================================

-- Core tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    priority TEXT NOT NULL DEFAULT 'medium',
    assignee_id UUID REFERENCES users(id) ON DELETE SET NULL,
    reporter_id UUID REFERENCES users(id) ON DELETE SET NULL,
    project_id UUID,
    due_date TIMESTAMPTZ,
    tags TEXT[] DEFAULT '{}',
    dependencies UUID[] DEFAULT '{}',
    estimated_hours FLOAT8,
    actual_hours FLOAT8,
    progress INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT check_task_status CHECK (status IN ('todo', 'in_progress', 'review', 'blocked', 'on_hold', 'done', 'completed', 'cancelled')),
    CONSTRAINT check_task_priority CHECK (priority IN ('low', 'medium', 'high', 'urgent'))
);

CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_reporter ON tasks(reporter_id);
CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created_at);

-- Task dependencies
CREATE TABLE IF NOT EXISTS task_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    dependency_type VARCHAR(20) DEFAULT 'finish_to_start',
    lag_days INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_dependency_type CHECK (dependency_type IN ('finish_to_start', 'start_to_start', 'finish_to_finish', 'start_to_finish')),
    CONSTRAINT unique_task_dependency UNIQUE (task_id, depends_on_task_id)
);

CREATE INDEX IF NOT EXISTS idx_task_dependencies_task ON task_dependencies(task_id);
CREATE INDEX IF NOT EXISTS idx_task_dependencies_depends ON task_dependencies(depends_on_task_id);

-- Task time tracking
CREATE TABLE IF NOT EXISTS task_time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_minutes INTEGER,
    is_billable BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_time_task ON task_time_entries(task_id);
CREATE INDEX IF NOT EXISTS idx_task_time_user ON task_time_entries(user_id, started_at);

-- Task recurring rules
CREATE TABLE IF NOT EXISTS task_recurrence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_template_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    recurrence_pattern VARCHAR(20) NOT NULL,
    interval_value INTEGER DEFAULT 1,
    days_of_week_json TEXT,
    day_of_month INTEGER,
    month_of_year INTEGER,
    end_date TIMESTAMPTZ,
    occurrence_count INTEGER,
    next_occurrence TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_recurrence CHECK (recurrence_pattern IN ('daily', 'weekly', 'monthly', 'yearly', 'custom'))
);

CREATE INDEX IF NOT EXISTS idx_task_recurrence_next ON task_recurrence(next_occurrence) WHERE is_active = true;

-- ============================================================================
-- CALENDAR ENTERPRISE FEATURES
-- ============================================================================

-- Resource booking (meeting rooms, equipment)
CREATE TABLE IF NOT EXISTS calendar_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location VARCHAR(255),
    capacity INTEGER,
    amenities_json TEXT DEFAULT '[]',
    availability_hours_json TEXT,
    booking_rules_json TEXT DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_resource_type CHECK (resource_type IN ('room', 'equipment', 'vehicle', 'other'))
);

CREATE INDEX IF NOT EXISTS idx_calendar_resources_bot ON calendar_resources(bot_id);
CREATE INDEX IF NOT EXISTS idx_calendar_resources_type ON calendar_resources(bot_id, resource_type);

CREATE TABLE IF NOT EXISTS calendar_resource_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES calendar_resources(id) ON DELETE CASCADE,
    event_id UUID,
    booked_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    notes TEXT,
    status VARCHAR(20) DEFAULT 'confirmed',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_booking_status CHECK (status IN ('pending', 'confirmed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_resource_bookings_resource ON calendar_resource_bookings(resource_id, start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_resource_bookings_user ON calendar_resource_bookings(booked_by);

-- Calendar sharing
CREATE TABLE IF NOT EXISTS calendar_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_with_user UUID REFERENCES users(id) ON DELETE CASCADE,
    shared_with_email VARCHAR(255),
    permission_level VARCHAR(20) DEFAULT 'view',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_cal_permission CHECK (permission_level IN ('free_busy', 'view', 'edit', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_calendar_shares_owner ON calendar_shares(owner_id);
CREATE INDEX IF NOT EXISTS idx_calendar_shares_shared ON calendar_shares(shared_with_user);

-- ============================================================================
-- TEST SUPPORT TABLES
-- ============================================================================

-- Test accounts for integration testing
CREATE TABLE IF NOT EXISTS test_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_type VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_test_account_type CHECK (account_type IN ('sender', 'receiver', 'bot', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_test_accounts_type ON test_accounts(account_type);

-- Test execution logs
CREATE TABLE IF NOT EXISTS test_execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_suite VARCHAR(100) NOT NULL,
    test_name VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL,
    duration_ms INTEGER,
    error_message TEXT,
    stack_trace TEXT,
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_test_status CHECK (status IN ('passed', 'failed', 'skipped', 'error'))
);

CREATE INDEX IF NOT EXISTS idx_test_logs_suite ON test_execution_logs(test_suite, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_test_logs_status ON test_execution_logs(status, created_at DESC);
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
-- Migration: 6.1.2_phase3_phase4
-- Description: Phase 3 and Phase 4 multi-agent features
-- Features:
--   - Episodic memory (conversation summaries)
--   - Knowledge graphs (entity relationships)
--   - Human-in-the-loop approvals
--   - LLM observability and cost tracking

-- ============================================
-- EPISODIC MEMORY TABLES
-- ============================================

-- Conversation episodes (summaries)
CREATE TABLE IF NOT EXISTS conversation_episodes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    summary TEXT NOT NULL,
    key_topics JSONB NOT NULL DEFAULT '[]',
    decisions JSONB NOT NULL DEFAULT '[]',
    action_items JSONB NOT NULL DEFAULT '[]',
    sentiment JSONB NOT NULL DEFAULT '{"score": 0, "label": "neutral", "confidence": 0.5}',
    resolution VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message_count INTEGER NOT NULL DEFAULT 0,
    message_ids JSONB NOT NULL DEFAULT '[]',
    conversation_start TIMESTAMP WITH TIME ZONE NOT NULL,
    conversation_end TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for episodic memory
CREATE INDEX IF NOT EXISTS idx_episodes_user_id ON conversation_episodes(user_id);
CREATE INDEX IF NOT EXISTS idx_episodes_bot_id ON conversation_episodes(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodes_session_id ON conversation_episodes(session_id);
CREATE INDEX IF NOT EXISTS idx_episodes_created_at ON conversation_episodes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_key_topics ON conversation_episodes USING GIN(key_topics);
CREATE INDEX IF NOT EXISTS idx_episodes_resolution ON conversation_episodes(resolution);

-- Full-text search on summaries
CREATE INDEX IF NOT EXISTS idx_episodes_summary_fts ON conversation_episodes
    USING GIN(to_tsvector('english', summary));

-- ============================================
-- KNOWLEDGE GRAPH TABLES - Add missing columns
-- (Tables created earlier in this migration)
-- ============================================

-- Add missing columns to kg_entities
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS aliases JSONB NOT NULL DEFAULT '[]';
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0;
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS source VARCHAR(50) NOT NULL DEFAULT 'manual';

-- Add missing columns to kg_relationships
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0;
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS bidirectional BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS source VARCHAR(50) NOT NULL DEFAULT 'manual';

-- Indexes for knowledge graph
CREATE INDEX IF NOT EXISTS idx_kg_entities_bot_id ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_lower ON kg_entities(LOWER(entity_name));
CREATE INDEX IF NOT EXISTS idx_kg_entities_aliases ON kg_entities USING GIN(aliases);

CREATE INDEX IF NOT EXISTS idx_kg_relationships_bot_id ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_type ON kg_relationships(relationship_type);

-- Full-text search on entity names
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_fts ON kg_entities
    USING GIN(to_tsvector('english', entity_name));

-- ============================================
-- HUMAN-IN-THE-LOOP APPROVAL TABLES
-- ============================================

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    approval_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    channel VARCHAR(50) NOT NULL,
    recipient VARCHAR(500) NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    message TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    default_action VARCHAR(50),
    current_level INTEGER NOT NULL DEFAULT 1,
    total_levels INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reminders_sent JSONB NOT NULL DEFAULT '[]',
    decision VARCHAR(50),
    decided_by VARCHAR(500),
    decided_at TIMESTAMP WITH TIME ZONE,
    comments TEXT
);

-- Approval chains
CREATE TABLE IF NOT EXISTS approval_chains (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    bot_id UUID NOT NULL,
    levels JSONB NOT NULL DEFAULT '[]',
    stop_on_reject BOOLEAN NOT NULL DEFAULT true,
    require_all BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Approval audit log
CREATE TABLE IF NOT EXISTS approval_audit_log (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    actor VARCHAR(500) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(50),
    user_agent TEXT
);

-- Approval tokens (for secure links)
CREATE TABLE IF NOT EXISTS approval_tokens (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL UNIQUE,
    action VARCHAR(50) NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for approval tables
CREATE INDEX IF NOT EXISTS idx_approval_requests_bot_id ON approval_requests(bot_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_session_id ON approval_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_status ON approval_requests(status);
CREATE INDEX IF NOT EXISTS idx_approval_requests_expires_at ON approval_requests(expires_at);
CREATE INDEX IF NOT EXISTS idx_approval_requests_pending ON approval_requests(status, expires_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_approval_audit_request_id ON approval_audit_log(request_id);
CREATE INDEX IF NOT EXISTS idx_approval_audit_timestamp ON approval_audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_approval_tokens_token ON approval_tokens(token);
CREATE INDEX IF NOT EXISTS idx_approval_tokens_request_id ON approval_tokens(request_id);

-- ============================================
-- LLM OBSERVABILITY TABLES
-- ============================================

-- LLM request metrics
CREATE TABLE IF NOT EXISTS llm_metrics (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model VARCHAR(200) NOT NULL,
    request_type VARCHAR(50) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    ttft_ms BIGINT,
    cached BOOLEAN NOT NULL DEFAULT false,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    estimated_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Aggregated metrics (hourly rollup)
CREATE TABLE IF NOT EXISTS llm_metrics_hourly (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    hour TIMESTAMP WITH TIME ZONE NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    successful_requests BIGINT NOT NULL DEFAULT 0,
    failed_requests BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p50_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p99_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_latency_ms BIGINT NOT NULL DEFAULT 0,
    min_latency_ms BIGINT NOT NULL DEFAULT 0,
    requests_by_model JSONB NOT NULL DEFAULT '{}',
    tokens_by_model JSONB NOT NULL DEFAULT '{}',
    cost_by_model JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, hour)
);

-- Budget tracking
CREATE TABLE IF NOT EXISTS llm_budget (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL UNIQUE,
    daily_limit DOUBLE PRECISION NOT NULL DEFAULT 100,
    monthly_limit DOUBLE PRECISION NOT NULL DEFAULT 2000,
    alert_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.8,
    daily_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    monthly_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    monthly_reset_date DATE NOT NULL DEFAULT DATE_TRUNC('month', CURRENT_DATE)::DATE,
    daily_alert_sent BOOLEAN NOT NULL DEFAULT false,
    monthly_alert_sent BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Trace events
CREATE TABLE IF NOT EXISTS llm_traces (
    id UUID PRIMARY KEY,
    parent_id UUID,
    trace_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    component VARCHAR(100) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    duration_ms BIGINT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    attributes JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress',
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for observability tables
CREATE INDEX IF NOT EXISTS idx_llm_metrics_bot_id ON llm_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_session_id ON llm_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_timestamp ON llm_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_model ON llm_metrics(model);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_bot_id ON llm_metrics_hourly(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_hour ON llm_metrics_hourly(hour DESC);

CREATE INDEX IF NOT EXISTS idx_llm_traces_trace_id ON llm_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_llm_traces_start_time ON llm_traces(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_llm_traces_component ON llm_traces(component);

-- ============================================
-- WORKFLOW TABLES
-- ============================================

-- Workflow definitions
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]',
    triggers JSONB NOT NULL DEFAULT '[]',
    error_handling JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Workflow executions
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL,
    session_id UUID,
    initiated_by UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    current_step INTEGER NOT NULL DEFAULT 0,
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    step_results JSONB NOT NULL DEFAULT '[]',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Workflow step executions
CREATE TABLE IF NOT EXISTS workflow_step_executions (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    step_name VARCHAR(200) NOT NULL,
    step_index INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms BIGINT
);

-- Indexes for workflow tables
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_bot_id ON workflow_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_bot_id ON workflow_executions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_step_executions_execution_id ON workflow_step_executions(execution_id);

-- ============================================
-- FUNCTIONS AND TRIGGERS
-- ============================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON workflow_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_llm_budget_updated_at ON llm_budget;
CREATE TRIGGER update_llm_budget_updated_at
    BEFORE UPDATE ON llm_budget
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to aggregate hourly metrics
CREATE OR REPLACE FUNCTION aggregate_llm_metrics_hourly()
RETURNS void AS $$
DECLARE
    last_hour TIMESTAMP WITH TIME ZONE;
BEGIN
    last_hour := DATE_TRUNC('hour', NOW() - INTERVAL '1 hour');

    INSERT INTO llm_metrics_hourly (
        id, bot_id, hour, total_requests, successful_requests, failed_requests,
        cache_hits, cache_misses, total_input_tokens, total_output_tokens,
        total_tokens, total_cost, avg_latency_ms, p50_latency_ms, p95_latency_ms,
        p99_latency_ms, max_latency_ms, min_latency_ms, requests_by_model,
        tokens_by_model, cost_by_model
    )
    SELECT
        gen_random_uuid(),
        bot_id,
        last_hour,
        COUNT(*),
        COUNT(*) FILTER (WHERE success = true),
        COUNT(*) FILTER (WHERE success = false),
        COUNT(*) FILTER (WHERE cached = true),
        COUNT(*) FILTER (WHERE cached = false),
        SUM(input_tokens),
        SUM(output_tokens),
        SUM(total_tokens),
        SUM(estimated_cost),
        AVG(latency_ms),
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms),
        MAX(latency_ms),
        MIN(latency_ms),
        jsonb_object_agg(model, model_count) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_tokens) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_cost) FILTER (WHERE model IS NOT NULL)
    FROM (
        SELECT
            bot_id, model, success, cached, input_tokens, output_tokens,
            total_tokens, estimated_cost, latency_ms,
            COUNT(*) OVER (PARTITION BY bot_id, model) as model_count,
            SUM(total_tokens) OVER (PARTITION BY bot_id, model) as model_tokens,
            SUM(estimated_cost) OVER (PARTITION BY bot_id, model) as model_cost
        FROM llm_metrics
        WHERE timestamp >= last_hour
        AND timestamp < last_hour + INTERVAL '1 hour'
    ) sub
    GROUP BY bot_id
    ON CONFLICT (bot_id, hour) DO UPDATE SET
        total_requests = EXCLUDED.total_requests,
        successful_requests = EXCLUDED.successful_requests,
        failed_requests = EXCLUDED.failed_requests,
        cache_hits = EXCLUDED.cache_hits,
        cache_misses = EXCLUDED.cache_misses,
        total_input_tokens = EXCLUDED.total_input_tokens,
        total_output_tokens = EXCLUDED.total_output_tokens,
        total_tokens = EXCLUDED.total_tokens,
        total_cost = EXCLUDED.total_cost,
        avg_latency_ms = EXCLUDED.avg_latency_ms,
        p50_latency_ms = EXCLUDED.p50_latency_ms,
        p95_latency_ms = EXCLUDED.p95_latency_ms,
        p99_latency_ms = EXCLUDED.p99_latency_ms,
        max_latency_ms = EXCLUDED.max_latency_ms,
        min_latency_ms = EXCLUDED.min_latency_ms,
        requests_by_model = EXCLUDED.requests_by_model,
        tokens_by_model = EXCLUDED.tokens_by_model,
        cost_by_model = EXCLUDED.cost_by_model;
END;
$$ LANGUAGE plpgsql;

-- Function to reset daily budget
CREATE OR REPLACE FUNCTION reset_daily_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET daily_spend = 0,
        daily_reset_date = CURRENT_DATE,
        daily_alert_sent = false
    WHERE daily_reset_date < CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Function to reset monthly budget
CREATE OR REPLACE FUNCTION reset_monthly_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET monthly_spend = 0,
        monthly_reset_date = DATE_TRUNC('month', CURRENT_DATE)::DATE,
        monthly_alert_sent = false
    WHERE monthly_reset_date < DATE_TRUNC('month', CURRENT_DATE)::DATE;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- VIEWS
-- ============================================

-- View for recent episode summaries with user info
CREATE OR REPLACE VIEW v_recent_episodes AS
SELECT
    e.id,
    e.user_id,
    e.bot_id,
    e.session_id,
    e.summary,
    e.key_topics,
    e.sentiment,
    e.resolution,
    e.message_count,
    e.created_at,
    e.conversation_start,
    e.conversation_end
FROM conversation_episodes e
ORDER BY e.created_at DESC;

-- View for knowledge graph statistics
CREATE OR REPLACE VIEW v_kg_stats AS
SELECT
    bot_id,
    COUNT(DISTINCT id) as total_entities,
    COUNT(DISTINCT entity_type) as entity_types,
    (SELECT COUNT(*) FROM kg_relationships r WHERE r.bot_id = e.bot_id) as total_relationships
FROM kg_entities e
GROUP BY bot_id;

-- View for approval status summary
CREATE OR REPLACE VIEW v_approval_summary AS
SELECT
    bot_id,
    status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (COALESCE(decided_at, NOW()) - created_at))) as avg_resolution_seconds
FROM approval_requests
GROUP BY bot_id, status;

-- View for LLM usage summary (last 24 hours)
CREATE OR REPLACE VIEW v_llm_usage_24h AS
SELECT
    bot_id,
    model,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    SUM(estimated_cost) as total_cost,
    AVG(latency_ms) as avg_latency_ms,
    SUM(CASE WHEN cached THEN 1 ELSE 0 END)::FLOAT / COUNT(*) as cache_hit_rate,
    SUM(CASE WHEN success THEN 0 ELSE 1 END)::FLOAT / COUNT(*) as error_rate
FROM llm_metrics
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY bot_id, model;

-- ============================================
-- CLEANUP POLICIES (retention)
-- ============================================

-- Create a cleanup function for old data
CREATE OR REPLACE FUNCTION cleanup_old_observability_data(retention_days INTEGER DEFAULT 30)
RETURNS void AS $$
BEGIN
    -- Delete old LLM metrics (keep hourly aggregates longer)
    DELETE FROM llm_metrics WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old traces
    DELETE FROM llm_traces WHERE start_time < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old approval audit logs
    DELETE FROM approval_audit_log WHERE timestamp < NOW() - (retention_days * 3 || ' days')::INTERVAL;

    -- Delete expired approval tokens
    DELETE FROM approval_tokens WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;
-- Suite Applications Migration
-- Adds tables for: Paper (Documents), Designer (Dialogs), and additional analytics support

-- Paper Documents table
CREATE TABLE IF NOT EXISTS paper_documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled Document',
    content TEXT NOT NULL DEFAULT '',
    owner_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_paper_documents_owner ON paper_documents(owner_id);
CREATE INDEX IF NOT EXISTS idx_paper_documents_updated ON paper_documents(updated_at DESC);

-- Designer Dialogs table
CREATE TABLE IF NOT EXISTS designer_dialogs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    bot_id TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_designer_dialogs_bot ON designer_dialogs(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_active ON designer_dialogs(is_active);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_updated ON designer_dialogs(updated_at DESC);

-- Sources Templates table (for template metadata caching)
CREATE TABLE IF NOT EXISTS source_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'General',
    preview_url TEXT,
    file_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_source_templates_category ON source_templates(category);

-- Analytics Events table (for additional event tracking)
CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    user_id UUID,
    session_id UUID,
    bot_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_session ON analytics_events(session_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at DESC);

-- Analytics Daily Aggregates (for faster dashboard queries)
CREATE TABLE IF NOT EXISTS analytics_daily_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    bot_id UUID,
    metric_name TEXT NOT NULL,
    metric_value BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(date, bot_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_analytics_daily_date ON analytics_daily_aggregates(date DESC);
CREATE INDEX IF NOT EXISTS idx_analytics_daily_bot ON analytics_daily_aggregates(bot_id);

-- Research Search History (for recent searches feature)
CREATE TABLE IF NOT EXISTS research_search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    collection_id TEXT,
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_history_user ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_history_created ON research_search_history(created_at DESC);
-- Email Read Tracking Table
-- Stores sent email tracking data for read receipt functionality
-- Enabled via config.csv: email-read-pixel,true

CREATE TABLE IF NOT EXISTS sent_email_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tracking_id UUID NOT NULL UNIQUE,
    bot_id UUID NOT NULL,
    account_id UUID NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    cc TEXT,
    bcc TEXT,
    subject TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    read_count INTEGER NOT NULL DEFAULT 0,
    first_read_ip VARCHAR(45),
    last_read_ip VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_tracking_id ON sent_email_tracking(tracking_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_bot_id ON sent_email_tracking(bot_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_account_id ON sent_email_tracking(account_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_to_email ON sent_email_tracking(to_email);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_sent_at ON sent_email_tracking(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_is_read ON sent_email_tracking(is_read);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_read_status ON sent_email_tracking(bot_id, is_read, sent_at DESC);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_sent_email_tracking_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_sent_email_tracking_updated_at ON sent_email_tracking;
CREATE TRIGGER trigger_update_sent_email_tracking_updated_at
    BEFORE UPDATE ON sent_email_tracking
    FOR EACH ROW
    EXECUTE FUNCTION update_sent_email_tracking_updated_at();

-- Add comment for documentation
COMMENT ON TABLE sent_email_tracking IS 'Tracks sent emails for read receipt functionality via tracking pixel';
COMMENT ON COLUMN sent_email_tracking.tracking_id IS 'Unique ID embedded in tracking pixel URL';
COMMENT ON COLUMN sent_email_tracking.is_read IS 'Whether the email has been opened (pixel loaded)';
COMMENT ON COLUMN sent_email_tracking.read_count IS 'Number of times the email was opened';
COMMENT ON COLUMN sent_email_tracking.first_read_ip IS 'IP address of first email open';
COMMENT ON COLUMN sent_email_tracking.last_read_ip IS 'IP address of most recent email open';
-- ============================================
-- TABLE KEYWORD SUPPORT (from 6.1.0_table_keyword)
-- ============================================

-- Migration for TABLE keyword support
-- Stores dynamic table definitions created via BASIC TABLE...END TABLE syntax

-- Table to store dynamic table definitions (metadata)
CREATE TABLE IF NOT EXISTS dynamic_table_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    table_name VARCHAR(255) NOT NULL,
    connection_name VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,

    -- Ensure unique table name per bot and connection
    CONSTRAINT unique_bot_table_connection UNIQUE (bot_id, table_name, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_dynamic_table_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Table to store field definitions for dynamic tables
CREATE TABLE IF NOT EXISTS dynamic_table_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_definition_id UUID NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    field_type VARCHAR(100) NOT NULL,
    field_length INTEGER,
    field_precision INTEGER,
    is_key BOOLEAN DEFAULT false,
    is_nullable BOOLEAN DEFAULT true,
    default_value TEXT,
    reference_table VARCHAR(255),
    field_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure unique field name per table definition
    CONSTRAINT unique_table_field UNIQUE (table_definition_id, field_name),

    -- Foreign key to table definitions
    CONSTRAINT fk_field_table_definition
        FOREIGN KEY (table_definition_id)
        REFERENCES dynamic_table_definitions(id)
        ON DELETE CASCADE
);

-- Table to store external database connections (from config.csv conn-* entries)
CREATE TABLE IF NOT EXISTS external_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    connection_name VARCHAR(255) NOT NULL,
    driver VARCHAR(100) NOT NULL,
    server VARCHAR(255) NOT NULL,
    port INTEGER,
    database_name VARCHAR(255),
    username VARCHAR(255),
    password_encrypted TEXT,
    additional_params JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_connected_at TIMESTAMPTZ,

    -- Ensure unique connection name per bot
    CONSTRAINT unique_bot_connection UNIQUE (bot_id, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_external_connection_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_bot_id
    ON dynamic_table_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_name
    ON dynamic_table_definitions(table_name);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_connection
    ON dynamic_table_definitions(connection_name);

CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_table_id
    ON dynamic_table_fields(table_definition_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_name
    ON dynamic_table_fields(field_name);

CREATE INDEX IF NOT EXISTS idx_external_connections_bot_id
    ON external_connections(bot_id);
CREATE INDEX IF NOT EXISTS idx_external_connections_name
    ON external_connections(connection_name);

-- Create trigger to update updated_at timestamp for dynamic_table_definitions
CREATE OR REPLACE FUNCTION update_dynamic_table_definitions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER dynamic_table_definitions_updated_at_trigger
    BEFORE UPDATE ON dynamic_table_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_dynamic_table_definitions_updated_at();

-- Create trigger to update updated_at timestamp for external_connections
CREATE OR REPLACE FUNCTION update_external_connections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER external_connections_updated_at_trigger
    BEFORE UPDATE ON external_connections
    FOR EACH ROW
    EXECUTE FUNCTION update_external_connections_updated_at();

-- ============================================================================
-- CONFIG ID TYPE FIXES (from 6.1.1)
-- Fix columns that were created as TEXT but should be UUID
-- ============================================================================

-- For bot_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'bot_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE bot_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For server_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'server_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE server_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For tenant_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'tenant_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE tenant_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For model_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'model_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE model_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For connection_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'connection_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE connection_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For component_installations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_installations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_installations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For component_logs
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_logs'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_logs
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For gbot_config_sync
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'gbot_config_sync'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE gbot_config_sync
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- ============================================================================
-- CONNECTED ACCOUNTS (from 6.1.2)
-- OAuth connected accounts for email providers
-- ============================================================================

CREATE TABLE IF NOT EXISTS connected_accounts (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id UUID,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    account_type TEXT NOT NULL DEFAULT 'email',
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    scopes TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 300,
    last_sync_at TIMESTAMPTZ,
    last_sync_status TEXT,
    last_sync_error TEXT,
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connected_accounts_bot_id ON connected_accounts(bot_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_user_id ON connected_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_email ON connected_accounts(email);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_provider ON connected_accounts(provider);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_status ON connected_accounts(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_connected_accounts_bot_email ON connected_accounts(bot_id, email);

CREATE TABLE IF NOT EXISTS session_account_associations (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT
);

CREATE INDEX IF NOT EXISTS idx_session_account_assoc_session ON session_account_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_account ON session_account_associations(account_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_active ON session_account_associations(session_id, is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_session_account_assoc_unique ON session_account_associations(session_id, account_id);

CREATE TABLE IF NOT EXISTS account_sync_items (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    subject TEXT,
    content_preview TEXT,
    sender TEXT,
    recipients TEXT,
    item_date TIMESTAMPTZ,
    folder TEXT,
    labels TEXT,
    has_attachments BOOLEAN DEFAULT false,
    qdrant_point_id TEXT,
    embedding_status TEXT DEFAULT 'pending',
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_account_sync_items_account ON account_sync_items(account_id);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_type ON account_sync_items(item_type);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_date ON account_sync_items(item_date);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_embedding ON account_sync_items(embedding_status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_account_sync_items_unique ON account_sync_items(account_id, item_type, item_id);

-- ============================================================================
-- BOT HIERARCHY AND MONITORS (from 6.1.3)
-- Sub-bots, ON EMAIL triggers, ON CHANGE triggers
-- ============================================================================

-- Bot Hierarchy: Add parent_bot_id to support sub-bots
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS parent_bot_id UUID REFERENCES public.bots(id) ON DELETE SET NULL;

-- Index for efficient hierarchy queries
CREATE INDEX IF NOT EXISTS idx_bots_parent_bot_id ON public.bots(parent_bot_id);

-- Bot enabled tabs configuration (which UI tabs are enabled for this bot)
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS enabled_tabs_json TEXT DEFAULT '["chat"]';

-- Bot configuration inheritance flag
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS inherit_parent_config BOOLEAN DEFAULT true;

-- Email monitoring table for ON EMAIL triggers
CREATE TABLE IF NOT EXISTS public.email_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
    email_address VARCHAR(500) NOT NULL,
    script_path VARCHAR(1000) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_check_at TIMESTAMPTZ,
    last_uid BIGINT DEFAULT 0,
    filter_from VARCHAR(500),
    filter_subject VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_bot_email UNIQUE (bot_id, email_address)
);

CREATE INDEX IF NOT EXISTS idx_email_monitors_bot_id ON public.email_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_email_monitors_email ON public.email_monitors(email_address);
CREATE INDEX IF NOT EXISTS idx_email_monitors_active ON public.email_monitors(is_active) WHERE is_active = true;

-- Folder monitoring table for ON CHANGE triggers (GDrive, OneDrive, Dropbox)
-- Uses account:// syntax: account://user@gmail.com/path or gdrive:///path
CREATE TABLE IF NOT EXISTS public.folder_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- 'gdrive', 'onedrive', 'dropbox', 'local'
    account_email VARCHAR(500), -- Email from account:// path (e.g., user@gmail.com)
    folder_path VARCHAR(2000) NOT NULL,
    folder_id VARCHAR(500), -- Provider-specific folder ID
    script_path VARCHAR(1000) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    watch_subfolders BOOLEAN DEFAULT true,
    last_check_at TIMESTAMPTZ,
    last_change_token VARCHAR(500), -- Provider-specific change token/page token
    event_types_json TEXT DEFAULT '["create", "modify", "delete"]',
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_bot_folder UNIQUE (bot_id, provider, folder_path)
);

CREATE INDEX IF NOT EXISTS idx_folder_monitors_bot_id ON public.folder_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_provider ON public.folder_monitors(provider);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_active ON public.folder_monitors(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_folder_monitors_account_email ON public.folder_monitors(account_email);

-- Folder change events log
CREATE TABLE IF NOT EXISTS public.folder_change_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES public.folder_monitors(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- 'create', 'modify', 'delete', 'rename', 'move'
    file_path VARCHAR(2000) NOT NULL,
    file_id VARCHAR(500),
    file_name VARCHAR(500),
    file_size BIGINT,
    mime_type VARCHAR(255),
    old_path VARCHAR(2000), -- For rename/move events
    processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_folder_events_monitor ON public.folder_change_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_folder_events_processed ON public.folder_change_events(processed) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_folder_events_created ON public.folder_change_events(created_at);

-- Email received events log
CREATE TABLE IF NOT EXISTS public.email_received_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES public.email_monitors(id) ON DELETE CASCADE,
    message_uid BIGINT NOT NULL,
    message_id VARCHAR(500),
    from_address VARCHAR(500) NOT NULL,
    to_addresses_json TEXT,
    subject VARCHAR(1000),
    received_at TIMESTAMPTZ,
    has_attachments BOOLEAN DEFAULT false,
    attachments_json TEXT,
    processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_email_events_monitor ON public.email_received_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_email_events_processed ON public.email_received_events(processed) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_email_events_received ON public.email_received_events(received_at);

-- Add new trigger kinds to system_automations
-- TriggerKind enum: 0=Scheduled, 1=TableUpdate, 2=TableInsert, 3=TableDelete, 4=Webhook, 5=EmailReceived, 6=FolderChange
COMMENT ON TABLE public.system_automations IS 'System automations with TriggerKind: 0=Scheduled, 1=TableUpdate, 2=TableInsert, 3=TableDelete, 4=Webhook, 5=EmailReceived, 6=FolderChange';

-- User organization memberships (users can belong to multiple orgs)
CREATE TABLE IF NOT EXISTS public.user_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member', -- 'owner', 'admin', 'member', 'viewer'
    is_default BOOLEAN DEFAULT false,
    joined_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_user_org UNIQUE (user_id, org_id)
);

CREATE INDEX IF NOT EXISTS idx_user_orgs_user ON public.user_organizations(user_id);
CREATE INDEX IF NOT EXISTS idx_user_orgs_org ON public.user_organizations(org_id);
CREATE INDEX IF NOT EXISTS idx_user_orgs_default ON public.user_organizations(user_id, is_default) WHERE is_default = true;

-- Comments for documentation
COMMENT ON COLUMN public.bots.parent_bot_id IS 'Parent bot ID for hierarchical bot structure. NULL means root bot.';
COMMENT ON COLUMN public.bots.enabled_tabs_json IS 'JSON array of enabled UI tabs for this bot. Root bots have all tabs.';
COMMENT ON COLUMN public.bots.inherit_parent_config IS 'If true, inherits config from parent bot for missing values.';
COMMENT ON TABLE public.email_monitors IS 'Email monitoring configuration for ON EMAIL triggers.';
COMMENT ON TABLE public.folder_monitors IS 'Folder monitoring configuration for ON CHANGE triggers (GDrive, OneDrive, Dropbox).';
COMMENT ON TABLE public.folder_change_events IS 'Log of detected folder changes to be processed by scripts.';
COMMENT ON TABLE public.email_received_events IS 'Log of received emails to be processed by scripts.';
COMMENT ON TABLE public.user_organizations IS 'User membership in organizations with roles.';
-- Migration: 6.1.1 AutoTask System
-- Description: Tables for the AutoTask system - autonomous task execution with LLM intent compilation
-- NOTE: TABLES AND INDEXES ONLY - No views, triggers, or functions per project standards

-- ============================================================================
-- PENDING INFO TABLE
-- ============================================================================
-- Stores information that the system needs to collect from users
-- Used by ASK LATER keyword to defer collecting config values

CREATE TABLE IF NOT EXISTS pending_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    field_name VARCHAR(100) NOT NULL,
    field_label VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL DEFAULT 'text',
    reason TEXT,
    config_key VARCHAR(255) NOT NULL,
    is_filled BOOLEAN DEFAULT false,
    filled_at TIMESTAMPTZ,
    filled_value TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pending_info_bot_id ON pending_info(bot_id);
CREATE INDEX IF NOT EXISTS idx_pending_info_config_key ON pending_info(config_key);
CREATE INDEX IF NOT EXISTS idx_pending_info_is_filled ON pending_info(is_filled);

-- ============================================================================
-- AUTO TASKS TABLE
-- ============================================================================
-- Stores autonomous tasks that can be executed by the system

CREATE TABLE IF NOT EXISTS auto_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    title VARCHAR(500) NOT NULL,
    intent TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    execution_mode VARCHAR(50) NOT NULL DEFAULT 'supervised',
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    plan_id UUID,
    basic_program TEXT,
    current_step INTEGER DEFAULT 0,
    total_steps INTEGER DEFAULT 0,
    progress FLOAT DEFAULT 0.0,
    step_results JSONB DEFAULT '[]'::jsonb,
    error TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_status CHECK (status IN ('pending', 'ready', 'running', 'paused', 'waiting_approval', 'completed', 'failed', 'cancelled')),
    CONSTRAINT check_execution_mode CHECK (execution_mode IN ('autonomous', 'supervised', 'manual')),
    CONSTRAINT check_priority CHECK (priority IN ('low', 'normal', 'high', 'urgent'))
);

CREATE INDEX IF NOT EXISTS idx_auto_tasks_bot_id ON auto_tasks(bot_id);
CREATE INDEX IF NOT EXISTS idx_auto_tasks_session_id ON auto_tasks(session_id);
CREATE INDEX IF NOT EXISTS idx_auto_tasks_status ON auto_tasks(status);
CREATE INDEX IF NOT EXISTS idx_auto_tasks_priority ON auto_tasks(priority);
CREATE INDEX IF NOT EXISTS idx_auto_tasks_created_at ON auto_tasks(created_at);

-- ============================================================================
-- EXECUTION PLANS TABLE
-- ============================================================================
-- Stores compiled execution plans from intent analysis

CREATE TABLE IF NOT EXISTS execution_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    task_id UUID REFERENCES auto_tasks(id) ON DELETE CASCADE,
    intent TEXT NOT NULL,
    intent_type VARCHAR(100),
    confidence FLOAT DEFAULT 0.0,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    basic_program TEXT,
    simulation_result JSONB,
    approved_at TIMESTAMPTZ,
    approved_by UUID,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_plan_status CHECK (status IN ('pending', 'approved', 'rejected', 'executing', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_execution_plans_bot_id ON execution_plans(bot_id);
CREATE INDEX IF NOT EXISTS idx_execution_plans_task_id ON execution_plans(task_id);
CREATE INDEX IF NOT EXISTS idx_execution_plans_status ON execution_plans(status);
CREATE INDEX IF NOT EXISTS idx_execution_plans_intent_type ON execution_plans(intent_type);

-- ============================================================================
-- TASK APPROVALS TABLE
-- ============================================================================
-- Stores approval requests and decisions for supervised tasks

CREATE TABLE IF NOT EXISTS task_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES auto_tasks(id) ON DELETE CASCADE,
    plan_id UUID REFERENCES execution_plans(id) ON DELETE CASCADE,
    step_index INTEGER,
    action_type VARCHAR(100) NOT NULL,
    action_description TEXT NOT NULL,
    risk_level VARCHAR(20) DEFAULT 'low',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    decision VARCHAR(20),
    decision_reason TEXT,
    decided_by UUID,
    decided_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_risk_level CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT check_approval_status CHECK (status IN ('pending', 'approved', 'rejected', 'expired', 'skipped')),
    CONSTRAINT check_decision CHECK (decision IS NULL OR decision IN ('approve', 'reject', 'skip'))
);

CREATE INDEX IF NOT EXISTS idx_task_approvals_bot_id ON task_approvals(bot_id);
CREATE INDEX IF NOT EXISTS idx_task_approvals_task_id ON task_approvals(task_id);
CREATE INDEX IF NOT EXISTS idx_task_approvals_status ON task_approvals(status);
CREATE INDEX IF NOT EXISTS idx_task_approvals_expires_at ON task_approvals(expires_at);

-- ============================================================================
-- TASK DECISIONS TABLE
-- ============================================================================
-- Stores user decisions requested during task execution

CREATE TABLE IF NOT EXISTS task_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    task_id UUID NOT NULL REFERENCES auto_tasks(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    options JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    selected_option VARCHAR(255),
    decision_reason TEXT,
    decided_by UUID,
    decided_at TIMESTAMPTZ,
    timeout_seconds INTEGER DEFAULT 3600,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_decision_status CHECK (status IN ('pending', 'answered', 'timeout', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_task_decisions_bot_id ON task_decisions(bot_id);
CREATE INDEX IF NOT EXISTS idx_task_decisions_task_id ON task_decisions(task_id);
CREATE INDEX IF NOT EXISTS idx_task_decisions_status ON task_decisions(status);

-- ============================================================================
-- SAFETY AUDIT LOG TABLE
-- ============================================================================
-- Stores audit trail of all safety checks and constraint validations

CREATE TABLE IF NOT EXISTS safety_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    task_id UUID REFERENCES auto_tasks(id) ON DELETE SET NULL,
    plan_id UUID REFERENCES execution_plans(id) ON DELETE SET NULL,
    action_type VARCHAR(100) NOT NULL,
    action_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    constraint_checks JSONB DEFAULT '[]'::jsonb,
    simulation_result JSONB,
    risk_assessment JSONB,
    outcome VARCHAR(50) NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_outcome CHECK (outcome IN ('allowed', 'blocked', 'warning', 'error'))
);

CREATE INDEX IF NOT EXISTS idx_safety_audit_log_bot_id ON safety_audit_log(bot_id);
CREATE INDEX IF NOT EXISTS idx_safety_audit_log_task_id ON safety_audit_log(task_id);
CREATE INDEX IF NOT EXISTS idx_safety_audit_log_outcome ON safety_audit_log(outcome);
CREATE INDEX IF NOT EXISTS idx_safety_audit_log_created_at ON safety_audit_log(created_at);

-- ============================================================================
-- GENERATED APPS TABLE
-- ============================================================================
-- Stores metadata about apps generated by the AppGenerator

CREATE TABLE IF NOT EXISTS generated_apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    domain VARCHAR(100),
    intent_source TEXT,
    pages JSONB DEFAULT '[]'::jsonb,
    tables_created JSONB DEFAULT '[]'::jsonb,
    tools JSONB DEFAULT '[]'::jsonb,
    schedulers JSONB DEFAULT '[]'::jsonb,
    app_path VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_bot_app_name UNIQUE (bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_generated_apps_bot_id ON generated_apps(bot_id);
CREATE INDEX IF NOT EXISTS idx_generated_apps_name ON generated_apps(name);
CREATE INDEX IF NOT EXISTS idx_generated_apps_is_active ON generated_apps(is_active);

-- ============================================================================
-- INTENT CLASSIFICATIONS TABLE
-- ============================================================================
-- Stores classified intents for analytics and learning

CREATE TABLE IF NOT EXISTS intent_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    original_text TEXT NOT NULL,
    intent_type VARCHAR(50) NOT NULL,
    confidence FLOAT NOT NULL DEFAULT 0.0,
    entities JSONB DEFAULT '{}'::jsonb,
    suggested_name VARCHAR(255),
    was_correct BOOLEAN,
    corrected_type VARCHAR(50),
    feedback TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_intent_type CHECK (intent_type IN ('APP_CREATE', 'TODO', 'MONITOR', 'ACTION', 'SCHEDULE', 'GOAL', 'TOOL', 'UNKNOWN'))
);

CREATE INDEX IF NOT EXISTS idx_intent_classifications_bot_id ON intent_classifications(bot_id);
CREATE INDEX IF NOT EXISTS idx_intent_classifications_intent_type ON intent_classifications(intent_type);
CREATE INDEX IF NOT EXISTS idx_intent_classifications_created_at ON intent_classifications(created_at);

-- ============================================================================
-- DESIGNER CHANGES TABLE
-- ============================================================================
-- Stores change history for Designer AI undo support

CREATE TABLE IF NOT EXISTS designer_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    change_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    original_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_designer_change_type CHECK (change_type IN ('STYLE', 'HTML', 'DATABASE', 'TOOL', 'SCHEDULER', 'MULTIPLE', 'UNKNOWN'))
);

CREATE INDEX IF NOT EXISTS idx_designer_changes_bot_id ON designer_changes(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_changes_created_at ON designer_changes(created_at);

-- ============================================================================
-- DESIGNER PENDING CHANGES TABLE
-- ============================================================================
-- Stores pending changes awaiting confirmation

CREATE TABLE IF NOT EXISTS designer_pending_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id UUID REFERENCES user_sessions(id) ON DELETE SET NULL,
    analysis_json TEXT NOT NULL,
    instruction TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_designer_pending_changes_bot_id ON designer_pending_changes(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_pending_changes_expires_at ON designer_pending_changes(expires_at);
-- Migration: 6.1.2_table_role_access
-- Add role-based access control columns to dynamic table definitions and fields
--
-- Syntax in .gbdialog TABLE definitions:
--   TABLE Contatos ON maria READ BY "admin;manager"
--       Id number key
--       Nome string(150)
--       NumeroDocumento string(25) READ BY "admin"
--       Celular string(20) WRITE BY "admin;manager"
--
-- Empty roles = everyone has access (default behavior)
-- Roles are semicolon-separated and match Zitadel directory roles

-- Add role columns to dynamic_table_definitions
ALTER TABLE dynamic_table_definitions
ADD COLUMN IF NOT EXISTS read_roles TEXT DEFAULT NULL,
ADD COLUMN IF NOT EXISTS write_roles TEXT DEFAULT NULL;

-- Add role columns to dynamic_table_fields
ALTER TABLE dynamic_table_fields
ADD COLUMN IF NOT EXISTS read_roles TEXT DEFAULT NULL,
ADD COLUMN IF NOT EXISTS write_roles TEXT DEFAULT NULL;

-- Add comments for documentation
COMMENT ON COLUMN dynamic_table_definitions.read_roles IS 'Semicolon-separated roles that can read from this table (empty = everyone)';
COMMENT ON COLUMN dynamic_table_definitions.write_roles IS 'Semicolon-separated roles that can write to this table (empty = everyone)';
COMMENT ON COLUMN dynamic_table_fields.read_roles IS 'Semicolon-separated roles that can read this field (empty = everyone)';
COMMENT ON COLUMN dynamic_table_fields.write_roles IS 'Semicolon-separated roles that can write this field (empty = everyone)';
-- Migration: Knowledge Base Sources
-- Description: Tables for document ingestion, chunking, and RAG support
-- Note: Vector embeddings are stored in Qdrant, not PostgreSQL

-- Drop existing tables for clean state
DROP TABLE IF EXISTS research_search_history CASCADE;
DROP TABLE IF EXISTS knowledge_chunks CASCADE;
DROP TABLE IF EXISTS knowledge_sources CASCADE;

-- Table for knowledge sources (uploaded documents)
CREATE TABLE IF NOT EXISTS knowledge_sources (
    id TEXT PRIMARY KEY,
    bot_id UUID,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'txt',
    file_path TEXT,
    url TEXT,
    content_hash TEXT NOT NULL,
    chunk_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    collection TEXT NOT NULL DEFAULT 'default',
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMPTZ
);

-- Indexes for knowledge_sources
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_bot_id ON knowledge_sources(bot_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_status ON knowledge_sources(status);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_collection ON knowledge_sources(collection);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_content_hash ON knowledge_sources(content_hash);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_created_at ON knowledge_sources(created_at);

-- Table for document chunks (text only - vectors stored in Qdrant)
CREATE TABLE IF NOT EXISTS knowledge_chunks (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES knowledge_sources(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for knowledge_chunks
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_source_id ON knowledge_chunks(source_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_chunk_index ON knowledge_chunks(chunk_index);

-- Full-text search index on content
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_content_fts
    ON knowledge_chunks USING gin(to_tsvector('english', content));

-- Table for search history
CREATE TABLE IF NOT EXISTS research_search_history (
    id TEXT PRIMARY KEY,
    bot_id UUID,
    user_id UUID,
    query TEXT NOT NULL,
    search_type TEXT NOT NULL DEFAULT 'web',
    results_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for search history
CREATE INDEX IF NOT EXISTS idx_research_search_history_bot_id ON research_search_history(bot_id);
CREATE INDEX IF NOT EXISTS idx_research_search_history_user_id ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_search_history_created_at ON research_search_history(created_at);

-- Trigger for updated_at on knowledge_sources
DROP TRIGGER IF EXISTS update_knowledge_sources_updated_at ON knowledge_sources;
CREATE TRIGGER update_knowledge_sources_updated_at
    BEFORE UPDATE ON knowledge_sources
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE knowledge_sources IS 'Uploaded documents for knowledge base ingestion';
COMMENT ON TABLE knowledge_chunks IS 'Text chunks extracted from knowledge sources - vectors stored in Qdrant';
COMMENT ON TABLE research_search_history IS 'History of web and knowledge base searches';

COMMENT ON COLUMN knowledge_sources.source_type IS 'Document type: pdf, docx, txt, markdown, html, csv, xlsx, url';
COMMENT ON COLUMN knowledge_sources.status IS 'Processing status: pending, processing, indexed, failed, reindexing';
COMMENT ON COLUMN knowledge_sources.collection IS 'Collection/namespace for organizing sources';
COMMENT ON COLUMN knowledge_chunks.token_count IS 'Estimated token count for the chunk';
