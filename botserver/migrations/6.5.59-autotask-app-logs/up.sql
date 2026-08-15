CREATE TABLE IF NOT EXISTS autotask_app_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    level VARCHAR(16) NOT NULL,
    source VARCHAR(16) NOT NULL,
    app_name VARCHAR(255) NOT NULL,
    bot_id UUID,
    user_id UUID,
    message TEXT NOT NULL,
    details TEXT,
    file_path TEXT,
    line_number INTEGER,
    stack_trace TEXT
);

CREATE INDEX IF NOT EXISTS idx_autotask_app_logs_app_time
    ON autotask_app_logs (app_name, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_autotask_app_logs_time
    ON autotask_app_logs (timestamp DESC);
