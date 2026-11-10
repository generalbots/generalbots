-- Plan ownership and tenant scope (issue #2136).
--
-- Plans used to be keyed only by a client-supplied id with `owner_id` hardcoded
-- to `"anonymous"`, so any caller who learned an id could read and edit the
-- plan. Ownership now comes from the verified JWT subject and the tenant from
-- the verified `org_id`/`branch_id` claims.
--
-- The JSONB payload stays the source of truth (it is what `load_from_db`
-- reads); these columns project the same values so ownership and tenant scope
-- can be audited — and later filtered — with plain SQL. Both are nullable: rows
-- written before this migration have no owner and no tenant, and are only
-- reachable by a branchless caller that also matches the payload owner.

ALTER TABLE plan_snapshots ADD COLUMN IF NOT EXISTS owner_id TEXT;
ALTER TABLE plan_snapshots ADD COLUMN IF NOT EXISTS org_id UUID;
ALTER TABLE plan_snapshots ADD COLUMN IF NOT EXISTS branch_id UUID;

-- Backfill from the payload for rows persisted before the columns existed.
UPDATE plan_snapshots
SET owner_id = payload ->> 'owner_id',
    org_id = NULLIF(payload ->> 'org_id', '')::uuid,
    branch_id = NULLIF(payload ->> 'branch_id', '')::uuid
WHERE owner_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_plan_snapshots_owner ON plan_snapshots(owner_id);
CREATE INDEX IF NOT EXISTS idx_plan_snapshots_branch ON plan_snapshots(branch_id);
