-- ════════════════════════════════════════════════════════════
-- Migration: 9.23-branch-scope-cleanup
-- Purpose: Remove bot_id from business tables, use branch_id as sole scope
-- Rule: Nothing belongs to the bot. Everything belongs to the branch.
-- ════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════
-- DROP ALL EXISTING TABLES (order: last created = first dropped)
-- ════════════════════════════════════════════════

DROP TABLE IF EXISTS social_accounts CASCADE;
DROP TABLE IF EXISTS reconciliation_runs CASCADE;
DROP TABLE IF EXISTS reconciliation_rules CASCADE;
DROP TABLE IF EXISTS bank_transactions CASCADE;
DROP TABLE IF EXISTS delivery_transactions CASCADE;
DROP TABLE IF EXISTS oauth_applications CASCADE;
DROP TABLE IF EXISTS attendance_sla_policies CASCADE;
DROP TABLE IF EXISTS attendance_webhooks CASCADE;
DROP TABLE IF EXISTS tasks CASCADE;
DROP TABLE IF EXISTS conversational_queries CASCADE;
DROP TABLE IF EXISTS dashboard_data_sources CASCADE;
DROP TABLE IF EXISTS dashboards CASCADE;
DROP TABLE IF EXISTS canvases CASCADE;
DROP TABLE IF EXISTS etl_jobs CASCADE;
DROP TABLE IF EXISTS connectors CASCADE;
DROP TABLE IF EXISTS research_projects CASCADE;
DROP TABLE IF EXISTS cloud_workspaces CASCADE;
DROP TABLE IF EXISTS aiworkspace_templates CASCADE;
DROP TABLE IF EXISTS aiworkspaces CASCADE;
DROP TABLE IF EXISTS conversation_metrics CASCADE;
DROP TABLE IF EXISTS conversation_ratings CASCADE;
DROP TABLE IF EXISTS conversation_analytics CASCADE;
DROP TABLE IF EXISTS handoff_contexts CASCADE;
DROP TABLE IF EXISTS handoff_queue CASCADE;
DROP TABLE IF EXISTS scheduled_meetings CASCADE;
DROP TABLE IF EXISTS whiteboard_exports CASCADE;
DROP TABLE IF EXISTS meeting_whiteboards CASCADE;
DROP TABLE IF EXISTS meeting_recordings CASCADE;
DROP TABLE IF EXISTS meeting_rooms CASCADE;
DROP TABLE IF EXISTS calendar_events CASCADE;
DROP TABLE IF EXISTS calendars CASCADE;
DROP TABLE IF EXISTS folder_monitors CASCADE;
DROP TABLE IF EXISTS drive_quotas CASCADE;
DROP TABLE IF EXISTS drive_files CASCADE;
DROP TABLE IF EXISTS shared_mailboxes CASCADE;
DROP TABLE IF EXISTS distribution_lists CASCADE;
DROP TABLE IF EXISTS email_labels CASCADE;
DROP TABLE IF EXISTS email_rules CASCADE;
DROP TABLE IF EXISTS email_auto_responders CASCADE;
DROP TABLE IF EXISTS email_templates CASCADE;
DROP TABLE IF EXISTS scheduled_emails CASCADE;
DROP TABLE IF EXISTS email_drafts CASCADE;
DROP TABLE IF EXISTS global_email_signatures CASCADE;
DROP TABLE IF EXISTS email_signatures CASCADE;
DROP TABLE IF EXISTS identity_kyc_workflows CASCADE;
DROP TABLE IF EXISTS identity_signed_documents CASCADE;
DROP TABLE IF EXISTS identity_signatures CASCADE;
DROP TABLE IF EXISTS identity_documents CASCADE;
DROP TABLE IF EXISTS identity_faces CASCADE;
DROP TABLE IF EXISTS identity_profiles CASCADE;
DROP TABLE IF EXISTS fraud_velocity CASCADE;
DROP TABLE IF EXISTS fraud_blocklist CASCADE;
DROP TABLE IF EXISTS fraud_events CASCADE;
DROP TABLE IF EXISTS fraud_rules CASCADE;
DROP TABLE IF EXISTS okr_activity_log CASCADE;
DROP TABLE IF EXISTS okr_comments CASCADE;
DROP TABLE IF EXISTS okr_templates CASCADE;
DROP TABLE IF EXISTS okr_alignments CASCADE;
DROP TABLE IF EXISTS okr_checkins CASCADE;
DROP TABLE IF EXISTS okr_key_results CASCADE;
DROP TABLE IF EXISTS okr_objectives CASCADE;
DROP TABLE IF EXISTS data_export_requests CASCADE;
DROP TABLE IF EXISTS data_deletion_requests CASCADE;
DROP TABLE IF EXISTS legal_acceptances CASCADE;
DROP TABLE IF EXISTS cookie_consents CASCADE;
DROP TABLE IF EXISTS legal_documents CASCADE;
DROP TABLE IF EXISTS compliance_access_reviews CASCADE;
DROP TABLE IF EXISTS compliance_training_records CASCADE;
DROP TABLE IF EXISTS compliance_risk_assessments CASCADE;
DROP TABLE IF EXISTS compliance_evidence CASCADE;
DROP TABLE IF EXISTS compliance_audit_log CASCADE;
DROP TABLE IF EXISTS compliance_issues CASCADE;
DROP TABLE IF EXISTS compliance_checks CASCADE;
DROP TABLE IF EXISTS marketing_contacts CASCADE;
DROP TABLE IF EXISTS marketing_templates CASCADE;
DROP TABLE IF EXISTS marketing_lists CASCADE;
DROP TABLE IF EXISTS marketing_campaigns CASCADE;
DROP TABLE IF EXISTS people_time_off CASCADE;
DROP TABLE IF EXISTS people_skills CASCADE;
DROP TABLE IF EXISTS people_org_chart CASCADE;
DROP TABLE IF EXISTS people_departments CASCADE;
DROP TABLE IF EXISTS people_teams CASCADE;
DROP TABLE IF EXISTS people CASCADE;
DROP TABLE IF EXISTS inventory_movements CASCADE;
DROP TABLE IF EXISTS pos_sales CASCADE;
DROP TABLE IF EXISTS pos_sessions CASCADE;
DROP TABLE IF EXISTS product_promotions CASCADE;
DROP TABLE IF EXISTS product_price_lists CASCADE;
DROP TABLE IF EXISTS product_stock CASCADE;
DROP TABLE IF EXISTS product_variations CASCADE;
DROP TABLE IF EXISTS price_lists CASCADE;
DROP TABLE IF EXISTS product_categories CASCADE;
DROP TABLE IF EXISTS services CASCADE;
DROP TABLE IF EXISTS products CASCADE;
DROP TABLE IF EXISTS billing_notification_preferences CASCADE;
DROP TABLE IF EXISTS billing_grace_periods CASCADE;
DROP TABLE IF EXISTS billing_alert_history CASCADE;
DROP TABLE IF EXISTS billing_usage_alerts CASCADE;
DROP TABLE IF EXISTS billing_tax_rates CASCADE;
DROP TABLE IF EXISTS billing_quotes CASCADE;
DROP TABLE IF EXISTS billing_payments CASCADE;
DROP TABLE IF EXISTS billing_recurring CASCADE;
DROP TABLE IF EXISTS billing_invoices CASCADE;
DROP TABLE IF EXISTS crm_deal_segments CASCADE;
DROP TABLE IF EXISTS crm_pipeline_stages CASCADE;
DROP TABLE IF EXISTS crm_activities CASCADE;
DROP TABLE IF EXISTS crm_opportunities CASCADE;
DROP TABLE IF EXISTS crm_leads CASCADE;
DROP TABLE IF EXISTS crm_accounts CASCADE;
DROP TABLE IF EXISTS crm_deals CASCADE;
DROP TABLE IF EXISTS crm_contacts CASCADE;
DROP TABLE IF EXISTS user_kb_associations CASCADE;
DROP TABLE IF EXISTS database_saved_queries CASCADE;
DROP TABLE IF EXISTS database_query_history CASCADE;
DROP TABLE IF EXISTS usage_analytics CASCADE;
DROP TABLE IF EXISTS directory_users CASCADE;
DROP TABLE IF EXISTS user_sessions CASCADE;
DROP TABLE IF EXISTS designer_changes CASCADE;
DROP TABLE IF EXISTS safety_constraints CASCADE;
DROP TABLE IF EXISTS intent_classifications CASCADE;
DROP TABLE IF EXISTS compiled_intents CASCADE;
DROP TABLE IF EXISTS website_crawls CASCADE;
DROP TABLE IF EXISTS system_automations CASCADE;
DROP TABLE IF EXISTS auto_tasks CASCADE;
DROP TABLE IF EXISTS workflow_executions CASCADE;
DROP TABLE IF EXISTS workflow_definitions CASCADE;
DROP TABLE IF EXISTS bot_configuration CASCADE;
DROP TABLE IF EXISTS basic_tools CASCADE;
DROP TABLE IF EXISTS bot_memories CASCADE;
DROP TABLE IF EXISTS bots CASCADE;

