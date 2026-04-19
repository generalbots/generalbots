ALTER TABLE bots ADD COLUMN IF NOT EXISTS database_name VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_bots_database_name ON bots(database_name);
