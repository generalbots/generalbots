-- ============================================================================
-- GENERAL BOTS - CONSOLIDATED SCHEMA v7.0.0 - ROLLBACK
-- ============================================================================
-- WARNING: This is a DESTRUCTIVE operation - all data will be lost
-- ============================================================================

-- Drop tables in reverse dependency order

-- Organizations
DROP TABLE IF EXISTS user_organizations CASCADE;
DROP TABLE IF EXISTS organizations CASCADE;

-- Context
DROP TABLE IF EXISTS context_injections CASCADE;

-- Table access control
DROP TABLE IF EXISTS table_role_access CASCADE;

-- Communication
DROP TABLE IF EXISTS clicks CASCADE;
DROP TABLE IF EXISTS whatsapp_numbers CASCADE;

-- Connected accounts
DROP TABLE IF EXISTS session_account_associations CASCADE;
DROP TABLE IF EXISTS connected_accounts CASCADE;

-- Tasks
DROP TABLE IF EXISTS task_comments CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;

-- Analytics
DROP TABLE IF EXISTS analytics_events CASCADE;
DROP TABLE IF EXISTS usage_analytics CASCADE;

-- Tools and automation
DROP TABLE IF EXISTS pending_info CASCADE;
DROP TABLE IF EXISTS system_automations CASCADE;
DROP TABLE IF EXISTS tools CASCADE;

-- Knowledge base
DROP TABLE IF EXISTS kb_sources CASCADE;
DROP TABLE IF EXISTS session_kb_associations CASCADE;
DROP TABLE IF EXISTS kb_documents CASCADE;
DROP TABLE IF EXISTS kb_collections CASCADE;

-- App generation
DROP TABLE IF EXISTS designer_pending_changes CASCADE;
DROP TABLE IF EXISTS designer_changes CASCADE;
DROP TABLE IF EXISTS generated_apps CASCADE;

-- Intent and classification
DROP TABLE IF EXISTS intent_classifications CASCADE;

-- Safety
DROP TABLE IF EXISTS safety_audit_log CASCADE;

-- Task decisions and approvals
DROP TABLE IF EXISTS task_decisions CASCADE;
DROP TABLE IF EXISTS task_approvals CASCADE;

-- Execution plans and auto tasks
DROP TABLE IF EXISTS execution_plans CASCADE;
DROP TABLE IF EXISTS auto_tasks CASCADE;

-- Memory
DROP TABLE IF EXISTS bot_memories CASCADE;

-- Messages and sessions
DROP TABLE IF EXISTS message_history CASCADE;
DROP TABLE IF EXISTS user_sessions CASCADE;

-- Bot configuration
DROP TABLE IF EXISTS bot_channels CASCADE;
DROP TABLE IF EXISTS bot_configuration CASCADE;
DROP TABLE IF EXISTS bots CASCADE;

-- Users
DROP TABLE IF EXISTS users CASCADE;

-- Tenants and sharding
DROP TABLE IF EXISTS tenant_shard_map CASCADE;
DROP TABLE IF EXISTS tenants CASCADE;
DROP TABLE IF EXISTS shard_config CASCADE;

-- Sequences
DROP SEQUENCE IF EXISTS global_id_seq;

-- Note: Diesel helper functions are kept (managed by 00000000000000_diesel_initial_setup)
