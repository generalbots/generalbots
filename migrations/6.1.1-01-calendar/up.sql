-- Legacy Calendar Tables extracted from consolidated

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

-- Calendar sharing (skip - already exists from 6.0.13-01-calendar)
-- CREATE TABLE IF NOT EXISTS calendar_shares (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
--     shared_with_user UUID REFERENCES users(id) ON DELETE CASCADE,
--     shared_with_email VARCHAR(255),
--     permission_level VARCHAR(20) DEFAULT 'view',
--     created_at TIMESTAMPTZ DEFAULT NOW(),
--     CONSTRAINT check_cal_permission CHECK (permission_level IN ('free_busy', 'view', 'edit', 'admin'))
-- );

-- CREATE INDEX IF NOT EXISTS idx_calendar_shares_owner ON calendar_shares(owner_id);
-- CREATE INDEX IF NOT EXISTS idx_calendar_shares_shared ON calendar_shares(shared_with_user);
