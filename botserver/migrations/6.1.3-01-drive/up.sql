-- Legacy Drive Tables extracted from consolidated

-- File version history
CREATE TABLE IF NOT EXISTS file_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    version_number INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(64) NOT NULL,
    modified_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    change_summary TEXT,
    is_current BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_file_version UNIQUE (file_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_file_versions_file ON file_versions(file_id);
CREATE INDEX IF NOT EXISTS idx_file_versions_current ON file_versions(file_id) WHERE is_current = true;

-- File comments
CREATE TABLE IF NOT EXISTS file_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES file_comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    anchor_data_json TEXT,
    is_resolved BOOLEAN DEFAULT false,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_file_comments_file ON file_comments(file_id);
CREATE INDEX IF NOT EXISTS idx_file_comments_unresolved ON file_comments(file_id) WHERE is_resolved = false;

-- File sharing permissions
CREATE TABLE IF NOT EXISTS file_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    shared_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_with_user UUID REFERENCES users(id) ON DELETE CASCADE,
    shared_with_email VARCHAR(255),
    shared_with_group UUID,
    permission_level VARCHAR(20) NOT NULL DEFAULT 'view',
    can_reshare BOOLEAN DEFAULT false,
    password_hash VARCHAR(255),
    expires_at TIMESTAMPTZ,
    link_token VARCHAR(64) UNIQUE,
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_share_permission CHECK (permission_level IN ('view', 'comment', 'edit', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_file_shares_file ON file_shares(file_id);
CREATE INDEX IF NOT EXISTS idx_file_shares_user ON file_shares(shared_with_user);
CREATE INDEX IF NOT EXISTS idx_file_shares_token ON file_shares(link_token) WHERE link_token IS NOT NULL;

-- File activity log
CREATE TABLE IF NOT EXISTS file_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    activity_type VARCHAR(50) NOT NULL,
    details_json TEXT DEFAULT '{}',
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_file_activities_file ON file_activities(file_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_file_activities_user ON file_activities(user_id, created_at DESC);

-- Trash bin (soft delete with restore)
CREATE TABLE IF NOT EXISTS file_trash (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_file_id UUID NOT NULL,
    original_path TEXT NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    file_metadata_json TEXT NOT NULL,
    deleted_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    deleted_at TIMESTAMPTZ DEFAULT NOW(),
    permanent_delete_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_file_trash_owner ON file_trash(owner_id);
CREATE INDEX IF NOT EXISTS idx_file_trash_expiry ON file_trash(permanent_delete_at);

-- Offline sync tracking
CREATE TABLE IF NOT EXISTS file_sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(255) NOT NULL,
    file_id UUID NOT NULL,
    local_path TEXT,
    sync_status VARCHAR(20) DEFAULT 'synced',
    local_version INTEGER,
    remote_version INTEGER,
    conflict_data_json TEXT,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_sync_status CHECK (sync_status IN ('synced', 'pending', 'conflict', 'error')),
    CONSTRAINT unique_sync_entry UNIQUE (user_id, device_id, file_id)
);

CREATE INDEX IF NOT EXISTS idx_file_sync_user ON file_sync_status(user_id, device_id);
CREATE INDEX IF NOT EXISTS idx_file_sync_pending ON file_sync_status(user_id) WHERE sync_status = 'pending';

-- Storage quotas
CREATE TABLE IF NOT EXISTS storage_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    bot_id UUID REFERENCES bots(id) ON DELETE CASCADE,
    quota_bytes BIGINT NOT NULL DEFAULT 5368709120,
    used_bytes BIGINT NOT NULL DEFAULT 0,
    warning_threshold_percent INTEGER DEFAULT 90,
    last_calculated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_user_quota UNIQUE (user_id),
    CONSTRAINT unique_bot_quota UNIQUE (bot_id)
);

CREATE INDEX IF NOT EXISTS idx_storage_quotas_user ON storage_quotas(user_id);

-- Document presence (who's viewing/editing)
CREATE TABLE IF NOT EXISTS document_presence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cursor_position_json TEXT,
    selection_range_json TEXT,
    color VARCHAR(7),
    is_editing BOOLEAN DEFAULT false,
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_doc_user_presence UNIQUE (document_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_document_presence_doc ON document_presence(document_id);

-- Folder monitoring table for ON CHANGE triggers (GDrive, OneDrive, Dropbox)
CREATE TABLE IF NOT EXISTS folder_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
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

CREATE INDEX IF NOT EXISTS idx_folder_monitors_bot_id ON folder_monitors(bot_id);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_provider ON folder_monitors(provider);
CREATE INDEX IF NOT EXISTS idx_folder_monitors_active ON folder_monitors(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_folder_monitors_account_email ON folder_monitors(account_email);

-- Folder change events log
CREATE TABLE IF NOT EXISTS folder_change_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    monitor_id UUID NOT NULL REFERENCES folder_monitors(id) ON DELETE CASCADE,
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

CREATE INDEX IF NOT EXISTS idx_folder_events_monitor ON folder_change_events(monitor_id);
CREATE INDEX IF NOT EXISTS idx_folder_events_processed ON folder_change_events(processed) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_folder_events_created ON folder_change_events(created_at);
