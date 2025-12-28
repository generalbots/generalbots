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
