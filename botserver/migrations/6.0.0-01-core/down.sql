DROP TABLE public.usage_analytics;
DROP TABLE public.message_history;
DROP TABLE public.context_injections;
DROP TABLE public.whatsapp_numbers;
DROP TABLE public.user_sessions;
DROP TABLE public.bot_channels;
DROP TABLE public.users;
DROP TABLE public.tools;
DROP TABLE public.system_automations;
DROP TABLE public.organizations;
DROP TABLE public.clicks;
DROP TABLE public.bots;
DROP INDEX idx_bot_memories_key;
DROP INDEX idx_bot_memories_bot_id;
DROP TABLE bot_memories;
-- Drop triggers
DROP TRIGGER IF EXISTS update_basic_tools_updated_at ON basic_tools;
DROP TRIGGER IF EXISTS update_kb_collections_updated_at ON kb_collections;
DROP TRIGGER IF EXISTS update_kb_documents_updated_at ON kb_documents;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column;

-- Drop indexes
DROP INDEX IF EXISTS idx_basic_tools_active;
DROP INDEX IF EXISTS idx_basic_tools_name;
DROP INDEX IF EXISTS idx_basic_tools_bot_id;
DROP INDEX IF EXISTS idx_kb_collections_name;
DROP INDEX IF EXISTS idx_kb_collections_bot_id;
DROP INDEX IF EXISTS idx_kb_documents_indexed_at;
DROP INDEX IF EXISTS idx_kb_documents_hash;
DROP INDEX IF EXISTS idx_kb_documents_collection;
DROP INDEX IF EXISTS idx_kb_documents_bot_id;

-- Drop tables
DROP TABLE IF EXISTS basic_tools;
DROP TABLE IF EXISTS kb_collections;
DROP TABLE IF EXISTS kb_documents;
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
-- Drop indexes first
DROP INDEX IF EXISTS idx_gbot_sync_bot;
DROP INDEX IF EXISTS idx_component_logs_created;
DROP INDEX IF EXISTS idx_component_logs_level;
DROP INDEX IF EXISTS idx_component_logs_component;
DROP INDEX IF EXISTS idx_component_status;
DROP INDEX IF EXISTS idx_component_name;
DROP INDEX IF EXISTS idx_connection_config_active;
DROP INDEX IF EXISTS idx_connection_config_name;
DROP INDEX IF EXISTS idx_connection_config_bot;
DROP INDEX IF EXISTS idx_model_config_default;
DROP INDEX IF EXISTS idx_model_config_active;
DROP INDEX IF EXISTS idx_model_config_type;
DROP INDEX IF EXISTS idx_bot_config_key;
DROP INDEX IF EXISTS idx_bot_config_bot;
DROP INDEX IF EXISTS idx_tenant_config_key;
DROP INDEX IF EXISTS idx_tenant_config_tenant;
DROP INDEX IF EXISTS idx_server_config_type;
DROP INDEX IF EXISTS idx_server_config_key;

-- Drop tables
DROP TABLE IF EXISTS gbot_config_sync;
DROP TABLE IF EXISTS component_logs;
DROP TABLE IF EXISTS component_installations;
DROP TABLE IF EXISTS connection_configurations;
DROP TABLE IF EXISTS model_configurations;
DROP TABLE IF EXISTS bot_configuration;
DROP TABLE IF EXISTS tenant_configuration;
DROP TABLE IF EXISTS server_configuration;

-- Remove added columns if they exist
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_sessions' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE user_sessions DROP COLUMN tenant_id;
    END IF;
    
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'bots' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE bots DROP COLUMN tenant_id;
    END IF;
END $$;

-- Drop tenant indexes if they exist
DROP INDEX IF EXISTS idx_user_sessions_tenant;
DROP INDEX IF EXISTS idx_bots_tenant;

-- Remove default tenant
DELETE FROM tenants WHERE slug = 'default';
-- Revert clicks table changes
CREATE TABLE IF NOT EXISTS public.old_clicks (
    campaign_id text NOT NULL,
    email text NOT NULL,
    updated_at timestamptz DEFAULT now() NULL,
    CONSTRAINT clicks_campaign_id_email_key UNIQUE (campaign_id, email)
);

INSERT INTO public.old_clicks (campaign_id, email, updated_at)
SELECT campaign_id, email, updated_at FROM public.clicks;

DROP TABLE public.clicks;
ALTER TABLE public.old_clicks RENAME TO clicks;

-- Remove system_automations constraints and indexes
DROP INDEX IF EXISTS idx_system_automations_bot_kind_param;
ALTER TABLE public.system_automations DROP CONSTRAINT IF EXISTS system_automations_bot_kind_param_unique;

