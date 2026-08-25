CREATE TABLE IF NOT EXISTS user_workspace_tabs (
    user_id UUID PRIMARY KEY,
    tabs JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
