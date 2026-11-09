---
name: release-management
description: |
  Version, changelog, deploy and roll back safely. Use when shipping.
version: 1.0.0
source: in-repo
---

# release-management

## Workflow

1. Semantic version from the change type; tag the release commit.
2. Changelog groups: added, changed, fixed, security; link issues.
3. Deploy through CI only; never copy binaries by hand.
4. Smoke test the deployed build (health, key flow, logs clean).
5. Rollback plan: previous artefact + migration compatibility.
6. Announce breaking changes before shipping them.

## Quality checks

- Every release has a reproducible artefact and a changelog entry.
- Rollback was rehearsed or is a single documented step.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
