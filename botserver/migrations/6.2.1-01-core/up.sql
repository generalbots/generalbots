-- Add refresh_policy column to website_crawls table
-- This column stores the user-configured refresh interval (e.g., "1d", "1w", "1m", "1y")

ALTER TABLE website_crawls
ADD COLUMN IF NOT EXISTS refresh_policy VARCHAR(20);

-- Update existing records to have a default refresh policy (1 month)
UPDATE website_crawls
SET refresh_policy = '1m'
WHERE refresh_policy IS NULL;

-- Add comment for documentation
COMMENT ON COLUMN website_crawls.refresh_policy IS 'User-configured refresh interval (e.g., "1d", "1w", "1m", "1y") - shortest interval is used when duplicates exist';
