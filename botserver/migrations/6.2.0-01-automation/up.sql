-- Drop existing workflow_executions table if it exists (from older schema)
DROP TABLE IF EXISTS workflow_executions CASCADE;

-- Workflow state persistence (survives server restart)
CREATE TABLE workflow_executions (
  id UUID PRIMARY KEY,
  bot_id UUID NOT NULL REFERENCES bots(id),
  workflow_name TEXT NOT NULL,
  current_step INTEGER NOT NULL DEFAULT 1,
  state_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'running',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Event subscriptions persistence
CREATE TABLE workflow_events (
  id UUID PRIMARY KEY,
  workflow_id UUID REFERENCES workflow_executions(id),
  event_name TEXT NOT NULL,
  event_data_json TEXT,
  processed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cross-bot memory sharing
CREATE TABLE bot_shared_memory (
  id UUID PRIMARY KEY,
  source_bot_id UUID NOT NULL REFERENCES bots(id),
  target_bot_id UUID NOT NULL REFERENCES bots(id),
  memory_key TEXT NOT NULL,
  memory_value TEXT NOT NULL,
  shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(target_bot_id, memory_key)
);

-- Indexes for performance
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_bot_id ON workflow_executions(bot_id);
CREATE INDEX idx_workflow_events_processed ON workflow_events(processed);
CREATE INDEX idx_workflow_events_name ON workflow_events(event_name);
CREATE INDEX idx_bot_shared_memory_target ON bot_shared_memory(target_bot_id, memory_key);
CREATE INDEX idx_bot_shared_memory_source ON bot_shared_memory(source_bot_id);
