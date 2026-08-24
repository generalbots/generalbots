# Agent Mode & Snapshots

The chat header exposes a **Chat / Agent** switcher. Enabling Agent mode for a
conversation provisions a dedicated Incus container bound to that session; the
binding persists across reloads (`agent_sessions` table) and the workspace is
stopped after an idle timeout, then resumed on demand.

## Snapshots

- Create: `POST /api/agent/sessions/{id}/snapshots` with optional label.
- List: `GET` on the same path. Restore: `POST /api/agent/snapshots/{id}/restore`.
- Retention is capped per session (FIFO eviction).
- The same lifecycle backs Vibe dev/prod environment checkpoints.

## Sandbox execution

`POST /api/v1/sandbox/exec` runs short-lived code (Python, Node, shell) inside
an ephemeral container with memory/CPU limits and network deny-by-default.
Authentication accepts either a JWT or an organization API key
(`GET|POST /api/cloud/orgkeys`, keys stored as SHA-256 hashes).

## Control frame

The switcher sends `{ "type": "agent_mode", "enabled": true|false }` over the
existing WebSocket. The server replies with an `agent_mode_status` frame;
no LLM invocation is involved.
