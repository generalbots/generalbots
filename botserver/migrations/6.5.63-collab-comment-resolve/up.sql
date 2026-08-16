-- Comment threads as resolvable tasks + per-user read tracking (issue #863).
-- A thread can be resolved/reopened (like Google Docs "resolve"), and each
-- user's last-read timestamp drives the unread badge on the comments button.

ALTER TABLE collab_comments
    ADD COLUMN IF NOT EXISTS resolved    BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS resolved_by VARCHAR(255),
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_collab_comments_resolved
    ON collab_comments (resolved);

-- Tracks when a user last read the comments of a resource. Unread count =
-- non-deleted comments (excluding the reader's own) created after this time.
CREATE TABLE IF NOT EXISTS collab_comment_reads (
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    user_id       VARCHAR(255) NOT NULL,
    last_read_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (resource_type, resource_id, user_id)
);
