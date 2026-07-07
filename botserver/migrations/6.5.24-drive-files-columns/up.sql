-- 9.24-drive-files-columns: Add missing columns (path, name, mime_type) to drive_files
-- These columns are expected by the Rust code in botdrive/src/drive_files.rs (raw SQL queries)
-- but were never added by earlier migrations. The consolidated (9.15.1) lacks them,
-- and 6.3.1-01-drive-files only creates file_path, file_type, etc.

ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS path TEXT;
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS name TEXT;
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS mime_type VARCHAR(50);

-- Backfill: set path = file_path for existing rows (they store the full MinIO key)
UPDATE drive_files SET path = file_path WHERE path IS NULL;
UPDATE drive_files SET name = split_part(file_path, '/', array_length(string_to_array(file_path, '/'), 1)) WHERE name IS NULL;
UPDATE drive_files SET mime_type = file_type WHERE mime_type IS NULL;

-- branch_id is already added by 9.23-branch-scope-cleanup; inferred here
ALTER TABLE drive_files ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE CASCADE;