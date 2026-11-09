---
name: performance-profiling
description: |
  Measure, locate the bottleneck, fix, and re-measure. Use when something is slow or memory-heavy.
version: 1.0.0
source: in-repo
---

# performance-profiling

## Workflow

1. Define the metric and the target (p95 latency, memory peak, throughput).
2. Capture a baseline under a realistic load.
3. Profile (sampling profiler, EXPLAIN, allocations) and find the top cost.
4. Fix the single largest cost; avoid speculative micro-optimisation.
5. Re-measure with the same harness and record the delta.
6. Guard the result with a benchmark or budget test.

## Quality checks

- Before/after numbers use the same conditions.
- The fix targets a measured hotspot, not a guess.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
