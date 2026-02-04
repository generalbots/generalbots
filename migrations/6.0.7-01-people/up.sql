CREATE TABLE IF NOT EXISTS crm_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    company VARCHAR(255),
    job_title VARCHAR(255),
    source VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    address_line1 VARCHAR(500),
    address_line2 VARCHAR(500),
    city VARCHAR(255),
    state VARCHAR(255),
    postal_code VARCHAR(50),
    country VARCHAR(100),
    notes TEXT,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crm_contacts_org ON crm_contacts(org_id);
CREATE INDEX idx_crm_contacts_bot ON crm_contacts(bot_id);
CREATE INDEX idx_crm_contacts_email ON crm_contacts(email);
CREATE INDEX idx_crm_contacts_owner ON crm_contacts(owner_id);
CREATE INDEX idx_crm_contacts_status ON crm_contacts(status);

CREATE TABLE IF NOT EXISTS crm_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    website VARCHAR(500),
    industry VARCHAR(100),
    employees_count INTEGER,
    annual_revenue DECIMAL(15,2),
    phone VARCHAR(50),
    email VARCHAR(255),
    address_line1 VARCHAR(500),
    address_line2 VARCHAR(500),
    city VARCHAR(255),
    state VARCHAR(255),
    postal_code VARCHAR(50),
    country VARCHAR(100),
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crm_accounts_org ON crm_accounts(org_id);
CREATE INDEX idx_crm_accounts_bot ON crm_accounts(bot_id);
CREATE INDEX idx_crm_accounts_name ON crm_accounts(name);
CREATE INDEX idx_crm_accounts_owner ON crm_accounts(owner_id);

CREATE TABLE IF NOT EXISTS crm_pipeline_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    stage_order INTEGER NOT NULL,
    probability INTEGER NOT NULL DEFAULT 0,
    is_won BOOLEAN NOT NULL DEFAULT FALSE,
    is_lost BOOLEAN NOT NULL DEFAULT FALSE,
    color VARCHAR(7) DEFAULT '#3b82f6',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT crm_pipeline_stages_unique UNIQUE (org_id, bot_id, name)
);

CREATE INDEX idx_crm_pipeline_stages_org ON crm_pipeline_stages(org_id);
CREATE INDEX idx_crm_pipeline_stages_bot ON crm_pipeline_stages(bot_id);
CREATE INDEX idx_crm_pipeline_stages_order ON crm_pipeline_stages(stage_order);

CREATE TABLE IF NOT EXISTS crm_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE SET NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    value DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'USD',
    stage_id UUID REFERENCES crm_pipeline_stages(id) ON DELETE SET NULL,
    stage VARCHAR(100) NOT NULL DEFAULT 'new',
    probability INTEGER NOT NULL DEFAULT 0,
    source VARCHAR(100),
    expected_close_date DATE,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    lost_reason VARCHAR(500),
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE INDEX idx_crm_leads_org ON crm_leads(org_id);
CREATE INDEX idx_crm_leads_bot ON crm_leads(bot_id);
CREATE INDEX idx_crm_leads_contact ON crm_leads(contact_id);
CREATE INDEX idx_crm_leads_account ON crm_leads(account_id);
CREATE INDEX idx_crm_leads_stage ON crm_leads(stage);
CREATE INDEX idx_crm_leads_owner ON crm_leads(owner_id);
CREATE INDEX idx_crm_leads_expected_close ON crm_leads(expected_close_date);

CREATE TABLE IF NOT EXISTS crm_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES crm_leads(id) ON DELETE SET NULL,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE SET NULL,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    value DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'USD',
    stage_id UUID REFERENCES crm_pipeline_stages(id) ON DELETE SET NULL,
    stage VARCHAR(100) NOT NULL DEFAULT 'qualification',
    probability INTEGER NOT NULL DEFAULT 0,
    source VARCHAR(100),
    expected_close_date DATE,
    actual_close_date DATE,
    won BOOLEAN,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    tags TEXT[] DEFAULT '{}',
    custom_fields JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crm_opportunities_org ON crm_opportunities(org_id);
CREATE INDEX idx_crm_opportunities_bot ON crm_opportunities(bot_id);
CREATE INDEX idx_crm_opportunities_lead ON crm_opportunities(lead_id);
CREATE INDEX idx_crm_opportunities_account ON crm_opportunities(account_id);
CREATE INDEX idx_crm_opportunities_stage ON crm_opportunities(stage);
CREATE INDEX idx_crm_opportunities_owner ON crm_opportunities(owner_id);
CREATE INDEX idx_crm_opportunities_won ON crm_opportunities(won);

CREATE TABLE IF NOT EXISTS crm_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES crm_leads(id) ON DELETE CASCADE,
    opportunity_id UUID REFERENCES crm_opportunities(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL,
    subject VARCHAR(500),
    description TEXT,
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    outcome VARCHAR(255),
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crm_activities_org ON crm_activities(org_id);
CREATE INDEX idx_crm_activities_contact ON crm_activities(contact_id);
CREATE INDEX idx_crm_activities_lead ON crm_activities(lead_id);
CREATE INDEX idx_crm_activities_opportunity ON crm_activities(opportunity_id);
CREATE INDEX idx_crm_activities_type ON crm_activities(activity_type);
CREATE INDEX idx_crm_activities_due ON crm_activities(due_date);
CREATE INDEX idx_crm_activities_owner ON crm_activities(owner_id);

CREATE TABLE IF NOT EXISTS crm_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE CASCADE,
    lead_id UUID REFERENCES crm_leads(id) ON DELETE CASCADE,
    opportunity_id UUID REFERENCES crm_opportunities(id) ON DELETE CASCADE,
    account_id UUID REFERENCES crm_accounts(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crm_notes_contact ON crm_notes(contact_id);
CREATE INDEX idx_crm_notes_lead ON crm_notes(lead_id);
CREATE INDEX idx_crm_notes_opportunity ON crm_notes(opportunity_id);
CREATE INDEX idx_crm_notes_account ON crm_notes(account_id);

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'New', 1, 10, FALSE, FALSE, '#94a3b8'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'Qualified', 2, 25, FALSE, FALSE, '#3b82f6'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'Proposal', 3, 50, FALSE, FALSE, '#8b5cf6'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'Negotiation', 4, 75, FALSE, FALSE, '#f59e0b'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'Won', 5, 100, TRUE, FALSE, '#22c55e'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO crm_pipeline_stages (org_id, bot_id, name, stage_order, probability, is_won, is_lost, color)
SELECT o.org_id, b.id, 'Lost', 6, 0, FALSE, TRUE, '#ef4444'
FROM organizations o
CROSS JOIN bots b
LIMIT 1
ON CONFLICT DO NOTHING;
