CREATE TABLE attendant_queues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    max_wait_minutes INTEGER NOT NULL DEFAULT 30,
    auto_assign BOOLEAN NOT NULL DEFAULT TRUE,
    working_hours JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    session_number VARCHAR(50) NOT NULL,
    channel VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255),
    customer_email VARCHAR(255),
    customer_phone VARCHAR(50),
    status VARCHAR(50) NOT NULL DEFAULT 'waiting',
    priority INTEGER NOT NULL DEFAULT 0,
    agent_id UUID,
    queue_id UUID REFERENCES attendant_queues(id) ON DELETE SET NULL,
    subject VARCHAR(500),
    initial_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_at TIMESTAMPTZ,
    first_response_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    wait_time_seconds INTEGER,
    handle_time_seconds INTEGER,
    satisfaction_rating INTEGER,
    satisfaction_comment TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    notes TEXT,
    transfer_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_session_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES attendant_sessions(id) ON DELETE CASCADE,
    sender_type VARCHAR(20) NOT NULL,
    sender_id UUID,
    sender_name VARCHAR(255),
    content TEXT NOT NULL,
    content_type VARCHAR(50) NOT NULL DEFAULT 'text',
    attachments JSONB NOT NULL DEFAULT '[]',
    is_internal BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_queue_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_id UUID NOT NULL REFERENCES attendant_queues(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    max_concurrent INTEGER NOT NULL DEFAULT 3,
    priority INTEGER NOT NULL DEFAULT 0,
    skills TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(queue_id, agent_id)
);

CREATE TABLE attendant_agent_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'offline',
    status_message VARCHAR(255),
    current_sessions INTEGER NOT NULL DEFAULT 0,
    max_sessions INTEGER NOT NULL DEFAULT 5,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    break_started_at TIMESTAMPTZ,
    break_reason VARCHAR(255),
    available_since TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, agent_id)
);

CREATE TABLE attendant_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES attendant_sessions(id) ON DELETE CASCADE,
    from_agent_id UUID,
    to_agent_id UUID,
    to_queue_id UUID REFERENCES attendant_queues(id) ON DELETE SET NULL,
    reason VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_canned_responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    shortcut VARCHAR(50),
    category VARCHAR(100),
    queue_id UUID REFERENCES attendant_queues(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(20),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_wrap_up_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    requires_notes BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE attendant_session_wrap_up (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES attendant_sessions(id) ON DELETE CASCADE,
    wrap_up_code_id UUID REFERENCES attendant_wrap_up_codes(id) ON DELETE SET NULL,
    notes TEXT,
    follow_up_required BOOLEAN NOT NULL DEFAULT FALSE,
    follow_up_date DATE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(session_id)
);

CREATE INDEX idx_attendant_queues_org_bot ON attendant_queues(org_id, bot_id);
CREATE INDEX idx_attendant_queues_active ON attendant_queues(is_active);

CREATE INDEX idx_attendant_sessions_org_bot ON attendant_sessions(org_id, bot_id);
CREATE INDEX idx_attendant_sessions_status ON attendant_sessions(status);
CREATE INDEX idx_attendant_sessions_agent ON attendant_sessions(agent_id);
CREATE INDEX idx_attendant_sessions_queue ON attendant_sessions(queue_id);
CREATE INDEX idx_attendant_sessions_customer ON attendant_sessions(customer_id);
CREATE INDEX idx_attendant_sessions_created ON attendant_sessions(created_at DESC);
CREATE UNIQUE INDEX idx_attendant_sessions_number ON attendant_sessions(org_id, session_number);

CREATE INDEX idx_attendant_session_messages_session ON attendant_session_messages(session_id);
CREATE INDEX idx_attendant_session_messages_created ON attendant_session_messages(created_at);

CREATE INDEX idx_attendant_queue_agents_queue ON attendant_queue_agents(queue_id);
CREATE INDEX idx_attendant_queue_agents_agent ON attendant_queue_agents(agent_id);

CREATE INDEX idx_attendant_agent_status_org ON attendant_agent_status(org_id, bot_id);
CREATE INDEX idx_attendant_agent_status_status ON attendant_agent_status(status);

CREATE INDEX idx_attendant_transfers_session ON attendant_transfers(session_id);

CREATE INDEX idx_attendant_canned_org_bot ON attendant_canned_responses(org_id, bot_id);
CREATE INDEX idx_attendant_canned_shortcut ON attendant_canned_responses(shortcut);

CREATE INDEX idx_attendant_tags_org_bot ON attendant_tags(org_id, bot_id);
CREATE UNIQUE INDEX idx_attendant_tags_org_name ON attendant_tags(org_id, bot_id, name);

CREATE INDEX idx_attendant_wrap_up_codes_org_bot ON attendant_wrap_up_codes(org_id, bot_id);
CREATE UNIQUE INDEX idx_attendant_wrap_up_codes_org_code ON attendant_wrap_up_codes(org_id, bot_id, code);

CREATE INDEX idx_attendant_session_wrap_up_session ON attendant_session_wrap_up(session_id);
