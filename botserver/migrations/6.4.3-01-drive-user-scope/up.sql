ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS user_id UUID;
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS scope VARCHAR(20) DEFAULT 'global';
CREATE INDEX IF NOT EXISTS idx_drive_files_user_id ON drive_files(user_id);
