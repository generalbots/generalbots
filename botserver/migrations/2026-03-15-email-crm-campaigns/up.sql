-- Email tables
CREATE TABLE IF NOT EXISTS emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    account_id UUID,
    folder VARCHAR(50) NOT NULL DEFAULT 'inbox',
    from_address VARCHAR(255) NOT NULL,
    to_address TEXT NOT NULL,
    cc_address TEXT,
    bcc_address TEXT,
    subject TEXT,
    body TEXT,
    html_body TEXT,
    is_read BOOLEAN DEFAULT FALSE,
    is_starred BOOLEAN DEFAULT FALSE,
    is_flagged BOOLEAN DEFAULT FALSE,
    thread_id UUID,
    in_reply_to UUID,
    message_id VARCHAR(255),
    ai_category VARCHAR(50),
    ai_confidence FLOAT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_emails_user_folder ON emails(user_id, folder);
CREATE INDEX idx_emails_thread ON emails(thread_id);
CREATE INDEX idx_emails_from ON emails(from_address);

-- Email accounts
CREATE TABLE IF NOT EXISTS email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    imap_server VARCHAR(255),
    imap_port INTEGER DEFAULT 993,
    smtp_server VARCHAR(255),
    smtp_port INTEGER DEFAULT 587,
    username VARCHAR(255),
    password_encrypted TEXT,
    use_ssl BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Email snooze
CREATE TABLE IF NOT EXISTS email_snooze (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    snooze_until TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_email_snooze_until ON email_snooze(snooze_until);

-- Email flags (follow-up)
CREATE TABLE IF NOT EXISTS email_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    follow_up_date DATE,
    flag_type VARCHAR(50),
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Email nudges
CREATE TABLE IF NOT EXISTS email_nudges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    last_sent TIMESTAMP,
    dismissed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Feature flags
CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    feature VARCHAR(50) NOT NULL,
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(org_id, feature)
);

-- Email-CRM links
CREATE TABLE IF NOT EXISTS email_crm_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    contact_id UUID,
    opportunity_id UUID,
    logged_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_email_crm_contact ON email_crm_links(contact_id);
CREATE INDEX idx_email_crm_opportunity ON email_crm_links(opportunity_id);

-- Email-Campaign links
CREATE TABLE IF NOT EXISTS email_campaign_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    campaign_id UUID,
    list_id UUID,
    sent_at TIMESTAMP DEFAULT NOW()
);

-- Offline queue
CREATE TABLE IF NOT EXISTS email_offline_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
