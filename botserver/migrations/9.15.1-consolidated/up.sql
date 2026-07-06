-- 9.15.1-consolidated: up.sql (was 6.5.0-consolidated, moved after 9.15-org-branches for branch_id ordering)
-- Consolidated schema creation from scratch

-- Ensure branch_id and tenant_id columns exist on all tables that need them
-- (tables may have been created by earlier migrations without these columns).
-- Also ensure UNIQUE constraints required by later migrations (9.15-org-branches)
-- are in place. This must run BEFORE ALTER TABLE ADD CONSTRAINT statements below.
DO $$
DECLARE
    tables_with_branch_id TEXT[] := ARRAY[
        'attendance_sla_policies', 'attendance_webhooks', 'attendant_agent_status',
        'attendant_canned_responses', 'attendant_queues', 'attendant_sessions',
        'attendant_tags', 'attendant_wrap_up_codes', 'billing_alert_history',
        'billing_grace_periods', 'billing_invoices', 'billing_notification_preferences',
        'billing_payments', 'billing_quotes', 'billing_recurring', 'billing_tax_rates',
        'billing_usage_alerts', 'bots', 'calendar_events', 'calendars', 'canvases',
        'compliance_access_reviews', 'compliance_audit_log', 'compliance_checks',
        'compliance_evidence', 'compliance_issues', 'compliance_risk_assessments',
        'compliance_training_records', 'conversational_queries', 'cookie_consents',
        'crm_accounts', 'crm_activities', 'crm_contacts', 'crm_deals', 'crm_deal_segments',
        'crm_leads', 'crm_notes', 'crm_opportunities', 'crm_pipeline_stages',
        'dashboard_data_sources', 'dashboards', 'data_deletion_requests',
        'data_export_requests', 'feature_flags', 'inventory_movements',
        'legal_acceptances', 'legal_documents', 'marketing_campaigns', 'marketing_lists',
        'marketing_templates', 'meeting_recordings', 'meeting_rooms',
        'meeting_transcriptions', 'meeting_whiteboards', 'okr_activity_log',
        'okr_alignments', 'okr_checkins', 'okr_comments', 'okr_key_results',
        'okr_objectives', 'okr_templates', 'organization_invitations', 'people',
        'people_departments', 'people_org_chart', 'people_skills', 'people_teams',
        'people_time_off', 'price_lists', 'product_categories', 'products',
        'research_projects', 'scheduled_meetings', 'services', 'social_announcements',
        'social_channel_accounts', 'social_communities', 'social_hashtags',
        'social_posts', 'social_praises', 'support_tickets', 'ticket_canned_responses',
        'ticket_categories', 'ticket_sla_policies', 'ticket_tags', 'whiteboard_exports',
        'workspaces', 'workspace_templates'
    ];
    tables_with_tenant_id TEXT[] := ARRAY[
        'organizations'
    ];
    t TEXT;
