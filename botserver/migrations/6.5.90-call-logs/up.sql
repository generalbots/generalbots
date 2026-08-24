CREATE TABLE IF NOT EXISTS call_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    direction VARCHAR(12) NOT NULL DEFAULT 'inbound',
    from_number VARCHAR(40),
    to_number VARCHAR(40),
    status VARCHAR(24) NOT NULL DEFAULT 'completed',
    duration_sec INTEGER,
    recording_ref TEXT,
    transcript TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_call_logs_bot_created ON call_logs(bot_id, created_at DESC);

CREATE TABLE IF NOT EXISTS channel_bindings (
    bot_id UUID PRIMARY KEY,
    phone_default VARCHAR(40),
    whatsapp_number VARCHAR(40),
    telegram_username VARCHAR(80),
    domains JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
