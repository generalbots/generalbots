-- Research / Knowledge Base / LLM Tables

-- Knowledge Base Documents
CREATE TABLE IF NOT EXISTS kb_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    collection_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL DEFAULT 0,
    file_hash TEXT NOT NULL,
    first_published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, collection_name, file_path)
);

CREATE INDEX IF NOT EXISTS idx_kb_documents_bot_id ON kb_documents(bot_id);
CREATE INDEX IF NOT EXISTS idx_kb_documents_collection ON kb_documents(collection_name);
CREATE INDEX IF NOT EXISTS idx_kb_documents_hash ON kb_documents(file_hash);
CREATE INDEX IF NOT EXISTS idx_kb_documents_indexed_at ON kb_documents(indexed_at);

-- Knowledge Base Collections
CREATE TABLE IF NOT EXISTS kb_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name TEXT NOT NULL,
    folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    document_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bot_id, name)
);

CREATE INDEX IF NOT EXISTS idx_kb_collections_bot_id ON kb_collections(bot_id);
CREATE INDEX IF NOT EXISTS idx_kb_collections_name ON kb_collections(name);

-- User KB Associations
CREATE TABLE IF NOT EXISTS user_kb_associations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    bot_id TEXT NOT NULL,
    kb_name TEXT NOT NULL,
    is_website INTEGER NOT NULL DEFAULT 0,
    website_url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(user_id, bot_id, kb_name)
);

CREATE INDEX IF NOT EXISTS idx_user_kb_user_id ON user_kb_associations(user_id);
CREATE INDEX IF NOT EXISTS idx_user_kb_bot_id ON user_kb_associations(bot_id);
CREATE INDEX IF NOT EXISTS idx_user_kb_name ON user_kb_associations(kb_name);
CREATE INDEX IF NOT EXISTS idx_user_kb_website ON user_kb_associations(is_website);

-- Session KB Associations
CREATE TABLE IF NOT EXISTS session_kb_associations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES user_sessions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    kb_name TEXT NOT NULL,
    kb_folder_path TEXT NOT NULL,
    qdrant_collection TEXT NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by_tool TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(session_id, kb_name)
);

CREATE INDEX IF NOT EXISTS idx_session_kb_session_id ON session_kb_associations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_bot_id ON session_kb_associations(bot_id);
CREATE INDEX IF NOT EXISTS idx_session_kb_name ON session_kb_associations(kb_name);
CREATE INDEX IF NOT EXISTS idx_session_kb_active ON session_kb_associations(is_active) WHERE is_active = true;

-- Research Search History
CREATE TABLE IF NOT EXISTS research_search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    collection_id TEXT,
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_history_user ON research_search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_research_history_created ON research_search_history(created_at DESC);

-- LLM Metrics & Budget (Moved from Core)
-- Note: Requires llm feature, but packed in research migration for simplicity or separated?
-- Plan says Research contains LLM tables.

-- Knowledge Graph Entities
CREATE TABLE IF NOT EXISTS kg_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name TEXT NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    description TEXT,
    properties JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(bot_id, name, entity_type)
);
CREATE INDEX IF NOT EXISTS idx_kg_entities_bot ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(bot_id, entity_type);

CREATE TABLE IF NOT EXISTS kg_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    source_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    relation_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, target_id, relation_type)
);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_bot ON kg_relationships(bot_id);

-- Triggers
DROP TRIGGER IF EXISTS update_kb_documents_updated_at ON kb_documents;
CREATE TRIGGER update_kb_documents_updated_at
    BEFORE UPDATE ON kb_documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_kb_collections_updated_at ON kb_collections;
CREATE TRIGGER update_kb_collections_updated_at
    BEFORE UPDATE ON kb_collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

