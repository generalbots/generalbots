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

CREATE INDEX IF NOT EXISTS idx_global_signatures_bot ON global_email_signatures(bot_id) WHERE is_active = true;

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

CREATE INDEX IF NOT EXISTS idx_meeting_recordings_meeting ON meeting_recordings(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_recordings_bot ON meeting_recordings(bot_id);

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

CREATE INDEX IF NOT EXISTS idx_breakout_rooms_meeting ON meeting_breakout_rooms(meeting_id);

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

CREATE INDEX IF NOT EXISTS idx_meeting_polls_meeting ON meeting_polls(meeting_id);

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

CREATE INDEX IF NOT EXISTS idx_meeting_questions_meeting ON meeting_questions(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meeting_questions_unanswered ON meeting_questions(meeting_id) WHERE is_answered = false;

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

CREATE INDEX IF NOT EXISTS idx_waiting_room_meeting ON meeting_waiting_room(meeting_id);
CREATE INDEX IF NOT EXISTS idx_waiting_room_status ON meeting_waiting_room(meeting_id, status);

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

CREATE INDEX IF NOT EXISTS idx_meeting_captions_meeting ON meeting_captions(meeting_id, timestamp_ms);

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

CREATE INDEX IF NOT EXISTS idx_virtual_backgrounds_user ON user_virtual_backgrounds(user_id);

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

CREATE INDEX IF NOT EXISTS idx_document_presence_doc ON document_presence(document_id);

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

CREATE INDEX IF NOT EXISTS idx_task_dependencies_task ON task_dependencies(task_id);
CREATE INDEX IF NOT EXISTS idx_task_dependencies_depends ON task_dependencies(depends_on_task_id);

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

CREATE INDEX IF NOT EXISTS idx_task_time_task ON task_time_entries(task_id);
CREATE INDEX IF NOT EXISTS idx_task_time_user ON task_time_entries(user_id, started_at);

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

CREATE INDEX IF NOT EXISTS idx_task_recurrence_next ON task_recurrence(next_occurrence) WHERE is_active = true;

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

CREATE INDEX IF NOT EXISTS idx_calendar_resources_bot ON calendar_resources(bot_id);
CREATE INDEX IF NOT EXISTS idx_calendar_resources_type ON calendar_resources(bot_id, resource_type);

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

CREATE INDEX IF NOT EXISTS idx_resource_bookings_resource ON calendar_resource_bookings(resource_id, start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_resource_bookings_user ON calendar_resource_bookings(booked_by);

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

CREATE INDEX IF NOT EXISTS idx_calendar_shares_owner ON calendar_shares(owner_id);
CREATE INDEX IF NOT EXISTS idx_calendar_shares_shared ON calendar_shares(shared_with_user);

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

CREATE INDEX IF NOT EXISTS idx_test_accounts_type ON test_accounts(account_type);

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

CREATE INDEX IF NOT EXISTS idx_test_logs_suite ON test_execution_logs(test_suite, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_test_logs_status ON test_execution_logs(status, created_at DESC);
-- Migration: 6.1.1 Multi-Agent Memory Support
-- Description: Adds tables for user memory, session preferences, and A2A protocol messaging

-- ============================================================================
-- User Memories Table
-- Cross-session memory that persists for users across all sessions and bots
-- ============================================================================
CREATE TABLE IF NOT EXISTS user_memories (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    memory_type VARCHAR(50) NOT NULL DEFAULT 'preference',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT user_memories_unique_key UNIQUE (user_id, key)
);

CREATE INDEX IF NOT EXISTS idx_user_memories_user_id ON user_memories(user_id);
CREATE INDEX IF NOT EXISTS idx_user_memories_type ON user_memories(user_id, memory_type);

-- ============================================================================
-- Session Preferences Table
-- Stores per-session configuration like current model, routing strategy, etc.
-- ============================================================================
CREATE TABLE IF NOT EXISTS session_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    preference_key VARCHAR(255) NOT NULL,
    preference_value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT session_preferences_unique UNIQUE (session_id, preference_key)
);

CREATE INDEX IF NOT EXISTS idx_session_preferences_session ON session_preferences(session_id);

-- ============================================================================
-- A2A Messages Table
-- Agent-to-Agent protocol messages for multi-agent orchestration
-- Based on https://a2a-protocol.org/latest/
-- ============================================================================
CREATE TABLE IF NOT EXISTS a2a_messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    from_agent VARCHAR(255) NOT NULL,
    to_agent VARCHAR(255),  -- NULL for broadcast messages
    message_type VARCHAR(50) NOT NULL,
    payload TEXT NOT NULL,
    correlation_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata TEXT DEFAULT '{}',
    ttl_seconds INTEGER NOT NULL DEFAULT 30,
    hop_count INTEGER NOT NULL DEFAULT 0,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_a2a_messages_session ON a2a_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_to_agent ON a2a_messages(session_id, to_agent);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_correlation ON a2a_messages(correlation_id);
CREATE INDEX IF NOT EXISTS idx_a2a_messages_pending ON a2a_messages(session_id, to_agent, processed) WHERE processed = FALSE;
CREATE INDEX IF NOT EXISTS idx_a2a_messages_timestamp ON a2a_messages(timestamp);

-- ============================================================================
-- Extended Bot Memory Table
-- Enhanced memory with TTL and different memory types
-- ============================================================================
CREATE TABLE IF NOT EXISTS bot_memory_extended (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID,  -- NULL for long-term memory
    memory_type VARCHAR(20) NOT NULL CHECK (memory_type IN ('short', 'long', 'episodic')),
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    ttl_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    CONSTRAINT bot_memory_extended_unique UNIQUE (bot_id, session_id, key)
);

CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_bot ON bot_memory_extended(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_session ON bot_memory_extended(bot_id, session_id);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_type ON bot_memory_extended(bot_id, memory_type);
CREATE INDEX IF NOT EXISTS idx_bot_memory_ext_expires ON bot_memory_extended(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- Knowledge Graph Entities Table
-- For graph-based memory and entity relationships
-- ============================================================================
CREATE TABLE IF NOT EXISTS kg_entities (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_name VARCHAR(500) NOT NULL,
    properties JSONB DEFAULT '{}',
    embedding_vector BYTEA,  -- For vector similarity search
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT kg_entities_unique UNIQUE (bot_id, entity_type, entity_name)
);

CREATE INDEX IF NOT EXISTS idx_kg_entities_bot ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(bot_id, entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);

-- ============================================================================
-- Knowledge Graph Relationships Table
-- For storing relationships between entities
-- ============================================================================
CREATE TABLE IF NOT EXISTS kg_relationships (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    from_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    weight FLOAT DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT kg_relationships_unique UNIQUE (from_entity_id, to_entity_id, relationship_type)
);

CREATE INDEX IF NOT EXISTS idx_kg_rel_bot ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_rel_type ON kg_relationships(bot_id, relationship_type);

-- ============================================================================
-- Episodic Memory Table
-- For storing conversation summaries and episodes
-- ============================================================================
CREATE TABLE IF NOT EXISTS episodic_memories (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    user_id UUID NOT NULL,
    session_id UUID,
    summary TEXT NOT NULL,
    key_topics JSONB DEFAULT '[]',
    decisions JSONB DEFAULT '[]',
    action_items JSONB DEFAULT '[]',
    message_count INTEGER NOT NULL DEFAULT 0,
    start_timestamp TIMESTAMPTZ NOT NULL,
    end_timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_episodic_bot ON episodic_memories(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodic_user ON episodic_memories(user_id);
CREATE INDEX IF NOT EXISTS idx_episodic_session ON episodic_memories(session_id);
CREATE INDEX IF NOT EXISTS idx_episodic_time ON episodic_memories(bot_id, user_id, created_at);

-- ============================================================================
-- Conversation Cost Tracking Table
-- For monitoring LLM usage and costs
-- ============================================================================
CREATE TABLE IF NOT EXISTS conversation_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model_used VARCHAR(100),
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd DECIMAL(10, 6) NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conv_costs_session ON conversation_costs(session_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_user ON conversation_costs(user_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_bot ON conversation_costs(bot_id);
CREATE INDEX IF NOT EXISTS idx_conv_costs_time ON conversation_costs(timestamp);

-- ============================================================================
-- Generated API Tools Table
-- For tracking tools generated from OpenAPI specs
-- ============================================================================
CREATE TABLE IF NOT EXISTS generated_api_tools (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    api_name VARCHAR(255) NOT NULL,
    spec_url TEXT NOT NULL,
    spec_hash VARCHAR(64) NOT NULL,
    tool_count INTEGER NOT NULL DEFAULT 0,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT generated_api_tools_unique UNIQUE (bot_id, api_name)
);

CREATE INDEX IF NOT EXISTS idx_gen_api_tools_bot ON generated_api_tools(bot_id);

-- ============================================================================
-- Session Bots Junction Table (if not exists)
-- For multi-agent sessions
-- ============================================================================
CREATE TABLE IF NOT EXISTS session_bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    bot_name VARCHAR(255) NOT NULL,
    trigger_config JSONB DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT session_bots_unique UNIQUE (session_id, bot_name)
);

CREATE INDEX IF NOT EXISTS idx_session_bots_session ON session_bots(session_id);
CREATE INDEX IF NOT EXISTS idx_session_bots_active ON session_bots(session_id, is_active);

-- ============================================================================
-- Cleanup function for expired A2A messages
-- ============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_a2a_messages()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM a2a_messages
    WHERE ttl_seconds > 0
    AND timestamp + (ttl_seconds || ' seconds')::INTERVAL < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Cleanup function for expired bot memory
-- ============================================================================
CREATE OR REPLACE FUNCTION cleanup_expired_bot_memory()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM bot_memory_extended
    WHERE expires_at IS NOT NULL AND expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Trigger to update updated_at timestamp
-- ============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables with updated_at
DROP TRIGGER IF EXISTS update_user_memories_updated_at ON user_memories;
CREATE TRIGGER update_user_memories_updated_at
    BEFORE UPDATE ON user_memories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_bot_memory_extended_updated_at ON bot_memory_extended;
CREATE TRIGGER update_bot_memory_extended_updated_at
    BEFORE UPDATE ON bot_memory_extended
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Bot Reflections Table
-- For storing agent self-reflection analysis results
-- ============================================================================
CREATE TABLE IF NOT EXISTS bot_reflections (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    reflection_type TEXT NOT NULL,
    score FLOAT NOT NULL DEFAULT 0.0,
    insights TEXT NOT NULL DEFAULT '[]',
    improvements TEXT NOT NULL DEFAULT '[]',
    positive_patterns TEXT NOT NULL DEFAULT '[]',
    concerns TEXT NOT NULL DEFAULT '[]',
    raw_response TEXT NOT NULL DEFAULT '',
    messages_analyzed INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bot_reflections_bot ON bot_reflections(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_session ON bot_reflections(session_id);
CREATE INDEX IF NOT EXISTS idx_bot_reflections_time ON bot_reflections(bot_id, created_at);

-- ============================================================================
-- Conversation Messages Table
-- For storing conversation history (if not already exists)
-- ============================================================================
CREATE TABLE IF NOT EXISTS conversation_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    user_id UUID,
    role VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    token_count INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conv_messages_session ON conversation_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_conv_messages_time ON conversation_messages(session_id, created_at);
CREATE INDEX IF NOT EXISTS idx_conv_messages_bot ON conversation_messages(bot_id);
-- Migration: 6.1.2_phase3_phase4
-- Description: Phase 3 and Phase 4 multi-agent features
-- Features:
--   - Episodic memory (conversation summaries)
--   - Knowledge graphs (entity relationships)
--   - Human-in-the-loop approvals
--   - LLM observability and cost tracking

-- ============================================
-- EPISODIC MEMORY TABLES
-- ============================================

-- Conversation episodes (summaries)
CREATE TABLE IF NOT EXISTS conversation_episodes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    summary TEXT NOT NULL,
    key_topics JSONB NOT NULL DEFAULT '[]',
    decisions JSONB NOT NULL DEFAULT '[]',
    action_items JSONB NOT NULL DEFAULT '[]',
    sentiment JSONB NOT NULL DEFAULT '{"score": 0, "label": "neutral", "confidence": 0.5}',
    resolution VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message_count INTEGER NOT NULL DEFAULT 0,
    message_ids JSONB NOT NULL DEFAULT '[]',
    conversation_start TIMESTAMP WITH TIME ZONE NOT NULL,
    conversation_end TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for episodic memory
CREATE INDEX IF NOT EXISTS idx_episodes_user_id ON conversation_episodes(user_id);
CREATE INDEX IF NOT EXISTS idx_episodes_bot_id ON conversation_episodes(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodes_session_id ON conversation_episodes(session_id);
CREATE INDEX IF NOT EXISTS idx_episodes_created_at ON conversation_episodes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_key_topics ON conversation_episodes USING GIN(key_topics);
CREATE INDEX IF NOT EXISTS idx_episodes_resolution ON conversation_episodes(resolution);

-- Full-text search on summaries
CREATE INDEX IF NOT EXISTS idx_episodes_summary_fts ON conversation_episodes
    USING GIN(to_tsvector('english', summary));

-- ============================================
-- KNOWLEDGE GRAPH TABLES - Add missing columns
-- (Tables created earlier in this migration)
-- ============================================

-- Add missing columns to kg_entities
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS aliases JSONB NOT NULL DEFAULT '[]';
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0;
ALTER TABLE kg_entities ADD COLUMN IF NOT EXISTS source VARCHAR(50) NOT NULL DEFAULT 'manual';

-- Add missing columns to kg_relationships
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0;
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS bidirectional BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE kg_relationships ADD COLUMN IF NOT EXISTS source VARCHAR(50) NOT NULL DEFAULT 'manual';

-- Indexes for knowledge graph
CREATE INDEX IF NOT EXISTS idx_kg_entities_bot_id ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_lower ON kg_entities(LOWER(entity_name));
CREATE INDEX IF NOT EXISTS idx_kg_entities_aliases ON kg_entities USING GIN(aliases);

CREATE INDEX IF NOT EXISTS idx_kg_relationships_bot_id ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_type ON kg_relationships(relationship_type);

-- Full-text search on entity names
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_fts ON kg_entities
    USING GIN(to_tsvector('english', entity_name));

-- ============================================
-- HUMAN-IN-THE-LOOP APPROVAL TABLES
-- ============================================

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    approval_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    channel VARCHAR(50) NOT NULL,
    recipient VARCHAR(500) NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    message TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    default_action VARCHAR(50),
    current_level INTEGER NOT NULL DEFAULT 1,
    total_levels INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reminders_sent JSONB NOT NULL DEFAULT '[]',
    decision VARCHAR(50),
    decided_by VARCHAR(500),
    decided_at TIMESTAMP WITH TIME ZONE,
    comments TEXT
);

-- Approval chains
CREATE TABLE IF NOT EXISTS approval_chains (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    bot_id UUID NOT NULL,
    levels JSONB NOT NULL DEFAULT '[]',
    stop_on_reject BOOLEAN NOT NULL DEFAULT true,
    require_all BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Approval audit log
CREATE TABLE IF NOT EXISTS approval_audit_log (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    actor VARCHAR(500) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(50),
    user_agent TEXT
);

-- Approval tokens (for secure links)
CREATE TABLE IF NOT EXISTS approval_tokens (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL UNIQUE,
    action VARCHAR(50) NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for approval tables
CREATE INDEX IF NOT EXISTS idx_approval_requests_bot_id ON approval_requests(bot_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_session_id ON approval_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_status ON approval_requests(status);
CREATE INDEX IF NOT EXISTS idx_approval_requests_expires_at ON approval_requests(expires_at);
CREATE INDEX IF NOT EXISTS idx_approval_requests_pending ON approval_requests(status, expires_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_approval_audit_request_id ON approval_audit_log(request_id);
CREATE INDEX IF NOT EXISTS idx_approval_audit_timestamp ON approval_audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_approval_tokens_token ON approval_tokens(token);
CREATE INDEX IF NOT EXISTS idx_approval_tokens_request_id ON approval_tokens(request_id);

-- ============================================
-- LLM OBSERVABILITY TABLES
-- ============================================

-- LLM request metrics
CREATE TABLE IF NOT EXISTS llm_metrics (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model VARCHAR(200) NOT NULL,
    request_type VARCHAR(50) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    latency_ms BIGINT NOT NULL DEFAULT 0,
    ttft_ms BIGINT,
    cached BOOLEAN NOT NULL DEFAULT false,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    estimated_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Aggregated metrics (hourly rollup)
CREATE TABLE IF NOT EXISTS llm_metrics_hourly (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    hour TIMESTAMP WITH TIME ZONE NOT NULL,
    total_requests BIGINT NOT NULL DEFAULT 0,
    successful_requests BIGINT NOT NULL DEFAULT 0,
    failed_requests BIGINT NOT NULL DEFAULT 0,
    cache_hits BIGINT NOT NULL DEFAULT 0,
    cache_misses BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p50_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    p99_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_latency_ms BIGINT NOT NULL DEFAULT 0,
    min_latency_ms BIGINT NOT NULL DEFAULT 0,
    requests_by_model JSONB NOT NULL DEFAULT '{}',
    tokens_by_model JSONB NOT NULL DEFAULT '{}',
    cost_by_model JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, hour)
);

-- Budget tracking
CREATE TABLE IF NOT EXISTS llm_budget (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL UNIQUE,
    daily_limit DOUBLE PRECISION NOT NULL DEFAULT 100,
    monthly_limit DOUBLE PRECISION NOT NULL DEFAULT 2000,
    alert_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.8,
    daily_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    monthly_spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_reset_date DATE NOT NULL DEFAULT CURRENT_DATE,
    monthly_reset_date DATE NOT NULL DEFAULT DATE_TRUNC('month', CURRENT_DATE)::DATE,
    daily_alert_sent BOOLEAN NOT NULL DEFAULT false,
    monthly_alert_sent BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Trace events
CREATE TABLE IF NOT EXISTS llm_traces (
    id UUID PRIMARY KEY,
    parent_id UUID,
    trace_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    component VARCHAR(100) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    duration_ms BIGINT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE,
    attributes JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress',
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for observability tables
CREATE INDEX IF NOT EXISTS idx_llm_metrics_bot_id ON llm_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_session_id ON llm_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_timestamp ON llm_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_model ON llm_metrics(model);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_bot_id ON llm_metrics_hourly(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_hour ON llm_metrics_hourly(hour DESC);

CREATE INDEX IF NOT EXISTS idx_llm_traces_trace_id ON llm_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_llm_traces_start_time ON llm_traces(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_llm_traces_component ON llm_traces(component);

-- ============================================
-- WORKFLOW TABLES
-- ============================================

-- Workflow definitions
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]',
    triggers JSONB NOT NULL DEFAULT '[]',
    error_handling JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Workflow executions
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL,
    session_id UUID,
    initiated_by UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    current_step INTEGER NOT NULL DEFAULT 0,
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    step_results JSONB NOT NULL DEFAULT '[]',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Workflow step executions
CREATE TABLE IF NOT EXISTS workflow_step_executions (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    step_name VARCHAR(200) NOT NULL,
    step_index INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms BIGINT
);

-- Indexes for workflow tables
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_bot_id ON workflow_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_bot_id ON workflow_executions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_step_executions_execution_id ON workflow_step_executions(execution_id);

-- ============================================
-- FUNCTIONS AND TRIGGERS
-- ============================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON workflow_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_llm_budget_updated_at ON llm_budget;
CREATE TRIGGER update_llm_budget_updated_at
    BEFORE UPDATE ON llm_budget
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to aggregate hourly metrics
CREATE OR REPLACE FUNCTION aggregate_llm_metrics_hourly()
RETURNS void AS $$
DECLARE
    last_hour TIMESTAMP WITH TIME ZONE;
BEGIN
    last_hour := DATE_TRUNC('hour', NOW() - INTERVAL '1 hour');

    INSERT INTO llm_metrics_hourly (
        id, bot_id, hour, total_requests, successful_requests, failed_requests,
        cache_hits, cache_misses, total_input_tokens, total_output_tokens,
        total_tokens, total_cost, avg_latency_ms, p50_latency_ms, p95_latency_ms,
        p99_latency_ms, max_latency_ms, min_latency_ms, requests_by_model,
        tokens_by_model, cost_by_model
    )
    SELECT
        gen_random_uuid(),
        bot_id,
        last_hour,
        COUNT(*),
        COUNT(*) FILTER (WHERE success = true),
        COUNT(*) FILTER (WHERE success = false),
        COUNT(*) FILTER (WHERE cached = true),
        COUNT(*) FILTER (WHERE cached = false),
        SUM(input_tokens),
        SUM(output_tokens),
        SUM(total_tokens),
        SUM(estimated_cost),
        AVG(latency_ms),
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms),
        MAX(latency_ms),
        MIN(latency_ms),
        jsonb_object_agg(model, model_count) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_tokens) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_cost) FILTER (WHERE model IS NOT NULL)
    FROM (
        SELECT
            bot_id, model, success, cached, input_tokens, output_tokens,
            total_tokens, estimated_cost, latency_ms,
            COUNT(*) OVER (PARTITION BY bot_id, model) as model_count,
            SUM(total_tokens) OVER (PARTITION BY bot_id, model) as model_tokens,
            SUM(estimated_cost) OVER (PARTITION BY bot_id, model) as model_cost
        FROM llm_metrics
        WHERE timestamp >= last_hour
        AND timestamp < last_hour + INTERVAL '1 hour'
    ) sub
    GROUP BY bot_id
    ON CONFLICT (bot_id, hour) DO UPDATE SET
        total_requests = EXCLUDED.total_requests,
        successful_requests = EXCLUDED.successful_requests,
        failed_requests = EXCLUDED.failed_requests,
        cache_hits = EXCLUDED.cache_hits,
        cache_misses = EXCLUDED.cache_misses,
        total_input_tokens = EXCLUDED.total_input_tokens,
        total_output_tokens = EXCLUDED.total_output_tokens,
        total_tokens = EXCLUDED.total_tokens,
        total_cost = EXCLUDED.total_cost,
        avg_latency_ms = EXCLUDED.avg_latency_ms,
        p50_latency_ms = EXCLUDED.p50_latency_ms,
        p95_latency_ms = EXCLUDED.p95_latency_ms,
        p99_latency_ms = EXCLUDED.p99_latency_ms,
        max_latency_ms = EXCLUDED.max_latency_ms,
        min_latency_ms = EXCLUDED.min_latency_ms,
        requests_by_model = EXCLUDED.requests_by_model,
        tokens_by_model = EXCLUDED.tokens_by_model,
        cost_by_model = EXCLUDED.cost_by_model;
END;
$$ LANGUAGE plpgsql;

-- Function to reset daily budget
CREATE OR REPLACE FUNCTION reset_daily_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET daily_spend = 0,
        daily_reset_date = CURRENT_DATE,
        daily_alert_sent = false
    WHERE daily_reset_date < CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Function to reset monthly budget
CREATE OR REPLACE FUNCTION reset_monthly_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET monthly_spend = 0,
        monthly_reset_date = DATE_TRUNC('month', CURRENT_DATE)::DATE,
        monthly_alert_sent = false
    WHERE monthly_reset_date < DATE_TRUNC('month', CURRENT_DATE)::DATE;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- VIEWS
-- ============================================

-- View for recent episode summaries with user info
CREATE OR REPLACE VIEW v_recent_episodes AS
SELECT
    e.id,
    e.user_id,
    e.bot_id,
    e.session_id,
    e.summary,
    e.key_topics,
    e.sentiment,
    e.resolution,
    e.message_count,
    e.created_at,
    e.conversation_start,
    e.conversation_end
FROM conversation_episodes e
ORDER BY e.created_at DESC;

-- View for knowledge graph statistics
CREATE OR REPLACE VIEW v_kg_stats AS
SELECT
    bot_id,
    COUNT(DISTINCT id) as total_entities,
    COUNT(DISTINCT entity_type) as entity_types,
    (SELECT COUNT(*) FROM kg_relationships r WHERE r.bot_id = e.bot_id) as total_relationships
FROM kg_entities e
GROUP BY bot_id;

-- View for approval status summary
CREATE OR REPLACE VIEW v_approval_summary AS
SELECT
    bot_id,
    status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (COALESCE(decided_at, NOW()) - created_at))) as avg_resolution_seconds
FROM approval_requests
GROUP BY bot_id, status;

-- View for LLM usage summary (last 24 hours)
CREATE OR REPLACE VIEW v_llm_usage_24h AS
SELECT
    bot_id,
    model,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    SUM(estimated_cost) as total_cost,
    AVG(latency_ms) as avg_latency_ms,
    SUM(CASE WHEN cached THEN 1 ELSE 0 END)::FLOAT / COUNT(*) as cache_hit_rate,
    SUM(CASE WHEN success THEN 0 ELSE 1 END)::FLOAT / COUNT(*) as error_rate
FROM llm_metrics
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY bot_id, model;

-- ============================================
-- CLEANUP POLICIES (retention)
-- ============================================

-- Create a cleanup function for old data
CREATE OR REPLACE FUNCTION cleanup_old_observability_data(retention_days INTEGER DEFAULT 30)
RETURNS void AS $$
BEGIN
    -- Delete old LLM metrics (keep hourly aggregates longer)
    DELETE FROM llm_metrics WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old traces
    DELETE FROM llm_traces WHERE start_time < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old approval audit logs
    DELETE FROM approval_audit_log WHERE timestamp < NOW() - (retention_days * 3 || ' days')::INTERVAL;

    -- Delete expired approval tokens
    DELETE FROM approval_tokens WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;
-- Suite Applications Migration
-- Adds tables for: Paper (Documents), Designer (Dialogs), and additional analytics support

-- Paper Documents table
CREATE TABLE IF NOT EXISTS paper_documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled Document',
    content TEXT NOT NULL DEFAULT '',
    owner_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_paper_documents_owner ON paper_documents(owner_id);
CREATE INDEX IF NOT EXISTS idx_paper_documents_updated ON paper_documents(updated_at DESC);

-- Designer Dialogs table
CREATE TABLE IF NOT EXISTS designer_dialogs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    bot_id TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_designer_dialogs_bot ON designer_dialogs(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_active ON designer_dialogs(is_active);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_updated ON designer_dialogs(updated_at DESC);

-- Sources Templates table (for template metadata caching)
CREATE TABLE IF NOT EXISTS source_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'General',
    preview_url TEXT,
    file_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_source_templates_category ON source_templates(category);

-- Analytics Events table (for additional event tracking)
CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    user_id UUID,
    session_id UUID,
    bot_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_session ON analytics_events(session_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at DESC);

-- Analytics Daily Aggregates (for faster dashboard queries)
CREATE TABLE IF NOT EXISTS analytics_daily_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    bot_id UUID,
    metric_name TEXT NOT NULL,
    metric_value BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(date, bot_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_analytics_daily_date ON analytics_daily_aggregates(date DESC);
CREATE INDEX IF NOT EXISTS idx_analytics_daily_bot ON analytics_daily_aggregates(bot_id);

-- Research Search History (for recent searches feature)
CREATE TABLE IF NOT EXISTS research_search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    collection_id TEXT,
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_history_user ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_history_created ON research_search_history(created_at DESC);
-- Email Read Tracking Table
-- Stores sent email tracking data for read receipt functionality
-- Enabled via config.csv: email-read-pixel,true

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

-- Trigger to auto-update updated_at
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
COMMENT ON COLUMN sent_email_tracking.tracking_id IS 'Unique ID embedded in tracking pixel URL';
COMMENT ON COLUMN sent_email_tracking.is_read IS 'Whether the email has been opened (pixel loaded)';
COMMENT ON COLUMN sent_email_tracking.read_count IS 'Number of times the email was opened';
COMMENT ON COLUMN sent_email_tracking.first_read_ip IS 'IP address of first email open';
COMMENT ON COLUMN sent_email_tracking.last_read_ip IS 'IP address of most recent email open';
-- ============================================
-- TABLE KEYWORD SUPPORT (from 6.1.0_table_keyword)
-- ============================================

-- Migration for TABLE keyword support
-- Stores dynamic table definitions created via BASIC TABLE...END TABLE syntax

-- Table to store dynamic table definitions (metadata)
CREATE TABLE IF NOT EXISTS dynamic_table_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    table_name VARCHAR(255) NOT NULL,
    connection_name VARCHAR(255) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,

    -- Ensure unique table name per bot and connection
    CONSTRAINT unique_bot_table_connection UNIQUE (bot_id, table_name, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_dynamic_table_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Table to store field definitions for dynamic tables
CREATE TABLE IF NOT EXISTS dynamic_table_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_definition_id UUID NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    field_type VARCHAR(100) NOT NULL,
    field_length INTEGER,
    field_precision INTEGER,
    is_key BOOLEAN DEFAULT false,
    is_nullable BOOLEAN DEFAULT true,
    default_value TEXT,
    reference_table VARCHAR(255),
    field_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure unique field name per table definition
    CONSTRAINT unique_table_field UNIQUE (table_definition_id, field_name),

    -- Foreign key to table definitions
    CONSTRAINT fk_field_table_definition
        FOREIGN KEY (table_definition_id)
        REFERENCES dynamic_table_definitions(id)
        ON DELETE CASCADE
);

-- Table to store external database connections (from config.csv conn-* entries)
CREATE TABLE IF NOT EXISTS external_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    connection_name VARCHAR(255) NOT NULL,
    driver VARCHAR(100) NOT NULL,
    server VARCHAR(255) NOT NULL,
    port INTEGER,
    database_name VARCHAR(255),
    username VARCHAR(255),
    password_encrypted TEXT,
    additional_params JSONB DEFAULT '{}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_connected_at TIMESTAMPTZ,

    -- Ensure unique connection name per bot
    CONSTRAINT unique_bot_connection UNIQUE (bot_id, connection_name),

    -- Foreign key to bots table
    CONSTRAINT fk_external_connection_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_bot_id
    ON dynamic_table_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_name
    ON dynamic_table_definitions(table_name);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_definitions_connection
    ON dynamic_table_definitions(connection_name);

CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_table_id
    ON dynamic_table_fields(table_definition_id);
CREATE INDEX IF NOT EXISTS idx_dynamic_table_fields_name
    ON dynamic_table_fields(field_name);

CREATE INDEX IF NOT EXISTS idx_external_connections_bot_id
    ON external_connections(bot_id);
CREATE INDEX IF NOT EXISTS idx_external_connections_name
    ON external_connections(connection_name);

-- Create trigger to update updated_at timestamp for dynamic_table_definitions
CREATE OR REPLACE FUNCTION update_dynamic_table_definitions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER dynamic_table_definitions_updated_at_trigger
    BEFORE UPDATE ON dynamic_table_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_dynamic_table_definitions_updated_at();

-- Create trigger to update updated_at timestamp for external_connections
CREATE OR REPLACE FUNCTION update_external_connections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER external_connections_updated_at_trigger
    BEFORE UPDATE ON external_connections
    FOR EACH ROW
    EXECUTE FUNCTION update_external_connections_updated_at();

-- ============================================================================
-- CONFIG ID TYPE FIXES (from 6.1.1)
-- Fix columns that were created as TEXT but should be UUID
-- ============================================================================

-- For bot_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'bot_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE bot_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For server_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'server_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE server_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For tenant_configuration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'tenant_configuration'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE tenant_configuration
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For model_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'model_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE model_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For connection_configurations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'connection_configurations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE connection_configurations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For component_installations
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_installations'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_installations
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For component_logs
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'component_logs'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE component_logs
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- For gbot_config_sync
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'gbot_config_sync'
               AND column_name = 'id'
               AND data_type = 'text') THEN
        ALTER TABLE gbot_config_sync
            ALTER COLUMN id TYPE UUID USING id::uuid;
    END IF;
END $$;

-- ============================================================================
-- CONNECTED ACCOUNTS (from 6.1.2)
-- OAuth connected accounts for email providers
-- ============================================================================

CREATE TABLE IF NOT EXISTS connected_accounts (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    user_id UUID,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    account_type TEXT NOT NULL DEFAULT 'email',
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    scopes TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    sync_enabled BOOLEAN NOT NULL DEFAULT true,
    sync_interval_seconds INTEGER NOT NULL DEFAULT 300,
    last_sync_at TIMESTAMPTZ,
    last_sync_status TEXT,
    last_sync_error TEXT,
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connected_accounts_bot_id ON connected_accounts(bot_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_user_id ON connected_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_email ON connected_accounts(email);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_provider ON connected_accounts(provider);
CREATE INDEX IF NOT EXISTS idx_connected_accounts_status ON connected_accounts(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_connected_accounts_bot_email ON connected_accounts(bot_id, email);

CREATE TABLE IF NOT EXISTS session_account_associations (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    provider TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT
);

CREATE INDEX IF NOT EXISTS idx_session_account_assoc_session ON session_account_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_account ON session_account_associations(account_id);
CREATE INDEX IF NOT EXISTS idx_session_account_assoc_active ON session_account_associations(session_id, is_active);
CREATE UNIQUE INDEX IF NOT EXISTS idx_session_account_assoc_unique ON session_account_associations(session_id, account_id);

CREATE TABLE IF NOT EXISTS account_sync_items (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    subject TEXT,
    content_preview TEXT,
    sender TEXT,
    recipients TEXT,
    item_date TIMESTAMPTZ,
    folder TEXT,
    labels TEXT,
    has_attachments BOOLEAN DEFAULT false,
    qdrant_point_id TEXT,
    embedding_status TEXT DEFAULT 'pending',
    metadata_json TEXT DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_account_sync_items_account ON account_sync_items(account_id);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_type ON account_sync_items(item_type);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_date ON account_sync_items(item_date);
CREATE INDEX IF NOT EXISTS idx_account_sync_items_embedding ON account_sync_items(embedding_status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_account_sync_items_unique ON account_sync_items(account_id, item_type, item_id);

-- ============================================================================
-- BOT HIERARCHY AND MONITORS (from 6.1.3)
-- Sub-bots, ON EMAIL triggers, ON CHANGE triggers
-- ============================================================================

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