-- ════════════════════════════════════════════════
-- CREATE ALL TABLES (branch-scope corrected)
-- ════════════════════════════════════════════════

-- [BOT] bots
CREATE TABLE IF NOT EXISTS bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    org_id UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id),
    is_default_for_branch BOOLEAN DEFAULT false,
    description TEXT,
    is_public BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    avatar_url VARCHAR(500),
    settings JSONB DEFAULT '{}'::jsonb,
    metadata JSONB DEFAULT '{}'::jsonb,
    UNIQUE(org_id, slug),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] bot_memories
CREATE TABLE IF NOT EXISTS bot_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    type VARCHAR(50) DEFAULT 'string',
    UNIQUE(branch_id, bot_id, key),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] basic_tools
CREATE TABLE IF NOT EXISTS basic_tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    source TEXT,
    compiled TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] bot_configuration
CREATE TABLE IF NOT EXISTS bot_configuration (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    config_key VARCHAR(255) NOT NULL,
    config_value TEXT NOT NULL,
    UNIQUE(branch_id, bot_id, config_key),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] workflow_definitions
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    trigger_type VARCHAR(50) NOT NULL,
    trigger_config JSONB DEFAULT '{}'::jsonb,
    steps JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] workflow_executions
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    input JSONB,
    output JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] auto_tasks
