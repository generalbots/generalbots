# Consent System

Agent-initiated actions on suite applications are gated by a per-user
permission matrix (`user × app × action class`). Destructive and payment
classes are deny-by-default; payment grants expire at month end even when
granted "always".

Flow: the planner requests an action → no grant exists → a pending request is
created and an interactive card is rendered in the conversation (or serialized
to remote channels). Decisions are **Allow once**, **Always allow** or **Deny**;
every decision is audited (`consent_audit`). Grants can be reviewed and revoked
in Settings.

Enforcement points: the chat command path (`__api_call__`) and the api/ui
loopback executor (classified by HTTP method). Manual use of the applications
is never gated.
