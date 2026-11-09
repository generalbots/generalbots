-- #2056 — posts published by BASIC `POST TO` to an external channel.
--
-- The keyword used to insert into `social_posts`, which is the internal social
-- network (org/bot/author/community) and has no platform, media_url or status
-- columns at all, so the insert failed. This table models exactly what an
-- external publish needs: the target platform, the payload, the provider
-- result and the scheduling fields.
CREATE TABLE IF NOT EXISTS social_external_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    user_id UUID,
    platform VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    media_url TEXT NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    remote_post_id VARCHAR(255),
    permalink TEXT,
    error TEXT,
    scheduled_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_social_external_posts_status
    ON social_external_posts(status, scheduled_at);
CREATE INDEX IF NOT EXISTS idx_social_external_posts_bot
    ON social_external_posts(bot_id, created_at DESC);
