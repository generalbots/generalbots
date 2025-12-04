-- Down migration: Remove email tracking table and related objects

-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_update_sent_email_tracking_updated_at ON sent_email_tracking;

-- Drop function
DROP FUNCTION IF EXISTS update_sent_email_tracking_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_sent_email_tracking_tracking_id;
DROP INDEX IF EXISTS idx_sent_email_tracking_bot_id;
DROP INDEX IF EXISTS idx_sent_email_tracking_account_id;
DROP INDEX IF EXISTS idx_sent_email_tracking_to_email;
DROP INDEX IF EXISTS idx_sent_email_tracking_sent_at;
DROP INDEX IF EXISTS idx_sent_email_tracking_is_read;
DROP INDEX IF EXISTS idx_sent_email_tracking_read_status;

-- Drop table
DROP TABLE IF EXISTS sent_email_tracking;
