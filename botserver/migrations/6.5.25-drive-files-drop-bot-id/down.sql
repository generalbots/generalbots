-- Rollback: Re-add bot_id column
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS bot_id UUID REFERENCES bots(id) ON DELETE CASCADE;
UPDATE drive_files SET bot_id = '00000000-0000-0000-0000-000000000000' WHERE bot_id IS NULL;
ALTER TABLE drive_files ALTER COLUMN bot_id SET NOT NULL;
CREATE INDEX IF NOT EXISTS idx_drive_files_bot ON drive_files(bot_id);
CREATE INDEX IF NOT EXISTS idx_drive_files_type ON drive_files(bot_id, file_type);
CREATE INDEX IF NOT EXISTS idx_drive_files_indexed ON drive_files(bot_id, indexed) WHERE NOT indexed;
CREATE INDEX IF NOT EXISTS idx_drive_files_fail ON drive_files(bot_id, fail_count) WHERE fail_count > 0;
