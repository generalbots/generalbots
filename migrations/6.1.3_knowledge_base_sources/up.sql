-- Migration: Knowledge Base Sources
-- Description: Tables for document ingestion, chunking, and RAG support
-- Note: Vector embeddings are stored in Qdrant, not PostgreSQL

-- Drop existing tables for clean state
DROP TABLE IF EXISTS research_search_history CASCADE;
DROP TABLE IF EXISTS knowledge_chunks CASCADE;
DROP TABLE IF EXISTS knowledge_sources CASCADE;

-- Table for knowledge sources (uploaded documents)
CREATE TABLE IF NOT EXISTS knowledge_sources (
    id TEXT PRIMARY KEY,
    bot_id UUID,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'txt',
    file_path TEXT,
    url TEXT,
    content_hash TEXT NOT NULL,
    chunk_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    collection TEXT NOT NULL DEFAULT 'default',
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMPTZ
);

-- Indexes for knowledge_sources
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_bot_id ON knowledge_sources(bot_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_status ON knowledge_sources(status);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_collection ON knowledge_sources(collection);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_content_hash ON knowledge_sources(content_hash);
CREATE INDEX IF NOT EXISTS idx_knowledge_sources_created_at ON knowledge_sources(created_at);

-- Table for document chunks (text only - vectors stored in Qdrant)
CREATE TABLE IF NOT EXISTS knowledge_chunks (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES knowledge_sources(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for knowledge_chunks
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_source_id ON knowledge_chunks(source_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_chunk_index ON knowledge_chunks(chunk_index);

-- Full-text search index on content
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_content_fts
    ON knowledge_chunks USING gin(to_tsvector('english', content));

-- Table for search history
CREATE TABLE IF NOT EXISTS research_search_history (
    id TEXT PRIMARY KEY,
    bot_id UUID,
    user_id UUID,
    query TEXT NOT NULL,
    search_type TEXT NOT NULL DEFAULT 'web',
    results_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for search history
CREATE INDEX IF NOT EXISTS idx_research_search_history_bot_id ON research_search_history(bot_id);
CREATE INDEX IF NOT EXISTS idx_research_search_history_user_id ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_search_history_created_at ON research_search_history(created_at);

-- Trigger for updated_at on knowledge_sources
DROP TRIGGER IF EXISTS update_knowledge_sources_updated_at ON knowledge_sources;
CREATE TRIGGER update_knowledge_sources_updated_at
    BEFORE UPDATE ON knowledge_sources
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE knowledge_sources IS 'Uploaded documents for knowledge base ingestion';
COMMENT ON TABLE knowledge_chunks IS 'Text chunks extracted from knowledge sources - vectors stored in Qdrant';
COMMENT ON TABLE research_search_history IS 'History of web and knowledge base searches';

COMMENT ON COLUMN knowledge_sources.source_type IS 'Document type: pdf, docx, txt, markdown, html, csv, xlsx, url';
COMMENT ON COLUMN knowledge_sources.status IS 'Processing status: pending, processing, indexed, failed, reindexing';
COMMENT ON COLUMN knowledge_sources.collection IS 'Collection/namespace for organizing sources';
COMMENT ON COLUMN knowledge_chunks.token_count IS 'Estimated token count for the chunk';
