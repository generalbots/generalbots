---
name: auth-sessions
description: |
  Implement tokens, sessions, RBAC and CSRF correctly. Use for authentication/authorization work.
version: 1.0.0
source: in-repo
---

# auth-sessions

## Workflow

1. Passwords hashed with argon2/bcrypt cost tuned; never reversible.
2. Sessions: httpOnly, secure, sameSite cookies; rotate on privilege change.
3. Access tokens short-lived; refresh tokens stored hashed and revocable.
4. Authorization checked server-side per request, not hidden in the UI.
5. CSRF token for cookie-authenticated state changes; bearer APIs exempt.
6. Rate limit login and token endpoints; log failures without secrets.

## Quality checks

- A revoked session cannot call protected endpoints.
- No token or password appears in logs.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
