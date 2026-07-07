CREATE TABLE IF NOT EXISTS drive_trash (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    original_path VARCHAR(1024) NOT NULL,
    is_dir BOOLEAN NOT NULL DEFAULT false,
    size BIGINT NOT NULL DEFAULT 0,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days')
);

CREATE INDEX idx_drive_trash_user_id ON drive_trash(user_id);
CREATE INDEX idx_drive_trash_expires_at ON drive_trash(expires_at);

CREATE TABLE IF NOT EXISTS drive_stars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, bucket, path)
);

CREATE INDEX idx_drive_stars_user_id ON drive_stars(user_id);

CREATE TABLE IF NOT EXISTS drive_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    bucket VARCHAR(255) NOT NULL,
    path VARCHAR(1024) NOT NULL,
    permissions VARCHAR(50) NOT NULL DEFAULT 'read',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(owner_id, recipient_id, bucket, path)
);

CREATE INDEX idx_drive_shares_recipient_id ON drive_shares(recipient_id);
CREATE INDEX idx_drive_shares_owner_id ON drive_shares(owner_id);
