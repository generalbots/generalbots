---
name: caching
description: |
  Choose cache keys, TTLs and invalidation that cannot serve stale data silently. Use when adding
  caches.
version: 1.0.0
source: in-repo
---

# caching

## Workflow

1. Key includes every input that changes the value (tenant, user, params, version).
2. TTL by volatility; a hard upper bound even for 'immutable' data.
3. Invalidate on write via version bump or delete, not by guessing TTLs.
4. Cache-aside reads; never cache errors for long.
5. Stampede protection (single-flight) for hot keys.
6. Measure hit rate and the cost of a miss.

## Quality checks

- Changing the underlying data is visible within the declared staleness.
- A cache outage degrades to the database, not to an error.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
