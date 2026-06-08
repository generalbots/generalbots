CREATE TABLE IF NOT EXISTS meet_recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    meeting_id UUID,
    title VARCHAR(500) NOT NULL,
    recording_path TEXT NOT NULL,
    duration_seconds INTEGER,
    file_size BIGINT,
    language VARCHAR(10) NOT NULL DEFAULT 'en-US',
    status VARCHAR(32) NOT NULL DEFAULT 'recorded',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meet_recordings_bot ON meet_recordings(bot_id);
CREATE INDEX IF NOT EXISTS idx_meet_recordings_meeting ON meet_recordings(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meet_recordings_status ON meet_recordings(status);

CREATE TABLE IF NOT EXISTS meet_transcriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recording_id UUID NOT NULL,
    full_text TEXT NOT NULL DEFAULT '',
    segments JSONB NOT NULL DEFAULT '[]',
    speakers JSONB NOT NULL DEFAULT '[]',
    language VARCHAR(10) NOT NULL DEFAULT 'en-US',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meet_transcriptions_recording ON meet_transcriptions(recording_id);

CREATE TABLE IF NOT EXISTS meet_minutes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    recording_id UUID,
    meeting_id UUID,
    title VARCHAR(500) NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    key_points JSONB NOT NULL DEFAULT '[]',
    action_items JSONB NOT NULL DEFAULT '[]',
    decisions JSONB NOT NULL DEFAULT '[]',
    attendees JSONB NOT NULL DEFAULT '[]',
    duration_minutes INTEGER,
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meet_minutes_bot ON meet_minutes(bot_id);
CREATE INDEX IF NOT EXISTS idx_meet_minutes_recording ON meet_minutes(recording_id);
CREATE INDEX IF NOT EXISTS idx_meet_minutes_meeting ON meet_minutes(meeting_id);
CREATE INDEX IF NOT EXISTS idx_meet_minutes_status ON meet_minutes(status);

CREATE TABLE IF NOT EXISTS meet_minute_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    minute_id UUID NOT NULL,
    user_id UUID NOT NULL,
    signature_id UUID,
    signed_hash VARCHAR(128) NOT NULL,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(64)
);

CREATE INDEX IF NOT EXISTS idx_meet_sigs_minute ON meet_minute_signatures(minute_id);
CREATE INDEX IF NOT EXISTS idx_meet_sigs_user ON meet_minute_signatures(user_id);
