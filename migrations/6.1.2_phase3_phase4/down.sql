-- Migration Rollback: 6.1.2_phase3_phase4
-- Description: Rollback Phase 3 and Phase 4 multi-agent features
-- WARNING: This will delete all data in the affected tables!

-- ============================================
-- DROP VIEWS
-- ============================================

DROP VIEW IF EXISTS v_llm_usage_24h;
DROP VIEW IF EXISTS v_approval_summary;
DROP VIEW IF EXISTS v_kg_stats;
DROP VIEW IF EXISTS v_recent_episodes;

-- ============================================
-- DROP FUNCTIONS
-- ============================================

DROP FUNCTION IF EXISTS cleanup_old_observability_data(INTEGER);
DROP FUNCTION IF EXISTS reset_monthly_budgets();
DROP FUNCTION IF EXISTS reset_daily_budgets();
DROP FUNCTION IF EXISTS aggregate_llm_metrics_hourly();

-- ============================================
-- DROP TRIGGERS
-- ============================================

DROP TRIGGER IF EXISTS update_llm_budget_updated_at ON llm_budget;
DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow_definitions;
DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;

-- Note: We don't drop the update_updated_at_column() function as it may be used by other tables

-- ============================================
-- DROP WORKFLOW TABLES
-- ============================================

DROP TABLE IF EXISTS workflow_step_executions CASCADE;
DROP TABLE IF EXISTS workflow_executions CASCADE;
DROP TABLE IF EXISTS workflow_definitions CASCADE;

-- ============================================
-- DROP LLM OBSERVABILITY TABLES
-- ============================================

DROP TABLE IF EXISTS llm_traces CASCADE;
DROP TABLE IF EXISTS llm_budget CASCADE;
DROP TABLE IF EXISTS llm_metrics_hourly CASCADE;
DROP TABLE IF EXISTS llm_metrics CASCADE;

-- ============================================
-- DROP APPROVAL TABLES
-- ============================================

DROP TABLE IF EXISTS approval_tokens CASCADE;
DROP TABLE IF EXISTS approval_audit_log CASCADE;
DROP TABLE IF EXISTS approval_chains CASCADE;
DROP TABLE IF EXISTS approval_requests CASCADE;

-- ============================================
-- DROP KNOWLEDGE GRAPH TABLES
-- ============================================

DROP TABLE IF EXISTS kg_relationships CASCADE;
DROP TABLE IF EXISTS kg_entities CASCADE;

-- ============================================
-- DROP EPISODIC MEMORY TABLES
-- ============================================

DROP TABLE IF EXISTS conversation_episodes CASCADE;

-- ============================================
-- DROP INDEXES (if any remain)
-- ============================================

-- Episodic memory indexes
DROP INDEX IF EXISTS idx_episodes_user_id;
DROP INDEX IF EXISTS idx_episodes_bot_id;
DROP INDEX IF EXISTS idx_episodes_session_id;
DROP INDEX IF EXISTS idx_episodes_created_at;
DROP INDEX IF EXISTS idx_episodes_key_topics;
DROP INDEX IF EXISTS idx_episodes_resolution;
DROP INDEX IF EXISTS idx_episodes_summary_fts;

-- Knowledge graph indexes
DROP INDEX IF EXISTS idx_kg_entities_bot_id;
DROP INDEX IF EXISTS idx_kg_entities_type;
DROP INDEX IF EXISTS idx_kg_entities_name;
DROP INDEX IF EXISTS idx_kg_entities_name_lower;
DROP INDEX IF EXISTS idx_kg_entities_aliases;
DROP INDEX IF EXISTS idx_kg_entities_name_fts;
DROP INDEX IF EXISTS idx_kg_relationships_bot_id;
DROP INDEX IF EXISTS idx_kg_relationships_from;
DROP INDEX IF EXISTS idx_kg_relationships_to;
DROP INDEX IF EXISTS idx_kg_relationships_type;

-- Approval indexes
DROP INDEX IF EXISTS idx_approval_requests_bot_id;
DROP INDEX IF EXISTS idx_approval_requests_session_id;
DROP INDEX IF EXISTS idx_approval_requests_status;
DROP INDEX IF EXISTS idx_approval_requests_expires_at;
DROP INDEX IF EXISTS idx_approval_requests_pending;
DROP INDEX IF EXISTS idx_approval_audit_request_id;
DROP INDEX IF EXISTS idx_approval_audit_timestamp;
DROP INDEX IF EXISTS idx_approval_tokens_token;
DROP INDEX IF EXISTS idx_approval_tokens_request_id;

-- Observability indexes
DROP INDEX IF EXISTS idx_llm_metrics_bot_id;
DROP INDEX IF EXISTS idx_llm_metrics_session_id;
DROP INDEX IF EXISTS idx_llm_metrics_timestamp;
DROP INDEX IF EXISTS idx_llm_metrics_model;
DROP INDEX IF EXISTS idx_llm_metrics_hourly_bot_id;
DROP INDEX IF EXISTS idx_llm_metrics_hourly_hour;
DROP INDEX IF EXISTS idx_llm_traces_trace_id;
DROP INDEX IF EXISTS idx_llm_traces_start_time;
DROP INDEX IF EXISTS idx_llm_traces_component;

-- Workflow indexes
DROP INDEX IF EXISTS idx_workflow_definitions_bot_id;
DROP INDEX IF EXISTS idx_workflow_executions_workflow_id;
DROP INDEX IF EXISTS idx_workflow_executions_bot_id;
DROP INDEX IF EXISTS idx_workflow_executions_status;
DROP INDEX IF EXISTS idx_workflow_step_executions_execution_id;
