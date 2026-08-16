-- Server-authoritative operation log (op-log) with Lamport clocks and version
-- vectors — replaces last-write-wins for concurrent collaborative editing.
--
-- Every mutation to a shared resource is an append-only op. Concurrency is
-- detected per op by comparing its `base_version` (the version the editor
-- observed before editing) with the resource's current converged version. An
-- op whose base is behind the current version is a concurrent/conflicting op
-- that the user must resolve (accept-server keeps the newer state, accept-client
-- rebases the editor's change on top of the newer state).
--
-- `actor_type` tags each op as 'human' or 'llm' so a concurrent AI agent's
-- edits are ordered, filtered and visualized separately from a person's —
-- the exact requirement for LLM + human co-editing.

CREATE TABLE IF NOT EXISTS collab_ops (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_type VARCHAR(64)  NOT NULL,
    resource_id   VARCHAR(255) NOT NULL,
    actor_id      VARCHAR(255) NOT NULL,
    actor_name    VARCHAR(255) NOT NULL DEFAULT '',
    -- 'human' | 'llm'
    actor_type    VARCHAR(16)  NOT NULL DEFAULT 'human',
    -- insert | delete | retain | attribute | full-snapshot
    op_type       VARCHAR(32)  NOT NULL,
    -- version this op was based on; < current_version means concurrent
    base_version  BIGINT       NOT NULL,
    -- Lamport clock: max(seen base, current) + 1
    lamport_ts    BIGINT       NOT NULL,
    -- op payload, e.g. {"index":N,"text":"..."} or {"attribute":"bold","value":true}
    payload       TEXT         NOT NULL DEFAULT '{}',
    conflict      BOOLEAN      NOT NULL DEFAULT FALSE,
    resolved      BOOLEAN      NOT NULL DEFAULT FALSE,
    -- 'pending' | 'accepted' | 'rejected' | 'merged'
    resolution    VARCHAR(16)  NOT NULL DEFAULT 'pending',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_collab_ops_resource
    ON collab_ops (resource_type, resource_id, lamport_ts);

CREATE INDEX IF NOT EXISTS idx_collab_ops_conflicts
    ON collab_ops (resource_type, resource_id, conflict, resolved);

-- Converged state per resource: the current version (Lamport clock) and the
-- per-actor version vector that tracks the last op seen from each editor.
CREATE TABLE IF NOT EXISTS collab_resource_state (
    resource_type   VARCHAR(64)  NOT NULL,
    resource_id     VARCHAR(255) NOT NULL,
    current_version BIGINT       NOT NULL DEFAULT 0,
    -- JSON version vector: { "actor_id": lamport_ts, ... }
    version_vector  TEXT         NOT NULL DEFAULT '{}',
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (resource_type, resource_id)
);
