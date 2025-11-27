-- Create website_crawls table for tracking crawled websites
CREATE TABLE IF NOT EXISTS website_crawls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    url TEXT NOT NULL,
    last_crawled TIMESTAMPTZ,
    next_crawl TIMESTAMPTZ,
    expires_policy VARCHAR(20) NOT NULL DEFAULT '1d',
    max_depth INTEGER DEFAULT 3,
    max_pages INTEGER DEFAULT 100,
    crawl_status SMALLINT DEFAULT 0, -- 0=pending, 1=success, 2=processing, 3=error
    pages_crawled INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure unique URL per bot
    CONSTRAINT unique_bot_url UNIQUE (bot_id, url),

    -- Foreign key to bots table
    CONSTRAINT fk_website_crawls_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_website_crawls_bot_id ON website_crawls(bot_id);
CREATE INDEX IF NOT EXISTS idx_website_crawls_next_crawl ON website_crawls(next_crawl);
CREATE INDEX IF NOT EXISTS idx_website_crawls_url ON website_crawls(url);
CREATE INDEX IF NOT EXISTS idx_website_crawls_status ON website_crawls(crawl_status);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_website_crawls_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER website_crawls_updated_at_trigger
    BEFORE UPDATE ON website_crawls
    FOR EACH ROW
    EXECUTE FUNCTION update_website_crawls_updated_at();

-- Create session_website_associations table for tracking websites added to sessions
-- Similar to session_kb_associations but for websites
CREATE TABLE IF NOT EXISTS session_website_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    website_url TEXT NOT NULL,
    collection_name TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    added_by_tool VARCHAR(255),

    -- Ensure unique website per session
    CONSTRAINT unique_session_website UNIQUE (session_id, website_url),

    -- Foreign key to sessions table
    CONSTRAINT fk_session_website_session
        FOREIGN KEY (session_id)
        REFERENCES sessions(id)
        ON DELETE CASCADE,

    -- Foreign key to bots table
    CONSTRAINT fk_session_website_bot
        FOREIGN KEY (bot_id)
        REFERENCES bots(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_session_website_associations_session_id
    ON session_website_associations(session_id) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_session_website_associations_bot_id
    ON session_website_associations(bot_id);

CREATE INDEX IF NOT EXISTS idx_session_website_associations_url
    ON session_website_associations(website_url);

CREATE INDEX IF NOT EXISTS idx_session_website_associations_collection
    ON session_website_associations(collection_name);
