-- Email Read Tracking Table
-- Stores sent email tracking data for read receipt functionality
-- Enabled via config.csv: email-read-pixel,true

CREATE TABLE IF NOT EXISTS sent_email_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tracking_id UUID NOT NULL UNIQUE,
    bot_id UUID NOT NULL,
    account_id UUID NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    cc TEXT,
    bcc TEXT,
    subject TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    read_count INTEGER NOT NULL DEFAULT 0,
    first_read_ip VARCHAR(45),
    last_read_ip VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_tracking_id ON sent_email_tracking(tracking_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_bot_id ON sent_email_tracking(bot_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_account_id ON sent_email_tracking(account_id);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_to_email ON sent_email_tracking(to_email);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_sent_at ON sent_email_tracking(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_is_read ON sent_email_tracking(is_read);
CREATE INDEX IF NOT EXISTS idx_sent_email_tracking_read_status ON sent_email_tracking(bot_id, is_read, sent_at DESC);

-- Trigger to auto-update updated_at
CREATE OR REPLACE FUNCTION update_sent_email_tracking_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_sent_email_tracking_updated_at ON sent_email_tracking;
CREATE TRIGGER trigger_update_sent_email_tracking_updated_at
    BEFORE UPDATE ON sent_email_tracking
    FOR EACH ROW
    EXECUTE FUNCTION update_sent_email_tracking_updated_at();

-- Add comment for documentation
COMMENT ON TABLE sent_email_tracking IS 'Tracks sent emails for read receipt functionality via tracking pixel';
COMMENT ON COLUMN sent_email_tracking.tracking_id IS 'Unique ID embedded in tracking pixel URL';
COMMENT ON COLUMN sent_email_tracking.is_read IS 'Whether the email has been opened (pixel loaded)';
COMMENT ON COLUMN sent_email_tracking.read_count IS 'Number of times the email was opened';
COMMENT ON COLUMN sent_email_tracking.first_read_ip IS 'IP address of first email open';
COMMENT ON COLUMN sent_email_tracking.last_read_ip IS 'IP address of most recent email open';
