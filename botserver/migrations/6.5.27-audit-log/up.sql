CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(100) NOT NULL,
    actor_type VARCHAR(50) NOT NULL,
    actor_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    outcome_success BOOLEAN NOT NULL DEFAULT TRUE,
    details TEXT,
    session_id UUID,
    bot_id UUID NOT NULL,
    task_id UUID,
    step_id UUID,
    risk_level VARCHAR(20) NOT NULL DEFAULT 'info'
);

CREATE INDEX IF NOT EXISTS idx_audit_log_bot ON audit_log (bot_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_event ON audit_log (event_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log (timestamp);
