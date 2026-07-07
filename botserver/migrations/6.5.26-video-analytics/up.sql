CREATE TABLE IF NOT EXISTS video_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    export_id UUID,
    views BIGINT NOT NULL DEFAULT 0,
    unique_viewers BIGINT NOT NULL DEFAULT 0,
    total_watch_time_ms BIGINT NOT NULL DEFAULT 0,
    avg_watch_percent REAL NOT NULL DEFAULT 0,
    completions BIGINT NOT NULL DEFAULT 0,
    shares BIGINT NOT NULL DEFAULT 0,
    likes BIGINT NOT NULL DEFAULT 0,
    engagement_score REAL NOT NULL DEFAULT 0,
    viewer_retention_json JSONB,
    geography_json JSONB,
    device_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_analytics_project ON video_analytics (project_id);
