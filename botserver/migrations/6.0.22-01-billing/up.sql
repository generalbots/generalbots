-- Billing Usage Alerts table
CREATE TABLE IF NOT EXISTS billing_usage_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    current_usage BIGINT NOT NULL,
    usage_limit BIGINT NOT NULL,
    percentage DECIMAL(5,2) NOT NULL,
    threshold DECIMAL(5,2) NOT NULL,
    message TEXT NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    notification_sent BOOLEAN NOT NULL DEFAULT FALSE,
    notification_channels JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_usage_alerts_org_id ON billing_usage_alerts(org_id);
CREATE INDEX idx_billing_usage_alerts_bot_id ON billing_usage_alerts(bot_id);
CREATE INDEX idx_billing_usage_alerts_severity ON billing_usage_alerts(severity);
CREATE INDEX idx_billing_usage_alerts_created_at ON billing_usage_alerts(created_at);
CREATE INDEX idx_billing_usage_alerts_acknowledged ON billing_usage_alerts(acknowledged_at);

-- Billing Alert History table
CREATE TABLE IF NOT EXISTS billing_alert_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    alert_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    current_usage BIGINT NOT NULL,
    usage_limit BIGINT NOT NULL,
    percentage DECIMAL(5,2) NOT NULL,
    message TEXT NOT NULL,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    resolved_at TIMESTAMPTZ,
    resolution_type VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_alert_history_org_id ON billing_alert_history(org_id);
CREATE INDEX idx_billing_alert_history_created_at ON billing_alert_history(created_at);

-- Billing Notification Preferences table
CREATE TABLE IF NOT EXISTS billing_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL UNIQUE,
    bot_id UUID NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    channels JSONB NOT NULL DEFAULT '["email", "in_app"]'::jsonb,
    email_recipients JSONB NOT NULL DEFAULT '[]'::jsonb,
    webhook_url TEXT,
    webhook_secret TEXT,
    slack_webhook_url TEXT,
    teams_webhook_url TEXT,
    sms_numbers JSONB NOT NULL DEFAULT '[]'::jsonb,
    min_severity VARCHAR(20) NOT NULL DEFAULT 'warning',
    quiet_hours_start INTEGER,
    quiet_hours_end INTEGER,
    quiet_hours_timezone VARCHAR(50),
    quiet_hours_days JSONB DEFAULT '[]'::jsonb,
    metric_overrides JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_notification_preferences_org_id ON billing_notification_preferences(org_id);

-- Grace Period Status table
CREATE TABLE IF NOT EXISTS billing_grace_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    overage_at_start DECIMAL(10,2) NOT NULL,
    current_overage DECIMAL(10,2) NOT NULL,
    max_allowed_overage DECIMAL(10,2) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ended_at TIMESTAMPTZ,
    end_reason VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, metric, is_active)
);

CREATE INDEX idx_billing_grace_periods_org_id ON billing_grace_periods(org_id);
CREATE INDEX idx_billing_grace_periods_active ON billing_grace_periods(is_active);
CREATE INDEX idx_billing_grace_periods_expires ON billing_grace_periods(expires_at);
