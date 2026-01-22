-- Legacy Meet Tables extracted from consolidated

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
