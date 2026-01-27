-- Drop workflow orchestration tables
DROP INDEX IF EXISTS idx_bot_shared_memory_source;
DROP INDEX IF EXISTS idx_bot_shared_memory_target;
DROP INDEX IF EXISTS idx_workflow_events_name;
DROP INDEX IF EXISTS idx_workflow_events_processed;
DROP INDEX IF EXISTS idx_workflow_executions_bot_id;
DROP INDEX IF EXISTS idx_workflow_executions_status;

DROP TABLE IF EXISTS bot_shared_memory;
DROP TABLE IF EXISTS workflow_events;
DROP TABLE IF EXISTS workflow_executions;
