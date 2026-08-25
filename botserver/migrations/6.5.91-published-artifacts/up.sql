CREATE TABLE IF NOT EXISTS published_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID,
    owner_user_id UUID NOT NULL,
    kind VARCHAR(24) NOT NULL,
    slug VARCHAR(120) NOT NULL UNIQUE,
    title TEXT,
    object_key TEXT NOT NULL,
    visibility VARCHAR(16) NOT NULL DEFAULT 'public',
    password_hash VARCHAR(128),
    expires_at TIMESTAMPTZ,
    view_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_published_artifacts_owner ON published_artifacts(owner_user_id, created_at DESC);
