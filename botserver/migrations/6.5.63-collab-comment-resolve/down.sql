DROP TABLE IF EXISTS collab_comment_reads;
DROP INDEX IF EXISTS idx_collab_comments_resolved;

ALTER TABLE collab_comments
    DROP COLUMN IF EXISTS resolved,
    DROP COLUMN IF EXISTS resolved_by,
    DROP COLUMN IF EXISTS resolved_at;
