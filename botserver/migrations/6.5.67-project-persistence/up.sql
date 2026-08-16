-- Project persistence (issue #871).
--
-- The botproject service keeps its in-memory store for the live UI, and this
-- table makes that state durable across restarts. Each organization's full
-- project set (projects + tasks + resources + assignments) is serialized to a
-- single JSONB document keyed by org_id, so data is tenant-scoped and loaded
-- back on startup. Upsert is last-write-wins per org.

CREATE TABLE IF NOT EXISTS project_snapshots (
    org_id     UUID PRIMARY KEY,
    payload    JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
