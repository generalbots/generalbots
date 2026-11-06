# Goal Runs

Long-horizon objective tracking executes as **goal runs**: a bounded background
runner advances an objective's key result one checkpoint per interval until the
target is reached or the checkpoint budget is spent.

## Checkpoints

- Each run stores `interval_seconds`, `step_value`, `checkpoints`,
  `last_value` and `target_value`. Runs created for a key result stop when the
  target is reached.
- When an `instruction` is supplied and an LLM is configured, the model chooses
  each checkpoint's increment instead of the fixed `step_value`
  (`core/bot/runs/goal_agent.rs`). The instruction is persisted on the run row
  (migration `6.8.10-goal-run-instruction`) so a resumed run keeps its agent
  steps.
- The runner is bounded by `MAX_CHECKPOINTS`, so no background task outlives
  its budget.

## Pause and resume

`POST /api/goals/runs/:id/pause` flips the run to `paused`; the runner exits at
its next checkpoint because `advance` refuses a non-running run. The checkpoint
count is frozen while paused.

`POST /api/goals/runs/:id/resume` flips a paused run back to `running`, resets
`next_checkpoint_at`, and restarts the runner with the same instruction and the
remaining checkpoint budget. The count continues from the paused value.

## Surfaces

- **Goals app → Runs** lists every run for the period's objectives with status,
  progress, checkpoints, interval and the last checkpoint value, and offers
  Pause / Resume / Stop actions. The list polls every 10 s while visible.
- **API** (`/api/goals/runs`) remains the source of truth used by agents and by
  the Goals app.

## Verification

The 2-hour wall test (60 s interval, 120 checkpoints) completes with progress
100% and status `completed`. Pause/resume is verified at runtime: while paused
the checkpoint count is unchanged across interval periods; after resume it
continues from the same value.
