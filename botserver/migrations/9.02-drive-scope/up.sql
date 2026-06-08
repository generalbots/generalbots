CREATE TABLE drive_user_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    granted_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    CONSTRAINT uq_user_bucket_path UNIQUE (user_id, bucket, path)
);

CREATE INDEX idx_drive_user_permissions_user ON drive_user_permissions(user_id);

CREATE TABLE drive_starred (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_star UNIQUE (user_id, bucket, path)
);

CREATE INDEX idx_drive_starred_user ON drive_starred(user_id);

CREATE TABLE drive_share_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    token VARCHAR(64) NOT NULL UNIQUE,
    permission VARCHAR(20) NOT NULL DEFAULT 'read',
    expires_at TIMESTAMPTZ,
    max_downloads INTEGER,
    download_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_drive_share_links_token ON drive_share_links(token);
