-- ============================================
-- Rollback: CRM v2.5 - Department + SLA Extension
-- Version: 6.2.5
-- ============================================

-- Drop views
DROP VIEW IF EXISTS crm_leads_compat;
DROP VIEW IF EXISTS crm_opportunities_compat;

-- Drop SLA tables
DROP TABLE IF EXISTS attendance_sla_events;
DROP TABLE IF EXISTS attendance_sla_policies;

-- Remove department_id from crm_deals
ALTER TABLE crm_deals DROP COLUMN IF EXISTS department_id;
