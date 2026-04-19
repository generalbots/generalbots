CREATE TABLE calendars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    color VARCHAR(20) DEFAULT '#3b82f6',
    timezone VARCHAR(100) DEFAULT 'UTC',
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    is_shared BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE calendar_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    calendar_id UUID NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    location VARCHAR(500),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    all_day BOOLEAN NOT NULL DEFAULT FALSE,
    recurrence_rule TEXT,
    recurrence_id UUID REFERENCES calendar_events(id) ON DELETE SET NULL,
    color VARCHAR(20),
    status VARCHAR(50) NOT NULL DEFAULT 'confirmed',
    visibility VARCHAR(50) NOT NULL DEFAULT 'default',
    busy_status VARCHAR(50) NOT NULL DEFAULT 'busy',
    reminders JSONB NOT NULL DEFAULT '[]',
    attendees JSONB NOT NULL DEFAULT '[]',
    conference_data JSONB,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE calendar_event_attendees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'needs-action',
    role VARCHAR(50) NOT NULL DEFAULT 'req-participant',
    rsvp_time TIMESTAMPTZ,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE calendar_event_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    reminder_type VARCHAR(50) NOT NULL DEFAULT 'notification',
    minutes_before INTEGER NOT NULL,
    is_sent BOOLEAN NOT NULL DEFAULT FALSE,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE calendar_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    calendar_id UUID NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    shared_with_user_id UUID,
    shared_with_email VARCHAR(255),
    permission VARCHAR(50) NOT NULL DEFAULT 'read',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(calendar_id, shared_with_user_id),
    UNIQUE(calendar_id, shared_with_email)
);

CREATE INDEX idx_calendars_org_bot ON calendars(org_id, bot_id);
CREATE INDEX idx_calendars_owner ON calendars(owner_id);
CREATE INDEX idx_calendars_primary ON calendars(owner_id, is_primary) WHERE is_primary = TRUE;

CREATE INDEX idx_calendar_events_org_bot ON calendar_events(org_id, bot_id);
CREATE INDEX idx_calendar_events_calendar ON calendar_events(calendar_id);
CREATE INDEX idx_calendar_events_owner ON calendar_events(owner_id);
CREATE INDEX idx_calendar_events_time_range ON calendar_events(start_time, end_time);
CREATE INDEX idx_calendar_events_status ON calendar_events(status);
CREATE INDEX idx_calendar_events_recurrence ON calendar_events(recurrence_id) WHERE recurrence_id IS NOT NULL;

CREATE INDEX idx_calendar_event_attendees_event ON calendar_event_attendees(event_id);
CREATE INDEX idx_calendar_event_attendees_email ON calendar_event_attendees(email);

CREATE INDEX idx_calendar_event_reminders_event ON calendar_event_reminders(event_id);
CREATE INDEX idx_calendar_event_reminders_pending ON calendar_event_reminders(is_sent, minutes_before) WHERE is_sent = FALSE;

CREATE INDEX idx_calendar_shares_calendar ON calendar_shares(calendar_id);
CREATE INDEX idx_calendar_shares_user ON calendar_shares(shared_with_user_id) WHERE shared_with_user_id IS NOT NULL;
CREATE INDEX idx_calendar_shares_email ON calendar_shares(shared_with_email) WHERE shared_with_email IS NOT NULL;