DROP INDEX IF EXISTS idx_system_automations_bot_id;
ALTER TABLE public.system_automations DROP COLUMN IF EXISTS bot_id;

DROP INDEX IF EXISTS idx_system_automations_name;
ALTER TABLE public.system_automations DROP COLUMN IF EXISTS name;

-- Remove bot_configuration constraint
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_config_key_unique;
-- Drop login tokens table
DROP TABLE IF EXISTS public.user_login_tokens;

-- Drop user preferences table
DROP TABLE IF EXISTS public.user_preferences;

-- Remove session enhancement
ALTER TABLE public.user_sessions
DROP CONSTRAINT IF EXISTS user_sessions_email_account_id_fkey,
DROP COLUMN IF EXISTS active_email_account_id;

-- Drop email folders table
DROP TABLE IF EXISTS public.email_folders;

-- Drop email drafts table
DROP TABLE IF EXISTS public.email_drafts;

-- Drop user email accounts table
DROP TABLE IF EXISTS public.user_email_accounts;
-- Drop triggers
DROP TRIGGER IF EXISTS update_directory_users_updated_at ON public.directory_users;
DROP TRIGGER IF EXISTS update_oauth_applications_updated_at ON public.oauth_applications;

-- Drop function if no other triggers use it
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS public.bot_access CASCADE;
DROP TABLE IF EXISTS public.oauth_applications CASCADE;
DROP TABLE IF EXISTS public.directory_users CASCADE;

-- Drop indexes
DROP INDEX IF EXISTS idx_bots_org_id;

-- Remove columns from bots table
ALTER TABLE public.bots
DROP CONSTRAINT IF EXISTS bots_org_id_fkey,
DROP COLUMN IF EXISTS org_id,
DROP COLUMN IF EXISTS is_default;

-- Note: We don't delete the default organization or bot data as they may have other relationships
-- The application should handle orphaned data appropriately
-- Drop session_website_associations table and related indexes
DROP TABLE IF EXISTS session_website_associations;

-- Drop website_crawls table and related objects
DROP TRIGGER IF EXISTS website_crawls_updated_at_trigger ON website_crawls;
DROP FUNCTION IF EXISTS update_website_crawls_updated_at();
DROP TABLE IF EXISTS website_crawls;
-- Rollback Migration: 6.1.0 Enterprise Features
-- WARNING: This will delete all enterprise feature data!
-- NOTE: TABLES AND INDEXES ONLY - No views, triggers, or functions per project standards
-- Includes rollback for: config ID fixes, connected accounts, bot hierarchy, monitors

-- ============================================================================
-- ROLLBACK: Bot Hierarchy and Monitors (from 6.1.3)
-- ============================================================================

-- Drop comments first
COMMENT ON TABLE public.user_organizations IS NULL;
COMMENT ON TABLE public.email_received_events IS NULL;
COMMENT ON TABLE public.folder_change_events IS NULL;
COMMENT ON TABLE public.folder_monitors IS NULL;
COMMENT ON TABLE public.email_monitors IS NULL;
COMMENT ON COLUMN public.bots.inherit_parent_config IS NULL;
COMMENT ON COLUMN public.bots.enabled_tabs_json IS NULL;
COMMENT ON COLUMN public.bots.parent_bot_id IS NULL;
COMMENT ON TABLE public.system_automations IS NULL;

-- Drop user organizations table
DROP INDEX IF EXISTS idx_user_orgs_default;
DROP INDEX IF EXISTS idx_user_orgs_org;
DROP INDEX IF EXISTS idx_user_orgs_user;
DROP TABLE IF EXISTS public.user_organizations;

-- Drop email received events table
DROP INDEX IF EXISTS idx_email_events_received;
DROP INDEX IF EXISTS idx_email_events_processed;
DROP INDEX IF EXISTS idx_email_events_monitor;
DROP TABLE IF EXISTS public.email_received_events;

-- Drop folder change events table
DROP INDEX IF EXISTS idx_folder_events_created;
DROP INDEX IF EXISTS idx_folder_events_processed;
DROP INDEX IF EXISTS idx_folder_events_monitor;
DROP TABLE IF EXISTS public.folder_change_events;

-- Drop folder monitors table
DROP INDEX IF EXISTS idx_folder_monitors_account_email;
DROP INDEX IF EXISTS idx_folder_monitors_active;
DROP INDEX IF EXISTS idx_folder_monitors_provider;
DROP INDEX IF EXISTS idx_folder_monitors_bot_id;
DROP TABLE IF EXISTS public.folder_monitors;

