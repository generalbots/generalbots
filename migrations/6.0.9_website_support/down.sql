-- Drop session_website_associations table and related indexes
DROP TABLE IF EXISTS session_website_associations;

-- Drop website_crawls table and related objects
DROP TRIGGER IF EXISTS website_crawls_updated_at_trigger ON website_crawls;
DROP FUNCTION IF EXISTS update_website_crawls_updated_at();
DROP TABLE IF EXISTS website_crawls;
