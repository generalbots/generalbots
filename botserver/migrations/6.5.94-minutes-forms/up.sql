CREATE TABLE IF NOT EXISTS minutes_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID,
    title TEXT NOT NULL,
    owner TEXT,
    due TIMESTAMPTZ,
    priority VARCHAR(20) NOT NULL DEFAULT 'medium',
    notes TEXT NOT NULL DEFAULT '',
    status VARCHAR(30) NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

CREATE TABLE IF NOT EXISTS minutes_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL,
    signer TEXT NOT NULL,
    signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

CREATE TABLE IF NOT EXISTS minutes_attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    attendee TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'confirmed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);