CREATE TABLE IF NOT EXISTS auto_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    schedule VARCHAR(100),
    task_type VARCHAR(50) NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    last_run_at TIMESTAMPTZ,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] system_automations
CREATE TABLE IF NOT EXISTS system_automations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    action_type VARCHAR(100) NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] website_crawls
CREATE TABLE IF NOT EXISTS website_crawls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    depth INTEGER DEFAULT 1,
    content TEXT,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] compiled_intents
CREATE TABLE IF NOT EXISTS compiled_intents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    patterns JSONB NOT NULL DEFAULT '[]'::jsonb,
    response TEXT,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] intent_classifications
CREATE TABLE IF NOT EXISTS intent_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    intent_id UUID NOT NULL REFERENCES compiled_intents(id) ON DELETE CASCADE,
    input TEXT NOT NULL,
    confidence REAL,
    classified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] safety_constraints
CREATE TABLE IF NOT EXISTS safety_constraints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    constraint_type VARCHAR(50) NOT NULL,
    pattern TEXT NOT NULL,
    action VARCHAR(50) NOT NULL DEFAULT 'block',
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, bot_id, constraint_type, pattern),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] designer_changes
CREATE TABLE IF NOT EXISTS designer_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    change_type VARCHAR(50) NOT NULL,
    target_type VARCHAR(50) NOT NULL,
    target_id UUID,
    payload JSONB NOT NULL,
    applied BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] user_sessions
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL UNIQUE,
    user_id VARCHAR(255),
    data JSONB DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ,
    UNIQUE(branch_id, bot_id, session_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] directory_users
CREATE TABLE IF NOT EXISTS directory_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE SET NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    display_name VARCHAR(255),
    roles JSONB DEFAULT '[]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, username),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] usage_analytics
CREATE TABLE IF NOT EXISTS usage_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB DEFAULT '{}'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] database_query_history
CREATE TABLE IF NOT EXISTS database_query_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    query TEXT NOT NULL,
    duration_ms INTEGER,
    rows_affected INTEGER,
    success BOOLEAN DEFAULT true,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] database_saved_queries
CREATE TABLE IF NOT EXISTS database_saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    query TEXT NOT NULL,
    description TEXT,
    UNIQUE(branch_id, bot_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BOT] user_kb_associations
