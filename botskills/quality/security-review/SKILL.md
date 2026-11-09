---
name: security-review
description: |
  Check secrets, injection, authz, headers and dependency risk. Use before shipping a
  security-sensitive change.
version: 1.0.0
source: in-repo
---

# security-review

## Workflow

1. Secrets: none in code/config/logs; loaded from Vault/env.
2. Injection: parameterised SQL, no shell string building, output encoding.
3. Authorization: every endpoint checks role/tenant server-side.
4. Transport/headers: TLS, HSTS, CSP, nosniff, frame options.
5. Dependencies: audit for known CVEs; pin critical ones.
6. Errors: sanitized responses, detailed logs without secrets.

## Quality checks

- No secret value appears in the repo or logs.
- Every state-changing endpoint enforces authz.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
