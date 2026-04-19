-- Legacy Mail Tables extracted from consolidated

-- Global email signature (applied to all emails from this bot)
CREATE TABLE IF NOT EXISTS global_email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL DEFAULT 'Default',
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    position VARCHAR(20) DEFAULT 'bottom',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_bot_global_signature UNIQUE (bot_id, name),
    CONSTRAINT check_signature_position CHECK (position IN ('top', 'bottom'))
);

CREATE INDEX IF NOT EXISTS idx_global_signatures_bot ON global_email_signatures(bot_id) WHERE is_active = true;

-- User email signatures (in addition to global)
CREATE TABLE IF NOT EXISTS email_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL DEFAULT 'Default',
    content_html TEXT NOT NULL,
    content_plain TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_signature_name UNIQUE (user_id, bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_email_signatures_user ON email_signatures(user_id);
CREATE INDEX IF NOT EXISTS idx_email_signatures_default ON email_signatures(user_id, bot_id) WHERE is_default = true;

-- Scheduled emails (send later)
CREATE TABLE IF NOT EXISTS scheduled_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    to_addresses TEXT NOT NULL,
    cc_addresses TEXT,
    bcc_addresses TEXT,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    attachments_json TEXT DEFAULT '[]',
    scheduled_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_scheduled_status CHECK (status IN ('pending', 'sent', 'failed', 'cancelled'))
);

CREATE INDEX IF NOT EXISTS idx_scheduled_emails_pending ON scheduled_emails(scheduled_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_user ON scheduled_emails(user_id, bot_id);

-- Email templates
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    subject_template TEXT NOT NULL,
    body_html_template TEXT NOT NULL,
    body_plain_template TEXT,
    variables_json TEXT DEFAULT '[]',
    category VARCHAR(100),
    is_shared BOOLEAN DEFAULT false,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_templates_bot ON email_templates(bot_id);
CREATE INDEX IF NOT EXISTS idx_email_templates_category ON email_templates(category);
CREATE INDEX IF NOT EXISTS idx_email_templates_shared ON email_templates(bot_id) WHERE is_shared = true;

-- Auto-responders (Out of Office) - works with Stalwart Sieve
CREATE TABLE IF NOT EXISTS email_auto_responders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    responder_type VARCHAR(50) NOT NULL DEFAULT 'out_of_office',
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_plain TEXT,
    start_date TIMESTAMPTZ,
    end_date TIMESTAMPTZ,
    send_to_internal_only BOOLEAN DEFAULT false,
    exclude_addresses TEXT,
    is_active BOOLEAN DEFAULT false,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_responder_type CHECK (responder_type IN ('out_of_office', 'vacation', 'custom')),
    CONSTRAINT unique_user_responder UNIQUE (user_id, bot_id, responder_type)
);

CREATE INDEX IF NOT EXISTS idx_auto_responders_active ON email_auto_responders(user_id, bot_id) WHERE is_active = true;

-- Email rules/filters - synced with Stalwart Sieve
CREATE TABLE IF NOT EXISTS email_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 0,
    conditions_json TEXT NOT NULL,
    actions_json TEXT NOT NULL,
    stop_processing BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    stalwart_sieve_id VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_rules_user ON email_rules(user_id, bot_id);
CREATE INDEX IF NOT EXISTS idx_email_rules_priority ON email_rules(user_id, bot_id, priority);

