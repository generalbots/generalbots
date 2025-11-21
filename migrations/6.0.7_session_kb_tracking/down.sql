-- Migration 6.0.7: Session KB Tracking (ROLLBACK)
-- Drops session KB tracking table

DROP INDEX IF EXISTS idx_session_kb_active;
DROP INDEX IF EXISTS idx_session_kb_name;
DROP INDEX IF EXISTS idx_session_kb_bot_id;
DROP INDEX IF EXISTS idx_session_kb_session_id;

DROP TABLE IF EXISTS session_kb_associations;
