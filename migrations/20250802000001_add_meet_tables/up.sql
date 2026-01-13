-- Meeting Rooms table
CREATE TABLE IF NOT EXISTS meeting_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    room_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_by UUID NOT NULL,
    max_participants INTEGER NOT NULL DEFAULT 100,
    is_recording BOOLEAN NOT NULL DEFAULT FALSE,
    is_transcribing BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL DEFAULT 'waiting',
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_rooms_org_id ON meeting_rooms(org_id);
CREATE INDEX idx_meeting_rooms_bot_id ON meeting_rooms(bot_id);
CREATE INDEX idx_meeting_rooms_room_code ON meeting_rooms(room_code);
CREATE INDEX idx_meeting_rooms_status ON meeting_rooms(status);
CREATE INDEX idx_meeting_rooms_created_by ON meeting_rooms(created_by);
CREATE INDEX idx_meeting_rooms_created_at ON meeting_rooms(created_at);

-- Meeting Participants table
CREATE TABLE IF NOT EXISTS meeting_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES meeting_rooms(id) ON DELETE CASCADE,
    user_id UUID,
    participant_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    role VARCHAR(20) NOT NULL DEFAULT 'participant',
    is_bot BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    has_video BOOLEAN NOT NULL DEFAULT FALSE,
    has_audio BOOLEAN NOT NULL DEFAULT FALSE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_participants_room_id ON meeting_participants(room_id);
CREATE INDEX idx_meeting_participants_user_id ON meeting_participants(user_id);
CREATE INDEX idx_meeting_participants_active ON meeting_participants(is_active) WHERE is_active = TRUE;

-- Meeting Recordings table
CREATE TABLE IF NOT EXISTS meeting_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES meeting_rooms(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    recording_type VARCHAR(20) NOT NULL DEFAULT 'video',
    file_url TEXT,
    file_size BIGINT,
    duration_seconds INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT 'recording',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stopped_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_recordings_room_id ON meeting_recordings(room_id);
CREATE INDEX idx_meeting_recordings_org_id ON meeting_recordings(org_id);
CREATE INDEX idx_meeting_recordings_status ON meeting_recordings(status);

-- Meeting Transcriptions table
CREATE TABLE IF NOT EXISTS meeting_transcriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES meeting_rooms(id) ON DELETE CASCADE,
    recording_id UUID REFERENCES meeting_recordings(id) ON DELETE SET NULL,
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    participant_id UUID REFERENCES meeting_participants(id) ON DELETE SET NULL,
    speaker_name VARCHAR(255),
    content TEXT NOT NULL,
    start_time DECIMAL(10,3) NOT NULL,
    end_time DECIMAL(10,3) NOT NULL,
    confidence DECIMAL(5,4),
    language VARCHAR(10) DEFAULT 'en',
    is_final BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_transcriptions_room_id ON meeting_transcriptions(room_id);
CREATE INDEX idx_meeting_transcriptions_recording_id ON meeting_transcriptions(recording_id);
CREATE INDEX idx_meeting_transcriptions_participant_id ON meeting_transcriptions(participant_id);
CREATE INDEX idx_meeting_transcriptions_created_at ON meeting_transcriptions(created_at);

-- Meeting Whiteboards table
CREATE TABLE IF NOT EXISTS meeting_whiteboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID REFERENCES meeting_rooms(id) ON DELETE SET NULL,
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    background_color VARCHAR(20) DEFAULT '#ffffff',
    grid_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    grid_size INTEGER DEFAULT 20,
    elements JSONB NOT NULL DEFAULT '[]'::jsonb,
    version INTEGER NOT NULL DEFAULT 1,
    created_by UUID NOT NULL,
    last_modified_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_whiteboards_room_id ON meeting_whiteboards(room_id);
CREATE INDEX idx_meeting_whiteboards_org_id ON meeting_whiteboards(org_id);
CREATE INDEX idx_meeting_whiteboards_created_by ON meeting_whiteboards(created_by);

-- Whiteboard Elements table (for granular element storage)
CREATE TABLE IF NOT EXISTS whiteboard_elements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    whiteboard_id UUID NOT NULL REFERENCES meeting_whiteboards(id) ON DELETE CASCADE,
    element_type VARCHAR(50) NOT NULL,
    position_x DECIMAL(10,2) NOT NULL,
    position_y DECIMAL(10,2) NOT NULL,
    width DECIMAL(10,2),
    height DECIMAL(10,2),
    rotation DECIMAL(5,2) DEFAULT 0,
    z_index INTEGER NOT NULL DEFAULT 0,
    properties JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_whiteboard_elements_whiteboard_id ON whiteboard_elements(whiteboard_id);
CREATE INDEX idx_whiteboard_elements_type ON whiteboard_elements(element_type);

-- Whiteboard Export History table
CREATE TABLE IF NOT EXISTS whiteboard_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    whiteboard_id UUID NOT NULL REFERENCES meeting_whiteboards(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    export_format VARCHAR(20) NOT NULL,
    file_url TEXT,
    file_size BIGINT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    requested_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_whiteboard_exports_whiteboard_id ON whiteboard_exports(whiteboard_id);
CREATE INDEX idx_whiteboard_exports_org_id ON whiteboard_exports(org_id);
CREATE INDEX idx_whiteboard_exports_status ON whiteboard_exports(status);

-- Meeting Chat Messages table
CREATE TABLE IF NOT EXISTS meeting_chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES meeting_rooms(id) ON DELETE CASCADE,
    participant_id UUID REFERENCES meeting_participants(id) ON DELETE SET NULL,
    sender_name VARCHAR(255) NOT NULL,
    message_type VARCHAR(20) NOT NULL DEFAULT 'text',
    content TEXT NOT NULL,
    reply_to_id UUID REFERENCES meeting_chat_messages(id) ON DELETE SET NULL,
    is_system_message BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_chat_messages_room_id ON meeting_chat_messages(room_id);
CREATE INDEX idx_meeting_chat_messages_participant_id ON meeting_chat_messages(participant_id);
CREATE INDEX idx_meeting_chat_messages_created_at ON meeting_chat_messages(created_at);

-- Scheduled Meetings table
CREATE TABLE IF NOT EXISTS scheduled_meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    room_id UUID REFERENCES meeting_rooms(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    organizer_id UUID NOT NULL,
    scheduled_start TIMESTAMPTZ NOT NULL,
    scheduled_end TIMESTAMPTZ NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    recurrence_rule TEXT,
    attendees JSONB NOT NULL DEFAULT '[]'::jsonb,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled',
    reminder_sent BOOLEAN NOT NULL DEFAULT FALSE,
    calendar_event_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scheduled_meetings_org_id ON scheduled_meetings(org_id);
CREATE INDEX idx_scheduled_meetings_organizer_id ON scheduled_meetings(organizer_id);
CREATE INDEX idx_scheduled_meetings_scheduled_start ON scheduled_meetings(scheduled_start);
CREATE INDEX idx_scheduled_meetings_status ON scheduled_meetings(status);
