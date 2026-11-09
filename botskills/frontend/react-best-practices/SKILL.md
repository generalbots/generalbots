---
name: react-best-practices
description: |
  Write React components with correct hooks, rendering and composition. Use when building or reviewing
  React UI.
version: 1.0.0
source: in-repo
---

# react-best-practices

## Workflow

1. Keep components pure; derive state instead of syncing it with effects.
2. Effects only for external systems (subscriptions, DOM, fetch) and always with cleanup.
3. Lift state to the nearest common parent; pass data down and callbacks up.
4. Memoise only measured hot paths (React.memo, useMemo, useCallback).
5. Lists: stable keys from data, never array index for reorderable items.
6. Split by responsibility: container fetches, presentational components render.

## Quality checks

- No state update during render; no missing dependency in useEffect.
- Every effect that opens something closes it on unmount.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
