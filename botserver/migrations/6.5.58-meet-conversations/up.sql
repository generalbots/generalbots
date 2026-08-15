-- Meet group conversations (standalone chat rooms, not meeting_rooms).
-- These back the /conversations/* endpoints; previously those routes had no
-- backing store and fabricated responses.

CREATE TABLE IF NOT EXISTS meet_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    conversation_type VARCHAR(20) NOT NULL DEFAULT 'group',
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT check_conv_type CHECK (conversation_type IN ('group', 'direct', 'channel'))
);

CREATE INDEX IF NOT EXISTS idx_meet_conversations_bot_id ON meet_conversations(bot_id);
CREATE INDEX IF NOT EXISTS idx_meet_conversations_updated_at ON meet_conversations(updated_at);

CREATE TABLE IF NOT EXISTS meet_conversation_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES meet_conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    display_name VARCHAR(255),
    role VARCHAR(20) NOT NULL DEFAULT 'member',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ,
    UNIQUE (conversation_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_meet_conv_members_conversation ON meet_conversation_members(conversation_id);

CREATE TABLE IF NOT EXISTS meet_conversation_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES meet_conversations(id) ON DELETE CASCADE,
    sender_id UUID,
    sender_name VARCHAR(255) NOT NULL,
    message_type VARCHAR(20) NOT NULL DEFAULT 'text',
    content TEXT NOT NULL,
    reply_to UUID REFERENCES meet_conversation_messages(id) ON DELETE SET NULL,
    attachments JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meet_conv_messages_conversation ON meet_conversation_messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_meet_conv_messages_created_at ON meet_conversation_messages(created_at);

CREATE TABLE IF NOT EXISTS meet_conversation_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES meet_conversation_messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    reaction VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (message_id, user_id, reaction)
);

CREATE INDEX IF NOT EXISTS idx_meet_conv_reactions_message ON meet_conversation_reactions(message_id);

CREATE TABLE IF NOT EXISTS meet_conversation_calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES meet_conversations(id) ON DELETE CASCADE,
    call_type VARCHAR(20) NOT NULL DEFAULT 'audio',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    started_by UUID,
    participants JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_recording BOOLEAN NOT NULL DEFAULT FALSE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    recording_url TEXT
);

CREATE INDEX IF NOT EXISTS idx_meet_conv_calls_conversation ON meet_conversation_calls(conversation_id);

CREATE TABLE IF NOT EXISTS meet_conversation_screen_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES meet_conversations(id) ON DELETE CASCADE,
    call_id UUID REFERENCES meet_conversation_calls(id) ON DELETE SET NULL,
    user_id UUID,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    quality VARCHAR(10) NOT NULL DEFAULT 'high',
    audio_included BOOLEAN NOT NULL DEFAULT FALSE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_meet_conv_screen_conversation ON meet_conversation_screen_shares(conversation_id);
