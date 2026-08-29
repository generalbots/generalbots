ALTER TABLE tasks ADD COLUMN IF NOT EXISTS stage TEXT NOT NULL DEFAULT 'plan';
ALTER TABLE tasks DROP CONSTRAINT IF EXISTS check_task_stage;
ALTER TABLE tasks ADD CONSTRAINT check_task_stage CHECK (stage IN ('plan','build','review','deploy','monitor'));
CREATE INDEX IF NOT EXISTS idx_tasks_stage ON tasks(stage);
