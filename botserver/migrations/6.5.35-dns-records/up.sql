CREATE TABLE IF NOT EXISTS dns_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hostname VARCHAR(255) NOT NULL,
    record_type VARCHAR(20) NOT NULL DEFAULT 'A',
    target VARCHAR(500) NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 300,
    priority INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_dns_records_hostname ON dns_records(hostname);
