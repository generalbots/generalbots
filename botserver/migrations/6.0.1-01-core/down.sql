DROP INDEX IF EXISTS idx_bots_tenant_id;
DROP INDEX IF EXISTS idx_bots_database_name;

ALTER TABLE bots DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE bots DROP COLUMN IF EXISTS database_name;
