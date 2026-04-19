-- Remove the refresh_policy column from website_crawls table
ALTER TABLE website_crawls
DROP COLUMN IF EXISTS refresh_policy;
