-- Drive public share links (enterprise): token-based, revocable, expiring.
-- The share_token is a random 128-bit UUID (unguessable) issued only for
-- files the owner explicitly shares. The public download endpoint resolves
-- the token, honours revoked/expiry flags, and streams the file with
-- attachment disposition — no listing, no enumeration.

CREATE TABLE IF NOT EXISTS drive_public_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    VARCHAR(255) NOT NULL,
    bucket      VARCHAR(255) NOT NULL,
    path        TEXT NOT NULL,
    key         TEXT NOT NULL,
    scope       VARCHAR(16) NOT NULL DEFAULT 'user',
    share_token VARCHAR(64) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    revoked     BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_drive_public_links_token
    ON drive_public_links (share_token);

CREATE INDEX IF NOT EXISTS idx_drive_public_links_owner
    ON drive_public_links (owner_id);

CREATE INDEX IF NOT EXISTS idx_drive_public_links_file
    ON drive_public_links (owner_id, bucket, path);
