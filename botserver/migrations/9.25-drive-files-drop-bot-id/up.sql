-- 9.25-drive-files-drop-bot-id: Remove bot_id from drive_files (scope is by branch)
-- The table was migrated to branch_id scope in 9.23 but bot_id column remained with NOT NULL.

ALTER TABLE drive_files DROP CONSTRAINT IF EXISTS drive_files_bot_id_fkey;
ALTER TABLE drive_files DROP CONSTRAINT IF EXISTS drive_files_bot_id_file_path_key;
DROP INDEX IF EXISTS idx_drive_files_bot;
DROP INDEX IF EXISTS idx_drive_files_type;
DROP INDEX IF EXISTS idx_drive_files_indexed;
DROP INDEX IF EXISTS idx_drive_files_fail;
ALTER TABLE drive_files DROP COLUMN IF EXISTS bot_id;
