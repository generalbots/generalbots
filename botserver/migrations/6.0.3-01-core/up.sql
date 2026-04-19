-- Add manifest_json column to store the full task manifest for historical viewing
ALTER TABLE auto_tasks ADD COLUMN IF NOT EXISTS manifest_json JSONB;

-- Add an index for faster lookups when manifest exists
CREATE INDEX IF NOT EXISTS idx_auto_tasks_manifest_json ON auto_tasks USING gin (manifest_json) WHERE manifest_json IS NOT NULL;
