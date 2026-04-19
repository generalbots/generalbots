-- ============================================
-- KB Documents Fail State
-- Version: 6.3.0
-- ============================================
-- Add fail_count and last_failed_at to kb_documents
-- for intelligent backoff retry logic

ALTER TABLE kb_documents 
    ADD COLUMN IF NOT EXISTS fail_count INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_failed_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_kb_documents_fail 
    ON kb_documents(bot_id, collection_name, fail_count) 
    WHERE fail_count > 0;
