-- Migration: 6.1.0 Enterprise Features
-- Description: MUST-HAVE features to compete with Microsoft 365 and Google Workspace
-- NOTE: TABLES AND INDEXES ONLY - No views, triggers, or functions per project standards

-- ============================================================================
-- GLOBAL CONFIGURATION
-- ============================================================================

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

CREATE INDEX idx_global_signatures_bot ON global_email_signatures(bot_id) WHERE is_active = true;

-- ============================================================================
-- EMAIL ENTERPRISE FEATURES (Outlook/Gmail parity)
-- Note: Many features controlled via Stalwart IMAP/JMAP API
-- ============================================================================

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

CREATE INDEX idx_email_signatures_user ON email_signatures(user_id);
CREATE INDEX idx_email_signatures_default ON email_signatures(user_id, bot_id) WHERE is_default = true;

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

CREATE INDEX idx_scheduled_emails_pending ON scheduled_emails(scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_scheduled_emails_user ON scheduled_emails(user_id, bot_id);

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

CREATE INDEX idx_email_templates_bot ON email_templates(bot_id);
CREATE INDEX idx_email_templates_category ON email_templates(category);
CREATE INDEX idx_email_templates_shared ON email_templates(bot_id) WHERE is_shared = true;

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

CREATE INDEX idx_auto_responders_active ON email_auto_responders(user_id, bot_id) WHERE is_active = true;

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

CREATE INDEX idx_email_rules_user ON email_rules(user_id, bot_id);
CREATE INDEX idx_email_rules_priority ON email_rules(user_id, bot_id, priority);

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

CREATE INDEX idx_email_labels_user ON email_labels(user_id, bot_id);

-- Email-label associations
CREATE TABLE IF NOT EXISTS email_label_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_message_id VARCHAR(255) NOT NULL,
    label_id UUID NOT NULL REFERENCES email_labels(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_email_label UNIQUE (email_message_id, label_id)
);

CREATE INDEX idx_label_assignments_email ON email_label_assignments(email_message_id);
CREATE INDEX idx_label_assignments_label ON email_label_assignments(label_id);

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

CREATE INDEX idx_distribution_lists_bot ON distribution_lists(bot_id);
CREATE INDEX idx_distribution_lists_owner ON distribution_lists(owner_id);

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

CREATE INDEX idx_shared_mailboxes_bot ON shared_mailboxes(bot_id);

CREATE TABLE IF NOT EXISTS shared_mailbox_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mailbox_id UUID NOT NULL REFERENCES shared_mailboxes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_level VARCHAR(20) DEFAULT 'read',
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT unique_mailbox_member UNIQUE (mailbox_id, user_id),
    CONSTRAINT check_permission CHECK (permission_level IN ('read', 'write', 'admin'))
);

CREATE INDEX idx_shared_mailbox_members ON shared_mailbox_members(mailbox_id);
CREATE INDEX idx_shared_mailbox_user ON shared_mailbox_members(user_id);

-- ============================================================================
-- VIDEO MEETING FEATURES (Google Meet/Zoom parity)
-- ============================================================================

-- Meeting recordings
CREATE TABLE IF NOT EXISTS meeting_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    recorded_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    format VARCHAR(20) DEFAULT 'mp4',
    thumbnail_path TEXT,
    transcription_path TEXT,
    transcription_status VARCHAR(20) DEFAULT 'pending',
    is_shared BOOLEAN DEFAULT false,
    shared_with_json TEXT DEFAULT '[]',
    retention_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_transcription_status CHECK (transcription_status IN ('pending', 'processing', 'completed', 'failed'))
);

CREATE INDEX idx_meeting_recordings_meeting ON meeting_recordings(meeting_id);
CREATE INDEX idx_meeting_recordings_bot ON meeting_recordings(bot_id);

-- Breakout rooms
CREATE TABLE IF NOT EXISTS meeting_breakout_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    room_number INTEGER NOT NULL,
    participants_json TEXT DEFAULT '[]',
    duration_minutes INTEGER,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_breakout_rooms_meeting ON meeting_breakout_rooms(meeting_id);

