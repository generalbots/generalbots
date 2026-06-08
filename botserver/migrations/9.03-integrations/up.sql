CREATE TABLE connectors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    connector_type VARCHAR(50) NOT NULL,
    description TEXT,
    auth_config JSONB NOT NULL DEFAULT '{}',
    endpoints JSONB NOT NULL DEFAULT '[]',
    schedule VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20),
    error_log TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_connectors_bot_id ON connectors(bot_id);
CREATE INDEX idx_connectors_type ON connectors(connector_type);
CREATE INDEX idx_connectors_active ON connectors(is_active);

CREATE TABLE etl_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    source_connector_id UUID REFERENCES connectors(id),
    destination_connector_id UUID REFERENCES connectors(id),
    transform_config JSONB NOT NULL DEFAULT '{}',
    schedule VARCHAR(50) NOT NULL DEFAULT 'manual',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    last_run_at TIMESTAMPTZ,
    last_run_status VARCHAR(20),
    run_count INTEGER NOT NULL DEFAULT 0,
    error_log TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_etl_jobs_bot_id ON etl_jobs(bot_id);
CREATE INDEX idx_etl_jobs_status ON etl_jobs(status);
