---
name: state-management
description: |
  Choose between local, lifted and server state; cache server data without duplicating it. Use when
  state is tangled.
version: 1.0.0
source: in-repo
---

# state-management

## Workflow

1. Classify state: local UI, shared UI, server cache, URL. Each has one home.
2. Server data belongs to a query cache (fetch keys), not to global stores.
3. URL for navigation state so links and reloads work.
4. Avoid derived state stored twice; compute from the source.
5. Keep stores small and typed; no god store.
6. Document the ownership of each state slice.

## Quality checks

- Reloading the page restores the same view for URL-driven state.
- No component holds a second copy of server data.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
