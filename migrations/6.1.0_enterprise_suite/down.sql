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
-- ROLLBACK: Original 6.1.0 Enterprise Features
-- ============================================================================

-- Drop test support tables
DROP TABLE IF EXISTS test_execution_logs;
DROP TABLE IF EXISTS test_accounts;

-- Drop calendar tables
DROP TABLE IF EXISTS calendar_shares;
DROP TABLE IF EXISTS calendar_resource_bookings;
DROP TABLE IF EXISTS calendar_resources;

-- Drop task tables (order matters due to foreign keys)
DROP TABLE IF EXISTS task_recurrence;
DROP TABLE IF EXISTS task_time_entries;
DROP TABLE IF EXISTS task_dependencies;
DROP TABLE IF EXISTS tasks;

-- Drop collaboration tables
DROP TABLE IF EXISTS document_presence;

-- Drop drive tables
DROP TABLE IF EXISTS storage_quotas;
DROP TABLE IF EXISTS file_sync_status;
DROP TABLE IF EXISTS file_trash;
DROP TABLE IF EXISTS file_activities;
DROP TABLE IF EXISTS file_shares;
DROP TABLE IF EXISTS file_comments;
DROP TABLE IF EXISTS file_versions;

-- Drop meet tables
DROP TABLE IF EXISTS user_virtual_backgrounds;
DROP TABLE IF EXISTS meeting_captions;
DROP TABLE IF EXISTS meeting_waiting_room;
DROP TABLE IF EXISTS meeting_questions;
DROP TABLE IF EXISTS meeting_polls;
DROP TABLE IF EXISTS meeting_breakout_rooms;
DROP TABLE IF EXISTS meeting_recordings;

-- Drop email tables (order matters due to foreign keys)
DROP TABLE IF EXISTS shared_mailbox_members;
DROP TABLE IF EXISTS shared_mailboxes;
DROP TABLE IF EXISTS distribution_lists;
DROP TABLE IF EXISTS email_label_assignments;
DROP TABLE IF EXISTS email_labels;
DROP TABLE IF EXISTS email_rules;
DROP TABLE IF EXISTS email_auto_responders;
DROP TABLE IF EXISTS email_templates;
DROP TABLE IF EXISTS scheduled_emails;
DROP TABLE IF EXISTS email_signatures;
DROP TABLE IF EXISTS global_email_signatures;
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
