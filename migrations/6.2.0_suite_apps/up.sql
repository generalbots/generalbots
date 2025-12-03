-- Suite Applications Migration
-- Adds tables for: Paper (Documents), Designer (Dialogs), and additional analytics support

-- Paper Documents table
CREATE TABLE IF NOT EXISTS paper_documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled Document',
    content TEXT NOT NULL DEFAULT '',
    owner_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_paper_documents_owner ON paper_documents(owner_id);
CREATE INDEX IF NOT EXISTS idx_paper_documents_updated ON paper_documents(updated_at DESC);

-- Designer Dialogs table
CREATE TABLE IF NOT EXISTS designer_dialogs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    bot_id TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_designer_dialogs_bot ON designer_dialogs(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_active ON designer_dialogs(is_active);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_updated ON designer_dialogs(updated_at DESC);

-- Sources Templates table (for template metadata caching)
CREATE TABLE IF NOT EXISTS source_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'General',
    preview_url TEXT,
    file_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_source_templates_category ON source_templates(category);

-- Analytics Events table (for additional event tracking)
CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    user_id UUID,
    session_id UUID,
    bot_id UUID,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_user ON analytics_events(user_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_session ON analytics_events(session_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at DESC);

-- Analytics Daily Aggregates (for faster dashboard queries)
CREATE TABLE IF NOT EXISTS analytics_daily_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    bot_id UUID,
    metric_name TEXT NOT NULL,
    metric_value BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(date, bot_id, metric_name)
);

CREATE INDEX IF NOT EXISTS idx_analytics_daily_date ON analytics_daily_aggregates(date DESC);
CREATE INDEX IF NOT EXISTS idx_analytics_daily_bot ON analytics_daily_aggregates(bot_id);

-- Research Search History (for recent searches feature)
CREATE TABLE IF NOT EXISTS research_search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    collection_id TEXT,
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_history_user ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_history_created ON research_search_history(created_at DESC);
