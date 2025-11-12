-- Drop triggers
DROP TRIGGER IF EXISTS update_basic_tools_updated_at ON basic_tools;
DROP TRIGGER IF EXISTS update_kb_collections_updated_at ON kb_collections;
DROP TRIGGER IF EXISTS update_kb_documents_updated_at ON kb_documents;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column;

-- Drop indexes
DROP INDEX IF EXISTS idx_basic_tools_active;
DROP INDEX IF EXISTS idx_basic_tools_name;
DROP INDEX IF EXISTS idx_basic_tools_bot_id;
DROP INDEX IF EXISTS idx_kb_collections_name;
DROP INDEX IF EXISTS idx_kb_collections_bot_id;
DROP INDEX IF EXISTS idx_kb_documents_indexed_at;
DROP INDEX IF EXISTS idx_kb_documents_hash;
DROP INDEX IF EXISTS idx_kb_documents_collection;
DROP INDEX IF EXISTS idx_kb_documents_bot_id;

-- Drop tables
DROP TABLE IF EXISTS basic_tools;
DROP TABLE IF EXISTS kb_collections;
DROP TABLE IF EXISTS kb_documents;
