DROP INDEX IF EXISTS idx_plan_snapshots_branch;
DROP INDEX IF EXISTS idx_plan_snapshots_owner;
ALTER TABLE plan_snapshots DROP COLUMN IF EXISTS branch_id;
ALTER TABLE plan_snapshots DROP COLUMN IF EXISTS org_id;
ALTER TABLE plan_snapshots DROP COLUMN IF EXISTS owner_id;
