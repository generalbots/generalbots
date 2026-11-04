# Conversation Workspace

The conversation workspace turns a chat turn into a durable, inspectable run. Progress, tool calls, questions, plans and interactive results are rendered inside the message body instead of a separate panel, and a closed tab no longer cancels the work.

Tracking: epic [`#1662`](https://github.com/generalbots/generalbots/issues/1662).

## Architecture

```text
source -> redactor -> validator -> sequencer -> run_events -> transport
```

- **`botevents`** (`botserver/crates/botevents`) owns the envelope, the run state machine, the thread sequencer, structural validation, redaction and the JSON Schema.
- **`botserver` event sink** (`src/core/bot/pipeline/event_sink.rs`) adapts the contract to the existing `ChannelSink`, persists accepted frames and emits them as `gb_event` WebSocket frames.
- **Run bus** (`src/core/bot/runs/bus.rs`) owns one run per thread. Sockets subscribe; the run task does not depend on the socket. Chat turns run in a background task, so closing the tab keeps the work alive.
- **`run_events`** (`botschema::runs`, migration `6.8.5-run-events`) is the append-only log. `seq` is dense per thread and enforced by `UNIQUE (thread_id, seq)`; `event_id` is content-derived and makes retries idempotent.
- **`agent_runs` / `agent_spans`** remain the durable summary and usage store. `message_history` gains only a nullable `run_id`/`thread_id` pointer; content stays encrypted.

## Enabling

| Variable | Effect |
|---|---|
| `GB_RUN_EVENTS=1` | Emit run frames and advertise structured tools. Off by default so an existing chat deployment is unchanged. |
| `GB_RUN_EVENTS_RETENTION_DAYS=30` | `run_events` retention. The run summary is never deleted by this job. |
| `GB_SANDBOX_PORT=8081` | Dedicated sandbox listener. Desktop defaults to loopback 8081; cloud sets this and proxies the sandbox hostname. |
| `GB_SANDBOX_ORIGIN=https://sandbox.example.com` | Origin the client uses for signed UI resources. |
| `GB_APP_ORIGIN=https://chat.example.com` | `frame-ancestors` for sandbox resources. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP/HTTP JSON trace export. Unset disables export. |

## Event vocabulary

`RUN_STARTED`, `RUN_FINISHED`, `RUN_ERROR`, `STEP_STARTED/FINISHED`, `TOOL_CALL_START/ARGS/END/RESULT`, `TEXT_MESSAGE_START/CONTENT/END`, `STATE_SNAPSHOT/DELTA`, `MESSAGES_SNAPSHOT`, `ACTIVITY_SNAPSHOT/DELTA`, `SUBAGENT_STARTED/FINISHED`, `CUSTOM`, `RAW`.

Product events travel as `CUSTOM` with a `gb.*` name:

| Name | Purpose |
|---|---|
| `gb.question` | Structured question card (`ask_user`) |
| `gb.plan` | Editable plan (`propose_plan`) |
| `gb.surface` | Tier 1 declarative surface (`show_surface`) |
| `gb.surface_update` | JSON Patch against a mounted surface data model |
| `gb.app` | Inline MCP app (`show_app`) |
| `gb.command_output` | Live command output line |

The checked-in schema is `botserver/crates/botevents/schemas/run-event.v1.json`; regenerate with `cargo run -p botevents --example dump_schema`.

## Tools

Native tools (no `.bas` required, advertised on web when run events are enabled):

- `ask_user` — one or more fields; blocking questions end the turn in `waiting_input` and the answer starts the next turn. Answered field ids are remembered per thread.
- `propose_plan` — steps the user can reorder/remove/approve before execution.
- `show_surface` — components from the trusted catalog `gb.basic.v1` plus a mandatory fallback.
- `show_app` — self-contained HTML published through the sandbox store and rendered in an opaque iframe.

Invalid calls return schema paths and the model is expected to correct them. The prompt includes the when-not-to-use rubric and few-shot intents (see `pipeline/surface_tools.rs`).

## Rendering

| Module | Responsibility |
|---|---|
| `chat/modules/71_run_timeline.js` | Run card, steps, tool rows, live log, Stop, resume |
| `chat/modules/72_surface_renderer.js` | Tier 1 catalog, forms, plans, data bindings, JSON Patch updates |
| `chat/modules/73_app_frame.js` | MCP Apps iframe, JSON-RPC bridge, hibernation, live-app cap |
| `chat/modules/74_viz.js` | Flow, timeline, metrics, bars, lines with fallbacks |
| `chat/modules/75_perf.js` | First-paint, log-latency and resume budgets (`window.GBPerf`) |
| `chat/chat-websocket.js` | Feeds frames to the timeline, surfaces and apps; `run_resume` |

Security invariants:

- Model HTML never runs on the app origin. Tier 2 apps render in an opaque iframe (`sandbox="allow-scripts"`, no `allow-same-origin`).
- UI resources are content-addressed and served by short-lived HMAC-signed URLs scoped to thread + hash (`/sandbox/ui/{hash}?t=…`). The session transport token is never reused.
- CSP is built from `_meta.ui.csp` (`connectDomains`, `resourceDomains`); camera, microphone, geolocation and clipboard are denied unless declared.
- `show_app` accepts at most 512 KiB and only https/wss/loopback CSP sources.
- App-initiated tool calls are confirmed and sent as a consented assistant turn; destructive tools always prompt.

## Storage

| Concern | Store |
|---|---|
| Ordered frames | `run_events` (new) |
| Run summary | `agent_runs` (extended with `session_id`, `thread_id`, `user_id`, `parent_run_id`) |
| Usage | `agent_spans`, `compute_usage_hourly` |
| Message pointer | `message_history.run_id`, `message_history.thread_id` |
| UI resource bytes | Drive `_sandbox/ui/{hash}.html` + in-process cache |
| Consent | `botconsent` (`app_permissions`, `consent_audit`) |

## Resume and steering

The client stores the highest `seq` per session and sends `{"type":"run_resume","last_seq":N}` on reconnect. The server replays persisted frames after `N`. A message sent while a run is active is queued (`run bus` steering queue) and delivered once at the next turn boundary. Stop sends `{"type":"run_cancel"}`; the model loop checks the cancel flag between chunks and skips tool execution.

## Tests and gates

- `cargo test -p botevents` — envelope, state machine, redaction, sequencer.
- `cargo test -p boteval` — frame fixtures (one per family), golden transcript replay, redaction cases, 220-prompt restraint set.
- `boteval::frames::gate::evaluate` — validity ≥ 99%, fallback coverage 100%, at most 5% UI on prose prompts, zero duplicate questions, deterministic replay.
- `window.GBPerf.metrics()` — first paint < 150 ms, log latency p95 < 250 ms, reconnect resume < 1 s.

## Quality dashboard

`GET /api/runs/quality` returns frame/UI/question/app counts, consent denials (24 h), sandbox errors and token totals. The admin fragment is `botui/ui/suite/admin/run-quality.html`.

## Tool discovery and manifests

The native surface tools (`ask_user`, `propose_plan`, `show_surface`, `show_app`) are declared once in `botserver/src/core/bot/pipeline/surface_catalog.rs`. The seed is the single source of truth for:

- the capability block injected into the web system prompt (`prompt_block()`), including the when-not-to-use rubric and one few-shot per tool;
- the MCP-spec manifests served at `GET /api/runs/tools` (`name`, `title`, `description`, `inputSchema`, `annotations` with `readOnlyHint` / `destructiveHint` / `openWorldHint`).

Adding a tool is a single entry in the seed; a test fails if the prompt, the dispatch table in `surface_tools::try_run` and the manifests drift apart.

Tool scripts can declare the same annotations in BASIC. The compiler parses the lines and emits them into the generated `.mcp.json`:

```basic
DESCRIPTION "Classify an inbound image or document and file it in Drive."
PARAM path AS string DESCRIPTION "Drive path of the received media"
ANNOTATION READ_ONLY
ANNOTATION TITLE "Classify and file media"
```

Unknown annotation hints are ignored (forward compatibility), and a tool without hints keeps the plain manifest. `cargo test -p botbasic_compiler` guards the source-to-manifest flow.

## Privacy: locked conversations (#1682)

A conversation can be locked from the sidebar (`Lock conversation`) with a
4–8 digit PIN. Locking stores an Argon2 hash on `user_sessions.locked` /
`lock_pin_hash` and takes the conversation out of every listing and search
path server-side — the filter lives in the query layer, not the UI.

Opening a locked conversation asks for the PIN. `POST
/api/chat/history/sessions/:id/unlock` verifies it and mints a short-lived
grant (5 minutes idle, extended on each read). The grant is what the chat tab
sends when replaying `/messages`; the conversation itself stays locked, so a
closed tab or an idle window re-locks it without a second state machine.
`GET /api/chat/history/sessions?locked=1` lists locked ids without titles so
the Locked view can offer them without leaking content.

## Portable skills (#1681)

`POST /api/sources/skills` imports a `SKILL.md` package (front-matter `name`,
`description`, `version` plus instructions). The import validates the slug,
hashes the document and stores it in `bot_skills` with `enabled`, `version`
and `content_hash` (migration 6.8.8). `GET` lists, `POST /:id/enable|disable`
toggles and `DELETE /:id` removes; the Settings → Integrations card is the
UI.

Enabled skills are rendered into the system prompt as an `INSTALLED SKILLS`
block under a clear data boundary: packages are guidance, never executable
instructions, and a hash change is visible in the list so re-consent follows
the same pinning idea as MCP UI resources.

## Generated apps for external hosts (#1676)

`show_app` publishes content-addressed resources (`_sandbox/ui/{hash}.html`
with a `.meta.json` sidecar). The built-in MCP server at `POST /api/mcp/apps`
implements `initialize`, `resources/list` and `resources/read` over
streamable-HTTP JSON-RPC:

- resources appear as `ui://generated/{hash}` with
  `text/html;profile=mcp-app` and the declared CSP in `_meta.ui`;
- `resources/read` returns the same sanitized HTML the sandbox serves;
- resources are immutable by hash, so any compliant host can list and render
  a generated app without touching our chat surface.

## Desktop settings and takeover (#1675, #1677)

The desktop edition mounts the MCP management component inside Settings →
Integrations against the same `/api/sources/mcp` endpoints as the
Integrations app (no parallel store); hash drift from approval is shown and
re-consent starts from Settings.

While a run holds a terminal or browser surface the run card exposes Take
over / Hand back. Takeover opens the real PTY of the workspace VM (the same
container the run executes in), and every segment is persisted as a `CUSTOM`
`gb.takeover` run event, replayed on reload and on a second device. ESC sends
`run_cancel`, while hand-back only returns input to the agent.

## Visual language (chat surface)

The conversation surface is styled by `botui/ui/suite/css/chat-modern.css`,
which is loaded from `desktop.html` after the chat partial and after the
desktop window shell, so it wins without `!important` except for the message
text colour the older partial forces.

Tokens (scoped to `#chat-app` and `.chat-sidebar`):

| Token | Light | Purpose |
|---|---|---|
| `--gb-bg` | `#fafafa` | Conversation canvas |
| `--gb-surface` | `#ffffff` | Bot bubble, composer, raised controls |
| `--gb-surface-muted` | `#f4f4f5` | User bubble |
| `--gb-border` | `#e4e4e7` | Default border — a true gray, never black or black-alpha |
| `--gb-border-soft` | `#ececef` | Bot bubble and run-card outline |
| `--gb-border-strong` | `#d4d4d8` | Window frame and hover outlines |

Rules the design review pinned:

- **User messages are a neutral gray surface with a gray border and dark
  text.** No saturated accent fill, no white text, no black border. The same
  contract holds in dark mode (`#26262b` fill, `#3f3f46` border).
- **Bot messages** are white (dark: `#1b1b1f`) with an `#ececef`
  (`#313138`) border, 18 px radius and a 1 px shadow.
- **Hover metadata:** `chat-messages.js` stamps `data-time` on live messages;
  CSS reveals the timestamp on hover. Replayed history keeps its stored time.
- **Message actions** (copy, listen) sit beside the bot bubble and are revealed
  on hover/focus; they are always visible on hover-less (touch) devices.
- **Run cards** are compact, gray-outlined cards. A finished run that arrived
  through `run_resume` replay is removed rather than parked at the bottom of
  the transcript; live cards keep their position for the turn.
- **Composer** has one gray border and one soft shadow. The focused state
  changes the border colour only — no double outline.
- **Window chrome** around the chat uses `--gb-border-strong`; the theme's
  near-black focused border is overridden for the window that hosts
  `#chat-app`.

`desktop.html` carries the cache-buster for `chat-modern.css` and
`partials/chat.html` for `chat-messages.js` / `71_run_timeline.js`; bump them
whenever those files change.

Tracking: [`#1689`](https://github.com/generalbots/generalbots/issues/1689).
