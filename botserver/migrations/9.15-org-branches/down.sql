-- Revert 9.15-org-branches

ALTER TABLE bots ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE bots ALTER COLUMN branch_id DROP NOT NULL;
ALTER TABLE bots DROP COLUMN IF EXISTS is_default_for_branch;
ALTER TABLE bots DROP COLUMN IF EXISTS branch_id;

DROP INDEX IF EXISTS idx_bots_branch;
DROP INDEX IF EXISTS idx_branches_slug;
DROP INDEX IF EXISTS idx_branches_tenant;
DROP INDEX IF EXISTS idx_branches_org;

DROP TABLE IF EXISTS branches;
