DROP TABLE IF EXISTS marketing_campaign_events;
ALTER TABLE marketing_campaigns DROP COLUMN IF EXISTS run_offset;
ALTER TABLE marketing_campaigns DROP COLUMN IF EXISTS pause_requested;
ALTER TABLE marketing_campaigns DROP COLUMN IF EXISTS stop_requested;
ALTER TABLE marketing_campaigns DROP COLUMN IF EXISTS started_at;
ALTER TABLE marketing_campaigns DROP COLUMN IF EXISTS completed_at;
