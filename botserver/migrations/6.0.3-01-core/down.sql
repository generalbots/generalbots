-- Remove manifest_json column from auto_tasks table
DROP INDEX IF EXISTS idx_auto_tasks_manifest_json;
ALTER TABLE auto_tasks DROP COLUMN IF EXISTS manifest_json;