-- Meeting polls
CREATE TABLE IF NOT EXISTS meeting_polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    poll_type VARCHAR(20) DEFAULT 'single',
    options_json TEXT NOT NULL,
    is_anonymous BOOLEAN DEFAULT false,
    allow_multiple BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT false,
    results_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    CONSTRAINT check_poll_type CHECK (poll_type IN ('single', 'multiple', 'open'))
);

CREATE INDEX idx_meeting_polls_meeting ON meeting_polls(meeting_id);

-- Meeting Q&A
CREATE TABLE IF NOT EXISTS meeting_questions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    asked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    question TEXT NOT NULL,
    is_anonymous BOOLEAN DEFAULT false,
    upvotes INTEGER DEFAULT 0,
    is_answered BOOLEAN DEFAULT false,
    answer TEXT,
    answered_by UUID REFERENCES users(id) ON DELETE SET NULL,
    answered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_meeting_questions_meeting ON meeting_questions(meeting_id);
CREATE INDEX idx_meeting_questions_unanswered ON meeting_questions(meeting_id) WHERE is_answered = false;

-- Meeting waiting room
CREATE TABLE IF NOT EXISTS meeting_waiting_room (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    guest_name VARCHAR(255),
    guest_email VARCHAR(255),
    device_info_json TEXT DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'waiting',
    admitted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    admitted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_waiting_status CHECK (status IN ('waiting', 'admitted', 'rejected', 'left'))
);

CREATE INDEX idx_waiting_room_meeting ON meeting_waiting_room(meeting_id);
CREATE INDEX idx_waiting_room_status ON meeting_waiting_room(meeting_id, status);

-- Meeting live captions
CREATE TABLE IF NOT EXISTS meeting_captions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    speaker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    speaker_name VARCHAR(255),
    caption_text TEXT NOT NULL,
    language VARCHAR(10) DEFAULT 'en',
    confidence REAL,
    timestamp_ms BIGINT NOT NULL,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_meeting_captions_meeting ON meeting_captions(meeting_id, timestamp_ms);

-- Virtual backgrounds
CREATE TABLE IF NOT EXISTS user_virtual_backgrounds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100),
    background_type VARCHAR(20) DEFAULT 'image',
    file_path TEXT,
    blur_intensity INTEGER,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_bg_type CHECK (background_type IN ('image', 'blur', 'none'))
);

CREATE INDEX idx_virtual_backgrounds_user ON user_virtual_backgrounds(user_id);

-- ============================================================================
-- DRIVE ENTERPRISE FEATURES (Google Drive/OneDrive parity)
-- ============================================================================

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

CREATE INDEX idx_file_versions_file ON file_versions(file_id);
CREATE INDEX idx_file_versions_current ON file_versions(file_id) WHERE is_current = true;

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

CREATE INDEX idx_file_comments_file ON file_comments(file_id);
CREATE INDEX idx_file_comments_unresolved ON file_comments(file_id) WHERE is_resolved = false;

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

CREATE INDEX idx_file_shares_file ON file_shares(file_id);
CREATE INDEX idx_file_shares_user ON file_shares(shared_with_user);
CREATE INDEX idx_file_shares_token ON file_shares(link_token) WHERE link_token IS NOT NULL;

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

CREATE INDEX idx_file_activities_file ON file_activities(file_id, created_at DESC);
CREATE INDEX idx_file_activities_user ON file_activities(user_id, created_at DESC);

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

CREATE INDEX idx_file_trash_owner ON file_trash(owner_id);
CREATE INDEX idx_file_trash_expiry ON file_trash(permanent_delete_at);

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

CREATE INDEX idx_file_sync_user ON file_sync_status(user_id, device_id);
CREATE INDEX idx_file_sync_pending ON file_sync_status(user_id) WHERE sync_status = 'pending';

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

CREATE INDEX idx_storage_quotas_user ON storage_quotas(user_id);

-- ============================================================================
-- COLLABORATION FEATURES
-- ============================================================================

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

CREATE INDEX idx_document_presence_doc ON document_presence(document_id);

-- ============================================================================
-- TASK ENTERPRISE FEATURES
-- ============================================================================

