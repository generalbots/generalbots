# Agent Mode & Snapshots

The chat header exposes a **Chat / Agent** switcher. Enabling Agent mode for a
conversation provisions a dedicated Incus container bound to that session; the
binding persists across reloads (`agent_sessions` table) and the workspace is
stopped after an idle timeout, then resumed on demand.

## Cold start

The first launch on a host downloads and unpacks the Incus image, which can take
several minutes. `botserver/crates/botagent/src/vm/lifecycle.rs` serializes
provisioning per user, so a second turn that arrives during that window waits
for the same container instead of launching a duplicate. Transient Incus
failures (`incus` busy, instance not created yet, already running) are retried
with exponential backoff in `vm/incus.rs`. A container that is still not running
after the cold budget (900 s, override with `AGENT_VM_LAUNCH_WAIT_SECS`) reports
a retryable "still starting" reason instead of the generic
`Agent VM operation failed`.

## Snapshots

- Create: `POST /api/agent/sessions/{id}/snapshots` with optional label.
- List: `GET` on the same path. Restore: `POST /api/agent/snapshots/{id}/restore`.
- Retention is capped per session (FIFO eviction).
- The same lifecycle backs Vibe dev/prod environment checkpoints.

## Sandbox execution

`POST /api/v1/sandbox/exec` (`botserver/crates/botagent/src/lib.rs`) runs short-lived code (Python, Node, shell) inside
an ephemeral container with memory/CPU limits and network deny-by-default.
Authentication accepts either a JWT or an organization API key
(`GET|POST /api/cloud/orgkeys`, keys stored as SHA-256 hashes).

## Control frame

The switcher sends `{ "type": "agent_mode", "enabled": true|false }` over the
existing WebSocket. The server replies with an `agent_mode_status` frame;
no LLM invocation is involved.
