-- 6.5.49-fraud-branch-scope
-- The botfraud model is tenant-scoped by branch_id (issue #734). Its tables were
-- created in 6.5.05 with a NON-NULL bot_id column that the branch-scoped INSERTs
-- never populate, causing every create to fail with a NOT NULL violation.
-- bot_id is a legacy single-tenant column; the branch column is the tenant
-- boundary now. We keep bot_id for backwards compatibility but let it default to
-- the nil UUID so inserts succeed and existing queries remain valid.

ALTER TABLE fraud_rules     ALTER COLUMN bot_id SET DEFAULT '00000000-0000-0000-0000-000000000000'::uuid;
ALTER TABLE fraud_events    ALTER COLUMN bot_id SET DEFAULT '00000000-0000-0000-0000-000000000000'::uuid;
ALTER TABLE fraud_blocklist ALTER COLUMN bot_id SET DEFAULT '00000000-0000-0000-0000-000000000000'::uuid;
ALTER TABLE fraud_velocity  ALTER COLUMN bot_id SET DEFAULT '00000000-0000-0000-0000-000000000000'::uuid;