-- Drop email monitors table
DROP INDEX IF EXISTS idx_email_monitors_active;
DROP INDEX IF EXISTS idx_email_monitors_email;
DROP INDEX IF EXISTS idx_email_monitors_bot_id;
DROP TABLE IF EXISTS public.email_monitors;

-- Remove bot hierarchy columns
DROP INDEX IF EXISTS idx_bots_parent_bot_id;
ALTER TABLE public.bots DROP COLUMN IF EXISTS inherit_parent_config;
ALTER TABLE public.bots DROP COLUMN IF EXISTS enabled_tabs_json;
ALTER TABLE public.bots DROP COLUMN IF EXISTS parent_bot_id;

-- ============================================================================
-- ROLLBACK: Connected Accounts (from 6.1.2)
-- ============================================================================

DROP INDEX IF EXISTS idx_account_sync_items_unique;
DROP INDEX IF EXISTS idx_account_sync_items_embedding;
DROP INDEX IF EXISTS idx_account_sync_items_date;
DROP INDEX IF EXISTS idx_account_sync_items_type;
DROP INDEX IF EXISTS idx_account_sync_items_account;
DROP TABLE IF EXISTS account_sync_items;

DROP INDEX IF EXISTS idx_session_account_assoc_unique;
DROP INDEX IF EXISTS idx_session_account_assoc_active;
DROP INDEX IF EXISTS idx_session_account_assoc_account;
DROP INDEX IF EXISTS idx_session_account_assoc_session;
DROP TABLE IF EXISTS session_account_associations;

DROP INDEX IF EXISTS idx_connected_accounts_bot_email;
DROP INDEX IF EXISTS idx_connected_accounts_status;
DROP INDEX IF EXISTS idx_connected_accounts_provider;
DROP INDEX IF EXISTS idx_connected_accounts_email;
DROP INDEX IF EXISTS idx_connected_accounts_user_id;
DROP INDEX IF EXISTS idx_connected_accounts_bot_id;
DROP TABLE IF EXISTS connected_accounts;

-- ============================================================================
-- ROLLBACK: Config ID Type Fixes (from 6.1.1)
-- Revert UUID columns back to TEXT
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'bot_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE bot_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'server_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE server_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'tenant_configuration'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE tenant_configuration
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'model_configurations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE model_configurations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'connection_configurations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE connection_configurations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_installations'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE component_installations
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_logs'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE component_logs
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'gbot_config_sync'
               AND column_name = 'id'
               AND data_type = 'uuid') THEN
        ALTER TABLE gbot_config_sync
            ALTER COLUMN id TYPE TEXT USING id::text;
    END IF;
END $$;

-- ============================================================================
-- FEATURE TABLES MOVED TO DEDICATED MIGRATIONS
-- ============================================================================

-- Drop triggers and functions
DROP TRIGGER IF EXISTS external_connections_updated_at_trigger ON external_connections;
DROP FUNCTION IF EXISTS update_external_connections_updated_at();

DROP TRIGGER IF EXISTS dynamic_table_definitions_updated_at_trigger ON dynamic_table_definitions;
DROP FUNCTION IF EXISTS update_dynamic_table_definitions_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_external_connections_name;
DROP INDEX IF EXISTS idx_external_connections_bot_id;

DROP INDEX IF EXISTS idx_dynamic_table_fields_name;
DROP INDEX IF EXISTS idx_dynamic_table_fields_table_id;

DROP INDEX IF EXISTS idx_dynamic_table_definitions_connection;
DROP INDEX IF EXISTS idx_dynamic_table_definitions_name;
DROP INDEX IF EXISTS idx_dynamic_table_definitions_bot_id;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS external_connections;
DROP TABLE IF EXISTS dynamic_table_fields;
DROP TABLE IF EXISTS dynamic_table_definitions;
-- Rollback Migration: 6.1.1 AutoTask System
-- Description: Drop tables for the AutoTask system

-- Drop indexes first (automatically dropped with tables, but explicit for clarity)

-- Drop designer_pending_changes
DROP INDEX IF EXISTS idx_designer_pending_changes_expires_at;
DROP INDEX IF EXISTS idx_designer_pending_changes_bot_id;
DROP TABLE IF EXISTS designer_pending_changes;

-- Drop designer_changes
DROP INDEX IF EXISTS idx_designer_changes_created_at;
DROP INDEX IF EXISTS idx_designer_changes_bot_id;
DROP TABLE IF EXISTS designer_changes;

