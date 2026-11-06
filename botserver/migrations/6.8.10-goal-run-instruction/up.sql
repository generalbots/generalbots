-- #1706 — pause/resume must keep the agent instruction across restarts.
ALTER TABLE goal_runs ADD COLUMN IF NOT EXISTS instruction TEXT;
