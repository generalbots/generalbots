DROP INDEX IF EXISTS idx_bots_database_name;
ALTER TABLE bots DROP COLUMN IF EXISTS database_name;
