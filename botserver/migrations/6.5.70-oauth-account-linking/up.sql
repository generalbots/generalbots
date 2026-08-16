-- OAuth account linking (enterprise-grade, #899): stores which external
-- identity (Google/Microsoft/GitHub) is linked to each local user. Only
-- non-sensitive profile metadata is kept here; access/refresh tokens are
-- stored in Vault (`secret/gbo/oauth/{user_id}/{provider}`), never in the DB.

CREATE TABLE IF NOT EXISTS oauth_account_links (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    provider        VARCHAR(32) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    email           VARCHAR(255),
    display_name    VARCHAR(255),
    avatar_url      TEXT,
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, provider),
    UNIQUE (provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_account_links_user
    ON oauth_account_links (user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_account_links_provider
    ON oauth_account_links (provider, provider_user_id);
