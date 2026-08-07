CREATE TABLE IF NOT EXISTS login_sessions (
    token TEXT PRIMARY KEY,
    user_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_login_sessions_created ON login_sessions(created_at);
