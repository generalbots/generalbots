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
