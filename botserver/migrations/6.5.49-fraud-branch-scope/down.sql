-- 6.5.49-fraud-branch-scope (rollback)
ALTER TABLE fraud_rules     ALTER COLUMN bot_id DROP DEFAULT;
ALTER TABLE fraud_events    ALTER COLUMN bot_id DROP DEFAULT;
ALTER TABLE fraud_blocklist ALTER COLUMN bot_id DROP DEFAULT;
ALTER TABLE fraud_velocity  ALTER COLUMN bot_id DROP DEFAULT;