CREATE TABLE IF NOT EXISTS user_kb_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,
    kb_name VARCHAR(255) NOT NULL,
    UNIQUE(branch_id, bot_id, user_id, kb_name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_contacts
CREATE TABLE IF NOT EXISTS crm_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    mobile VARCHAR(50),
    company VARCHAR(255),
    job_title VARCHAR(255),
    source VARCHAR(100),
    status VARCHAR(50) DEFAULT 'active',
    tags TEXT[],
    custom_fields JSONB DEFAULT '{}'::jsonb,
    address_street VARCHAR(255),
    address_city VARCHAR(255),
    address_state VARCHAR(100),
    address_country VARCHAR(100),
    address_zip VARCHAR(20),
    notes TEXT,
    owner_id UUID,
    pass_hash TEXT,
    UNIQUE(branch_id, email),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_deals
CREATE TABLE IF NOT EXISTS crm_deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    title VARCHAR(255),
    value DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    stage VARCHAR(100) DEFAULT 'qualification',
    probability INTEGER DEFAULT 10,
    won BOOLEAN DEFAULT false,
    closed_at TIMESTAMPTZ,
    owner_id UUID,
    department_id UUID,
    source VARCHAR(100),
    lost_reason TEXT,
    notes TEXT,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_accounts
CREATE TABLE IF NOT EXISTS crm_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    industry VARCHAR(100),
    website VARCHAR(500),
    phone VARCHAR(50),
    email VARCHAR(255),
    address_street VARCHAR(255),
    address_city VARCHAR(255),
    address_state VARCHAR(100),
    address_country VARCHAR(100),
    address_zip VARCHAR(20),
    notes TEXT,
    owner_id UUID,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_leads
CREATE TABLE IF NOT EXISTS crm_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    company VARCHAR(255),
    source VARCHAR(100),
    status VARCHAR(50) DEFAULT 'new',
    score INTEGER DEFAULT 0,
    notes TEXT,
    assigned_to UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_opportunities
CREATE TABLE IF NOT EXISTS crm_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    value DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    stage VARCHAR(100) DEFAULT 'qualification',
    probability INTEGER DEFAULT 10,
    expected_close_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_activities
CREATE TABLE IF NOT EXISTS crm_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    activity_type VARCHAR(100) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    due_date TIMESTAMPTZ,
    completed BOOLEAN DEFAULT false,
    completed_at TIMESTAMPTZ,
    assigned_to UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_pipeline_stages
CREATE TABLE IF NOT EXISTS crm_pipeline_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    probability INTEGER DEFAULT 0,
    color VARCHAR(50),
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] crm_deal_segments
CREATE TABLE IF NOT EXISTS crm_deal_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    deal_id UUID NOT NULL REFERENCES crm_deals(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    value DECIMAL(15,2) DEFAULT 0,
    quantity INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_invoices
CREATE TABLE IF NOT EXISTS billing_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    invoice_number VARCHAR(50) NOT NULL,
    customer_name VARCHAR(255),
    customer_email VARCHAR(255),
    status VARCHAR(50) DEFAULT 'draft',
    total DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    due_date DATE,
    paid_at TIMESTAMPTZ,
    notes TEXT,
    UNIQUE(branch_id, invoice_number),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_recurring
CREATE TABLE IF NOT EXISTS billing_recurring (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    plan_name VARCHAR(255) NOT NULL,
    amount DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(50) DEFAULT 'active',
    trial_end TIMESTAMPTZ,
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_payments
CREATE TABLE IF NOT EXISTS billing_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    invoice_id UUID REFERENCES billing_invoices(id) ON DELETE SET NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    payment_method VARCHAR(100),
    status VARCHAR(50) DEFAULT 'pending',
    paid_at TIMESTAMPTZ,
    gateway_response JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_quotes
CREATE TABLE IF NOT EXISTS billing_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    quote_number VARCHAR(50) NOT NULL,
    customer_name VARCHAR(255),
    customer_email VARCHAR(255),
    items JSONB DEFAULT '[]'::jsonb,
    total DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(50) DEFAULT 'draft',
    valid_until DATE,
    notes TEXT,
    UNIQUE(branch_id, quote_number),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_tax_rates
CREATE TABLE IF NOT EXISTS billing_tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    rate DECIMAL(5,2) NOT NULL,
    country VARCHAR(3),
    region VARCHAR(100),
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_usage_alerts
CREATE TABLE IF NOT EXISTS billing_usage_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    threshold DECIMAL(15,2) NOT NULL,
    metric VARCHAR(100) NOT NULL,
    recipients TEXT[] NOT NULL DEFAULT '{}'::text[],
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_alert_history
CREATE TABLE IF NOT EXISTS billing_alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    alert_id UUID REFERENCES billing_usage_alerts(id) ON DELETE SET NULL,
    metric VARCHAR(100) NOT NULL,
    value DECIMAL(15,2) NOT NULL,
    threshold DECIMAL(15,2) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_grace_periods
CREATE TABLE IF NOT EXISTS billing_grace_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES billing_recurring(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] billing_notification_preferences
CREATE TABLE IF NOT EXISTS billing_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    channel VARCHAR(50) NOT NULL DEFAULT 'email',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] products
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    sku VARCHAR(100) NOT NULL,
    description TEXT,
    price DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    stock_quantity INTEGER DEFAULT 0,
    category_id UUID,
    attributes JSONB DEFAULT '{}'::jsonb,
    is_public BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, sku),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] services
CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    sku VARCHAR(100) NOT NULL,
    description TEXT,
    price DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'USD',
    is_recurring BOOLEAN DEFAULT false,
    billing_cycle VARCHAR(50),
    attributes JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, sku),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] product_categories
CREATE TABLE IF NOT EXISTS product_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES product_categories(id) ON DELETE SET NULL,
    display_order INTEGER DEFAULT 0,
    UNIQUE(branch_id, slug),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] price_lists
CREATE TABLE IF NOT EXISTS price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    is_active BOOLEAN DEFAULT true,
    valid_from DATE,
    valid_until DATE,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] product_variations
CREATE TABLE IF NOT EXISTS product_variations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    sku VARCHAR(100) NOT NULL,
    price DECIMAL(15,2),
    attributes JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, sku),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] product_stock
CREATE TABLE IF NOT EXISTS product_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 0,
    min_quantity INTEGER DEFAULT 0,
    UNIQUE(branch_id, product_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] product_price_lists
CREATE TABLE IF NOT EXISTS product_price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    price_list_id UUID NOT NULL REFERENCES price_lists(id) ON DELETE CASCADE,
    price DECIMAL(15,2) NOT NULL,
    UNIQUE(branch_id, product_id, price_list_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] product_promotions
