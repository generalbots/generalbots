DROP INDEX IF EXISTS idx_message_history_run;
ALTER TABLE message_history DROP COLUMN IF EXISTS thread_id;
ALTER TABLE message_history DROP COLUMN IF EXISTS run_id;

DROP INDEX IF EXISTS idx_agent_runs_session;
DROP INDEX IF EXISTS idx_agent_runs_thread;
ALTER TABLE agent_runs DROP COLUMN IF EXISTS parent_run_id;
ALTER TABLE agent_runs DROP COLUMN IF EXISTS user_id;
ALTER TABLE agent_runs DROP COLUMN IF EXISTS thread_id;
ALTER TABLE agent_runs DROP COLUMN IF EXISTS session_id;

DROP TABLE IF EXISTS run_events;
