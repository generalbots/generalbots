-- Cross-app version history (Google Docs/M365-grade revision history).
--
-- Immutable snapshots keyed by a generic (resource_type, resource_id) address
-- so the same table serves docs, slides, sheets and any other collaboration
-- app. Snapshots are deduped by content hash (a save that changes nothing
-- produces no new row). Restore never mutates history: it inserts a NEW
-- version whose content equals the restored one, keeping the old rows intact.
--
--   resource_type: 'docs' | 'slides' | 'sheet' | ...
--   resource_id:   doc id / presentation id / sheet id
--   content:       serialized document content (JSON/HTML as the app emits)
--   content_hash:  SHA-256 hex of content, used for dedup + integrity display

CREATE TABLE IF NOT EXISTS collab_versions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    actor_id      VARCHAR(255) NOT NULL,
    actor_name    VARCHAR(255) NOT NULL DEFAULT '',
    content       TEXT NOT NULL,
    content_hash  CHAR(64) NOT NULL,
    name          VARCHAR(255) NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_collab_versions_resource
    ON collab_versions (resource_type, resource_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_collab_versions_hash
    ON collab_versions (resource_type, resource_id, content_hash);
