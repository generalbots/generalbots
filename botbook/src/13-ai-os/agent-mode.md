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

## Artifacts and Drive

The conversation folder is ephemeral: it disappears with the session, so a file
left only in `output/` used to vanish when the user closed the chat unless they
clicked Save/Open or the model called `drive_save`.

At the end of every turn that produced files, `pipeline/output_publish.rs`
publishes the new or changed files from `output/` to the user's Drive through
the same bridge the Save button uses
(`{bot}.gbdrive/conversations/{title}/`) and attaches the `vmfile:` link to the
answer, so the artifact survives the conversation.

- Bounded: at most 5 files, 25 MB per file and 50 MB per turn.
- Idempotent: an unchanged file (same size and modification time) is neither
  read nor uploaded again; a file that was only touched is not re-uploaded when
  its content digest still matches.
- Best effort: if Drive is unavailable the turn still succeeds, a warning is
  logged, and the next turn retries. Nothing is claimed as saved.
- `drive_save` remains available for an explicit, on-demand publish (for
  example into a folder the user named).

Channels other than the web chat get the durable Drive copy but no `vmfile:`
message, because that link form is only rendered by the suite chat.
