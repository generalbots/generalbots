-- 6.5.37-unified-search-fts: PostgreSQL full-text search support for the
-- universal search endpoint (/api/ui/search).
--
-- Each searchable entity table gets a GIN index over an expression that
-- concatenates its searchable columns, so tsquery lookups stay index-backed.
-- The unified search handler uses `websearch_to_tsquery` with ts_rank ordering.

CREATE INDEX IF NOT EXISTS idx_unified_search_people
    ON people USING GIN (to_tsvector('english',
        COALESCE(first_name::text, '') || ' ' ||
        COALESCE(last_name::text, '') || ' ' ||
        COALESCE(email::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_crm_contacts
    ON crm_contacts USING GIN (to_tsvector('english',
        COALESCE(first_name::text, '') || ' ' ||
        COALESCE(last_name::text, '') || ' ' ||
        COALESCE(email::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_products
    ON products USING GIN (to_tsvector('english',
        COALESCE(name::text, '') || ' ' ||
        COALESCE(sku::text, '') || ' ' ||
        COALESCE(description::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_support_tickets
    ON support_tickets USING GIN (to_tsvector('english',
        COALESCE(subject::text, '') || ' ' ||
        COALESCE(description::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_kb_documents
    ON kb_documents USING GIN (to_tsvector('english',
        COALESCE(file_path::text, '') || ' ' ||
        COALESCE(collection_name::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_drive_files
    ON drive_files USING GIN (to_tsvector('english',
        COALESCE(name::text, '') || ' ' ||
        COALESCE(file_path::text, '')));

CREATE INDEX IF NOT EXISTS idx_unified_search_bots
    ON bots USING GIN (to_tsvector('english',
        COALESCE(name::text, '') || ' ' ||
        COALESCE(description::text, '')));
