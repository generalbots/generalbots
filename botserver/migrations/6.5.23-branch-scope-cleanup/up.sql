-- ════════════════════════════════════════════════════════════
-- Migration: 9.23-branch-scope-cleanup (up)
-- Adds branch_id as the sole scope column to all business tables.
-- Pattern: ADD COLUMN IF NOT EXISTS → UPDATE NULLs → SET NOT NULL → CREATE INDEX
-- ════════════════════════════════════════════════════════════

-- Ensure the default fallback branch exists (used by backfill UPDATEs below).
-- 9.15-org-branches creates branch '00000000-0000-0000-0000-000000000001';
-- this migration uses '00000000-0000-0000-0000-000000000000' as fallback.
INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at)
SELECT '00000000-0000-0000-0000-000000000000', org_id, tenant_id, '__default_fallback', '__default_fallback', NOW()
FROM branches
WHERE slug = 'default'
LIMIT 1
ON CONFLICT (id) DO NOTHING;

-- ── Bot Configuration (special — backfill from bots table) ──────────────

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'bot_configuration') THEN
ALTER TABLE bot_configuration ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;

-- Backfill branch_id from bots table
UPDATE bot_configuration bc
SET branch_id = b.branch_id
FROM bots b
WHERE bc.bot_id = b.id AND bc.branch_id IS NULL;

-- Default branch for any remaining NULLs
UPDATE bot_configuration
SET branch_id = '00000000-0000-0000-0000-000000000000'
WHERE branch_id IS NULL;

ALTER TABLE bot_configuration ALTER COLUMN branch_id SET NOT NULL;

-- Drop old unique constraint, create scoped one
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_bot_id_config_key_key;
ALTER TABLE bot_configuration DROP CONSTRAINT IF EXISTS bot_configuration_pkey CASCADE;
ALTER TABLE bot_configuration ADD PRIMARY KEY (id);
ALTER TABLE bot_configuration ADD UNIQUE (branch_id, bot_id, config_key);

-- Remove deprecated columns
ALTER TABLE bot_configuration DROP COLUMN IF EXISTS config_type;
ALTER TABLE bot_configuration DROP COLUMN IF EXISTS is_encrypted;

CREATE INDEX IF NOT EXISTS idx_bot_configuration_branch_id ON bot_configuration(branch_id);

-- ── AI Workspace ────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'aiworkspace_templates') THEN
ALTER TABLE aiworkspace_templates ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE aiworkspace_templates SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE aiworkspace_templates ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_aiworkspace_templates_branch_id ON aiworkspace_templates(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'aiworkspaces') THEN
ALTER TABLE aiworkspaces ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE aiworkspaces SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE aiworkspaces ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_aiworkspaces_branch_id ON aiworkspaces(branch_id);

-- ── Attendance ──────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'attendance_sla_policies') THEN
ALTER TABLE attendance_sla_policies ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE attendance_sla_policies SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE attendance_sla_policies ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_attendance_sla_policies_branch_id ON attendance_sla_policies(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'attendance_webhooks') THEN
ALTER TABLE attendance_webhooks ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE attendance_webhooks SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE attendance_webhooks ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_attendance_webhooks_branch_id ON attendance_webhooks(branch_id);

-- ── Auto Tasks / System Automations ─────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'auto_tasks') THEN
ALTER TABLE auto_tasks ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE auto_tasks SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE auto_tasks ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_auto_tasks_branch_id ON auto_tasks(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'system_automations') THEN
ALTER TABLE system_automations ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE system_automations SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE system_automations ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_system_automations_branch_id ON system_automations(branch_id);

-- ── Banking / Financial Transactions ────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'bank_transactions') THEN
ALTER TABLE bank_transactions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE bank_transactions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE bank_transactions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_bank_transactions_branch_id ON bank_transactions(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'delivery_transactions') THEN
ALTER TABLE delivery_transactions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE delivery_transactions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE delivery_transactions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_delivery_transactions_branch_id ON delivery_transactions(branch_id);