CREATE TABLE IF NOT EXISTS product_promotions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    product_id UUID REFERENCES products(id) ON DELETE CASCADE,
    discount_type VARCHAR(50) NOT NULL,
    discount_value DECIMAL(15,2) NOT NULL,
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] pos_sessions
CREATE TABLE IF NOT EXISTS pos_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_number VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'open',
    opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    opened_by UUID,
    UNIQUE(branch_id, session_number),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] pos_sales
CREATE TABLE IF NOT EXISTS pos_sales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_id UUID REFERENCES pos_sessions(id) ON DELETE SET NULL,
    sale_number VARCHAR(50) NOT NULL,
    items JSONB DEFAULT '[]'::jsonb,
    total DECIMAL(15,2) DEFAULT 0,
    payment_method VARCHAR(100),
    status VARCHAR(50) DEFAULT 'completed',
    UNIQUE(branch_id, sale_number),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] inventory_movements
CREATE TABLE IF NOT EXISTS inventory_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL,
    movement_type VARCHAR(50) NOT NULL,
    reference VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people
CREATE TABLE IF NOT EXISTS people (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(50),
    job_title VARCHAR(255),
    department_id UUID,
    manager_id UUID REFERENCES people(id) ON DELETE SET NULL,
    status VARCHAR(50) DEFAULT 'active',
    hire_date DATE,
    metadata JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, email),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people_teams
CREATE TABLE IF NOT EXISTS people_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    lead_id UUID REFERENCES people(id) ON DELETE SET NULL,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people_departments
CREATE TABLE IF NOT EXISTS people_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50),
    parent_id UUID REFERENCES people_departments(id) ON DELETE SET NULL,
    head_id UUID,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people_org_chart
CREATE TABLE IF NOT EXISTS people_org_chart (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    reports_to_id UUID REFERENCES people(id) ON DELETE SET NULL,
    position VARCHAR(255),
    UNIQUE(branch_id, person_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people_skills
CREATE TABLE IF NOT EXISTS people_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    skill VARCHAR(255) NOT NULL,
    level INTEGER DEFAULT 1,
    UNIQUE(branch_id, person_id, skill),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] people_time_off
CREATE TABLE IF NOT EXISTS people_time_off (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    leave_type VARCHAR(100) NOT NULL,
    starts_at DATE NOT NULL,
    ends_at DATE NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    approved_by UUID,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] marketing_campaigns
CREATE TABLE IF NOT EXISTS marketing_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    campaign_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    budget DECIMAL(15,2),
    metrics JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] marketing_lists
CREATE TABLE IF NOT EXISTS marketing_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    member_count INTEGER DEFAULT 0,
    is_dynamic BOOLEAN DEFAULT false,
    criteria JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] marketing_templates
CREATE TABLE IF NOT EXISTS marketing_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    template_type VARCHAR(100) NOT NULL,
    subject VARCHAR(500),
    body TEXT NOT NULL,
    variables JSONB DEFAULT '[]'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] marketing_contacts
CREATE TABLE IF NOT EXISTS marketing_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    list_id UUID REFERENCES marketing_lists(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    phone VARCHAR(50),
    metadata JSONB DEFAULT '{}'::jsonb,
    subscribed BOOLEAN DEFAULT true,
    UNIQUE(branch_id, list_id, email),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_checks
CREATE TABLE IF NOT EXISTS compliance_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    check_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    target_type VARCHAR(100),
    target_id UUID,
    result JSONB,
    checked_at TIMESTAMPTZ,
    checked_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_issues
CREATE TABLE IF NOT EXISTS compliance_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    status VARCHAR(50) DEFAULT 'open',
    description TEXT,
    assigned_to UUID,
    remediation TEXT,
    due_date DATE,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_audit_log
CREATE TABLE IF NOT EXISTS compliance_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    action VARCHAR(255) NOT NULL,
    actor_id UUID,
    target_type VARCHAR(100),
    target_id UUID,
    details JSONB DEFAULT '{}'::jsonb,
    ip_address VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_evidence
CREATE TABLE IF NOT EXISTS compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    check_id UUID REFERENCES compliance_checks(id) ON DELETE CASCADE,
    file_path VARCHAR(500) NOT NULL,
    description TEXT,
    uploaded_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_risk_assessments
CREATE TABLE IF NOT EXISTS compliance_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    risk_level VARCHAR(50) NOT NULL,
    probability INTEGER,
    impact INTEGER,
    mitigation TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    assessed_by UUID,
    assessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_training_records
CREATE TABLE IF NOT EXISTS compliance_training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    person_id UUID REFERENCES people(id) ON DELETE SET NULL,
    course_name VARCHAR(255) NOT NULL,
    completed BOOLEAN DEFAULT false,
    completed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] compliance_access_reviews
