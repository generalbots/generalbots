-- Cross-app collaboration layer (M365/Google-grade expectations):
-- threaded comments with @-mentions and emoji reactions, plus live
-- presence (who is viewing/typing) on any resource.
--
-- Resource addressing is generic (resource_type + resource_id) so every
-- collaboration app (drive files, sheets, docs, tasks, calendar events)
-- can attach comments and presence without schema changes:
--   resource_type: 'drive:file' | 'sheet' | 'doc' | 'task' | 'calendar' ...
--   resource_id:   file path / sheet id / doc id / task id / event id

CREATE TABLE IF NOT EXISTS collab_comments (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    author_id     VARCHAR(255) NOT NULL,
    author_name   VARCHAR(255) NOT NULL DEFAULT '',
    parent_id     UUID,
    body          TEXT NOT NULL,
    -- JSON array of @mention tokens (e.g. ["maria@example.com"]).
    mentions      TEXT NOT NULL DEFAULT '[]',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted       BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_collab_comments_resource
    ON collab_comments (resource_type, resource_id, created_at);

CREATE INDEX IF NOT EXISTS idx_collab_comments_parent
    ON collab_comments (parent_id);

CREATE TABLE IF NOT EXISTS collab_comment_reactions (
    comment_id UUID NOT NULL REFERENCES collab_comments(id) ON DELETE CASCADE,
    user_id    VARCHAR(255) NOT NULL,
    emoji      VARCHAR(32)  NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (comment_id, user_id, emoji)
);

-- Presence heartbeat. Rows older than the active window (60s) are treated
-- as offline by readers; heartbeats refresh last_seen.
CREATE TABLE IF NOT EXISTS collab_presence (
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    user_id       VARCHAR(255) NOT NULL,
    user_name     VARCHAR(255) NOT NULL DEFAULT '',
    last_seen     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    typing        BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (resource_type, resource_id, user_id)
);
