-- ============================================
-- CRM v2 - Unified Deals Table
-- Version: 6.2.3
-- ============================================

-- 1. Create domain: segments
CREATE TABLE crm_deal_segments (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id      uuid NOT NULL,
  bot_id      uuid NOT NULL,
  name        varchar(50) NOT NULL,
  description varchar(255),
  created_at  timestamptz DEFAULT now()
);

-- Insert default segments (from gb.rob data)
INSERT INTO crm_deal_segments (org_id, bot_id, name) 
SELECT org_id, id, 'Default' FROM bots LIMIT 1;

-- 2. Create main deals table
CREATE TABLE crm_deals (
  -- 🆔 Key
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id          uuid NOT NULL,
  bot_id          uuid NOT NULL,

  -- 🔗 Links to Contact/Account (NO DUPLICATE!)
  contact_id      uuid REFERENCES crm_contacts(id),
  account_id      uuid REFERENCES crm_accounts(id),
  
  -- 🔗 Owner/Team (FK to users)
  am_id           uuid REFERENCES users(id),
  owner_id        uuid REFERENCES users(id),
  lead_id         uuid REFERENCES crm_leads(id),
  
  -- 💰 Deal
  title           varchar(100),
  name            varchar(100),
  description     text,
  value           double precision,
  currency        varchar(10),
  
  -- 📊 Pipeline (use existing crm_pipeline_stages!)
  stage_id        uuid REFERENCES crm_pipeline_stages(id),
  stage           varchar(30),  -- new, qualified, proposal, negotiation, won, lost
  probability     integer DEFAULT 0,
  won             boolean,
  
  -- 🎯 Classification
  source          varchar(50),  -- EMAIL, CALL, WEBSITE, REFERAL
  segment_id      uuid REFERENCES crm_deal_segments(id),
  
  -- 📅 Dates
  expected_close_date date,
  actual_close_date   date,
  period           integer,  -- 1=manhã, 2=tarde, 3=noite (or hour 1-24)
  deal_date       date,
  closed_at        timestamptz,
  created_at      timestamptz DEFAULT now(),
  updated_at      timestamptz,
  
  -- 📝 Notes (only current, history goes to crm_activities)
  notes           text,
  
  -- 🏷️ Tags
  tags            text[],
  
  -- 📦 Custom Fields (social media: linkedin, facebook, twitter, instagram, territory, hard_to_find)
  custom_fields   jsonb DEFAULT '{}'
);

-- 3. Add deal_id to crm_activities (for history migration)
-- ALTER TABLE crm_activities ADD COLUMN deal_id uuid REFERENCES crm_deals(id);

-- 4. Create indexes
CREATE INDEX idx_crm_deals_org_bot ON crm_deals(org_id, bot_id);
CREATE INDEX idx_crm_deals_contact ON crm_deals(contact_id);
CREATE INDEX idx_crm_deals_account ON crm_deals(account_id);
CREATE INDEX idx_crm_deals_stage ON crm_deals(stage_id);
CREATE INDEX idx_crm_deals_am ON crm_deals(am_id);
CREATE INDEX idx_crm_deals_source ON crm_deals(source);