CREATE TABLE IF NOT EXISTS compliance_access_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    reviewer_id UUID,
    reviewed_type VARCHAR(100) NOT NULL,
    reviewed_id UUID NOT NULL,
    decision VARCHAR(50),
    comments TEXT,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] legal_documents
CREATE TABLE IF NOT EXISTS legal_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    document_type VARCHAR(100) NOT NULL,
    version VARCHAR(50) DEFAULT '1.0',
    content TEXT NOT NULL,
    is_published BOOLEAN DEFAULT false,
    published_at TIMESTAMPTZ,
    UNIQUE(branch_id, document_type, version),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] cookie_consents
CREATE TABLE IF NOT EXISTS cookie_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    visitor_id VARCHAR(255) NOT NULL,
    consent_type VARCHAR(100) NOT NULL,
    granted BOOLEAN NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] legal_acceptances
CREATE TABLE IF NOT EXISTS legal_acceptances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES legal_documents(id) ON DELETE CASCADE,
    acceptor_id VARCHAR(255) NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(branch_id, document_id, acceptor_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] data_deletion_requests
CREATE TABLE IF NOT EXISTS data_deletion_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    requester_email VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] data_export_requests
CREATE TABLE IF NOT EXISTS data_export_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    requester_email VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    export_format VARCHAR(50) DEFAULT 'json',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    file_path VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_objectives
CREATE TABLE IF NOT EXISTS okr_objectives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id UUID,
    cycle VARCHAR(50),
    status VARCHAR(50) DEFAULT 'draft',
    progress INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_key_results
CREATE TABLE IF NOT EXISTS okr_key_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    start_value DECIMAL(15,2) DEFAULT 0,
    target_value DECIMAL(15,2) NOT NULL,
    current_value DECIMAL(15,2) DEFAULT 0,
    unit VARCHAR(100),
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_checkins
CREATE TABLE IF NOT EXISTS okr_checkins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    key_result_id UUID NOT NULL REFERENCES okr_key_results(id) ON DELETE CASCADE,
    value DECIMAL(15,2) NOT NULL,
    confidence INTEGER,
    notes TEXT,
    checked_in_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_alignments
CREATE TABLE IF NOT EXISTS okr_alignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    objective_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    aligned_with_id UUID NOT NULL REFERENCES okr_objectives(id) ON DELETE CASCADE,
    alignment_type VARCHAR(50) DEFAULT 'contributes_to',
    UNIQUE(branch_id, objective_id, aligned_with_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_templates
CREATE TABLE IF NOT EXISTS okr_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    structure JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_comments
CREATE TABLE IF NOT EXISTS okr_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    objective_id UUID REFERENCES okr_objectives(id) ON DELETE CASCADE,
    key_result_id UUID REFERENCES okr_key_results(id) ON DELETE CASCADE,
    author_id UUID,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] okr_activity_log
CREATE TABLE IF NOT EXISTS okr_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    objective_id UUID REFERENCES okr_objectives(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    actor_id UUID,
    details JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] fraud_rules
CREATE TABLE IF NOT EXISTS fraud_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(100) NOT NULL,
    conditions JSONB NOT NULL,
    action VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] fraud_events
CREATE TABLE IF NOT EXISTS fraud_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    score REAL,
    details JSONB DEFAULT '{}'::jsonb,
    rule_id UUID REFERENCES fraud_rules(id) ON DELETE SET NULL,
    ip_address VARCHAR(50),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] fraud_blocklist
CREATE TABLE IF NOT EXISTS fraud_blocklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    target_type VARCHAR(50) NOT NULL,
    target_value VARCHAR(255) NOT NULL,
    reason TEXT,
    expires_at TIMESTAMPTZ,
    created_by UUID,
    UNIQUE(branch_id, target_type, target_value),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] fraud_velocity
CREATE TABLE IF NOT EXISTS fraud_velocity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    count INTEGER DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(branch_id, key, window_start),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_profiles
CREATE TABLE IF NOT EXISTS identity_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    person_id UUID REFERENCES people(id) ON DELETE SET NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    verification_level VARCHAR(50) DEFAULT 'none',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_faces
CREATE TABLE IF NOT EXISTS identity_faces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    profile_id UUID NOT NULL REFERENCES identity_profiles(id) ON DELETE CASCADE,
    image_hash VARCHAR(255) NOT NULL,
    encoding BYTEA,
    verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_documents
CREATE TABLE IF NOT EXISTS identity_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    profile_id UUID NOT NULL REFERENCES identity_profiles(id) ON DELETE CASCADE,
    document_type VARCHAR(100) NOT NULL,
    document_number VARCHAR(255),
    file_path VARCHAR(500),
    verified BOOLEAN DEFAULT false,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_signatures
CREATE TABLE IF NOT EXISTS identity_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    profile_id UUID REFERENCES identity_profiles(id) ON DELETE SET NULL,
    signature_hash VARCHAR(255) NOT NULL,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_signed_documents
