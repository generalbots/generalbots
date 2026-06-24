-- ============================================
-- Drive Files State Table
-- Version: 6.3.1
-- ============================================
-- Unifies file state tracking from JSON file to database
-- Used by DriveMonitor to track file changes across all file types

CREATE TABLE IF NOT EXISTS drive_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_type VARCHAR(20) NOT NULL, -- 'gbdialog', 'gbot', 'gbkb', etc.
    etag TEXT,
    last_modified TIMESTAMPTZ,
    file_size BIGINT,
    indexed BOOLEAN NOT NULL DEFAULT FALSE,
    fail_count INT NOT NULL DEFAULT 0,
    last_failed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, file_path)
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_drive_files_bot ON drive_files(bot_id);
CREATE INDEX IF NOT EXISTS idx_drive_files_type ON drive_files(bot_id, file_type);
CREATE INDEX IF NOT EXISTS idx_drive_files_indexed ON drive_files(bot_id, indexed) WHERE NOT indexed;
CREATE INDEX IF NOT EXISTS idx_drive_files_fail ON drive_files(bot_id, fail_count) WHERE fail_count > 0;

-- Migrate existing kb_documents state to drive_files
-- This preserves fail_count and last_failed_at from kb_documents
INSERT INTO drive_files (
    bot_id,
    file_path,
    file_type,
    etag,
    last_modified,
    indexed,
    fail_count,
    last_failed_at
)
SELECT 
    bot_id,
    file_path,
    'gbkb' as file_type,
    file_hash as etag,
    last_modified_at as last_modified,
    indexed_at IS NOT NULL as indexed,
    fail_count,
    last_failed_at
FROM kb_documents
ON CONFLICT (bot_id, file_path) DO UPDATE SET
    etag = EXCLUDED.etag,
    last_modified = EXCLUDED.last_modified,
    indexed = EXCLUDED.indexed,
    fail_count = EXCLUDED.fail_count,
    last_failed_at = EXCLUDED.last_failed_at;
