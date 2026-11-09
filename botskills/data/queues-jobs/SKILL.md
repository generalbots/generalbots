---
name: queues-jobs
description: |
  Run background jobs with retries, idempotency and dead letters. Use for async work.
version: 1.0.0
source: in-repo
---

# queues-jobs

## Workflow

1. Enqueue a job id, not the payload blob; the worker loads current state.
2. Idempotent handlers keyed by job id; safe to retry.
3. Exponential backoff with a max attempt count; move exhausted jobs to a dead-letter queue.
4. Visibility timeout longer than the slowest expected run; heartbeats for long jobs.
5. Observability: queue depth, age, failure rate, dead-letter count.
6. Cancellation is cooperative and recorded.

## Quality checks

- Killing the worker mid-job does not lose the job.
- A poisoned job stops retrying and lands in the dead-letter queue.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