CREATE TABLE IF NOT EXISTS identity_signed_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    signature_id UUID NOT NULL REFERENCES identity_signatures(id) ON DELETE CASCADE,
    document_content TEXT NOT NULL,
    document_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] identity_kyc_workflows
CREATE TABLE IF NOT EXISTS identity_kyc_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    profile_id UUID NOT NULL REFERENCES identity_profiles(id) ON DELETE CASCADE,
    workflow_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    steps JSONB DEFAULT '[]'::jsonb,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_signatures
CREATE TABLE IF NOT EXISTS email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] global_email_signatures
CREATE TABLE IF NOT EXISTS global_email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_drafts
CREATE TABLE IF NOT EXISTS email_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    subject VARCHAR(500),
    body TEXT,
    recipient VARCHAR(255),
    cc VARCHAR(500),
    bcc VARCHAR(500),
    attachments JSONB DEFAULT '[]'::jsonb,
    saved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] scheduled_emails
CREATE TABLE IF NOT EXISTS scheduled_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    recipient VARCHAR(255),
    cc VARCHAR(500),
    bcc VARCHAR(500),
    send_at TIMESTAMPTZ NOT NULL,
    sent BOOLEAN DEFAULT false,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_templates
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    subject VARCHAR(500),
    body TEXT NOT NULL,
    variables JSONB DEFAULT '[]'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_auto_responders
CREATE TABLE IF NOT EXISTS email_auto_responders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    trigger_type VARCHAR(100) NOT NULL,
    trigger_value VARCHAR(500),
    subject VARCHAR(500),
    body TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_rules
CREATE TABLE IF NOT EXISTS email_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    condition_field VARCHAR(100) NOT NULL,
    condition_operator VARCHAR(50) NOT NULL,
    condition_value VARCHAR(500),
    action_type VARCHAR(100) NOT NULL,
    action_value VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] email_labels
CREATE TABLE IF NOT EXISTS email_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(50),
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] distribution_lists
CREATE TABLE IF NOT EXISTS distribution_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    members TEXT[] DEFAULT '{}'::text[],
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] shared_mailboxes
CREATE TABLE IF NOT EXISTS shared_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    members JSONB DEFAULT '[]'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, email),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] drive_files
CREATE TABLE IF NOT EXISTS drive_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    mime_type VARCHAR(255),
    size_bytes BIGINT DEFAULT 0,
    folder_id UUID REFERENCES drive_files(id) ON DELETE SET NULL,
    is_folder BOOLEAN DEFAULT false,
    is_starred BOOLEAN DEFAULT false,
    is_trashed BOOLEAN DEFAULT false,
    shared_with TEXT[] DEFAULT '{}'::text[],
    created_by UUID,
    UNIQUE(branch_id, path),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] drive_quotas
CREATE TABLE IF NOT EXISTS drive_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    max_bytes BIGINT DEFAULT 1073741824,
    used_bytes BIGINT DEFAULT 0,
    UNIQUE(branch_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] folder_monitors
CREATE TABLE IF NOT EXISTS folder_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    folder_path TEXT NOT NULL,
    poll_interval_seconds INTEGER DEFAULT 60,
    is_active BOOLEAN DEFAULT true,
    last_checked_at TIMESTAMPTZ,
    UNIQUE(branch_id, folder_path),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] calendars
CREATE TABLE IF NOT EXISTS calendars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(50),
    is_public BOOLEAN DEFAULT false,
    owner_id UUID,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] calendar_events
CREATE TABLE IF NOT EXISTS calendar_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    calendar_id UUID NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    is_all_day BOOLEAN DEFAULT false,
    location TEXT,
    attendees TEXT[] DEFAULT '{}'::text[],
    recurrence_rule VARCHAR(255),
    status VARCHAR(50) DEFAULT 'confirmed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] meeting_rooms
CREATE TABLE IF NOT EXISTS meeting_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    capacity INTEGER DEFAULT 10,
    amenities TEXT[] DEFAULT '{}'::text[],
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] meeting_recordings
CREATE TABLE IF NOT EXISTS meeting_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    room_id UUID REFERENCES meeting_rooms(id) ON DELETE SET NULL,
    file_path VARCHAR(500) NOT NULL,
    duration_seconds INTEGER,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] meeting_whiteboards
CREATE TABLE IF NOT EXISTS meeting_whiteboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '{}'::jsonb,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] whiteboard_exports
CREATE TABLE IF NOT EXISTS whiteboard_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    whiteboard_id UUID NOT NULL REFERENCES meeting_whiteboards(id) ON DELETE CASCADE,
    format VARCHAR(50) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    exported_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] scheduled_meetings
