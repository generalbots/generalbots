-- 6.8.17-ops-analytics
-- #2116 — operational visibility inside the Analytics app.
--
-- `ops_request_metrics` is the per-endpoint request counter the server writes
-- from its HTTP middleware (`botserver/src/main_module/server/ops_metrics_layer.rs`).
-- The middleware keeps a process-local registry and folds it into this table
-- every 30 s: one row per (minute, endpoint class, status class). Latency is
-- stored as a fixed 8-bucket histogram so p95 stays queryable without keeping
-- one row per request.
--
-- `ops_deploy_markers` records the version/commit/build date of each boot that
-- changed the running build, so the traffic chart can overlay deploys.
--
-- Read by `botanalytics/src/handlers_ops*.rs`. Not declared as a Diesel
-- `table!` because the crate only uses `sql_query` (schema ownership stays in
-- `botschema` for tables that have typed models).

CREATE TABLE IF NOT EXISTS ops_request_metrics (
    bucket TIMESTAMPTZ NOT NULL,
    endpoint_class VARCHAR(48) NOT NULL,
    status_class VARCHAR(8) NOT NULL,
    request_count BIGINT NOT NULL DEFAULT 0,
    latency_sum_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    latency_max_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Fixed histogram, upper bounds: 50, 100, 250, 500, 1000, 2500, 5000 ms
    -- and an open last bucket (>= 5000 ms).
    b_le_50 BIGINT NOT NULL DEFAULT 0,
    b_le_100 BIGINT NOT NULL DEFAULT 0,
    b_le_250 BIGINT NOT NULL DEFAULT 0,
    b_le_500 BIGINT NOT NULL DEFAULT 0,
    b_le_1000 BIGINT NOT NULL DEFAULT 0,
    b_le_2500 BIGINT NOT NULL DEFAULT 0,
    b_le_5000 BIGINT NOT NULL DEFAULT 0,
    b_gt_5000 BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (bucket, endpoint_class, status_class)
);

CREATE INDEX IF NOT EXISTS idx_ops_request_metrics_bucket
    ON ops_request_metrics (bucket DESC);

CREATE TABLE IF NOT EXISTS ops_deploy_markers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version VARCHAR(64) NOT NULL,
    commit_hash VARCHAR(64) NOT NULL DEFAULT 'unknown',
    build_date VARCHAR(64) NOT NULL DEFAULT 'unknown',
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    note TEXT
);

CREATE INDEX IF NOT EXISTS idx_ops_deploy_markers_deployed_at
    ON ops_deploy_markers (deployed_at DESC);
