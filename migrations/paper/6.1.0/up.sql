CREATE TABLE IF NOT EXISTS paper_documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT 'Untitled Document',
    content TEXT NOT NULL DEFAULT '',
    owner_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_paper_documents_owner ON paper_documents(owner_id);
CREATE INDEX IF NOT EXISTS idx_paper_documents_updated ON paper_documents(updated_at DESC);