-- Drop intent_classifications
DROP INDEX IF EXISTS idx_intent_classifications_created_at;
DROP INDEX IF EXISTS idx_intent_classifications_intent_type;
DROP INDEX IF EXISTS idx_intent_classifications_bot_id;
DROP TABLE IF EXISTS intent_classifications;

-- Drop generated_apps
DROP INDEX IF EXISTS idx_generated_apps_is_active;
DROP INDEX IF EXISTS idx_generated_apps_name;
DROP INDEX IF EXISTS idx_generated_apps_bot_id;
DROP TABLE IF EXISTS generated_apps;

-- Drop safety_audit_log
DROP INDEX IF EXISTS idx_safety_audit_log_created_at;
DROP INDEX IF EXISTS idx_safety_audit_log_outcome;
DROP INDEX IF EXISTS idx_safety_audit_log_task_id;
DROP INDEX IF EXISTS idx_safety_audit_log_bot_id;
DROP TABLE IF EXISTS safety_audit_log;

-- Drop task_decisions
DROP INDEX IF EXISTS idx_task_decisions_status;
DROP INDEX IF EXISTS idx_task_decisions_task_id;
DROP INDEX IF EXISTS idx_task_decisions_bot_id;
DROP TABLE IF EXISTS task_decisions;

-- Drop task_approvals
DROP INDEX IF EXISTS idx_task_approvals_expires_at;
DROP INDEX IF EXISTS idx_task_approvals_status;
DROP INDEX IF EXISTS idx_task_approvals_task_id;
DROP INDEX IF EXISTS idx_task_approvals_bot_id;
DROP TABLE IF EXISTS task_approvals;

-- Drop execution_plans
DROP INDEX IF EXISTS idx_execution_plans_intent_type;
DROP INDEX IF EXISTS idx_execution_plans_status;
DROP INDEX IF EXISTS idx_execution_plans_task_id;
DROP INDEX IF EXISTS idx_execution_plans_bot_id;
DROP TABLE IF EXISTS execution_plans;

-- Drop auto_tasks
DROP INDEX IF EXISTS idx_auto_tasks_created_at;
DROP INDEX IF EXISTS idx_auto_tasks_priority;
DROP INDEX IF EXISTS idx_auto_tasks_status;
DROP INDEX IF EXISTS idx_auto_tasks_session_id;
DROP INDEX IF EXISTS idx_auto_tasks_bot_id;
DROP TABLE IF EXISTS auto_tasks;

-- Drop pending_info
DROP INDEX IF EXISTS idx_pending_info_is_filled;
DROP INDEX IF EXISTS idx_pending_info_config_key;
DROP INDEX IF EXISTS idx_pending_info_bot_id;
DROP TABLE IF EXISTS pending_info;
-- Rollback: Remove role-based access control columns from dynamic tables
-- Migration: 6.1.2_table_role_access

-- Remove columns from dynamic_table_definitions
ALTER TABLE dynamic_table_definitions
    DROP COLUMN IF EXISTS read_roles,
    DROP COLUMN IF EXISTS write_roles;

-- Remove columns from dynamic_table_fields
ALTER TABLE dynamic_table_fields
    DROP COLUMN IF EXISTS read_roles,
    DROP COLUMN IF EXISTS write_roles;
-- Rollback Migration: Knowledge Base Sources

-- Drop triggers first
DROP TRIGGER IF EXISTS update_knowledge_sources_updated_at ON knowledge_sources;

-- Drop indexes
DROP INDEX IF EXISTS idx_knowledge_sources_bot_id;
DROP INDEX IF EXISTS idx_knowledge_sources_status;
DROP INDEX IF EXISTS idx_knowledge_sources_collection;
DROP INDEX IF EXISTS idx_knowledge_sources_content_hash;
DROP INDEX IF EXISTS idx_knowledge_sources_created_at;

DROP INDEX IF EXISTS idx_knowledge_chunks_source_id;
DROP INDEX IF EXISTS idx_knowledge_chunks_chunk_index;
DROP INDEX IF EXISTS idx_knowledge_chunks_content_fts;
DROP INDEX IF EXISTS idx_knowledge_chunks_embedding;

DROP INDEX IF EXISTS idx_research_search_history_bot_id;
DROP INDEX IF EXISTS idx_research_search_history_user_id;
DROP INDEX IF EXISTS idx_research_search_history_created_at;

-- Drop tables (order matters due to foreign key constraints)
DROP TABLE IF EXISTS research_search_history;
DROP TABLE IF EXISTS knowledge_chunks;
DROP TABLE IF EXISTS knowledge_sources;
