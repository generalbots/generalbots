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

**Allow once** is a one-shot handoff, not a stored grant. The decision lands in
an in-memory approved map, and the gate consumes the matching approval
(`take_approved_match`) before it looks for a grant, so the assistant's retry
runs exactly once and the next identical request prompts again. A destructive
class always prompts, even when a broader grant exists.

Enforcement points: the chat command path (`__api_call__`) and the api/ui
loopback executor (classified by HTTP method). Manual use of the applications
is never gated.
