-- ============================================
-- Rollback KB Fail State
-- ============================================

DROP INDEX IF EXISTS idx_kb_documents_fail;
ALTER TABLE kb_documents 
    DROP COLUMN IF EXISTS fail_count,
    DROP COLUMN IF EXISTS last_failed_at;
