CREATE TABLE IF NOT EXISTS browser_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    org_id UUID,
    bot_id UUID,
    goal TEXT NOT NULL,
    domains JSONB NOT NULL DEFAULT '[]'::jsonb,
    budget_steps INTEGER NOT NULL DEFAULT 60,
    status VARCHAR(24) NOT NULL DEFAULT 'queued',
    plan JSONB,
    progress JSONB,
    result JSONB,
    citations JSONB,
    error TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_browser_tasks_user_created ON browser_tasks(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS page_facts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    facts JSONB NOT NULL DEFAULT '{}'::jsonb,
    visit_count INTEGER NOT NULL DEFAULT 1,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, url)
);

CREATE TABLE IF NOT EXISTS browse_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    task_id UUID,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_browse_sessions_user ON browse_sessions(user_id, started_at DESC);
