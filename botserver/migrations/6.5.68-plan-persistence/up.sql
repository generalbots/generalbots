-- Plan persistence (issue #875).
--
-- The botplan service keeps its in-memory store for the live UI, and this
-- table makes plans durable across restarts. Each plan (with its tasks) is
-- serialized to a single JSONB document keyed by plan_id; upsert is
-- last-write-wins per plan.

CREATE TABLE IF NOT EXISTS plan_snapshots (
    plan_id    TEXT PRIMARY KEY,
    payload    JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
