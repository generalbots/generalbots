' CRM Database Tables Definition
' This file defines all CRM tables using the TABLE keyword
' Tables are automatically created and managed by the system

' Leads table - stores potential customers
TABLE leads
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    company_name VARCHAR(255) NOT NULL
    contact_name VARCHAR(255)
    email VARCHAR(255) UNIQUE
    phone VARCHAR(50)
    website VARCHAR(255)
    industry VARCHAR(100)
    company_size VARCHAR(50)
    lead_source VARCHAR(100)
    lead_status VARCHAR(50) DEFAULT 'new'
    score INTEGER DEFAULT 0
    assigned_to VARCHAR(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
    converted_at TIMESTAMP
    converted_to_account_id UUID
END TABLE

' Accounts table - stores customer organizations
TABLE accounts
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(255) NOT NULL
    type VARCHAR(50) DEFAULT 'customer'
    industry VARCHAR(100)
    annual_revenue DECIMAL(15,2)
    employees INTEGER
    website VARCHAR(255)
    phone VARCHAR(50)
    billing_address TEXT
    shipping_address TEXT
    owner_id VARCHAR(100)
    parent_account_id UUID
    status VARCHAR(50) DEFAULT 'active'
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Contacts table - stores individual people
TABLE contacts
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    account_id UUID REFERENCES accounts(id)
    first_name VARCHAR(100)
    last_name VARCHAR(100)
    full_name VARCHAR(255) GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED
    email VARCHAR(255) UNIQUE
    phone VARCHAR(50)
    mobile VARCHAR(50)
    title VARCHAR(100)
    department VARCHAR(100)
    lead_id UUID REFERENCES leads(id)
    primary_contact BOOLEAN DEFAULT FALSE
    do_not_call BOOLEAN DEFAULT FALSE
    do_not_email BOOLEAN DEFAULT FALSE
    preferred_contact_method VARCHAR(50)
    linkedin_url VARCHAR(255)
    twitter_handle VARCHAR(100)
    notes TEXT
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Opportunities table - stores sales opportunities
TABLE opportunities
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(255) NOT NULL
    account_id UUID REFERENCES accounts(id)
    contact_id UUID REFERENCES contacts(id)
    amount DECIMAL(15,2)
    probability INTEGER CHECK (probability >= 0 AND probability <= 100)
    expected_revenue DECIMAL(15,2) GENERATED ALWAYS AS (amount * probability / 100) STORED
    stage VARCHAR(100) DEFAULT 'qualification'
    close_date DATE
    type VARCHAR(50)
    lead_source VARCHAR(100)
    next_step TEXT
    description TEXT
    owner_id VARCHAR(100)
    campaign_id UUID
    competitor_names TEXT[]
    won BOOLEAN
    closed_at TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Activities table - stores all customer interactions
TABLE activities
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    type VARCHAR(50) NOT NULL
    subject VARCHAR(255) NOT NULL
    description TEXT
    status VARCHAR(50) DEFAULT 'open'
    priority VARCHAR(20) DEFAULT 'normal'
    due_date TIMESTAMP
    completed_date TIMESTAMP
    duration_minutes INTEGER
    location VARCHAR(255)

    ' Related entities
    account_id UUID REFERENCES accounts(id)
    contact_id UUID REFERENCES contacts(id)
    opportunity_id UUID REFERENCES opportunities(id)
    lead_id UUID REFERENCES leads(id)
    parent_activity_id UUID REFERENCES activities(id)

    ' Assignment and tracking
    assigned_to VARCHAR(100)
    created_by VARCHAR(100)
    modified_by VARCHAR(100)

    ' Activity-specific fields
    call_result VARCHAR(100)
    call_duration INTEGER
    email_message_id VARCHAR(255)
    meeting_notes TEXT
    meeting_attendees TEXT[]

    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Products table - stores product catalog
TABLE products
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(255) NOT NULL
    code VARCHAR(100) UNIQUE
    description TEXT
    category VARCHAR(100)
    unit_price DECIMAL(10,2)
    cost DECIMAL(10,2)
    margin DECIMAL(5,2) GENERATED ALWAYS AS ((unit_price - cost) / unit_price * 100) STORED
    quantity_in_stock INTEGER DEFAULT 0
    active BOOLEAN DEFAULT TRUE
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Quotes table - stores sales quotes
TABLE quotes
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    quote_number VARCHAR(50) UNIQUE
    opportunity_id UUID REFERENCES opportunities(id)
    account_id UUID REFERENCES accounts(id)
    contact_id UUID REFERENCES contacts(id)
    status VARCHAR(50) DEFAULT 'draft'
    valid_until DATE
    subtotal DECIMAL(15,2)
    discount_percent DECIMAL(5,2) DEFAULT 0
    discount_amount DECIMAL(15,2) DEFAULT 0
    tax_rate DECIMAL(5,2) DEFAULT 0
    tax_amount DECIMAL(15,2)
    total DECIMAL(15,2)
    terms_conditions TEXT
    notes TEXT
    approved_by VARCHAR(100)
    approved_at TIMESTAMP
    sent_at TIMESTAMP
    created_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Quote line items
TABLE quote_items
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    quote_id UUID REFERENCES quotes(id) ON DELETE CASCADE
    product_id UUID REFERENCES products(id)
    description TEXT
    quantity INTEGER NOT NULL
    unit_price DECIMAL(10,2) NOT NULL
    discount_percent DECIMAL(5,2) DEFAULT 0
    total DECIMAL(10,2) GENERATED ALWAYS AS (quantity * unit_price * (1 - discount_percent/100)) STORED
    position INTEGER
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Campaigns table - stores marketing campaigns
TABLE campaigns
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(255) NOT NULL
    type VARCHAR(50)
    status VARCHAR(50) DEFAULT 'planning'
    start_date DATE
    end_date DATE
    budget DECIMAL(15,2)
    actual_cost DECIMAL(15,2)
    expected_revenue DECIMAL(15,2)
    expected_response DECIMAL(5,2)
    description TEXT
    objective TEXT
    num_sent INTEGER DEFAULT 0
    num_responses INTEGER DEFAULT 0
    num_leads INTEGER DEFAULT 0
    num_opportunities INTEGER DEFAULT 0
    num_won_opportunities INTEGER DEFAULT 0
    revenue_generated DECIMAL(15,2)
    roi DECIMAL(10,2) GENERATED ALWAYS AS
        (CASE WHEN actual_cost > 0 THEN (revenue_generated - actual_cost) / actual_cost * 100 ELSE 0 END) STORED
    owner_id VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Campaign members
TABLE campaign_members
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    campaign_id UUID REFERENCES campaigns(id) ON DELETE CASCADE
    lead_id UUID REFERENCES leads(id)
    contact_id UUID REFERENCES contacts(id)
    status VARCHAR(50) DEFAULT 'sent'
    responded BOOLEAN DEFAULT FALSE
    response_date TIMESTAMP
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Cases/Tickets table - stores customer support cases
TABLE cases
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    case_number VARCHAR(50) UNIQUE
    subject VARCHAR(255) NOT NULL
    description TEXT
    status VARCHAR(50) DEFAULT 'new'
    priority VARCHAR(20) DEFAULT 'medium'
    type VARCHAR(50)
    origin VARCHAR(50)
    reason VARCHAR(100)

    account_id UUID REFERENCES accounts(id)
    contact_id UUID REFERENCES contacts(id)
    parent_case_id UUID REFERENCES cases(id)

    assigned_to VARCHAR(100)
    escalated_to VARCHAR(100)

    resolution TEXT
    resolved_at TIMESTAMP
    satisfaction_score INTEGER CHECK (satisfaction_score >= 1 AND satisfaction_score <= 5)

    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
    closed_at TIMESTAMP
END TABLE

' Email tracking table
TABLE email_tracking
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    message_id VARCHAR(255) UNIQUE
    from_address VARCHAR(255)
    to_addresses TEXT[]
    cc_addresses TEXT[]
    subject VARCHAR(255)
    body TEXT
    html_body TEXT

    ' Related entities
    account_id UUID REFERENCES accounts(id)
    contact_id UUID REFERENCES contacts(id)
    opportunity_id UUID REFERENCES opportunities(id)
    lead_id UUID REFERENCES leads(id)
    case_id UUID REFERENCES cases(id)
    activity_id UUID REFERENCES activities(id)

    ' Tracking
    sent_at TIMESTAMP
    delivered_at TIMESTAMP
    opened_at TIMESTAMP
    clicked_at TIMESTAMP
    bounced BOOLEAN DEFAULT FALSE
    bounce_reason TEXT

    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Documents/Attachments table
TABLE documents
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(255) NOT NULL
    file_path VARCHAR(500)
    file_size INTEGER
    mime_type VARCHAR(100)
    description TEXT

    ' Polymorphic associations
    entity_type VARCHAR(50)
    entity_id UUID

    uploaded_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Notes table - stores notes for any entity
TABLE notes
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    title VARCHAR(255)
    body TEXT NOT NULL

    ' Polymorphic associations
    entity_type VARCHAR(50)
    entity_id UUID

    is_private BOOLEAN DEFAULT FALSE
    created_by VARCHAR(100)
    modified_by VARCHAR(100)
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Tags table for categorization
TABLE tags
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    name VARCHAR(100) UNIQUE NOT NULL
    color VARCHAR(7)
    description TEXT
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Entity tags junction table
TABLE entity_tags
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    tag_id UUID REFERENCES tags(id) ON DELETE CASCADE
    entity_type VARCHAR(50)
    entity_id UUID
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(tag_id, entity_type, entity_id)
END TABLE

' Pipeline stages configuration
TABLE pipeline_stages
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    pipeline_type VARCHAR(50) NOT NULL
    stage_name VARCHAR(100) NOT NULL
    stage_order INTEGER NOT NULL
    probability INTEGER DEFAULT 0
    is_won BOOLEAN DEFAULT FALSE
    is_closed BOOLEAN DEFAULT FALSE
    color VARCHAR(7)
    created_at TIMESTAMP DEFAULT NOW()
    UNIQUE(pipeline_type, stage_order)
END TABLE

' User preferences and settings
TABLE crm_user_settings
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    user_id VARCHAR(100) UNIQUE NOT NULL
    default_pipeline VARCHAR(50)
    email_signature TEXT
    notification_preferences JSONB
    dashboard_layout JSONB
    list_view_preferences JSONB
    timezone VARCHAR(50) DEFAULT 'UTC'
    date_format VARCHAR(20) DEFAULT 'YYYY-MM-DD'
    created_at TIMESTAMP DEFAULT NOW()
    updated_at TIMESTAMP DEFAULT NOW()
END TABLE

' Audit log for tracking changes
TABLE audit_log
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
    entity_type VARCHAR(50) NOT NULL
    entity_id UUID NOT NULL
    action VARCHAR(50) NOT NULL
    field_name VARCHAR(100)
    old_value TEXT
    new_value TEXT
    user_id VARCHAR(100)
    ip_address VARCHAR(45)
    user_agent TEXT
    created_at TIMESTAMP DEFAULT NOW()
END TABLE

' Indexes for performance
CREATE INDEX idx_leads_status ON leads(lead_status)
CREATE INDEX idx_leads_assigned ON leads(assigned_to)
CREATE INDEX idx_accounts_owner ON accounts(owner_id)
CREATE INDEX idx_contacts_account ON contacts(account_id)
CREATE INDEX idx_opportunities_account ON opportunities(account_id)
CREATE INDEX idx_opportunities_stage ON opportunities(stage)
CREATE INDEX idx_opportunities_close_date ON opportunities(close_date)
CREATE INDEX idx_activities_due_date ON activities(due_date)
CREATE INDEX idx_activities_assigned ON activities(assigned_to)
CREATE INDEX idx_cases_status ON cases(status)
CREATE INDEX idx_cases_assigned ON cases(assigned_to)
CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id)
CREATE INDEX idx_entity_tags ON entity_tags(entity_type, entity_id)
