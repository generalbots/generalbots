-- #2538 — durable state for the WebA site builder.
--
-- The builder used to keep apps, pages and published sites in a process-local
-- RwLock<HashMap<..>>, so everything vanished on restart. One JSONB row per
-- branch holds the whole snapshot: apps, draft pages, immutable publish
-- snapshots and captured form submissions. Reload happens in WebaState::new()
-- on boot; every mutation writes the row back.
--
-- A whole-snapshot row is intentionally chosen over one table per entity: the
-- builder is a single-tenant-per-branch tree that is always read and written as
-- a unit, and this keeps the persistence boundary small and atomic. The
-- branch_id primary key is the tenant boundary.

CREATE TABLE IF NOT EXISTS weba_snapshots (
    branch_id UUID PRIMARY KEY,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_weba_snapshots_updated
    ON weba_snapshots (updated_at DESC);
