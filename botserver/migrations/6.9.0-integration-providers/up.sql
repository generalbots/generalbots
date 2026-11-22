-- #2346 — operator-added integration providers. Seeded providers stay in Rust;
-- rows here are additions/edits made through the provider editor.
CREATE TABLE IF NOT EXISTS integration_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    module TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'planned',
    source TEXT NOT NULL DEFAULT 'operator',
    actions JSONB NOT NULL DEFAULT '[]'::jsonb,
    setup_steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    credentials_url TEXT NOT NULL DEFAULT '',
    official_docs TEXT NOT NULL DEFAULT '',
    guidance TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
