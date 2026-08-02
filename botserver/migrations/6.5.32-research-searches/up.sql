CREATE TABLE IF NOT EXISTS research_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    query TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_searches_created_at ON research_searches(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_research_searches_user ON research_searches(user_id);
