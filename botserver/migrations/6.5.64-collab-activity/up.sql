-- Cross-app activity log / audit trail (enterprise compliance).
--
-- Every mutating collaboration event is recorded against a generic
-- (resource_type, resource_id) address so the same table serves drive files,
-- sheets, docs, slides, tasks and calendar events. Readers query a timeline
-- with pagination via GET /api/activity.
--
--   action: 'create' | 'edit' | 'comment' | 'delete' | 'resolve' | 'reopen'
--         | 'reaction' | 'share' | 'restore' | 'transfer'
--   payload: small JSON blob with event-specific detail (e.g. { "body_len": 42 }).

CREATE TABLE IF NOT EXISTS collab_activity (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    actor_id      VARCHAR(255) NOT NULL,
    actor_name    VARCHAR(255) NOT NULL DEFAULT '',
    action        VARCHAR(32)  NOT NULL,
    payload       TEXT NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_collab_activity_resource
    ON collab_activity (resource_type, resource_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_collab_activity_actor
    ON collab_activity (actor_id, created_at DESC);