-- ── Bot Engine Tables ───────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'basic_tools') THEN
ALTER TABLE basic_tools ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE basic_tools SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE basic_tools ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_basic_tools_branch_id ON basic_tools(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'bot_memories') THEN
ALTER TABLE bot_memories ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE bot_memories SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE bot_memories ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_bot_memories_branch_id ON bot_memories(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'bots') THEN
ALTER TABLE bots ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE bots SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE bots ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_bots_branch_id ON bots(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compiled_intents') THEN
ALTER TABLE compiled_intents ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compiled_intents SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compiled_intents ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compiled_intents_branch_id ON compiled_intents(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'designer_changes') THEN
ALTER TABLE designer_changes ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE designer_changes SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE designer_changes ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_designer_changes_branch_id ON designer_changes(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'intent_classifications') THEN
ALTER TABLE intent_classifications ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE intent_classifications SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE intent_classifications ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_intent_classifications_branch_id ON intent_classifications(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'safety_constraints') THEN
ALTER TABLE safety_constraints ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE safety_constraints SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE safety_constraints ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_safety_constraints_branch_id ON safety_constraints(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'website_crawls') THEN
ALTER TABLE website_crawls ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE website_crawls SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE website_crawls ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_website_crawls_branch_id ON website_crawls(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'workflow_definitions') THEN
ALTER TABLE workflow_definitions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE workflow_definitions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE workflow_definitions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_branch_id ON workflow_definitions(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'workflow_executions') THEN
ALTER TABLE workflow_executions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE workflow_executions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE workflow_executions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_workflow_executions_branch_id ON workflow_executions(branch_id);

-- ── Billing ─────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_alert_history') THEN
ALTER TABLE billing_alert_history ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_alert_history SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_alert_history ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_alert_history_branch_id ON billing_alert_history(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_grace_periods') THEN
ALTER TABLE billing_grace_periods ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_grace_periods SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_grace_periods ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_grace_periods_branch_id ON billing_grace_periods(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_invoices') THEN
ALTER TABLE billing_invoices ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_invoices SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_invoices ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_invoices_branch_id ON billing_invoices(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_notification_preferences') THEN
ALTER TABLE billing_notification_preferences ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_notification_preferences SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_notification_preferences ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_notification_preferences_branch_id ON billing_notification_preferences(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_payments') THEN
ALTER TABLE billing_payments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_payments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_payments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_payments_branch_id ON billing_payments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_quotes') THEN
ALTER TABLE billing_quotes ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_quotes SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_quotes ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_quotes_branch_id ON billing_quotes(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_recurring') THEN
ALTER TABLE billing_recurring ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_recurring SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_recurring ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_recurring_branch_id ON billing_recurring(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_tax_rates') THEN
ALTER TABLE billing_tax_rates ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_tax_rates SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_tax_rates ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_tax_rates_branch_id ON billing_tax_rates(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'billing_usage_alerts') THEN
ALTER TABLE billing_usage_alerts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE billing_usage_alerts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE billing_usage_alerts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_billing_usage_alerts_branch_id ON billing_usage_alerts(branch_id);

-- ── Calendar ────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'calendar_events') THEN
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE calendar_events SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE calendar_events ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_calendar_events_branch_id ON calendar_events(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'calendars') THEN
ALTER TABLE calendars ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE calendars SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE calendars ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_calendars_branch_id ON calendars(branch_id);

-- ── Canvases ────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'canvases') THEN
ALTER TABLE canvases ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE canvases SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE canvases ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_canvases_branch_id ON canvases(branch_id);

-- ── Cloud Workspaces ────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'cloud_workspaces') THEN
ALTER TABLE cloud_workspaces ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE cloud_workspaces SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE cloud_workspaces ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_cloud_workspaces_branch_id ON cloud_workspaces(branch_id);

-- ── Compliance ──────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_access_reviews') THEN
ALTER TABLE compliance_access_reviews ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_access_reviews SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_access_reviews ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_access_reviews_branch_id ON compliance_access_reviews(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_audit_log') THEN
ALTER TABLE compliance_audit_log ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_audit_log SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_audit_log ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_audit_log_branch_id ON compliance_audit_log(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_checks') THEN
ALTER TABLE compliance_checks ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_checks SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_checks ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_checks_branch_id ON compliance_checks(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_evidence') THEN
ALTER TABLE compliance_evidence ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_evidence SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_evidence ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_evidence_branch_id ON compliance_evidence(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_issues') THEN
ALTER TABLE compliance_issues ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_issues SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_issues ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_issues_branch_id ON compliance_issues(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_risk_assessments') THEN
ALTER TABLE compliance_risk_assessments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_risk_assessments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_risk_assessments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_risk_assessments_branch_id ON compliance_risk_assessments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'compliance_training_records') THEN
ALTER TABLE compliance_training_records ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE compliance_training_records SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compliance_training_records ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_training_records_branch_id ON compliance_training_records(branch_id);

-- ── Connectors / ETL ────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'connectors') THEN
ALTER TABLE connectors ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE connectors SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE connectors ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_connectors_branch_id ON connectors(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'etl_jobs') THEN
ALTER TABLE etl_jobs ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE etl_jobs SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE etl_jobs ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_etl_jobs_branch_id ON etl_jobs(branch_id);

-- ── Conversation Analytics ──────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'conversation_analytics') THEN
ALTER TABLE conversation_analytics ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE conversation_analytics SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE conversation_analytics ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversation_analytics_branch_id ON conversation_analytics(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'conversation_metrics') THEN
ALTER TABLE conversation_metrics ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE conversation_metrics SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE conversation_metrics ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversation_metrics_branch_id ON conversation_metrics(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'conversation_ratings') THEN
ALTER TABLE conversation_ratings ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE conversation_ratings SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE conversation_ratings ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversation_ratings_branch_id ON conversation_ratings(branch_id);

-- ── Conversational Queries / Handoff ────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'conversational_queries') THEN
ALTER TABLE conversational_queries ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE conversational_queries SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE conversational_queries ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversational_queries_branch_id ON conversational_queries(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'handoff_contexts') THEN
ALTER TABLE handoff_contexts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE handoff_contexts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE handoff_contexts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_handoff_contexts_branch_id ON handoff_contexts(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'handoff_queue') THEN
ALTER TABLE handoff_queue ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE handoff_queue SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE handoff_queue ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_handoff_queue_branch_id ON handoff_queue(branch_id);

-- ── Cookie / Legal ──────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'cookie_consents') THEN
ALTER TABLE cookie_consents ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE cookie_consents SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE cookie_consents ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_cookie_consents_branch_id ON cookie_consents(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'legal_acceptances') THEN
ALTER TABLE legal_acceptances ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE legal_acceptances SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE legal_acceptances ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_legal_acceptances_branch_id ON legal_acceptances(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'legal_documents') THEN
ALTER TABLE legal_documents ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE legal_documents SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE legal_documents ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_legal_documents_branch_id ON legal_documents(branch_id);

-- ── CRM ─────────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_accounts') THEN
ALTER TABLE crm_accounts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_accounts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_accounts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_accounts_branch_id ON crm_accounts(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_activities') THEN
ALTER TABLE crm_activities ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_activities SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_activities ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_activities_branch_id ON crm_activities(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_contacts') THEN
ALTER TABLE crm_contacts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_contacts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_contacts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_contacts_branch_id ON crm_contacts(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_deal_segments') THEN
ALTER TABLE crm_deal_segments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_deal_segments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_deal_segments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_deal_segments_branch_id ON crm_deal_segments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_deals') THEN
ALTER TABLE crm_deals ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_deals SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_deals ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_deals_branch_id ON crm_deals(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_leads') THEN
ALTER TABLE crm_leads ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_leads SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_leads ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_leads_branch_id ON crm_leads(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_opportunities') THEN
ALTER TABLE crm_opportunities ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_opportunities SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_opportunities ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_opportunities_branch_id ON crm_opportunities(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'crm_pipeline_stages') THEN
ALTER TABLE crm_pipeline_stages ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE crm_pipeline_stages SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE crm_pipeline_stages ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_crm_pipeline_stages_branch_id ON crm_pipeline_stages(branch_id);

-- ── Dashboards / Data Sources ───────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'dashboard_data_sources') THEN
ALTER TABLE dashboard_data_sources ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE dashboard_data_sources SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE dashboard_data_sources ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dashboard_data_sources_branch_id ON dashboard_data_sources(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'dashboards') THEN
ALTER TABLE dashboards ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE dashboards SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE dashboards ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dashboards_branch_id ON dashboards(branch_id);

-- ── Data Deletion / Export Requests ─────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'data_deletion_requests') THEN
ALTER TABLE data_deletion_requests ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE data_deletion_requests SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE data_deletion_requests ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_data_deletion_requests_branch_id ON data_deletion_requests(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'data_export_requests') THEN
ALTER TABLE data_export_requests ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE data_export_requests SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE data_export_requests ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_data_export_requests_branch_id ON data_export_requests(branch_id);

-- ── Database Manager ────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'database_query_history') THEN
ALTER TABLE database_query_history ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE database_query_history SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE database_query_history ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_database_query_history_branch_id ON database_query_history(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'database_saved_queries') THEN
ALTER TABLE database_saved_queries ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE database_saved_queries SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE database_saved_queries ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_database_saved_queries_branch_id ON database_saved_queries(branch_id);

-- ── Directory Users / User Sessions ─────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'directory_users') THEN
ALTER TABLE directory_users ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE directory_users SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE directory_users ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_directory_users_branch_id ON directory_users(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'user_sessions') THEN
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE user_sessions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE user_sessions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_sessions_branch_id ON user_sessions(branch_id);

-- ── Drive / Storage ─────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'drive_files') THEN
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE drive_files SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE drive_files ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_drive_files_branch_id ON drive_files(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'drive_quotas') THEN
ALTER TABLE drive_quotas ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE drive_quotas SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE drive_quotas ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_drive_quotas_branch_id ON drive_quotas(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'folder_monitors') THEN
ALTER TABLE folder_monitors ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE folder_monitors SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE folder_monitors ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_folder_monitors_branch_id ON folder_monitors(branch_id);

-- ── Email ───────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'distribution_lists') THEN
ALTER TABLE distribution_lists ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE distribution_lists SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE distribution_lists ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_distribution_lists_branch_id ON distribution_lists(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_auto_responders') THEN
ALTER TABLE email_auto_responders ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_auto_responders SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_auto_responders ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_auto_responders_branch_id ON email_auto_responders(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_drafts') THEN
ALTER TABLE email_drafts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_drafts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_drafts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_drafts_branch_id ON email_drafts(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_labels') THEN
ALTER TABLE email_labels ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_labels SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_labels ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_labels_branch_id ON email_labels(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_rules') THEN
ALTER TABLE email_rules ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_rules SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_rules ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_rules_branch_id ON email_rules(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_signatures') THEN
ALTER TABLE email_signatures ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_signatures SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_signatures ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_signatures_branch_id ON email_signatures(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'email_templates') THEN
ALTER TABLE email_templates ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE email_templates SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE email_templates ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_email_templates_branch_id ON email_templates(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'global_email_signatures') THEN
ALTER TABLE global_email_signatures ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE global_email_signatures SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE global_email_signatures ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_global_email_signatures_branch_id ON global_email_signatures(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'scheduled_emails') THEN
ALTER TABLE scheduled_emails ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE scheduled_emails SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE scheduled_emails ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_branch_id ON scheduled_emails(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'shared_mailboxes') THEN
ALTER TABLE shared_mailboxes ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE shared_mailboxes SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE shared_mailboxes ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_shared_mailboxes_branch_id ON shared_mailboxes(branch_id);

-- ── Fraud ───────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'fraud_blocklist') THEN
ALTER TABLE fraud_blocklist ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE fraud_blocklist SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE fraud_blocklist ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_fraud_blocklist_branch_id ON fraud_blocklist(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'fraud_events') THEN
ALTER TABLE fraud_events ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE fraud_events SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE fraud_events ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_fraud_events_branch_id ON fraud_events(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'fraud_rules') THEN
ALTER TABLE fraud_rules ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE fraud_rules SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE fraud_rules ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_fraud_rules_branch_id ON fraud_rules(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'fraud_velocity') THEN
ALTER TABLE fraud_velocity ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE fraud_velocity SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE fraud_velocity ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_fraud_velocity_branch_id ON fraud_velocity(branch_id);

-- ── Identity / KYC ──────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_documents') THEN
ALTER TABLE identity_documents ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_documents SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_documents ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_documents_branch_id ON identity_documents(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_faces') THEN
ALTER TABLE identity_faces ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_faces SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_faces ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_faces_branch_id ON identity_faces(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_kyc_workflows') THEN
ALTER TABLE identity_kyc_workflows ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_kyc_workflows SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_kyc_workflows ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_kyc_workflows_branch_id ON identity_kyc_workflows(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_profiles') THEN
ALTER TABLE identity_profiles ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_profiles SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_profiles ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_profiles_branch_id ON identity_profiles(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_signatures') THEN
ALTER TABLE identity_signatures ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_signatures SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_signatures ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_signatures_branch_id ON identity_signatures(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'identity_signed_documents') THEN
ALTER TABLE identity_signed_documents ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE identity_signed_documents SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE identity_signed_documents ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_identity_signed_documents_branch_id ON identity_signed_documents(branch_id);

-- ── Inventory / POS ─────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'inventory_movements') THEN
ALTER TABLE inventory_movements ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE inventory_movements SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE inventory_movements ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_movements_branch_id ON inventory_movements(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'pos_sales') THEN
ALTER TABLE pos_sales ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE pos_sales SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE pos_sales ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_pos_sales_branch_id ON pos_sales(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'pos_sessions') THEN
ALTER TABLE pos_sessions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE pos_sessions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE pos_sessions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_pos_sessions_branch_id ON pos_sessions(branch_id);

-- ── Marketing ───────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'marketing_campaigns') THEN
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE marketing_campaigns SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE marketing_campaigns ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_marketing_campaigns_branch_id ON marketing_campaigns(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'marketing_contacts') THEN
ALTER TABLE marketing_contacts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE marketing_contacts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE marketing_contacts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_marketing_contacts_branch_id ON marketing_contacts(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'marketing_lists') THEN
ALTER TABLE marketing_lists ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE marketing_lists SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE marketing_lists ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_marketing_lists_branch_id ON marketing_lists(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'marketing_templates') THEN
ALTER TABLE marketing_templates ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE marketing_templates SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE marketing_templates ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_marketing_templates_branch_id ON marketing_templates(branch_id);

-- ── Meetings / Whiteboard ───────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'meeting_recordings') THEN
ALTER TABLE meeting_recordings ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE meeting_recordings SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE meeting_recordings ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_meeting_recordings_branch_id ON meeting_recordings(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'meeting_rooms') THEN
ALTER TABLE meeting_rooms ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE meeting_rooms SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE meeting_rooms ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_meeting_rooms_branch_id ON meeting_rooms(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'meeting_whiteboards') THEN
ALTER TABLE meeting_whiteboards ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE meeting_whiteboards SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE meeting_whiteboards ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_meeting_whiteboards_branch_id ON meeting_whiteboards(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'scheduled_meetings') THEN
ALTER TABLE scheduled_meetings ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE scheduled_meetings SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE scheduled_meetings ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_scheduled_meetings_branch_id ON scheduled_meetings(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'whiteboard_exports') THEN
ALTER TABLE whiteboard_exports ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE whiteboard_exports SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE whiteboard_exports ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_whiteboard_exports_branch_id ON whiteboard_exports(branch_id);

-- ── OAuth ───────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'oauth_applications') THEN
ALTER TABLE oauth_applications ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE oauth_applications SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE oauth_applications ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_applications_branch_id ON oauth_applications(branch_id);

-- ── OKR ─────────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_activity_log') THEN
ALTER TABLE okr_activity_log ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_activity_log SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_activity_log ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_activity_log_branch_id ON okr_activity_log(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_alignments') THEN
ALTER TABLE okr_alignments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_alignments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_alignments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_alignments_branch_id ON okr_alignments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_checkins') THEN
ALTER TABLE okr_checkins ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_checkins SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_checkins ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_checkins_branch_id ON okr_checkins(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_comments') THEN
ALTER TABLE okr_comments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_comments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_comments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_comments_branch_id ON okr_comments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_key_results') THEN
ALTER TABLE okr_key_results ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_key_results SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_key_results ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_key_results_branch_id ON okr_key_results(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_objectives') THEN
ALTER TABLE okr_objectives ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_objectives SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_objectives ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_objectives_branch_id ON okr_objectives(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'okr_templates') THEN
ALTER TABLE okr_templates ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE okr_templates SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE okr_templates ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_okr_templates_branch_id ON okr_templates(branch_id);

-- ── People / HR ─────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people') THEN
ALTER TABLE people ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_branch_id ON people(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people_departments') THEN
ALTER TABLE people_departments ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people_departments SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people_departments ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_departments_branch_id ON people_departments(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people_org_chart') THEN
ALTER TABLE people_org_chart ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people_org_chart SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people_org_chart ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_org_chart_branch_id ON people_org_chart(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people_skills') THEN
ALTER TABLE people_skills ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people_skills SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people_skills ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_skills_branch_id ON people_skills(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people_teams') THEN
ALTER TABLE people_teams ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people_teams SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people_teams ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_teams_branch_id ON people_teams(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'people_time_off') THEN
ALTER TABLE people_time_off ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE people_time_off SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE people_time_off ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_people_time_off_branch_id ON people_time_off(branch_id);

-- ── Products / Services / Price Lists ───────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'price_lists') THEN
ALTER TABLE price_lists ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE price_lists SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE price_lists ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_price_lists_branch_id ON price_lists(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_categories') THEN
ALTER TABLE product_categories ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE product_categories SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE product_categories ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_product_categories_branch_id ON product_categories(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_price_lists') THEN
ALTER TABLE product_price_lists ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE product_price_lists SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE product_price_lists ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_product_price_lists_branch_id ON product_price_lists(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_promotions') THEN
ALTER TABLE product_promotions ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE product_promotions SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE product_promotions ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_product_promotions_branch_id ON product_promotions(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_stock') THEN
ALTER TABLE product_stock ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE product_stock SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE product_stock ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_product_stock_branch_id ON product_stock(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_variations') THEN
ALTER TABLE product_variations ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE product_variations SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE product_variations ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_product_variations_branch_id ON product_variations(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'products') THEN
ALTER TABLE products ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE products SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE products ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_products_branch_id ON products(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'services') THEN
ALTER TABLE services ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE services SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE services ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_services_branch_id ON services(branch_id);

-- ── Reconciliation ──────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'reconciliation_rules') THEN
ALTER TABLE reconciliation_rules ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE reconciliation_rules SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE reconciliation_rules ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_reconciliation_rules_branch_id ON reconciliation_rules(branch_id);
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'reconciliation_runs') THEN
ALTER TABLE reconciliation_runs ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE reconciliation_runs SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE reconciliation_runs ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_reconciliation_runs_branch_id ON reconciliation_runs(branch_id);

-- ── Research ────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'research_projects') THEN
ALTER TABLE research_projects ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE research_projects SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE research_projects ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_research_projects_branch_id ON research_projects(branch_id);

-- ── Social ──────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'social_accounts') THEN
ALTER TABLE social_accounts ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE social_accounts SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE social_accounts ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_social_accounts_branch_id ON social_accounts(branch_id);

-- ── Tasks ───────────────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'tasks') THEN
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE tasks SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE tasks ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_branch_id ON tasks(branch_id);

-- ── Usage Analytics ─────────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'usage_analytics') THEN
ALTER TABLE usage_analytics ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE usage_analytics SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE usage_analytics ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_usage_analytics_branch_id ON usage_analytics(branch_id);

-- ── User KB Associations ────────────────────────────────────────────────
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'user_kb_associations') THEN
ALTER TABLE user_kb_associations ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;
UPDATE user_kb_associations SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE user_kb_associations ALTER COLUMN branch_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_user_kb_associations_branch_id ON user_kb_associations(branch_id);
    END IF;
END $$;