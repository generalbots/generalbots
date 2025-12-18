-- Bot Hierarchy: Add parent_bot_id to support sub-bots
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS parent_bot_id UUID REFERENCES public.bots(id) ON DELETE SET NULL;

-- Index for efficient hierarchy queries
CREATE INDEX IF NOT EXISTS idx_bots_parent_bot_id ON public.bots(parent_bot_id);

-- Bot enabled tabs configuration (which UI tabs are enabled for this bot)
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS enabled_tabs_json TEXT DEFAULT '["chat"]';

-- Bot configuration inheritance flag
ALTER TABLE public.bots
ADD COLUMN IF NOT EXISTS inherit_parent_config BOOLEAN DEFAULT true;

-- Email monitoring table for ON EMAIL triggers
CREATE TABLE IF NOT EXISTS public.email_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
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

CREATE INDEX IF NOT EXISTS idx_email_monitors_bot_id ON public.email_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_email_monitors_email ON public.email_monitors(email_address);
CREATE INDEX IF NOT EXISTS idx_email_monitors_active ON public.email_monitors(is_active) WHERE is_active = true;

-- Folder monitoring table for ON CHANGE triggers (GDrive, OneDrive, Dropbox)
-- Uses account:// syntax: account://user@gmail.com/path or gdrive:///path
CREATE TABLE IF NOT EXISTS public.folder_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES public.bots(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- 'gdrive', 'onedrive', 'dropbox', 'local'
    account_email VARCHAR(500), -- Email from account:// path (e.g., user@gmail.com)
    folder_path VARCHAR(2000) NOT NULL,
    folder_id VARCHAR(500), -- Provider-specific folder ID
    script_path VARCHAR(1000) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    watch_subfolders BOOLEAN DEFAULT true,
    last_check_at TIMESTAMPTZ,
    last_change_token VARCHAR(500), -- Provider-specific change token/page token
    event_types_json TEXT DEFAULT '["create", "modify", "delete"]',
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_bot_folder UNIQUE (bot_id, provider, folder_path)
);

CREATE INDEX IF NOT EXISTS idx_folder_monitors_bot_id ON public.folder_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_provider ON public.folder_monitors(provider);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_active ON public.folder_monitors(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_folder_monitors_account_email ON public.folder_monitors(account_email);

-- Folder change events log
CREATE TABLE IF NOT EXISTS public.folder_change_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES public.folder_monitors(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- 'create', 'modify', 'delete', 'rename', 'move'
    file_path VARCHAR(2000) NOT NULL,
    file_id VARCHAR(500),
    file_name VARCHAR(500),
    file_size BIGINT,
    mime_type VARCHAR(255),
    old_path VARCHAR(2000), -- For rename/move events
    processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_folder_events_monitor ON public.folder_change_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_folder_events_processed ON public.folder_change_events(processed) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_folder_events_created ON public.folder_change_events(created_at);

-- Email received events log
CREATE TABLE IF NOT EXISTS public.email_received_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES public.email_monitors(id) ON DELETE CASCADE,
    message_uid BIGINT NOT NULL,
    message_id VARCHAR(500),
    from_address VARCHAR(500) NOT NULL,
    to_addresses_json TEXT,
    subject VARCHAR(1000),
    received_at TIMESTAMPTZ,
    has_attachments BOOLEAN DEFAULT false,
    attachments_json TEXT,
    processed BOOLEAN DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_email_events_monitor ON public.email_received_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_email_events_processed ON public.email_received_events(processed) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_email_events_received ON public.email_received_events(received_at);

-- Add new trigger kinds to system_automations
-- TriggerKind enum: 0=Scheduled, 1=TableUpdate, 2=TableInsert, 3=TableDelete, 4=Webhook, 5=EmailReceived, 6=FolderChange
COMMENT ON TABLE public.system_automations IS 'System automations with TriggerKind: 0=Scheduled, 1=TableUpdate, 2=TableInsert, 3=TableDelete, 4=Webhook, 5=EmailReceived, 6=FolderChange';

-- User organization memberships (users can belong to multiple orgs)
CREATE TABLE IF NOT EXISTS public.user_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES public.users(user_id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES public.organizations(org_id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member', -- 'owner', 'admin', 'member', 'viewer'
    is_default BOOLEAN DEFAULT false,
    joined_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    CONSTRAINT unique_user_org UNIQUE (user_id, org_id)
);

CREATE INDEX IF NOT EXISTS idx_user_orgs_user ON public.user_organizations(user_id);
CREATE INDEX IF NOT EXISTS idx_user_orgs_org ON public.user_organizations(org_id);
CREATE INDEX IF NOT EXISTS idx_user_orgs_default ON public.user_organizations(user_id, is_default) WHERE is_default = true;

-- Comments for documentation
COMMENT ON COLUMN public.bots.parent_bot_id IS 'Parent bot ID for hierarchical bot structure. NULL means root bot.';
COMMENT ON COLUMN public.bots.enabled_tabs_json IS 'JSON array of enabled UI tabs for this bot. Root bots have all tabs.';
COMMENT ON COLUMN public.bots.inherit_parent_config IS 'If true, inherits config from parent bot for missing values.';
COMMENT ON TABLE public.email_monitors IS 'Email monitoring configuration for ON EMAIL triggers.';
COMMENT ON TABLE public.folder_monitors IS 'Folder monitoring configuration for ON CHANGE triggers (GDrive, OneDrive, Dropbox).';
COMMENT ON TABLE public.folder_change_events IS 'Log of detected folder changes to be processed by scripts.';
COMMENT ON TABLE public.email_received_events IS 'Log of received emails to be processed by scripts.';
COMMENT ON TABLE public.user_organizations IS 'User membership in organizations with roles.';