-- Email labels/categories
CREATE TABLE IF NOT EXISTS email_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7) DEFAULT '#3b82f6',
    parent_id UUID REFERENCES email_labels(id) ON DELETE CASCADE,
    is_system BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_label UNIQUE (user_id, bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_email_labels_user ON email_labels(user_id, bot_id);

-- Email-label associations
CREATE TABLE IF NOT EXISTS email_label_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_message_id VARCHAR(255) NOT NULL,
    label_id UUID NOT NULL REFERENCES email_labels(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_email_label UNIQUE (email_message_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_label_assignments_email ON email_label_assignments(email_message_id);
CREATE INDEX IF NOT EXISTS idx_label_assignments_label ON email_label_assignments(label_id);

-- Distribution lists
CREATE TABLE IF NOT EXISTS distribution_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email_alias VARCHAR(255),
    description TEXT,
    members_json TEXT NOT NULL DEFAULT '[]',
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_distribution_lists_bot ON distribution_lists(bot_id);
CREATE INDEX IF NOT EXISTS idx_distribution_lists_owner ON distribution_lists(owner_id);

-- Shared mailboxes - managed via Stalwart
CREATE TABLE IF NOT EXISTS shared_mailboxes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    email_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    settings_json TEXT DEFAULT '{}',
    stalwart_account_id VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_shared_mailbox_email UNIQUE (bot_id, email_address)
);

CREATE INDEX IF NOT EXISTS idx_shared_mailboxes_bot ON shared_mailboxes(bot_id);

CREATE TABLE IF NOT EXISTS shared_mailbox_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID NOT NULL REFERENCES shared_mailboxes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_level VARCHAR(20) DEFAULT 'read',
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_mailbox_member UNIQUE (mailbox_id, user_id),
    CONSTRAINT check_permission CHECK (permission_level IN ('read', 'write', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_shared_mailbox_members ON shared_mailbox_members(mailbox_id);
CREATE INDEX IF NOT EXISTS idx_shared_mailbox_user ON shared_mailbox_members(user_id);

-- Add user_email_accounts table for storing user email credentials
CREATE TABLE IF NOT EXISTS user_email_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email varchar(255) NOT NULL,
    display_name varchar(255) NULL,
    imap_server varchar(255) NOT NULL,
    imap_port int4 DEFAULT 993 NOT NULL,
    smtp_server varchar(255) NOT NULL,
    smtp_port int4 DEFAULT 587 NOT NULL,
    username varchar(255) NOT NULL,
    password_encrypted text NOT NULL,
    is_primary bool DEFAULT false NOT NULL,
    is_active bool DEFAULT true NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT user_email_accounts_pkey PRIMARY KEY (id),
    CONSTRAINT user_email_accounts_user_email_key UNIQUE (user_id, email)
);

CREATE INDEX IF NOT EXISTS idx_user_email_accounts_user_id ON user_email_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_user_email_accounts_active ON user_email_accounts(is_active) WHERE is_active;

-- Add email drafts table
CREATE TABLE IF NOT EXISTS email_drafts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id uuid NOT NULL REFERENCES user_email_accounts(id) ON DELETE CASCADE,
    to_address text NOT NULL,
    cc_address text NULL,
    bcc_address text NULL,
    subject varchar(500) NULL,
    body text NULL,
    attachments jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_drafts_pkey PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_email_drafts_user_id ON email_drafts(user_id);
CREATE INDEX IF NOT EXISTS idx_email_drafts_account_id ON email_drafts(account_id);

-- Add email folders metadata table (for caching and custom folders)
CREATE TABLE IF NOT EXISTS email_folders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    account_id uuid NOT NULL REFERENCES user_email_accounts(id) ON DELETE CASCADE,
    folder_name varchar(255) NOT NULL,
    folder_path varchar(500) NOT NULL,
    unread_count int4 DEFAULT 0 NOT NULL,
    total_count int4 DEFAULT 0 NOT NULL,
    last_synced timestamptz NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT email_folders_pkey PRIMARY KEY (id),
    CONSTRAINT email_folders_account_path_key UNIQUE (account_id, folder_path)
);

CREATE INDEX IF NOT EXISTS idx_email_folders_account_id ON email_folders(account_id);

-- Add sessions table enhancement for storing current email account
-- Check if column exists, if not add it (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'user_sessions' AND column_name = 'active_email_account_id'
    ) THEN
        ALTER TABLE user_sessions ADD COLUMN active_email_account_id uuid NULL;
        ALTER TABLE user_sessions ADD CONSTRAINT user_sessions_email_account_id_fkey
        FOREIGN KEY (active_email_account_id) REFERENCES user_email_accounts(id) ON DELETE SET NULL;
    END IF;
END $$;

-- Email Read Tracking Table
CREATE TABLE IF NOT EXISTS sent_email_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tracking_id UUID NOT NULL UNIQUE,
    bot_id UUID NOT NULL,
    account_id UUID NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    cc TEXT,
    bcc TEXT,
    subject TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    read_count INTEGER NOT NULL DEFAULT 0,
    first_read_ip VARCHAR(45),
    last_read_ip VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_tracking_id ON sent_email_tracking(tracking_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_bot_id ON sent_email_tracking(bot_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_account_id ON sent_email_tracking(account_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_to_email ON sent_email_tracking(to_email);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_sent_at ON sent_email_tracking(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_is_read ON sent_email_tracking(is_read);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_read_status ON sent_email_tracking(bot_id, is_read, sent_at DESC);

-- Trigger to auto-update updated_at for tracking
CREATE OR REPLACE FUNCTION update_sent_email_tracking_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_sent_email_tracking_updated_at ON sent_email_tracking;
CREATE TRIGGER trigger_update_sent_email_tracking_updated_at
    BEFORE UPDATE ON sent_email_tracking
    FOR EACH ROW
    EXECUTE FUNCTION update_sent_email_tracking_updated_at();

-- Add comment for documentation
COMMENT ON TABLE sent_email_tracking IS 'Tracks sent emails for read receipt functionality via tracking pixel';

-- Email monitoring table for ON EMAIL triggers
CREATE TABLE IF NOT EXISTS email_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    email_address VARCHAR(500) NOT NULL,
    script_path VARCHAR(1000) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_check_at TIMESTAMPTZ,
    last_uid BIGINT DEFAULT 0,
    filter_from VARCHAR(500),
    filter_subject VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_bot_email UNIQUE (bot_id, email_address)
);

CREATE INDEX IF NOT EXISTS idx_email_monitors_bot_id ON email_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_email_monitors_email ON email_monitors(email_address);
CREATE INDEX IF NOT EXISTS idx_email_monitors_active ON email_monitors(is_active) WHERE is_active = true;

-- Email received events log
CREATE TABLE IF NOT EXISTS email_received_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES email_monitors(id) ON DELETE CASCADE,
    message_uid BIGINT NOT NULL,
    message_id VARCHAR(500),
    from_address VARCHAR(500) NOT NULL,
    to_addresses_json TEXT,
    subject VARCHAR(1000),
    received_at TIMESTAMPTZ,
    has_attachments BOOLEAN DEFAULT false,
    content_preview TEXT,
    processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_email_events_monitor ON email_received_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_email_events_received ON email_received_events(received_at DESC);
CREATE INDEX IF NOT EXISTS idx_email_events_processed ON email_received_events(processed) WHERE processed = false;
