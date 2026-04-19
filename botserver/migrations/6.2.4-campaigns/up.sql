-- ============================================
-- Campaigns - Multichannel Marketing Platform
-- Version: 6.2.4
-- ============================================

-- 1. Marketing Campaigns
CREATE TABLE marketing_campaigns (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id          uuid NOT NULL,
  bot_id          uuid NOT NULL,
  name            varchar(100) NOT NULL,
  status          varchar(20) DEFAULT 'draft',  -- draft, scheduled, running, paused, completed
  channel         varchar(20) NOT NULL,  -- email, whatsapp, instagram, facebook, multi
  content_template jsonb DEFAULT '{}',
  scheduled_at    timestamptz,
  sent_at         timestamptz,
  completed_at    timestamptz,
  metrics         jsonb DEFAULT '{}',
  budget          double precision,
  created_at      timestamptz DEFAULT now(),
  updated_at      timestamptz
);

-- 2. Marketing Lists (saved recipient lists)
CREATE TABLE marketing_lists (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id          uuid NOT NULL,
  bot_id          uuid NOT NULL,
  name            varchar(100) NOT NULL,
  list_type       varchar(20) NOT NULL,  -- static, dynamic
  query_text      text,  -- SQL filter or broadcast.bas path
  contact_count   integer DEFAULT 0,
  created_at      timestamptz DEFAULT now(),
  updated_at      timestamptz
);

-- 3. Marketing List Contacts (junction for static lists)
CREATE TABLE marketing_list_contacts (
  list_id         uuid REFERENCES marketing_lists(id) ON DELETE CASCADE,
  contact_id      uuid REFERENCES crm_contacts(id) ON DELETE CASCADE,
  added_at        timestamptz DEFAULT now(),
  PRIMARY KEY (list_id, contact_id)
);

-- 4. Marketing Recipients (track delivery per contact)
CREATE TABLE marketing_recipients (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  campaign_id     uuid REFERENCES marketing_campaigns(id) ON DELETE CASCADE,
  contact_id      uuid REFERENCES crm_contacts(id) ON DELETE CASCADE,
  deal_id         uuid REFERENCES crm_deals(id) ON DELETE SET NULL,
  channel         varchar(20) NOT NULL,
  status          varchar(20) DEFAULT 'pending',  -- pending, sent, delivered, failed
  sent_at         timestamptz,
  delivered_at    timestamptz,
  failed_at       timestamptz,
  error_message   text,
  response        jsonb,
  created_at      timestamptz DEFAULT now()
);

-- 5. Marketing Templates
CREATE TABLE marketing_templates (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id          uuid NOT NULL,
  bot_id          uuid NOT NULL,
  name            varchar(100) NOT NULL,
  channel         varchar(20) NOT NULL,
  subject         varchar(200),
  body            text,
  media_url       varchar(500),
  ai_prompt       text,
  variables       jsonb DEFAULT '[]',
  approved        boolean DEFAULT false,
  meta_template_id varchar(100),
  created_at      timestamptz DEFAULT now(),
  updated_at      timestamptz
);

-- 6. Email Tracking
CREATE TABLE email_tracking (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  recipient_id    uuid REFERENCES marketing_recipients(id) ON DELETE CASCADE,
  campaign_id     uuid REFERENCES marketing_campaigns(id) ON DELETE CASCADE,
  message_id      varchar(100),
  open_token      uuid UNIQUE,
  open_tracking_enabled boolean DEFAULT true,
  opened          boolean DEFAULT false,
  opened_at       timestamptz,
  clicked         boolean DEFAULT false,
  clicked_at      timestamptz,
  ip_address      varchar(45),
  user_agent      varchar(500),
  created_at      timestamptz DEFAULT now()
);

-- 7. WhatsApp Business Config
CREATE TABLE whatsapp_business (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  bot_id          uuid NOT NULL UNIQUE,
  phone_number_id varchar(50),
  business_account_id varchar(50),
  access_token    varchar(500),
  webhooks_verified boolean DEFAULT false,
  created_at      timestamptz DEFAULT now(),
  updated_at      timestamptz
);

-- 8. Add deal_id column to marketing_campaigns (link campaigns to deals)
ALTER TABLE marketing_campaigns ADD COLUMN deal_id uuid REFERENCES crm_deals(id) ON DELETE SET NULL;

-- Indexes
CREATE INDEX idx_marketing_campaigns_org_bot ON marketing_campaigns(org_id, bot_id);
CREATE INDEX idx_marketing_campaigns_status ON marketing_campaigns(status);
CREATE INDEX idx_marketing_lists_org_bot ON marketing_lists(org_id, bot_id);
CREATE INDEX idx_marketing_recipients_campaign ON marketing_recipients(campaign_id);
CREATE INDEX idx_marketing_recipients_contact ON marketing_recipients(contact_id);
CREATE INDEX idx_marketing_recipients_deal ON marketing_recipients(deal_id);
CREATE INDEX idx_email_tracking_token ON email_tracking(open_token);
CREATE INDEX idx_email_tracking_campaign ON email_tracking(campaign_id);