BEGIN
    FOREACH t IN ARRAY tables_with_branch_id LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = t) THEN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = t AND column_name = 'branch_id') THEN
                EXECUTE format('ALTER TABLE public.%I ADD COLUMN branch_id UUID', t);
            END IF;
        END IF;
    END LOOP;
    FOREACH t IN ARRAY tables_with_tenant_id LOOP
        IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = t) THEN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = t AND column_name = 'tenant_id') THEN
                EXECUTE format('ALTER TABLE public.%I ADD COLUMN tenant_id UUID', t);
            END IF;
        END IF;
    END LOOP;
    -- Ensure UNIQUE constraints needed by 9.15-org-branches ON CONFLICT clauses
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'branches') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE table_schema = 'public' AND table_name = 'branches' AND constraint_type = 'UNIQUE' AND constraint_name = 'branches_org_id_slug_key') THEN
            ALTER TABLE branches ADD CONSTRAINT branches_org_id_slug_key UNIQUE (org_id, slug);
        END IF;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'tenants') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE table_schema = 'public' AND table_name = 'tenants' AND constraint_type = 'UNIQUE' AND constraint_name = 'tenants_slug_key') THEN
            ALTER TABLE tenants ADD CONSTRAINT tenants_slug_key UNIQUE (slug);
        END IF;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'organizations') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE table_schema = 'public' AND table_name = 'organizations' AND constraint_type = 'UNIQUE' AND constraint_name = 'organizations_slug_key') THEN
            ALTER TABLE organizations ADD CONSTRAINT organizations_slug_key UNIQUE (slug);
        END IF;
    END IF;
    -- Add missing columns to product_categories (used by botproducts seed)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'product_categories') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'product_categories' AND column_name = 'display_order') THEN
            ALTER TABLE product_categories ADD COLUMN display_order INTEGER;
        END IF;
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'product_categories' AND column_name = 'updated_at') THEN
            ALTER TABLE product_categories ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
        END IF;
        -- Make org_id and bot_id nullable (botproducts seed may not provide them)
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'product_categories' AND column_name = 'org_id' AND is_nullable = 'NO') THEN
            ALTER TABLE product_categories ALTER COLUMN org_id DROP NOT NULL;
        END IF;
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'product_categories' AND column_name = 'bot_id' AND is_nullable = 'NO') THEN
            ALTER TABLE product_categories ALTER COLUMN bot_id DROP NOT NULL;
        END IF;
    END IF;
    -- Add category_id to products (used by botproducts seed)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'products') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'products' AND column_name = 'category_id') THEN
            ALTER TABLE products ADD COLUMN category_id UUID;
        END IF;
        -- Make org_id and bot_id nullable (botproducts seed may not provide them)
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'products' AND column_name = 'org_id' AND is_nullable = 'NO') THEN
            ALTER TABLE products ALTER COLUMN org_id DROP NOT NULL;
        END IF;
        IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'products' AND column_name = 'bot_id' AND is_nullable = 'NO') THEN
            ALTER TABLE products ALTER COLUMN bot_id DROP NOT NULL;
        END IF;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS attendance_sla_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    sla_policy_id UUID NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    due_at TIMESTAMPTZ NOT NULL,
    met_at TIMESTAMPTZ,
    breached_at TIMESTAMPTZ,
    status VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendance_sla_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    channel VARCHAR(255),
    priority VARCHAR(255),
    first_response_minutes INTEGER,
    resolution_minutes INTEGER,
    escalate_on_breach BOOLEAN,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendance_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    webhook_url VARCHAR(255) NOT NULL,
    events TEXT[],
    is_active BOOLEAN DEFAULT TRUE,
    secret_key TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_agent_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    agent_id UUID NOT NULL,
    status VARCHAR(255) NOT NULL,
    status_message VARCHAR(255),
    current_sessions INTEGER NOT NULL,
    max_sessions INTEGER NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL,
    break_started_at TIMESTAMPTZ,
    break_reason VARCHAR(255),
    available_since TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_canned_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    shortcut VARCHAR(255),
    category VARCHAR(255),
    queue_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    usage_count INTEGER NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_queue_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    max_concurrent INTEGER NOT NULL,
    priority INTEGER NOT NULL,
    skills TEXT[] NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_queues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL,
    max_wait_minutes INTEGER NOT NULL,
    auto_assign BOOLEAN NOT NULL,
    working_hours JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_session_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    sender_type VARCHAR(255) NOT NULL,
    sender_id UUID,
    sender_name VARCHAR(255),
    content TEXT NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    attachments JSONB NOT NULL,
    is_internal BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_session_wrap_up (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    wrap_up_code_id UUID,
    notes TEXT,
    follow_up_required BOOLEAN NOT NULL,
    follow_up_date VARCHAR(255),
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    session_number VARCHAR(255) NOT NULL,
    channel VARCHAR(255) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255),
    customer_email VARCHAR(255),
    customer_phone VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL,
    agent_id UUID,
    queue_id UUID,
    subject VARCHAR(255),
    initial_message TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    assigned_at TIMESTAMPTZ,
    first_response_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    wait_time_seconds INTEGER,
    handle_time_seconds INTEGER,
    satisfaction_rating INTEGER,
    satisfaction_comment TEXT,
    tags TEXT[] NOT NULL,
    metadata JSONB NOT NULL,
    notes TEXT,
    transfer_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(255),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    from_agent_id UUID,
    to_agent_id UUID,
    to_queue_id UUID,
    reason VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS attendant_wrap_up_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    code VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    requires_notes BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS basic_tools (
    id TEXT PRIMARY KEY,
    bot_id TEXT,
    tool_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    ast_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    mcp_json TEXT,
    tool_json TEXT,
    compiled_at TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT NOW(),
    updated_at TEXT NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    alert_id UUID NOT NULL,
    metric VARCHAR(255) NOT NULL,
    severity VARCHAR(255) NOT NULL,
    current_usage BIGINT NOT NULL,
    usage_limit BIGINT NOT NULL,
    percentage NUMERIC NOT NULL,
    message TEXT NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    resolved_at TIMESTAMPTZ,
    resolution_type VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_grace_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    metric VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    overage_at_start NUMERIC NOT NULL,
    current_overage NUMERIC NOT NULL,
    max_allowed_overage NUMERIC NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ended_at TIMESTAMPTZ,
    end_reason VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL,
    product_id UUID,
    description VARCHAR(255) NOT NULL,
    quantity NUMERIC NOT NULL,
    unit_price NUMERIC NOT NULL,
    discount_percent NUMERIC NOT NULL,
    tax_rate NUMERIC NOT NULL,
    amount NUMERIC NOT NULL,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    invoice_number VARCHAR(255) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    customer_address TEXT,
    status VARCHAR(255) NOT NULL,
    issue_date VARCHAR(255) NOT NULL,
    due_date VARCHAR(255) NOT NULL,
    subtotal NUMERIC NOT NULL,
    tax_rate NUMERIC NOT NULL,
    tax_amount NUMERIC NOT NULL,
    discount_percent NUMERIC NOT NULL,
    discount_amount NUMERIC NOT NULL,
    total NUMERIC NOT NULL,
    amount_paid NUMERIC NOT NULL,
    amount_due NUMERIC NOT NULL,
    currency VARCHAR(255) NOT NULL,
    notes TEXT,
    terms TEXT,
    footer TEXT,
    paid_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    voided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    enabled BOOLEAN NOT NULL,
    channels JSONB NOT NULL,
    email_recipients JSONB NOT NULL,
    webhook_url TEXT,
    webhook_secret TEXT,
    slack_webhook_url TEXT,
    teams_webhook_url TEXT,
    sms_numbers JSONB NOT NULL,
    min_severity VARCHAR(255) NOT NULL,
    quiet_hours_start INTEGER,
    quiet_hours_end INTEGER,
    quiet_hours_timezone VARCHAR(255),
    quiet_hours_days JSONB,
    metric_overrides JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    invoice_id UUID,
    payment_number VARCHAR(255) NOT NULL,
    amount NUMERIC NOT NULL,
    currency VARCHAR(255) NOT NULL,
    payment_method VARCHAR(255) NOT NULL,
    payment_reference VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    payer_name VARCHAR(255),
    payer_email VARCHAR(255),
    notes TEXT,
    paid_at TIMESTAMPTZ NOT NULL,
    refunded_at TIMESTAMPTZ,
    refund_amount NUMERIC,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_quote_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quote_id UUID NOT NULL,
    product_id UUID,
    description VARCHAR(255) NOT NULL,
    quantity NUMERIC NOT NULL,
    unit_price NUMERIC NOT NULL,
    discount_percent NUMERIC NOT NULL,
    tax_rate NUMERIC NOT NULL,
    amount NUMERIC NOT NULL,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    quote_number VARCHAR(255) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    customer_address TEXT,
    status VARCHAR(255) NOT NULL,
    issue_date VARCHAR(255) NOT NULL,
    valid_until VARCHAR(255) NOT NULL,
    subtotal NUMERIC NOT NULL,
    tax_rate NUMERIC NOT NULL,
    tax_amount NUMERIC NOT NULL,
    discount_percent NUMERIC NOT NULL,
    discount_amount NUMERIC NOT NULL,
    total NUMERIC NOT NULL,
    currency VARCHAR(255) NOT NULL,
    notes TEXT,
    terms TEXT,
    accepted_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    converted_invoice_id UUID,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_recurring (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    frequency VARCHAR(255) NOT NULL,
    interval_count INTEGER NOT NULL,
    amount NUMERIC NOT NULL,
    currency VARCHAR(255) NOT NULL,
    description TEXT,
    next_invoice_date VARCHAR(255) NOT NULL,
    last_invoice_date VARCHAR(255),
    last_invoice_id UUID,
    start_date VARCHAR(255) NOT NULL,
    end_date VARCHAR(255),
    invoices_generated INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    rate NUMERIC NOT NULL,
    description TEXT,
    region VARCHAR(255),
    is_default BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS billing_usage_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    metric VARCHAR(255) NOT NULL,
    severity VARCHAR(255) NOT NULL,
    current_usage BIGINT NOT NULL,
    usage_limit BIGINT NOT NULL,
    percentage NUMERIC NOT NULL,
    threshold NUMERIC NOT NULL,
    message TEXT NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    notification_sent BOOLEAN NOT NULL,
    notification_channels JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bot_configuration (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    is_encrypted BOOLEAN NOT NULL,
    config_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bot_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS bot_shared_memory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_bot_id UUID NOT NULL,
    target_bot_id UUID NOT NULL,
    memory_key TEXT NOT NULL,
    memory_value TEXT NOT NULL,
    shared_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    llm_provider VARCHAR(255) NOT NULL,
    llm_config JSONB NOT NULL,
    context_provider VARCHAR(255) NOT NULL,
    context_config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE,
    database_name VARCHAR(255),
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_default_for_branch BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    slug VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendar_event_attendees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    role VARCHAR(255) NOT NULL,
    rsvp_time TIMESTAMPTZ,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendar_event_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    reminder_type VARCHAR(255) NOT NULL,
    minutes_before INTEGER NOT NULL,
    is_sent BOOLEAN NOT NULL,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendar_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    calendar_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    location VARCHAR(255),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    all_day BOOLEAN NOT NULL,
    recurrence_rule TEXT,
    recurrence_id UUID,
    color VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    visibility VARCHAR(255) NOT NULL,
    busy_status VARCHAR(255) NOT NULL,
    reminders JSONB NOT NULL,
    attendees JSONB NOT NULL,
    conference_data JSONB,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendar_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_id UUID NOT NULL,
    shared_with_user_id UUID,
    shared_with_email VARCHAR(255),
    permission VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS calendars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(255),
    timezone VARCHAR(255),
    is_primary BOOLEAN NOT NULL,
    is_visible BOOLEAN NOT NULL,
    is_shared BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS canvas_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL,
    user_id UUID NOT NULL,
    permission VARCHAR(255) NOT NULL,
    added_by UUID,
    added_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS canvas_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL,
    element_id UUID,
    parent_comment_id UUID,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    x_position DOUBLE PRECISION,
    y_position DOUBLE PRECISION,
    resolved BOOLEAN NOT NULL,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS canvas_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL,
    element_type VARCHAR(255) NOT NULL,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    width DOUBLE PRECISION NOT NULL,
    height DOUBLE PRECISION NOT NULL,
    rotation DOUBLE PRECISION NOT NULL,
    z_index INTEGER NOT NULL,
    locked BOOLEAN NOT NULL,
    properties JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS canvas_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canvas_id UUID NOT NULL,
    version_number INTEGER NOT NULL,
    name VARCHAR(255),
    elements_snapshot JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS canvases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    background_color VARCHAR(255),
    thumbnail_url TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_template BOOLEAN NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clicks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id TEXT NOT NULL,
    email TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_access_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID NOT NULL,
    reviewer_id UUID NOT NULL,
    review_date TIMESTAMPTZ NOT NULL,
    permissions_reviewed JSONB NOT NULL,
    anomalies JSONB NOT NULL,
    recommendations JSONB NOT NULL,
    status VARCHAR(255) NOT NULL,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    event_type VARCHAR(255) NOT NULL,
    user_id UUID,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL,
    result VARCHAR(255) NOT NULL,
    ip_address VARCHAR(255),
    user_agent TEXT,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    framework VARCHAR(255) NOT NULL,
    control_id VARCHAR(255) NOT NULL,
    control_name VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    score NUMERIC NOT NULL,
    checked_at TIMESTAMPTZ NOT NULL,
    checked_by UUID,
    evidence JSONB NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    check_id UUID,
    issue_id UUID,
    evidence_type VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    file_url TEXT,
    file_name VARCHAR(255),
    file_size INTEGER,
    mime_type VARCHAR(255),
    metadata JSONB NOT NULL,
    collected_at TIMESTAMPTZ NOT NULL,
    collected_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    check_id UUID,
    severity VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    remediation TEXT,
    due_date TIMESTAMPTZ,
    assigned_to UUID,
    status VARCHAR(255) NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    title VARCHAR(255) NOT NULL,
    assessor_id UUID NOT NULL,
    methodology VARCHAR(255) NOT NULL,
    overall_risk_score NUMERIC NOT NULL,
    status VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    next_review_date VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_risks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(255) NOT NULL,
    likelihood_score INTEGER NOT NULL,
    impact_score INTEGER NOT NULL,
    risk_score INTEGER NOT NULL,
    risk_level VARCHAR(255) NOT NULL,
    current_controls JSONB NOT NULL,
    treatment_strategy VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    owner_id UUID,
    due_date VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS compliance_training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID NOT NULL,
    training_type VARCHAR(255) NOT NULL,
    training_name VARCHAR(255) NOT NULL,
    provider VARCHAR(255),
    score INTEGER,
    passed BOOLEAN NOT NULL,
    completion_date TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    certificate_url TEXT,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS consent_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consent_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    previous_consents JSONB NOT NULL,
    new_consents JSONB NOT NULL,
    ip_address VARCHAR(255),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS conversational_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    dashboard_id UUID,
    user_id UUID NOT NULL,
    natural_language TEXT NOT NULL,
    generated_query TEXT,
    result_widget_config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS cookie_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID,
    session_id VARCHAR(255),
    ip_address VARCHAR(255),
    user_agent TEXT,
    country_code VARCHAR(255),
    consent_necessary BOOLEAN NOT NULL,
    consent_analytics BOOLEAN NOT NULL,
    consent_marketing BOOLEAN NOT NULL,
    consent_preferences BOOLEAN NOT NULL,
    consent_functional BOOLEAN NOT NULL,
    consent_version VARCHAR(255) NOT NULL,
    consent_given_at TIMESTAMPTZ NOT NULL,
    consent_updated_at TIMESTAMPTZ NOT NULL,
    consent_withdrawn_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    website VARCHAR(255),
    industry VARCHAR(255),
    employees_count INTEGER,
    annual_revenue DOUBLE PRECISION,
    phone VARCHAR(255),
    email VARCHAR(255),
    address_line1 VARCHAR(255),
    address_line2 VARCHAR(255),
    city VARCHAR(255),
    state VARCHAR(255),
    postal_code VARCHAR(255),
    country VARCHAR(255),
    description TEXT,
    tags TEXT[] NOT NULL,
    custom_fields JSONB NOT NULL,
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    contact_id UUID,
    lead_id UUID,
    opportunity_id UUID,
    account_id UUID,
    activity_type VARCHAR(255) NOT NULL,
    subject VARCHAR(255),
    description TEXT,
    due_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    outcome VARCHAR(255),
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(255),
    mobile VARCHAR(255),
    company VARCHAR(255),
    job_title VARCHAR(255),
    source VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    tags TEXT[] NOT NULL,
    custom_fields JSONB NOT NULL,
    address_line1 VARCHAR(255),
    address_line2 VARCHAR(255),
    city VARCHAR(255),
    state VARCHAR(255),
    postal_code VARCHAR(255),
    country VARCHAR(255),
    notes TEXT,
    owner_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_deal_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    contact_id UUID,
    account_id UUID,
    am_id UUID,
    owner_id UUID,
    lead_id UUID,
    title VARCHAR(255),
    name VARCHAR(255),
    description TEXT,
    value DOUBLE PRECISION,
    currency VARCHAR(255),
    stage_id UUID,
    stage VARCHAR(255),
    probability INTEGER NOT NULL,
    source VARCHAR(255),
    segment_id UUID,
    department_id UUID,
    expected_close_date VARCHAR(255),
    actual_close_date VARCHAR(255),
    period INTEGER,
    deal_date VARCHAR(255),
    closed_at TIMESTAMPTZ,
    lost_reason VARCHAR(255),
    won BOOLEAN,
    notes TEXT,
    tags TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    custom_fields JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS crm_leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    contact_id UUID,
    account_id UUID,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    value DOUBLE PRECISION,
    currency VARCHAR(255),
    stage_id UUID,
    stage VARCHAR(255) NOT NULL,
    probability INTEGER NOT NULL,
    source VARCHAR(255),
    expected_close_date VARCHAR(255),
    owner_id UUID,
    lost_reason VARCHAR(255),
    tags TEXT[] NOT NULL,
    custom_fields JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS crm_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    contact_id UUID,
    lead_id UUID,
    opportunity_id UUID,
    account_id UUID,
    content TEXT NOT NULL,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    lead_id UUID,
    account_id UUID,
    contact_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    value DOUBLE PRECISION,
    currency VARCHAR(255),
    stage_id UUID,
    stage VARCHAR(255) NOT NULL,
    probability INTEGER NOT NULL,
    source VARCHAR(255),
    expected_close_date VARCHAR(255),
    actual_close_date VARCHAR(255),
    won BOOLEAN,
    owner_id UUID,
    tags TEXT[] NOT NULL,
    custom_fields JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS crm_pipeline_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    stage_order INTEGER NOT NULL,
    probability INTEGER NOT NULL,
    is_won BOOLEAN NOT NULL,
    is_lost BOOLEAN NOT NULL,
    color VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboard_data_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    source_type VARCHAR(255) NOT NULL,
    connection JSONB NOT NULL,
    schema_definition JSONB NOT NULL,
    refresh_schedule VARCHAR(255),
    last_sync TIMESTAMPTZ,
    status VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboard_filters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    field VARCHAR(255) NOT NULL,
    filter_type VARCHAR(255) NOT NULL,
    default_value JSONB,
    options JSONB NOT NULL,
    linked_widgets JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboard_widget_data_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    widget_id UUID NOT NULL,
    data_source_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL,
    widget_type VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    position_x INTEGER NOT NULL,
    position_y INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    config JSONB NOT NULL,
    data_query JSONB,
    style JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    layout JSONB NOT NULL,
    refresh_interval INTEGER,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    is_template BOOLEAN NOT NULL,
    tags TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_deletion_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID NOT NULL,
    request_type VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    reason TEXT,
    requested_at TIMESTAMPTZ NOT NULL,
    scheduled_for TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    confirmation_token VARCHAR(255) NOT NULL,
    confirmed_at TIMESTAMPTZ,
    processed_by UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS data_export_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID NOT NULL,
    status VARCHAR(255) NOT NULL,
    format VARCHAR(255) NOT NULL,
    include_sections JSONB NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    file_url TEXT,
    file_size INTEGER,
    expires_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS database_query_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query_text TEXT NOT NULL,
    is_mutation BOOLEAN NOT NULL,
    row_count INTEGER,
    duration_ms INTEGER,
    error_message TEXT,
    executed_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS database_saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    query_text TEXT NOT NULL,
    description TEXT,
    is_shared BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS desktop_connection_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id UUID,
    user_id UUID NOT NULL,
    session_id UUID NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR(255) NOT NULL,
    connected_at TIMESTAMPTZ NOT NULL,
    disconnected_at TIMESTAMPTZ,
    bytes_transferred BIGINT,
    disconnect_reason VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS desktop_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR(255) NOT NULL,
    auth_type VARCHAR(255),
    auto_connect BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS distribution_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    email_alias VARCHAR(255),
    description TEXT,
    members_json TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    stalwart_principal_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path TEXT NOT NULL,
    file_type VARCHAR(255) NOT NULL,
    etag TEXT,
    last_modified TIMESTAMPTZ,
    file_size BIGINT,
    indexed BOOLEAN NOT NULL,
    fail_count INTEGER NOT NULL,
    last_failed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_share_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    token VARCHAR(255) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ,
    max_downloads INTEGER,
    download_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_starred (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drive_user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    permission VARCHAR(255) NOT NULL,
    granted_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS email_auto_responders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    responder_type VARCHAR(255) NOT NULL,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    send_to_internal_only BOOLEAN NOT NULL,
    exclude_addresses TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_campaign_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL,
    campaign_id UUID,
    list_id UUID,
    sent_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS email_crm_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL,
    contact_id UUID,
    opportunity_id UUID,
    logged_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS email_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    account_id UUID NOT NULL,
    to_address TEXT NOT NULL,
    cc_address TEXT,
    bcc_address TEXT,
    subject VARCHAR(255),
    body TEXT,
    attachments JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL,
    follow_up_date VARCHAR(255),
    flag_type VARCHAR(255),
    completed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL,
    folder_name VARCHAR(255) NOT NULL,
    folder_path VARCHAR(255) NOT NULL,
    unread_count INTEGER NOT NULL,
    total_count INTEGER NOT NULL,
    last_synced TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_label_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_message_id VARCHAR(255) NOT NULL,
    label_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS email_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(255) NOT NULL,
    parent_id UUID,
    is_system BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_nudges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL,
    last_sent TIMESTAMP,
    dismissed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL,
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    stop_processing BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    is_default BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_snooze (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL,
    snooze_until TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    subject_template TEXT NOT NULL,
    body_html_template TEXT NOT NULL,
    body_plain_template TEXT,
    variables_json TEXT NOT NULL,
    category VARCHAR(255),
    is_shared BOOLEAN NOT NULL,
    usage_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID,
    campaign_id UUID,
    message_id VARCHAR(255),
    open_token UUID,
    open_tracking_enabled BOOLEAN,
    opened BOOLEAN,
    opened_at TIMESTAMPTZ,
    clicked BOOLEAN,
    clicked_at TIMESTAMPTZ,
    ip_address VARCHAR(255),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    feature VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS global_email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    position VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    product_id UUID NOT NULL,
    movement_type VARCHAR(255) NOT NULL,
    quantity INTEGER NOT NULL,
    reference_type VARCHAR(255),
    reference_id UUID,
    notes TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS kb_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    name TEXT NOT NULL,
    folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    document_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS kb_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    collection_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash TEXT NOT NULL,
    first_published_at TIMESTAMPTZ NOT NULL,
    last_modified_at TIMESTAMPTZ NOT NULL,
    indexed_at TIMESTAMPTZ,
    fail_count INTEGER NOT NULL,
    last_failed_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS kb_group_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kb_id UUID NOT NULL,
    group_id UUID NOT NULL,
    granted_by UUID,
    granted_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS learn_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
    parent_id UUID,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learn_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    course_id UUID NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    score INTEGER NOT NULL,
    certificate_url TEXT,
    verification_code TEXT NOT NULL,
    expires_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS learn_course_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL,
    user_id UUID NOT NULL,
    assigned_by UUID,
    due_date TIMESTAMPTZ,
    is_mandatory BOOLEAN NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    reminder_sent BOOLEAN NOT NULL,
    reminder_sent_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS learn_courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL,
    thumbnail_url TEXT,
    is_mandatory BOOLEAN NOT NULL,
    due_days INTEGER,
    is_published BOOLEAN NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learn_lessons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    content_type TEXT NOT NULL,
    lesson_order INTEGER NOT NULL,
    duration_minutes INTEGER NOT NULL,
    video_url TEXT,
    attachments JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learn_quizzes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id UUID,
    course_id UUID NOT NULL,
    title TEXT NOT NULL,
    passing_score INTEGER NOT NULL,
    time_limit_minutes INTEGER,
    max_attempts INTEGER,
    questions JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS learn_user_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    course_id UUID NOT NULL,
    lesson_id UUID,
    status TEXT NOT NULL,
    quiz_score INTEGER,
    quiz_attempts INTEGER NOT NULL,
    time_spent_minutes INTEGER NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    last_accessed_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS legal_acceptances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID NOT NULL,
    document_id UUID NOT NULL,
    document_version VARCHAR(255) NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL,
    ip_address VARCHAR(255),
    user_agent TEXT
);

CREATE TABLE IF NOT EXISTS legal_document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    version VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS legal_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    document_type VARCHAR(255) NOT NULL,
    version VARCHAR(255) NOT NULL,
    effective_date TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    requires_acceptance BOOLEAN NOT NULL,
    metadata JSONB NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketing_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    deal_id UUID,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    channel VARCHAR(255) NOT NULL,
    content_template JSONB NOT NULL,
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metrics JSONB NOT NULL,
    budget DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketing_list_contacts (
    list_id UUID,
    contact_id UUID,
    added_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (list_id, contact_id)
);

CREATE TABLE IF NOT EXISTS marketing_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    list_type VARCHAR(255) NOT NULL,
    query_text TEXT,
    contact_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketing_recipients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID,
    contact_id UUID,
    deal_id UUID,
    channel VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    error_message TEXT,
    response JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS marketing_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    channel VARCHAR(255) NOT NULL,
    subject VARCHAR(255),
    body TEXT,
    media_url VARCHAR(255),
    ai_prompt TEXT,
    variables JSONB NOT NULL,
    approved BOOLEAN,
    meta_template_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL,
    participant_id UUID,
    sender_name VARCHAR(255) NOT NULL,
    message_type VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    reply_to_id UUID,
    is_system_message BOOLEAN NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL,
    user_id UUID,
    participant_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    role VARCHAR(255) NOT NULL,
    is_bot BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    has_video BOOLEAN NOT NULL,
    has_audio BOOLEAN NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL,
    left_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL,
    org_id UUID NOT NULL,
    branch_id UUID,
    recording_type VARCHAR(255) NOT NULL,
    file_url TEXT,
    file_size BIGINT,
    duration_seconds INTEGER,
    status VARCHAR(255) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    stopped_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    room_code VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_by UUID NOT NULL,
    max_participants INTEGER NOT NULL,
    is_recording BOOLEAN NOT NULL,
    is_transcribing BOOLEAN NOT NULL,
    status VARCHAR(255) NOT NULL,
    settings JSONB NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_transcriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL,
    recording_id UUID,
    org_id UUID NOT NULL,
    branch_id UUID,
    participant_id UUID,
    speaker_name VARCHAR(255),
    content TEXT NOT NULL,
    start_time NUMERIC NOT NULL,
    end_time NUMERIC NOT NULL,
    confidence NUMERIC,
    language VARCHAR(255),
    is_final BOOLEAN NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meeting_whiteboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID,
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    background_color VARCHAR(255),
    grid_enabled BOOLEAN NOT NULL,
    grid_size INTEGER,
    elements JSONB NOT NULL,
    version INTEGER NOT NULL,
    created_by UUID NOT NULL,
    last_modified_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS message_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role INTEGER NOT NULL,
    content_encrypted TEXT NOT NULL,
    message_type INTEGER NOT NULL,
    message_index INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    objective_id UUID,
    key_result_id UUID,
    user_id UUID NOT NULL,
    activity_type VARCHAR(255) NOT NULL,
    description TEXT,
    old_value TEXT,
    new_value TEXT,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_alignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    child_objective_id UUID NOT NULL,
    parent_objective_id UUID NOT NULL,
    alignment_type VARCHAR(255) NOT NULL,
    weight NUMERIC NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_checkins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    key_result_id UUID NOT NULL,
    user_id UUID NOT NULL,
    previous_value NUMERIC,
    new_value NUMERIC NOT NULL,
    note TEXT,
    confidence VARCHAR(255),
    blockers TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    objective_id UUID,
    key_result_id UUID,
    user_id UUID NOT NULL,
    content TEXT NOT NULL,
    parent_comment_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_key_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    objective_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    metric_type VARCHAR(255) NOT NULL,
    start_value NUMERIC NOT NULL,
    target_value NUMERIC NOT NULL,
    current_value NUMERIC NOT NULL,
    unit VARCHAR(255),
    weight NUMERIC NOT NULL,
    status VARCHAR(255) NOT NULL,
    due_date VARCHAR(255),
    scoring_type VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_objectives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    owner_id UUID NOT NULL,
    parent_id UUID,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    period VARCHAR(255) NOT NULL,
    period_start VARCHAR(255),
    period_end VARCHAR(255),
    status VARCHAR(255) NOT NULL,
    progress NUMERIC NOT NULL,
    visibility VARCHAR(255) NOT NULL,
    weight NUMERIC NOT NULL,
    tags VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS okr_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(255),
    objective_template JSONB NOT NULL,
    key_result_templates JSONB NOT NULL,
    is_system BOOLEAN NOT NULL,
    usage_count INTEGER NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS organization_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    message TEXT,
    invited_by UUID NOT NULL,
    token VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,
    accepted_by UUID
);

CREATE TABLE IF NOT EXISTS organizations (
    org_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    user_id UUID,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255),
    email VARCHAR(255),
    phone VARCHAR(255),
    mobile VARCHAR(255),
    job_title VARCHAR(255),
    department VARCHAR(255),
    manager_id UUID,
    office_location VARCHAR(255),
    hire_date VARCHAR(255),
    birthday VARCHAR(255),
    avatar_url TEXT,
    bio TEXT,
    skills TEXT[] NOT NULL,
    social_links JSONB NOT NULL,
    custom_fields JSONB NOT NULL,
    timezone VARCHAR(255),
    locale VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    code VARCHAR(255),
    parent_id UUID,
    head_id UUID,
    cost_center VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_org_chart (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    person_id UUID NOT NULL,
    reports_to_id UUID,
    position_title VARCHAR(255),
    position_level INTEGER NOT NULL,
    position_order INTEGER NOT NULL,
    effective_from VARCHAR(255),
    effective_until VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_person_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id UUID NOT NULL,
    skill_id UUID NOT NULL,
    proficiency_level INTEGER NOT NULL,
    years_experience NUMERIC,
    verified_by UUID,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(255),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL,
    person_id UUID NOT NULL,
    role VARCHAR(255),
    is_primary BOOLEAN NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS people_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    leader_id UUID,
    parent_team_id UUID,
    color VARCHAR(255),
    icon VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS people_time_off (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    person_id UUID NOT NULL,
    time_off_type VARCHAR(255) NOT NULL,
    status VARCHAR(255) NOT NULL,
    start_date VARCHAR(255) NOT NULL,
    end_date VARCHAR(255) NOT NULL,
    hours_requested NUMERIC,
    reason TEXT,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS price_list_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    price_list_id UUID NOT NULL,
    product_id UUID,
    service_id UUID,
    price NUMERIC NOT NULL,
    min_quantity INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS price_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    currency VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL,
    valid_from VARCHAR(255),
    valid_until VARCHAR(255),
    customer_group VARCHAR(255),
    discount_percent NUMERIC NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS product_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID,
    slug VARCHAR(255),
    image_url TEXT,
    sort_order INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    sku VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    price_adjustment NUMERIC NOT NULL,
    stock_quantity INTEGER NOT NULL,
    attributes JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    global_trade_number VARCHAR(255),
    net_weight NUMERIC,
    gross_weight NUMERIC,
    width NUMERIC,
    height NUMERIC,
    length NUMERIC,
    color VARCHAR(255),
    size VARCHAR(255),
    images JSONB
);

CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    sku VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(255),
    product_type VARCHAR(255) NOT NULL,
    price NUMERIC NOT NULL,
    cost NUMERIC,
    currency VARCHAR(255) NOT NULL,
    tax_rate NUMERIC NOT NULL,
    unit VARCHAR(255) NOT NULL,
    stock_quantity INTEGER NOT NULL,
    low_stock_threshold INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    images JSONB NOT NULL,
    attributes JSONB NOT NULL,
    weight NUMERIC,
    dimensions JSONB,
    barcode VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS project_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    resource_id UUID NOT NULL,
    units VARCHAR(255) NOT NULL,
    work_hours VARCHAR(255) NOT NULL,
    start_date VARCHAR(255) NOT NULL,
    end_date VARCHAR(255) NOT NULL,
    cost DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS project_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    predecessor_id UUID NOT NULL,
    dependency_type TEXT NOT NULL,
    lag_days INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS project_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    user_id UUID,
    name TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    email TEXT,
    max_units VARCHAR(255) NOT NULL,
    standard_rate DOUBLE PRECISION,
    overtime_rate DOUBLE PRECISION,
    cost_per_use DOUBLE PRECISION,
    calendar_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS project_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    parent_id UUID,
    name TEXT NOT NULL,
    description TEXT,
    task_type TEXT NOT NULL,
    start_date VARCHAR(255) NOT NULL,
    end_date VARCHAR(255) NOT NULL,
    duration_days INTEGER NOT NULL,
    percent_complete INTEGER NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    assigned_to VARCHAR(255) NOT NULL,
    estimated_hours VARCHAR(255),
    actual_hours VARCHAR(255),
    cost DOUBLE PRECISION,
    notes TEXT,
    wbs TEXT NOT NULL,
    outline_level INTEGER NOT NULL,
    is_milestone BOOLEAN NOT NULL,
    is_summary BOOLEAN NOT NULL,
    is_critical BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    start_date VARCHAR(255) NOT NULL,
    end_date VARCHAR(255),
    status TEXT NOT NULL,
    owner_id UUID NOT NULL,
    settings JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rbac_group_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL,
    role_id UUID NOT NULL,
    granted_by UUID,
    granted_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS rbac_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_group_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rbac_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL,
    category VARCHAR(255) NOT NULL,
    is_system BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rbac_role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    granted_by UUID,
    granted_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS rbac_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rbac_user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    group_id UUID NOT NULL,
    added_by UUID,
    added_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS rbac_user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    granted_by UUID,
    granted_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS research_citations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL,
    citation_style VARCHAR(255) NOT NULL,
    formatted_citation TEXT NOT NULL,
    bibtex TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS research_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(255) NOT NULL,
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS research_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    export_type VARCHAR(255) NOT NULL,
    format VARCHAR(255) NOT NULL,
    file_url TEXT,
    file_size INTEGER,
    status VARCHAR(255) NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS research_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    finding_type VARCHAR(255) NOT NULL,
    confidence_level VARCHAR(255),
    supporting_sources JSONB NOT NULL,
    related_findings JSONB NOT NULL,
    status VARCHAR(255) NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS research_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    source_id UUID,
    title VARCHAR(255),
    content TEXT NOT NULL,
    note_type VARCHAR(255) NOT NULL,
    tags VARCHAR(255) NOT NULL,
    highlight_text TEXT,
    highlight_position JSONB,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS research_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(255) NOT NULL,
    owner_id UUID NOT NULL,
    tags VARCHAR(255) NOT NULL,
    settings JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS research_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    source_type VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    url TEXT,
    content TEXT,
    summary TEXT,
    metadata JSONB NOT NULL,
    credibility_score INTEGER,
    is_verified BOOLEAN NOT NULL,
    added_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduled_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    to_addresses TEXT NOT NULL,
    cc_addresses TEXT,
    bcc_addresses TEXT,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    attachments_json TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    status VARCHAR(255) NOT NULL,
    retry_count INTEGER NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduled_meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    room_id UUID,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    organizer_id UUID NOT NULL,
    scheduled_start TIMESTAMPTZ NOT NULL,
    scheduled_end TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(255) NOT NULL,
    recurrence_rule TEXT,
    attendees JSONB NOT NULL,
    settings JSONB NOT NULL,
    status VARCHAR(255) NOT NULL,
    reminder_sent BOOLEAN NOT NULL,
    calendar_event_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(255),
    service_type VARCHAR(255) NOT NULL,
    hourly_rate NUMERIC,
    fixed_price NUMERIC,
    currency VARCHAR(255) NOT NULL,
    duration_minutes INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    attributes JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS session_tool_associations (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    added_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS shared_mailbox_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID NOT NULL,
    user_id UUID NOT NULL,
    permission_level VARCHAR(255) NOT NULL,
    added_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS shared_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    settings_json TEXT NOT NULL,
    stalwart_account_id VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_announcements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    author_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    priority VARCHAR(255) NOT NULL,
    target_audience JSONB NOT NULL,
    is_pinned BOOLEAN NOT NULL,
    requires_acknowledgment BOOLEAN NOT NULL,
    acknowledged_by JSONB NOT NULL,
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    post_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_channel_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    channel_type VARCHAR(255) NOT NULL,
    credentials JSONB NOT NULL,
    settings JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL,
    parent_comment_id UUID,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    mentions JSONB NOT NULL,
    reaction_counts JSONB NOT NULL,
    reply_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS social_communities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image TEXT,
    icon TEXT,
    visibility VARCHAR(255) NOT NULL,
    join_policy VARCHAR(255) NOT NULL,
    owner_id UUID NOT NULL,
    member_count INTEGER NOT NULL,
    post_count INTEGER NOT NULL,
    is_official BOOLEAN NOT NULL,
    is_featured BOOLEAN NOT NULL,
    settings JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS social_community_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(255) NOT NULL,
    notifications_enabled BOOLEAN NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS social_hashtags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    tag VARCHAR(255) NOT NULL,
    post_count INTEGER NOT NULL,
    last_used_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_poll_options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL,
    text VARCHAR(255) NOT NULL,
    vote_count INTEGER NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_poll_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL,
    option_id UUID NOT NULL,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL,
    question TEXT NOT NULL,
    allow_multiple BOOLEAN NOT NULL,
    allow_add_options BOOLEAN NOT NULL,
    anonymous BOOLEAN NOT NULL,
    total_votes INTEGER NOT NULL,
    ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    author_id UUID NOT NULL,
    community_id UUID,
    parent_id UUID,
    content TEXT NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    attachments JSONB NOT NULL,
    mentions JSONB NOT NULL,
    hashtags VARCHAR(255) NOT NULL,
    visibility VARCHAR(255) NOT NULL,
    is_announcement BOOLEAN NOT NULL,
    is_pinned BOOLEAN NOT NULL,
    poll_id UUID,
    reaction_counts JSONB NOT NULL,
    comment_count INTEGER NOT NULL,
    share_count INTEGER NOT NULL,
    view_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS social_praises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    badge_type VARCHAR(255) NOT NULL,
    message TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    post_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS social_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID,
    comment_id UUID,
    user_id UUID NOT NULL,
    reaction_type VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS support_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    ticket_number VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(255) NOT NULL,
    priority VARCHAR(255) NOT NULL,
    category VARCHAR(255),
    source VARCHAR(255) NOT NULL,
    requester_id UUID,
    requester_email VARCHAR(255),
    requester_name VARCHAR(255),
    assignee_id UUID,
    team_id UUID,
    due_date TIMESTAMPTZ,
    first_response_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    satisfaction_rating INTEGER,
    tags TEXT[] NOT NULL,
    custom_fields JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS system_automations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    kind INTEGER NOT NULL,
    target TEXT,
    schedule TEXT,
    param TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    assignee_id UUID,
    reporter_id UUID,
    project_id UUID,
    due_date TIMESTAMPTZ,
    tags TEXT[] NOT NULL,
    dependencies UUID[] NOT NULL,
    estimated_hours DOUBLE PRECISION,
    actual_hours DOUBLE PRECISION,
    progress INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_canned_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    category VARCHAR(255),
    shortcut VARCHAR(255),
    created_by UUID,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    parent_id UUID,
    color VARCHAR(255),
    icon VARCHAR(255),
    sort_order INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL,
    author_id UUID,
    author_name VARCHAR(255),
    author_email VARCHAR(255),
    content TEXT NOT NULL,
    is_internal BOOLEAN NOT NULL,
    attachments JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_sla_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    priority VARCHAR(255) NOT NULL,
    first_response_hours INTEGER NOT NULL,
    resolution_hours INTEGER NOT NULL,
    business_hours_only BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    imap_server VARCHAR(255) NOT NULL,
    imap_port INTEGER NOT NULL,
    smtp_server VARCHAR(255) NOT NULL,
    smtp_port INTEGER NOT NULL,
    username VARCHAR(255) NOT NULL,
    password_encrypted TEXT NOT NULL,
    is_primary BOOLEAN NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_kb_associations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    bot_id TEXT,
    kb_name TEXT NOT NULL,
    is_website INTEGER NOT NULL,
    website_url TEXT,
    created_at TEXT NOT NULL DEFAULT NOW(),
    updated_at TEXT NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_login_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ NOT NULL,
    user_agent TEXT,
    ip_address VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS user_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    org_id UUID NOT NULL,
    role VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    preference_key VARCHAR(255) NOT NULL,
    preference_value JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bot_id UUID,
    title TEXT NOT NULL,
    context_data JSONB NOT NULL,
    current_tool TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_admin BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS website_crawls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    url TEXT NOT NULL,
    last_crawled TIMESTAMPTZ,
    next_crawl TIMESTAMPTZ,
    expires_policy VARCHAR(255) NOT NULL,
    max_depth INTEGER,
    max_pages INTEGER,
    crawl_status SMALLINT,
    pages_crawled INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    refresh_policy VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS whatsapp_business (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number_id VARCHAR(255),
    business_account_id VARCHAR(255),
    access_token TEXT,
    webhooks_verified BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS whiteboard_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    whiteboard_id UUID NOT NULL,
    element_type VARCHAR(255) NOT NULL,
    position_x NUMERIC NOT NULL,
    position_y NUMERIC NOT NULL,
    width NUMERIC,
    height NUMERIC,
    rotation NUMERIC,
    z_index INTEGER NOT NULL,
    properties JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS whiteboard_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    whiteboard_id UUID NOT NULL,
    org_id UUID NOT NULL,
    branch_id UUID,
    export_format VARCHAR(255) NOT NULL,
    file_url TEXT,
    file_size BIGINT,
    status VARCHAR(255) NOT NULL,
    error_message TEXT,
    requested_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS workflow_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL,
    workflow_id UUID NOT NULL,
    event_name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    event_data_json JSONB,
    processed BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID,
    workflow_name TEXT NOT NULL,
    current_step INTEGER,
    state_json JSONB,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_comment_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id UUID NOT NULL,
    user_id UUID NOT NULL,
    emoji VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    page_id UUID NOT NULL,
    block_id UUID,
    parent_comment_id UUID,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    resolved BOOLEAN NOT NULL,
    resolved_by UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(255) NOT NULL,
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS workspace_page_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL,
    user_id UUID,
    role VARCHAR(255),
    permission VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_page_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id UUID NOT NULL,
    version_number INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    content JSONB NOT NULL,
    change_summary TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    parent_id UUID,
    title VARCHAR(255) NOT NULL,
    icon_type VARCHAR(255),
    icon_value VARCHAR(255),
    cover_image TEXT,
    content JSONB NOT NULL,
    properties JSONB NOT NULL,
    is_template BOOLEAN NOT NULL,
    template_id UUID,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    public_edit BOOLEAN NOT NULL,
    position INTEGER NOT NULL,
    created_by UUID NOT NULL,
    last_edited_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(255),
    icon_type VARCHAR(255),
    icon_value VARCHAR(255),
    cover_image TEXT,
    content JSONB NOT NULL,
    is_system BOOLEAN NOT NULL,
    usage_count INTEGER NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon_type VARCHAR(255),
    icon_value VARCHAR(255),
    cover_image TEXT,
    settings JSONB NOT NULL,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Foreign keys (extracted from CREATE TABLE to avoid ordering issues)

ALTER TABLE ONLY attendance_sla_events ADD CONSTRAINT fk_attendance_sla_events_sla_policy_id FOREIGN KEY (sla_policy_id) REFERENCES attendance_sla_policies(id) ON DELETE CASCADE;
ALTER TABLE ONLY attendance_sla_policies ADD CONSTRAINT fk_attendance_sla_policies_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendance_sla_policies ADD CONSTRAINT fk_attendance_sla_policies_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendance_webhooks ADD CONSTRAINT fk_attendance_webhooks_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendance_webhooks ADD CONSTRAINT fk_attendance_webhooks_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_agent_status ADD CONSTRAINT fk_attendant_agent_status_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_agent_status ADD CONSTRAINT fk_attendant_agent_status_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_canned_responses ADD CONSTRAINT fk_attendant_canned_responses_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_canned_responses ADD CONSTRAINT fk_attendant_canned_responses_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_queue_agents ADD CONSTRAINT fk_attendant_queue_agents_queue_id FOREIGN KEY (queue_id) REFERENCES attendant_queues(id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_queues ADD CONSTRAINT fk_attendant_queues_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_queues ADD CONSTRAINT fk_attendant_queues_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_session_messages ADD CONSTRAINT fk_attendant_session_messages_session_id FOREIGN KEY (session_id) REFERENCES attendant_sessions(id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_session_wrap_up ADD CONSTRAINT fk_attendant_session_wrap_up_session_id FOREIGN KEY (session_id) REFERENCES attendant_sessions(id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_session_wrap_up ADD CONSTRAINT fk_attendant_session_wrap_up_wrap_up_code_id FOREIGN KEY (wrap_up_code_id) REFERENCES attendant_wrap_up_codes(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_sessions ADD CONSTRAINT fk_attendant_sessions_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_sessions ADD CONSTRAINT fk_attendant_sessions_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_sessions ADD CONSTRAINT fk_attendant_sessions_queue_id FOREIGN KEY (queue_id) REFERENCES attendant_queues(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_tags ADD CONSTRAINT fk_attendant_tags_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_tags ADD CONSTRAINT fk_attendant_tags_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY attendant_transfers ADD CONSTRAINT fk_attendant_transfers_session_id FOREIGN KEY (session_id) REFERENCES attendant_sessions(id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_wrap_up_codes ADD CONSTRAINT fk_attendant_wrap_up_codes_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY attendant_wrap_up_codes ADD CONSTRAINT fk_attendant_wrap_up_codes_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_alert_history ADD CONSTRAINT fk_billing_alert_history_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_alert_history ADD CONSTRAINT fk_billing_alert_history_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_grace_periods ADD CONSTRAINT fk_billing_grace_periods_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_grace_periods ADD CONSTRAINT fk_billing_grace_periods_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_invoice_items ADD CONSTRAINT fk_billing_invoice_items_invoice_id FOREIGN KEY (invoice_id) REFERENCES billing_invoices(id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_invoices ADD CONSTRAINT fk_billing_invoices_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_invoices ADD CONSTRAINT fk_billing_invoices_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_notification_preferences ADD CONSTRAINT fk_billing_notification_preferences_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_notification_preferences ADD CONSTRAINT fk_billing_notification_preferences_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_payments ADD CONSTRAINT fk_billing_payments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_payments ADD CONSTRAINT fk_billing_payments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_payments ADD CONSTRAINT fk_billing_payments_invoice_id FOREIGN KEY (invoice_id) REFERENCES billing_invoices(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_quote_items ADD CONSTRAINT fk_billing_quote_items_quote_id FOREIGN KEY (quote_id) REFERENCES billing_quotes(id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_quotes ADD CONSTRAINT fk_billing_quotes_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_quotes ADD CONSTRAINT fk_billing_quotes_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_recurring ADD CONSTRAINT fk_billing_recurring_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_recurring ADD CONSTRAINT fk_billing_recurring_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_tax_rates ADD CONSTRAINT fk_billing_tax_rates_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_tax_rates ADD CONSTRAINT fk_billing_tax_rates_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY billing_usage_alerts ADD CONSTRAINT fk_billing_usage_alerts_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY billing_usage_alerts ADD CONSTRAINT fk_billing_usage_alerts_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY bot_configuration ADD CONSTRAINT fk_bot_configuration_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY bot_memories ADD CONSTRAINT fk_bot_memories_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY bots ADD CONSTRAINT fk_bots_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY bots ADD CONSTRAINT fk_bots_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE;
ALTER TABLE ONLY branches ADD CONSTRAINT fk_branches_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY branches ADD CONSTRAINT fk_branches_tenant_id FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;
ALTER TABLE ONLY calendar_event_attendees ADD CONSTRAINT fk_calendar_event_attendees_event_id FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE;
ALTER TABLE ONLY calendar_event_reminders ADD CONSTRAINT fk_calendar_event_reminders_event_id FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE;
ALTER TABLE ONLY calendar_events ADD CONSTRAINT fk_calendar_events_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY calendar_events ADD CONSTRAINT fk_calendar_events_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY calendar_events ADD CONSTRAINT fk_calendar_events_calendar_id FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE;
ALTER TABLE ONLY calendar_shares ADD CONSTRAINT fk_calendar_shares_calendar_id FOREIGN KEY (calendar_id) REFERENCES calendars(id) ON DELETE CASCADE;
ALTER TABLE ONLY calendars ADD CONSTRAINT fk_calendars_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY calendars ADD CONSTRAINT fk_calendars_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY canvas_collaborators ADD CONSTRAINT fk_canvas_collaborators_canvas_id FOREIGN KEY (canvas_id) REFERENCES canvases(id) ON DELETE CASCADE;
ALTER TABLE ONLY canvas_comments ADD CONSTRAINT fk_canvas_comments_canvas_id FOREIGN KEY (canvas_id) REFERENCES canvases(id) ON DELETE CASCADE;
ALTER TABLE ONLY canvas_elements ADD CONSTRAINT fk_canvas_elements_canvas_id FOREIGN KEY (canvas_id) REFERENCES canvases(id) ON DELETE CASCADE;
ALTER TABLE ONLY canvas_versions ADD CONSTRAINT fk_canvas_versions_canvas_id FOREIGN KEY (canvas_id) REFERENCES canvases(id) ON DELETE CASCADE;
ALTER TABLE ONLY canvases ADD CONSTRAINT fk_canvases_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY canvases ADD CONSTRAINT fk_canvases_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_access_reviews ADD CONSTRAINT fk_compliance_access_reviews_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_access_reviews ADD CONSTRAINT fk_compliance_access_reviews_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_audit_log ADD CONSTRAINT fk_compliance_audit_log_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_audit_log ADD CONSTRAINT fk_compliance_audit_log_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_checks ADD CONSTRAINT fk_compliance_checks_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_checks ADD CONSTRAINT fk_compliance_checks_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_evidence ADD CONSTRAINT fk_compliance_evidence_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_evidence ADD CONSTRAINT fk_compliance_evidence_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_issues ADD CONSTRAINT fk_compliance_issues_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_issues ADD CONSTRAINT fk_compliance_issues_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_issues ADD CONSTRAINT fk_compliance_issues_check_id FOREIGN KEY (check_id) REFERENCES compliance_checks(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_risk_assessments ADD CONSTRAINT fk_compliance_risk_assessments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_risk_assessments ADD CONSTRAINT fk_compliance_risk_assessments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY compliance_risks ADD CONSTRAINT fk_compliance_risks_assessment_id FOREIGN KEY (assessment_id) REFERENCES compliance_risk_assessments(id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_training_records ADD CONSTRAINT fk_compliance_training_records_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY compliance_training_records ADD CONSTRAINT fk_compliance_training_records_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY consent_history ADD CONSTRAINT fk_consent_history_consent_id FOREIGN KEY (consent_id) REFERENCES cookie_consents(id) ON DELETE CASCADE;
ALTER TABLE ONLY conversational_queries ADD CONSTRAINT fk_conversational_queries_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY conversational_queries ADD CONSTRAINT fk_conversational_queries_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY cookie_consents ADD CONSTRAINT fk_cookie_consents_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY cookie_consents ADD CONSTRAINT fk_cookie_consents_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_accounts ADD CONSTRAINT fk_crm_accounts_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_accounts ADD CONSTRAINT fk_crm_accounts_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_activities ADD CONSTRAINT fk_crm_activities_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_activities ADD CONSTRAINT fk_crm_activities_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_contacts ADD CONSTRAINT fk_crm_contacts_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_contacts ADD CONSTRAINT fk_crm_contacts_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_deal_segments ADD CONSTRAINT fk_crm_deal_segments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_deal_segments ADD CONSTRAINT fk_crm_deal_segments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_deals ADD CONSTRAINT fk_crm_deals_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_deals ADD CONSTRAINT fk_crm_deals_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_deals ADD CONSTRAINT fk_crm_deals_department_id FOREIGN KEY (department_id) REFERENCES people_departments(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_leads ADD CONSTRAINT fk_crm_leads_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_leads ADD CONSTRAINT fk_crm_leads_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_notes ADD CONSTRAINT fk_crm_notes_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_notes ADD CONSTRAINT fk_crm_notes_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_opportunities ADD CONSTRAINT fk_crm_opportunities_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_opportunities ADD CONSTRAINT fk_crm_opportunities_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY crm_pipeline_stages ADD CONSTRAINT fk_crm_pipeline_stages_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY crm_pipeline_stages ADD CONSTRAINT fk_crm_pipeline_stages_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY dashboard_data_sources ADD CONSTRAINT fk_dashboard_data_sources_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboard_data_sources ADD CONSTRAINT fk_dashboard_data_sources_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY dashboard_filters ADD CONSTRAINT fk_dashboard_filters_dashboard_id FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboard_widget_data_sources ADD CONSTRAINT fk_dashboard_widget_data_sources_widget_id FOREIGN KEY (widget_id) REFERENCES dashboard_widgets(id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboard_widget_data_sources ADD CONSTRAINT fk_dashboard_widget_data_sources_data_source_id FOREIGN KEY (data_source_id) REFERENCES dashboard_data_sources(id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboard_widgets ADD CONSTRAINT fk_dashboard_widgets_dashboard_id FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboards ADD CONSTRAINT fk_dashboards_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY dashboards ADD CONSTRAINT fk_dashboards_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY data_deletion_requests ADD CONSTRAINT fk_data_deletion_requests_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY data_deletion_requests ADD CONSTRAINT fk_data_deletion_requests_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY data_export_requests ADD CONSTRAINT fk_data_export_requests_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY data_export_requests ADD CONSTRAINT fk_data_export_requests_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY desktop_connection_log ADD CONSTRAINT fk_desktop_connection_log_connection_id FOREIGN KEY (connection_id) REFERENCES desktop_connections(id) ON DELETE SET NULL;
ALTER TABLE ONLY feature_flags ADD CONSTRAINT fk_feature_flags_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY feature_flags ADD CONSTRAINT fk_feature_flags_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY inventory_movements ADD CONSTRAINT fk_inventory_movements_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY inventory_movements ADD CONSTRAINT fk_inventory_movements_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY inventory_movements ADD CONSTRAINT fk_inventory_movements_product_id FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE;
ALTER TABLE ONLY kb_collections ADD CONSTRAINT fk_kb_collections_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY kb_documents ADD CONSTRAINT fk_kb_documents_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY kb_group_associations ADD CONSTRAINT fk_kb_group_associations_kb_id FOREIGN KEY (kb_id) REFERENCES kb_collections(id) ON DELETE CASCADE;
ALTER TABLE ONLY kb_group_associations ADD CONSTRAINT fk_kb_group_associations_group_id FOREIGN KEY (group_id) REFERENCES rbac_groups(id) ON DELETE CASCADE;
ALTER TABLE ONLY kb_group_associations ADD CONSTRAINT fk_kb_group_associations_granted_by FOREIGN KEY (granted_by) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE ONLY legal_acceptances ADD CONSTRAINT fk_legal_acceptances_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY legal_acceptances ADD CONSTRAINT fk_legal_acceptances_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY legal_acceptances ADD CONSTRAINT fk_legal_acceptances_document_id FOREIGN KEY (document_id) REFERENCES legal_documents(id) ON DELETE CASCADE;
ALTER TABLE ONLY legal_document_versions ADD CONSTRAINT fk_legal_document_versions_document_id FOREIGN KEY (document_id) REFERENCES legal_documents(id) ON DELETE CASCADE;
ALTER TABLE ONLY legal_documents ADD CONSTRAINT fk_legal_documents_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY legal_documents ADD CONSTRAINT fk_legal_documents_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY marketing_campaigns ADD CONSTRAINT fk_marketing_campaigns_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY marketing_campaigns ADD CONSTRAINT fk_marketing_campaigns_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY marketing_lists ADD CONSTRAINT fk_marketing_lists_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY marketing_lists ADD CONSTRAINT fk_marketing_lists_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY marketing_templates ADD CONSTRAINT fk_marketing_templates_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY marketing_templates ADD CONSTRAINT fk_marketing_templates_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_chat_messages ADD CONSTRAINT fk_meeting_chat_messages_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_chat_messages ADD CONSTRAINT fk_meeting_chat_messages_participant_id FOREIGN KEY (participant_id) REFERENCES meeting_participants(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_participants ADD CONSTRAINT fk_meeting_participants_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_recordings ADD CONSTRAINT fk_meeting_recordings_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_recordings ADD CONSTRAINT fk_meeting_recordings_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_recordings ADD CONSTRAINT fk_meeting_recordings_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_rooms ADD CONSTRAINT fk_meeting_rooms_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_rooms ADD CONSTRAINT fk_meeting_rooms_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_transcriptions ADD CONSTRAINT fk_meeting_transcriptions_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_transcriptions ADD CONSTRAINT fk_meeting_transcriptions_recording_id FOREIGN KEY (recording_id) REFERENCES meeting_recordings(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_transcriptions ADD CONSTRAINT fk_meeting_transcriptions_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_transcriptions ADD CONSTRAINT fk_meeting_transcriptions_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_transcriptions ADD CONSTRAINT fk_meeting_transcriptions_participant_id FOREIGN KEY (participant_id) REFERENCES meeting_participants(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_whiteboards ADD CONSTRAINT fk_meeting_whiteboards_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE SET NULL;
ALTER TABLE ONLY meeting_whiteboards ADD CONSTRAINT fk_meeting_whiteboards_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY meeting_whiteboards ADD CONSTRAINT fk_meeting_whiteboards_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_activity_log ADD CONSTRAINT fk_okr_activity_log_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_activity_log ADD CONSTRAINT fk_okr_activity_log_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_alignments ADD CONSTRAINT fk_okr_alignments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_alignments ADD CONSTRAINT fk_okr_alignments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_checkins ADD CONSTRAINT fk_okr_checkins_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_checkins ADD CONSTRAINT fk_okr_checkins_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_checkins ADD CONSTRAINT fk_okr_checkins_key_result_id FOREIGN KEY (key_result_id) REFERENCES okr_key_results(id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_comments ADD CONSTRAINT fk_okr_comments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_comments ADD CONSTRAINT fk_okr_comments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_key_results ADD CONSTRAINT fk_okr_key_results_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_key_results ADD CONSTRAINT fk_okr_key_results_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_key_results ADD CONSTRAINT fk_okr_key_results_objective_id FOREIGN KEY (objective_id) REFERENCES okr_objectives(id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_objectives ADD CONSTRAINT fk_okr_objectives_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_objectives ADD CONSTRAINT fk_okr_objectives_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY okr_templates ADD CONSTRAINT fk_okr_templates_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY okr_templates ADD CONSTRAINT fk_okr_templates_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY organization_invitations ADD CONSTRAINT fk_organization_invitations_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY organization_invitations ADD CONSTRAINT fk_organization_invitations_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY organizations ADD CONSTRAINT fk_organizations_tenant_id FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;
ALTER TABLE ONLY people ADD CONSTRAINT fk_people_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people ADD CONSTRAINT fk_people_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY people_departments ADD CONSTRAINT fk_people_departments_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people_departments ADD CONSTRAINT fk_people_departments_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY people_org_chart ADD CONSTRAINT fk_people_org_chart_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people_org_chart ADD CONSTRAINT fk_people_org_chart_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY people_person_skills ADD CONSTRAINT fk_people_person_skills_person_id FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE;
ALTER TABLE ONLY people_person_skills ADD CONSTRAINT fk_people_person_skills_skill_id FOREIGN KEY (skill_id) REFERENCES people_skills(id) ON DELETE CASCADE;
ALTER TABLE ONLY people_skills ADD CONSTRAINT fk_people_skills_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people_skills ADD CONSTRAINT fk_people_skills_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY people_team_members ADD CONSTRAINT fk_people_team_members_team_id FOREIGN KEY (team_id) REFERENCES people_teams(id) ON DELETE CASCADE;
ALTER TABLE ONLY people_team_members ADD CONSTRAINT fk_people_team_members_person_id FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE;
ALTER TABLE ONLY people_teams ADD CONSTRAINT fk_people_teams_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people_teams ADD CONSTRAINT fk_people_teams_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY people_time_off ADD CONSTRAINT fk_people_time_off_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY people_time_off ADD CONSTRAINT fk_people_time_off_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY price_list_items ADD CONSTRAINT fk_price_list_items_price_list_id FOREIGN KEY (price_list_id) REFERENCES price_lists(id) ON DELETE CASCADE;
ALTER TABLE ONLY price_list_items ADD CONSTRAINT fk_price_list_items_product_id FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL;
ALTER TABLE ONLY price_list_items ADD CONSTRAINT fk_price_list_items_service_id FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE SET NULL;
ALTER TABLE ONLY price_lists ADD CONSTRAINT fk_price_lists_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY price_lists ADD CONSTRAINT fk_price_lists_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY product_categories ADD CONSTRAINT fk_product_categories_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY product_categories ADD CONSTRAINT fk_product_categories_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY product_variants ADD CONSTRAINT fk_product_variants_product_id FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE;
ALTER TABLE ONLY products ADD CONSTRAINT fk_products_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY products ADD CONSTRAINT fk_products_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY project_assignments ADD CONSTRAINT fk_project_assignments_task_id FOREIGN KEY (task_id) REFERENCES project_tasks(id) ON DELETE CASCADE;
ALTER TABLE ONLY project_assignments ADD CONSTRAINT fk_project_assignments_resource_id FOREIGN KEY (resource_id) REFERENCES project_resources(id) ON DELETE CASCADE;
ALTER TABLE ONLY project_dependencies ADD CONSTRAINT fk_project_dependencies_task_id FOREIGN KEY (task_id) REFERENCES project_tasks(id) ON DELETE CASCADE;
ALTER TABLE ONLY project_resources ADD CONSTRAINT fk_project_resources_project_id FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY project_tasks ADD CONSTRAINT fk_project_tasks_project_id FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_group_roles ADD CONSTRAINT fk_rbac_group_roles_group_id FOREIGN KEY (group_id) REFERENCES rbac_groups(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_group_roles ADD CONSTRAINT fk_rbac_group_roles_role_id FOREIGN KEY (role_id) REFERENCES rbac_roles(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_role_permissions ADD CONSTRAINT fk_rbac_role_permissions_role_id FOREIGN KEY (role_id) REFERENCES rbac_roles(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_role_permissions ADD CONSTRAINT fk_rbac_role_permissions_permission_id FOREIGN KEY (permission_id) REFERENCES rbac_permissions(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_user_groups ADD CONSTRAINT fk_rbac_user_groups_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_user_groups ADD CONSTRAINT fk_rbac_user_groups_group_id FOREIGN KEY (group_id) REFERENCES rbac_groups(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_user_roles ADD CONSTRAINT fk_rbac_user_roles_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE ONLY rbac_user_roles ADD CONSTRAINT fk_rbac_user_roles_role_id FOREIGN KEY (role_id) REFERENCES rbac_roles(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_citations ADD CONSTRAINT fk_research_citations_source_id FOREIGN KEY (source_id) REFERENCES research_sources(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_collaborators ADD CONSTRAINT fk_research_collaborators_project_id FOREIGN KEY (project_id) REFERENCES research_projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_exports ADD CONSTRAINT fk_research_exports_project_id FOREIGN KEY (project_id) REFERENCES research_projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_findings ADD CONSTRAINT fk_research_findings_project_id FOREIGN KEY (project_id) REFERENCES research_projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_notes ADD CONSTRAINT fk_research_notes_project_id FOREIGN KEY (project_id) REFERENCES research_projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY research_projects ADD CONSTRAINT fk_research_projects_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY research_projects ADD CONSTRAINT fk_research_projects_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY research_sources ADD CONSTRAINT fk_research_sources_project_id FOREIGN KEY (project_id) REFERENCES research_projects(id) ON DELETE CASCADE;
ALTER TABLE ONLY scheduled_meetings ADD CONSTRAINT fk_scheduled_meetings_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY scheduled_meetings ADD CONSTRAINT fk_scheduled_meetings_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY scheduled_meetings ADD CONSTRAINT fk_scheduled_meetings_room_id FOREIGN KEY (room_id) REFERENCES meeting_rooms(id) ON DELETE SET NULL;
ALTER TABLE ONLY services ADD CONSTRAINT fk_services_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY services ADD CONSTRAINT fk_services_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_announcements ADD CONSTRAINT fk_social_announcements_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_announcements ADD CONSTRAINT fk_social_announcements_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_bookmarks ADD CONSTRAINT fk_social_bookmarks_post_id FOREIGN KEY (post_id) REFERENCES social_posts(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_channel_accounts ADD CONSTRAINT fk_social_channel_accounts_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_channel_accounts ADD CONSTRAINT fk_social_channel_accounts_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_comments ADD CONSTRAINT fk_social_comments_post_id FOREIGN KEY (post_id) REFERENCES social_posts(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_communities ADD CONSTRAINT fk_social_communities_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_communities ADD CONSTRAINT fk_social_communities_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_community_members ADD CONSTRAINT fk_social_community_members_community_id FOREIGN KEY (community_id) REFERENCES social_communities(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_hashtags ADD CONSTRAINT fk_social_hashtags_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_hashtags ADD CONSTRAINT fk_social_hashtags_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_poll_options ADD CONSTRAINT fk_social_poll_options_poll_id FOREIGN KEY (poll_id) REFERENCES social_polls(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_poll_votes ADD CONSTRAINT fk_social_poll_votes_poll_id FOREIGN KEY (poll_id) REFERENCES social_polls(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_poll_votes ADD CONSTRAINT fk_social_poll_votes_option_id FOREIGN KEY (option_id) REFERENCES social_poll_options(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_polls ADD CONSTRAINT fk_social_polls_post_id FOREIGN KEY (post_id) REFERENCES social_posts(id) ON DELETE CASCADE;
ALTER TABLE ONLY social_posts ADD CONSTRAINT fk_social_posts_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_posts ADD CONSTRAINT fk_social_posts_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY social_praises ADD CONSTRAINT fk_social_praises_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY social_praises ADD CONSTRAINT fk_social_praises_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY support_tickets ADD CONSTRAINT fk_support_tickets_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY support_tickets ADD CONSTRAINT fk_support_tickets_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY system_automations ADD CONSTRAINT fk_system_automations_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY ticket_canned_responses ADD CONSTRAINT fk_ticket_canned_responses_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY ticket_canned_responses ADD CONSTRAINT fk_ticket_canned_responses_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY ticket_categories ADD CONSTRAINT fk_ticket_categories_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY ticket_categories ADD CONSTRAINT fk_ticket_categories_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY ticket_sla_policies ADD CONSTRAINT fk_ticket_sla_policies_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY ticket_sla_policies ADD CONSTRAINT fk_ticket_sla_policies_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY ticket_tags ADD CONSTRAINT fk_ticket_tags_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY ticket_tags ADD CONSTRAINT fk_ticket_tags_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY user_organizations ADD CONSTRAINT fk_user_organizations_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE ONLY user_organizations ADD CONSTRAINT fk_user_organizations_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY user_sessions ADD CONSTRAINT fk_user_sessions_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY website_crawls ADD CONSTRAINT fk_website_crawls_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY whiteboard_elements ADD CONSTRAINT fk_whiteboard_elements_whiteboard_id FOREIGN KEY (whiteboard_id) REFERENCES meeting_whiteboards(id) ON DELETE CASCADE;
ALTER TABLE ONLY whiteboard_exports ADD CONSTRAINT fk_whiteboard_exports_whiteboard_id FOREIGN KEY (whiteboard_id) REFERENCES meeting_whiteboards(id) ON DELETE CASCADE;
ALTER TABLE ONLY whiteboard_exports ADD CONSTRAINT fk_whiteboard_exports_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY whiteboard_exports ADD CONSTRAINT fk_whiteboard_exports_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY workflow_executions ADD CONSTRAINT fk_workflow_executions_bot_id FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE SET NULL;
ALTER TABLE ONLY workspace_comment_reactions ADD CONSTRAINT fk_workspace_comment_reactions_comment_id FOREIGN KEY (comment_id) REFERENCES workspace_comments(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_comments ADD CONSTRAINT fk_workspace_comments_workspace_id FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_comments ADD CONSTRAINT fk_workspace_comments_page_id FOREIGN KEY (page_id) REFERENCES workspace_pages(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_members ADD CONSTRAINT fk_workspace_members_workspace_id FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_page_permissions ADD CONSTRAINT fk_workspace_page_permissions_page_id FOREIGN KEY (page_id) REFERENCES workspace_pages(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_page_versions ADD CONSTRAINT fk_workspace_page_versions_page_id FOREIGN KEY (page_id) REFERENCES workspace_pages(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_pages ADD CONSTRAINT fk_workspace_pages_workspace_id FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_templates ADD CONSTRAINT fk_workspace_templates_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY workspace_templates ADD CONSTRAINT fk_workspace_templates_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE ONLY workspaces ADD CONSTRAINT fk_workspaces_org_id FOREIGN KEY (org_id) REFERENCES organizations(org_id) ON DELETE CASCADE;
ALTER TABLE ONLY workspaces ADD CONSTRAINT fk_workspaces_branch_id FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE SET NULL;