-- Task dependencies
CREATE TABLE IF NOT EXISTS task_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    depends_on_task_id UUID NOT NULL,
    dependency_type VARCHAR(20) DEFAULT 'finish_to_start',
    lag_days INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_dependency_type CHECK (dependency_type IN ('finish_to_start', 'start_to_start', 'finish_to_finish', 'start_to_finish')),
    CONSTRAINT unique_task_dependency UNIQUE (task_id, depends_on_task_id)
);

CREATE INDEX idx_task_dependencies_task ON task_dependencies(task_id);
CREATE INDEX idx_task_dependencies_depends ON task_dependencies(depends_on_task_id);

-- Task time tracking
CREATE TABLE IF NOT EXISTS task_time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_minutes INTEGER,
    is_billable BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_task_time_task ON task_time_entries(task_id);
CREATE INDEX idx_task_time_user ON task_time_entries(user_id, started_at);

-- Task recurring rules
CREATE TABLE IF NOT EXISTS task_recurrence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_template_id UUID NOT NULL,
    recurrence_pattern VARCHAR(20) NOT NULL,
    interval_value INTEGER DEFAULT 1,
    days_of_week_json TEXT,
    day_of_month INTEGER,
    month_of_year INTEGER,
    end_date TIMESTAMPTZ,
    occurrence_count INTEGER,
    next_occurrence TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_recurrence CHECK (recurrence_pattern IN ('daily', 'weekly', 'monthly', 'yearly', 'custom'))
);

CREATE INDEX idx_task_recurrence_next ON task_recurrence(next_occurrence) WHERE is_active = true;

-- ============================================================================
-- CALENDAR ENTERPRISE FEATURES
-- ============================================================================

-- Resource booking (meeting rooms, equipment)
CREATE TABLE IF NOT EXISTS calendar_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    location VARCHAR(255),
    capacity INTEGER,
    amenities_json TEXT DEFAULT '[]',
    availability_hours_json TEXT,
    booking_rules_json TEXT DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_resource_type CHECK (resource_type IN ('room', 'equipment', 'vehicle', 'other'))
);

CREATE INDEX idx_calendar_resources_bot ON calendar_resources(bot_id);
CREATE INDEX idx_calendar_resources_type ON calendar_resources(bot_id, resource_type);

CREATE TABLE IF NOT EXISTS calendar_resource_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES calendar_resources(id) ON DELETE CASCADE,
    event_id UUID,
    booked_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    notes TEXT,
    status VARCHAR(20) DEFAULT 'confirmed',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_booking_status CHECK (status IN ('pending', 'confirmed', 'cancelled'))
);

CREATE INDEX idx_resource_bookings_resource ON calendar_resource_bookings(resource_id, start_time, end_time);
CREATE INDEX idx_resource_bookings_user ON calendar_resource_bookings(booked_by);

-- Calendar sharing
CREATE TABLE IF NOT EXISTS calendar_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_with_user UUID REFERENCES users(id) ON DELETE CASCADE,
    shared_with_email VARCHAR(255),
    permission_level VARCHAR(20) DEFAULT 'view',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_cal_permission CHECK (permission_level IN ('free_busy', 'view', 'edit', 'admin'))
);

CREATE INDEX idx_calendar_shares_owner ON calendar_shares(owner_id);
CREATE INDEX idx_calendar_shares_shared ON calendar_shares(shared_with_user);

-- ============================================================================
-- TEST SUPPORT TABLES
-- ============================================================================

-- Test accounts for integration testing
CREATE TABLE IF NOT EXISTS test_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_type VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_test_account_type CHECK (account_type IN ('sender', 'receiver', 'bot', 'admin'))
);

CREATE INDEX idx_test_accounts_type ON test_accounts(account_type);

-- Test execution logs
CREATE TABLE IF NOT EXISTS test_execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_suite VARCHAR(100) NOT NULL,
    test_name VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL,
    duration_ms INTEGER,
    error_message TEXT,
    stack_trace TEXT,
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT check_test_status CHECK (status IN ('passed', 'failed', 'skipped', 'error'))
);

CREATE INDEX idx_test_logs_suite ON test_execution_logs(test_suite, created_at DESC);
CREATE INDEX idx_test_logs_status ON test_execution_logs(status, created_at DESC);
