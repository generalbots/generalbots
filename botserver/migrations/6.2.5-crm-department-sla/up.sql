-- ============================================
-- CRM v2.5 - Department + SLA Extension
-- Version: 6.2.5
-- ============================================

-- 1. Add department_id to crm_deals (links to people_departments)
ALTER TABLE crm_deals ADD COLUMN IF NOT EXISTS department_id uuid REFERENCES people_departments(id);
CREATE INDEX IF NOT EXISTS idx_crm_deals_department ON crm_deals(department_id);

-- 2. Create SLA Policies table
CREATE TABLE IF NOT EXISTS attendance_sla_policies (
    id                      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  uuid NOT NULL,
    bot_id                  uuid NOT NULL,
    name                    varchar(100) NOT NULL,
    channel                 varchar(20),
    priority                varchar(20),
    first_response_minutes   integer DEFAULT 15,
    resolution_minutes       integer DEFAULT 240,
    escalate_on_breach       boolean DEFAULT TRUE,
    is_active               boolean DEFAULT TRUE,
    created_at              timestamptz DEFAULT now()
);

-- 3. Create SLA Events table
CREATE TABLE IF NOT EXISTS attendance_sla_events (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      uuid NOT NULL,
    sla_policy_id   uuid NOT NULL REFERENCES attendance_sla_policies(id) ON DELETE CASCADE,
    event_type      varchar(50) NOT NULL,
    due_at          timestamptz NOT NULL,
    met_at          timestamptz,
    breached_at     timestamptz,
    status          varchar(20) DEFAULT 'pending',
    created_at      timestamptz DEFAULT now()
);

-- 4. Insert default SLA policies
INSERT INTO attendance_sla_policies (org_id, bot_id, name, channel, priority, first_response_minutes, resolution_minutes)
SELECT DISTINCT org_id, id as bot_id, 'Default - Urgent', NULL, 'urgent', 5, 60
FROM bots WHERE org_id IS NOT NULL ON CONFLICT DO NOTHING;

INSERT INTO attendance_sla_policies (org_id, bot_id, name, channel, priority, first_response_minutes, resolution_minutes)
SELECT DISTINCT org_id, id as bot_id, 'Default - High', NULL, 'high', 15, 240
FROM bots WHERE org_id IS NOT NULL ON CONFLICT DO NOTHING;

INSERT INTO attendance_sla_policies (org_id, bot_id, name, channel, priority, first_response_minutes, resolution_minutes)
SELECT DISTINCT org_id, id as bot_id, 'Default - Normal', NULL, 'normal', 30, 480
FROM bots WHERE org_id IS NOT NULL ON CONFLICT DO NOTHING;

INSERT INTO attendance_sla_policies (org_id, bot_id, name, channel, priority, first_response_minutes, resolution_minutes)
SELECT DISTINCT org_id, id as bot_id, 'Default - Low', NULL, 'low', 60, 1440
FROM bots WHERE org_id IS NOT NULL ON CONFLICT DO NOTHING;

-- 5. Add lost_reason column BEFORE views that reference it
ALTER TABLE crm_deals ADD COLUMN IF NOT EXISTS lost_reason varchar(255);

-- 6. Create legacy compat views for leads/opportunities (from crm-sales.md)
CREATE OR REPLACE VIEW crm_leads_compat AS
    SELECT id, org_id, bot_id, contact_id, account_id,
           COALESCE(title, name, '') as title, description, value, currency,
           stage_id, COALESCE(stage, 'new') as stage, probability, source,
           expected_close_date, owner_id, lost_reason,
           tags, custom_fields, created_at, updated_at, closed_at
    FROM crm_deals
    WHERE stage IN ('new', 'qualified') OR stage IS NULL;

CREATE OR REPLACE VIEW crm_opportunities_compat AS
    SELECT id, org_id, bot_id, lead_id, account_id, contact_id,
           COALESCE(name, title, '') as name, description, value, currency,
           stage_id, COALESCE(stage, 'proposal') as stage, probability, source,
           expected_close_date, actual_close_date, won, owner_id,
           tags, custom_fields, created_at, updated_at
    FROM crm_deals
    WHERE stage IN ('proposal', 'negotiation', 'won', 'lost');

-- 7. Create index for SLA events
CREATE INDEX IF NOT EXISTS idx_sla_events_status ON attendance_sla_events(status);
CREATE INDEX IF NOT EXISTS idx_sla_events_due ON attendance_sla_events(due_at);
CREATE INDEX IF NOT EXISTS idx_sla_events_session ON attendance_sla_events(session_id);
CREATE INDEX IF NOT EXISTS idx_sla_policies_org_bot ON attendance_sla_policies(org_id, bot_id);
