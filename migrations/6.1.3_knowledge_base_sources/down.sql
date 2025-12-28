-- Rollback Migration: Knowledge Base Sources

-- Drop triggers first
DROP TRIGGER IF EXISTS update_knowledge_sources_updated_at ON knowledge_sources;

-- Drop indexes
DROP INDEX IF EXISTS idx_knowledge_sources_bot_id;
DROP INDEX IF EXISTS idx_knowledge_sources_status;
DROP INDEX IF EXISTS idx_knowledge_sources_collection;
DROP INDEX IF EXISTS idx_knowledge_sources_content_hash;
DROP INDEX IF EXISTS idx_knowledge_sources_created_at;

DROP INDEX IF EXISTS idx_knowledge_chunks_source_id;
DROP INDEX IF EXISTS idx_knowledge_chunks_chunk_index;
DROP INDEX IF EXISTS idx_knowledge_chunks_content_fts;
DROP INDEX IF EXISTS idx_knowledge_chunks_embedding;

DROP INDEX IF EXISTS idx_research_search_history_bot_id;
DROP INDEX IF EXISTS idx_research_search_history_user_id;
DROP INDEX IF EXISTS idx_research_search_history_created_at;

-- Drop tables (order matters due to foreign key constraints)
DROP TABLE IF EXISTS research_search_history;
DROP TABLE IF EXISTS knowledge_chunks;
DROP TABLE IF EXISTS knowledge_sources;
