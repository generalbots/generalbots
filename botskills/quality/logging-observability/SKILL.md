---
name: logging-observability
description: |
  Structured logs, traces and metrics that answer what happened. Use when adding service behaviour.
version: 1.0.0
source: in-repo
---

# logging-observability

## Workflow

1. Structured key-value logs with request/job ids; no string concatenation.
2. Log at the boundary: input accepted, external call, outcome, latency.
3. Never log secrets, tokens or full PII.
4. Trace external calls and background jobs with the same correlation id.
5. Metrics: counters for outcomes, histograms for latency, gauges for queue depth.
6. Alerts on symptoms users feel, not on every error line.

## Quality checks

- A failed request can be reconstructed from logs alone.
- No secret or credential appears in any log line.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
