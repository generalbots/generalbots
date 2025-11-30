-- Multi-Agent Bots Migration
-- Enables multiple bots to participate in conversations based on triggers

-- ============================================================================
-- BOTS TABLE - Bot definitions
-- ============================================================================

CREATE TABLE IF NOT EXISTS bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    system_prompt TEXT,
    model_config JSONB DEFAULT '{}',
    tools JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_bots_name ON bots(name);
CREATE INDEX idx_bots_active ON bots(is_active) WHERE is_active = true;

-- ============================================================================
-- BOT_TRIGGERS TABLE - Trigger configurations for bots
-- ============================================================================

CREATE TABLE IF NOT EXISTS bot_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    trigger_type VARCHAR(50) NOT NULL, -- 'keyword', 'tool', 'schedule', 'event', 'always'
    trigger_config JSONB NOT NULL DEFAULT '{}',
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_trigger_type CHECK (
        trigger_type IN ('keyword', 'tool', 'schedule', 'event', 'always')
    )
);

CREATE INDEX idx_bot_triggers_bot_id ON bot_triggers(bot_id);
CREATE INDEX idx_bot_triggers_type ON bot_triggers(trigger_type);

-- ============================================================================
-- SESSION_BOTS TABLE - Bots active in a session
-- ============================================================================

CREATE TABLE IF NOT EXISTS session_bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    bot_name VARCHAR(255) NOT NULL,
    trigger_config JSONB NOT NULL DEFAULT '{}',
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    left_at TIMESTAMPTZ,

    CONSTRAINT unique_session_bot UNIQUE (session_id, bot_name)
);

CREATE INDEX idx_session_bots_session ON session_bots(session_id);
CREATE INDEX idx_session_bots_active ON session_bots(session_id, is_active) WHERE is_active = true;

-- ============================================================================
-- BOT_MESSAGES TABLE - Messages from bots in conversations
-- ============================================================================

CREATE TABLE IF NOT EXISTS bot_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID REFERENCES bots(id) ON DELETE SET NULL,
    bot_name VARCHAR(255) NOT NULL,
    user_message_id UUID, -- Reference to the user message this responds to
    content TEXT NOT NULL,
    role VARCHAR(50) DEFAULT 'assistant',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_bot_messages_session ON bot_messages(session_id);
CREATE INDEX idx_bot_messages_bot ON bot_messages(bot_id);
CREATE INDEX idx_bot_messages_created ON bot_messages(created_at);

-- ============================================================================
-- CONVERSATION_BRANCHES TABLE - Branch conversations from a point
-- ============================================================================

CREATE TABLE IF NOT EXISTS conversation_branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_session_id UUID NOT NULL,
    branch_session_id UUID NOT NULL UNIQUE,
    branch_from_message_id UUID NOT NULL,
    branch_name VARCHAR(255),
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_branches_parent ON conversation_branches(parent_session_id);
CREATE INDEX idx_branches_session ON conversation_branches(branch_session_id);

-- ============================================================================
-- ATTACHMENTS TABLE - Files attached to messages
-- ============================================================================

CREATE TABLE IF NOT EXISTS attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID,
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    file_type VARCHAR(50) NOT NULL, -- 'image', 'document', 'audio', 'video', 'code', 'archive', 'other'
    file_name VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(255),
    storage_path TEXT NOT NULL,
    thumbnail_path TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT valid_file_type CHECK (
        file_type IN ('image', 'document', 'audio', 'video', 'code', 'archive', 'other')
    )
);

CREATE INDEX idx_attachments_session ON attachments(session_id);
CREATE INDEX idx_attachments_user ON attachments(user_id);
CREATE INDEX idx_attachments_message ON attachments(message_id);
CREATE INDEX idx_attachments_type ON attachments(file_type);

-- ============================================================================
-- HEAR_WAIT_STATE TABLE - Track HEAR keyword wait states
-- ============================================================================

CREATE TABLE IF NOT EXISTS hear_wait_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    variable_name VARCHAR(255) NOT NULL,
    input_type VARCHAR(50) NOT NULL DEFAULT 'any',
    options JSONB, -- For menu type
    retry_count INT DEFAULT 0,
    max_retries INT DEFAULT 3,
    is_waiting BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '1 hour',
    completed_at TIMESTAMPTZ,

    CONSTRAINT unique_hear_wait UNIQUE (session_id, variable_name)
);

CREATE INDEX idx_hear_wait_session ON hear_wait_states(session_id);
CREATE INDEX idx_hear_wait_active ON hear_wait_states(session_id, is_waiting) WHERE is_waiting = true;

-- ============================================================================
-- PLAY_CONTENT TABLE - Track content projector state
-- ============================================================================

CREATE TABLE IF NOT EXISTS play_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    content_type VARCHAR(50) NOT NULL,
    source_url TEXT NOT NULL,
    title VARCHAR(500),
    options JSONB DEFAULT '{}',
    is_playing BOOLEAN DEFAULT true,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    stopped_at TIMESTAMPTZ,

    CONSTRAINT valid_content_type CHECK (
        content_type IN ('video', 'audio', 'image', 'presentation', 'document',
                        'code', 'spreadsheet', 'pdf', 'markdown', 'html', 'iframe', 'unknown')
    )
);

CREATE INDEX idx_play_content_session ON play_content(session_id);
CREATE INDEX idx_play_content_active ON play_content(session_id, is_playing) WHERE is_playing = true;

-- ============================================================================
-- DEFAULT BOTS - Insert some default specialized bots
-- ============================================================================

INSERT INTO bots (id, name, description, system_prompt, is_active) VALUES
    (gen_random_uuid(), 'fraud-detector',
     'Specialized bot for detecting and handling fraud-related inquiries',
     'You are a fraud detection specialist. Help users identify suspicious activities,
      report unauthorized transactions, and guide them through security procedures.
      Always prioritize user security and recommend immediate action for urgent cases.',
     true),

    (gen_random_uuid(), 'investment-advisor',
     'Specialized bot for investment and financial planning advice',
     'You are an investment advisor. Help users understand investment options,
      analyze portfolio performance, and make informed financial decisions.
      Always remind users that past performance does not guarantee future results.',
     true),

    (gen_random_uuid(), 'loan-specialist',
     'Specialized bot for loan and financing inquiries',
     'You are a loan specialist. Help users understand loan options,
      simulate payments, and guide them through the application process.
      Always disclose interest rates and total costs clearly.',
     true),

    (gen_random_uuid(), 'card-services',
     'Specialized bot for credit and debit card services',
     'You are a card services specialist. Help users manage their cards,
      understand benefits, handle disputes, and manage limits.
      For security, never ask for full card numbers in chat.',
     true)
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- TRIGGERS - Update timestamps automatically
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_bots_updated_at
    BEFORE UPDATE ON bots
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
