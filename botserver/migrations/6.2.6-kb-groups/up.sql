-- ============================================
-- KB Groups 2.0 - Access Control by RBAC Group
-- Version: 6.2.6
-- ============================================
-- Associates kb_collections with rbac_groups so that
-- THINK KB only returns results from KBs accessible to
-- the caller's groups. KBs with no associations remain public.

CREATE TABLE IF NOT EXISTS kb_group_associations (
    id          uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    kb_id       uuid        NOT NULL REFERENCES kb_collections(id) ON DELETE CASCADE,
    group_id    uuid        NOT NULL REFERENCES rbac_groups(id)    ON DELETE CASCADE,
    granted_by  uuid        REFERENCES users(id) ON DELETE SET NULL,
    granted_at  timestamptz NOT NULL DEFAULT NOW(),
    UNIQUE (kb_id, group_id)
);

CREATE INDEX IF NOT EXISTS idx_kb_group_kb   ON kb_group_associations(kb_id);
CREATE INDEX IF NOT EXISTS idx_kb_group_grp  ON kb_group_associations(group_id);
