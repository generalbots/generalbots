-- The legacy schema (user_id/key/value/memory_type) was created by 6.0.0-01-core.
-- The new owner-scoped schema (owner_user_id/kind/content/...) is incompatible, so
-- drop the old table and recreate with the current shape.
DROP TABLE IF EXISTS user_memories CASCADE;

CREATE TABLE IF NOT EXISTS user_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID,
    branch_id UUID,
    owner_user_id UUID NOT NULL,
    scope VARCHAR(16) NOT NULL DEFAULT 'private',
    kind VARCHAR(32) NOT NULL DEFAULT 'fact',
    content TEXT NOT NULL,
    source VARCHAR(120) NOT NULL DEFAULT 'conversation',
    confidence REAL NOT NULL DEFAULT 0.8,
    pinned BOOLEAN NOT NULL DEFAULT FALSE,
    superseded_by UUID,
    embedding_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_memories_owner_branch ON user_memories(owner_user_id, branch_id);
CREATE INDEX IF NOT EXISTS idx_user_memories_kind ON user_memories(kind);