CREATE TABLE IF NOT EXISTS scheduled_meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    room_id UUID REFERENCES meeting_rooms(id) ON DELETE SET NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    attendees TEXT[] DEFAULT '{}'::text[],
    status VARCHAR(50) DEFAULT 'scheduled',
    recording_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] handoff_queue
CREATE TABLE IF NOT EXISTS handoff_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'waiting',
    priority INTEGER DEFAULT 0,
    assigned_to UUID,
    channel VARCHAR(100),
    context JSONB DEFAULT '{}'::jsonb,
    entered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] handoff_contexts
CREATE TABLE IF NOT EXISTS handoff_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    context_type VARCHAR(100) NOT NULL,
    context_data JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] conversation_analytics
CREATE TABLE IF NOT EXISTS conversation_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    message_count INTEGER DEFAULT 0,
    duration_seconds INTEGER,
    resolved BOOLEAN DEFAULT false,
    rating INTEGER,
    UNIQUE(branch_id, session_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] conversation_ratings
CREATE TABLE IF NOT EXISTS conversation_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL,
    score INTEGER NOT NULL,
    feedback TEXT,
    rated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] conversation_metrics
CREATE TABLE IF NOT EXISTS conversation_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    metric_date DATE NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(15,2) NOT NULL,
    UNIQUE(branch_id, metric_date, metric_name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] aiworkspaces
CREATE TABLE IF NOT EXISTS aiworkspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    workspace_type VARCHAR(100) DEFAULT 'default',
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, slug),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] aiworkspace_templates
CREATE TABLE IF NOT EXISTS aiworkspace_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    structure JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] cloud_workspaces
CREATE TABLE IF NOT EXISTS cloud_workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, slug),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] research_projects
CREATE TABLE IF NOT EXISTS research_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'draft',
    lead_id UUID,
    start_date DATE,
    end_date DATE,
    budget DECIMAL(15,2),
    tags TEXT[] DEFAULT '{}'::text[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] connectors
CREATE TABLE IF NOT EXISTS connectors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    connector_type VARCHAR(100) NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] etl_jobs
CREATE TABLE IF NOT EXISTS etl_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    source_connector_id UUID REFERENCES connectors(id) ON DELETE SET NULL,
    target_connector_id UUID REFERENCES connectors(id) ON DELETE SET NULL,
    schedule VARCHAR(100),
    config JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    last_run_at TIMESTAMPTZ,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] canvases
CREATE TABLE IF NOT EXISTS canvases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '{}'::jsonb,
    is_published BOOLEAN DEFAULT false,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] dashboards
CREATE TABLE IF NOT EXISTS dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    config JSONB DEFAULT '{}'::jsonb,
    is_default BOOLEAN DEFAULT false,
    UNIQUE(branch_id, slug),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] dashboard_data_sources
CREATE TABLE IF NOT EXISTS dashboard_data_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(100) NOT NULL,
    config JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] conversational_queries
CREATE TABLE IF NOT EXISTS conversational_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    query_text TEXT NOT NULL,
    result JSONB,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    execution_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] tasks
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'todo',
    priority INTEGER DEFAULT 0,
    assignee_id UUID,
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    parent_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] attendance_webhooks
CREATE TABLE IF NOT EXISTS attendance_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    event_types TEXT[] DEFAULT '{}'::text[],
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, url),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] attendance_sla_policies
CREATE TABLE IF NOT EXISTS attendance_sla_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    response_time_minutes INTEGER NOT NULL,
    resolution_time_minutes INTEGER,
    priority INTEGER DEFAULT 0,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] oauth_applications
CREATE TABLE IF NOT EXISTS oauth_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    client_id VARCHAR(255) NOT NULL UNIQUE,
    client_secret TEXT,
    redirect_uris TEXT[] DEFAULT '{}'::text[],
    scopes TEXT[] DEFAULT '{}'::text[],
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] delivery_transactions
CREATE TABLE IF NOT EXISTS delivery_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    transaction_id VARCHAR(255) NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(50) DEFAULT 'pending',
    source VARCHAR(255),
    destination VARCHAR(255),
    metadata JSONB DEFAULT '{}'::jsonb,
    UNIQUE(branch_id, transaction_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] bank_transactions
CREATE TABLE IF NOT EXISTS bank_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    external_id VARCHAR(255),
    description TEXT,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    transaction_date DATE NOT NULL,
    category VARCHAR(255),
    reconciled BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] reconciliation_rules
CREATE TABLE IF NOT EXISTS reconciliation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(100) NOT NULL,
    conditions JSONB NOT NULL,
    action VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] reconciliation_runs
CREATE TABLE IF NOT EXISTS reconciliation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    run_date DATE NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    total_matched INTEGER DEFAULT 0,
    total_unmatched INTEGER DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- [BRANCH] social_accounts
CREATE TABLE IF NOT EXISTS social_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    platform VARCHAR(100) NOT NULL,
    account_id VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    UNIQUE(branch_id, platform, account_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

