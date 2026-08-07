-- 6.5.47-campaign-worker
-- Durable multi-channel campaign sender (#731). Adds run-control state to
-- marketing_campaigns (pause/stop/resume, restart offset, timing) and a
-- campaign event log for the realtime monitor.

ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS run_offset INTEGER DEFAULT 0;
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS pause_requested BOOLEAN DEFAULT false;
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS stop_requested BOOLEAN DEFAULT false;
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS started_at timestamptz;
ALTER TABLE marketing_campaigns ADD COLUMN IF NOT EXISTS completed_at timestamptz;

CREATE TABLE IF NOT EXISTS marketing_campaign_events (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id       uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    campaign_id     uuid NOT NULL REFERENCES marketing_campaigns(id) ON DELETE CASCADE,
    channel         varchar(20),
    event_type      varchar(20) NOT NULL,   -- send, deliver, open, click, fail, pause, resume, stop
    recipient_email varchar(255),
    status          varchar(20),
    error_message   text,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_marketing_campaign_events_campaign ON marketing_campaign_events(campaign_id, created_at DESC);
