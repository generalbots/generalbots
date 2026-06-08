CREATE TABLE fraud_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    rule_type VARCHAR(50) NOT NULL,
    condition_json JSONB NOT NULL,
    action VARCHAR(20) NOT NULL DEFAULT 'flag',
    severity VARCHAR(20) NOT NULL DEFAULT 'medium',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fraud_rules_bot ON fraud_rules(bot_id);
CREATE INDEX idx_fraud_rules_type ON fraud_rules(rule_type);

CREATE TABLE fraud_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    risk_score INT NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    triggered_rules JSONB DEFAULT '[]',
    ml_score DECIMAL(5,4),
    action_taken VARCHAR(20) NOT NULL,
    details JSONB DEFAULT '{}',
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fraud_events_bot ON fraud_events(bot_id);
CREATE INDEX idx_fraud_events_type ON fraud_events(event_type);
CREATE INDEX idx_fraud_events_level ON fraud_events(risk_level);
CREATE INDEX idx_fraud_events_entity ON fraud_events(entity_type, entity_id);
CREATE INDEX idx_fraud_events_created ON fraud_events(created_at DESC);

CREATE TABLE fraud_blocklist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    block_type VARCHAR(50) NOT NULL,
    block_value VARCHAR(255) NOT NULL,
    reason TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blocklist_bot ON fraud_blocklist(bot_id);
CREATE INDEX idx_blocklist_type_value ON fraud_blocklist(block_type, block_value);

CREATE TABLE fraud_velocity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    identifier VARCHAR(255) NOT NULL,
    identifier_type VARCHAR(50) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    count INT NOT NULL DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_velocity_lookup ON fraud_velocity(bot_id, identifier_type, identifier, event_type);
