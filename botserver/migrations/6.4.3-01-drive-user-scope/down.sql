DROP INDEX IF EXISTS idx_drive_files_user_id;
ALTER TABLE drive_files DROP COLUMN IF EXISTS scope;
ALTER TABLE drive_files DROP COLUMN IF EXISTS user_id;
