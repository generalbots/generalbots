# Agentic UI Orchestration & Universal Search

Enterprise-grade feature that lets a user drive **any** of the ~80 suite
applications from the Chat window, and search across **all** applications
with a single interface.

```
User: "create a new customer named Jeff"
  └─► Chat WS (message_type 1)
        └─► LLM generates {"__ui_plan__": {...}} JSON
              └─► Backend validates plan (ops/app allowlists, value escaping)
                    └─► WS message_type 9 (UI_ACTION) → frontend
                          └─► GBUiOrchestrator executes steps:
                                open window → resolve field by label →
                                animated typing → click → submit
```

## 1. Message Types

| ID | Name | Direction | Purpose |
|----|------|-----------|---------|
| 9 | `UI_ACTION` | server → client | JSON plan of UI steps to animate |

- Backend: `botlib/src/message_types.rs` (`UI_ACTION = 9`)
- Frontend: `botui/ui/suite/chat/chat-state.js` (`MessageType.UI_ACTION = 9`)

## 2. UI Plan Protocol

The LLM emits a self-contained JSON object (mirroring the existing
`__tool_call__` mechanism) as the **first** thing in its response when the
user requests a UI-level operation:

```json
{"__ui_plan__": {"app": "crm", "steps": [
  {"op": "open", "app": "crm"},
  {"op": "click", "label": "New Lead"},
  {"op": "fill", "field": "First Name", "value": "Jeff"},
  {"op": "fill", "field": "Email", "value": "jeff@example.com"},
  {"op": "submit"}
]}}
```

### Step operations

| op | args | description |
|----|------|-------------|
| `open` | `app` | Open the app window via WindowManager |
| `click` | `label` | Click element matching visible text |
| `fill` | `field`, `value` | Type into field resolved by label/placeholder/name |
| `select` | `field`, `value` | Choose option in a `<select>` |
| `submit` | — | Submit the active form |
| `wait` | `ms` | Pause execution |

### Backend validation (`server/src/main_module/ui_plan.rs`)

- `op` must be in the allowlist above
- `app` must exist in `apps::registry::all_apps()` (or be `chat`)
- `value`/`field`/`label` stripped of HTML, truncated (≤ 500 chars value)
- Plan capped at 32 steps
- Invalid plans are logged + rejected (no client echo)

## 3. LLM Instruction Injection

`exec.rs` appends an "Agentic UI" instruction block to the system prompt for
the web channel listing available apps (id + title + description) and the
exact `__ui_plan__` JSON contract. The instruction tells the LLM:

- Emit the plan as the first line, then a short user-facing confirmation
- Use only apps present in the list; use `click`/`fill`/`select`/`submit`
- For search requests, emit `{"__ui_plan__": {"op": "open", ...}}` or let the
  Universal Search handle it

## 4. Streaming Interception (`llm.rs`)

In the stream loop, chunks containing `"__ui_plan__"` are accumulated
separately (like `__tool_call__`), never sent as bot content. After the
stream completes:

1. Parse the plan JSON
2. Validate via `ui_plan.rs`
3. If valid → send WS frame `{message_type: 9, plan: {...}}` to the client
4. If invalid → log + continue with normal text response

## 5. Universal Search

### Backend endpoint

`GET /api/ui/search?q=<query>` (Bearer token, rate-limited)

Scans the following entity sources with `ILIKE` and returns a flat result set:

| source | table | app |
|--------|-------|-----|
| people | `people` | people |
| crm contacts | `crm_contacts` | crm |
| products | `products` | products |
| services | `services` | products |
| tickets | `tickets` | tickets |
| kb documents | `kb_documents` | research |
| drive files | `drive_objects` | drive |
| bots | `bots` | admin |

Response:

```json
{"results": [
  {"app": "crm", "type": "contact", "id": "...", "title": "Jeff Bezos",
   "subtitle": "jeff@example.com", "url": "/suite/crm/crm.html"},
  ...
]}
```

### Frontend (`js/gb-search.js`)

- Search input in the desktop shell (taskbar area) + chat window
- Debounced (300 ms) GET, results dropdown grouped by app
- Clicking a result opens the app window and **focuses the entity**:
  1. `WindowManager.open(appId, title, "")` → fetch `hxGet` URL → inject
  2. Wait for rows to render (poll for `tbody tr` or `[data-entity]`)
  3. Flash-highlight + scroll to the matching row (by id/text match)
  4. If the app exposes a detail handler, open it

## 6. Frontend Orchestrator (`js/ui-orchestrator.js`)

`window.GBUiOrchestrator` executes plans with visible animation so users see
forms being controlled:

- **Ghost cursor** — an animated SVG cursor that glides to the target element
- **Focus ring** — pulsing highlight on the target field/button
- **Typing animation** — characters appear one by one with a blinking caret
- **Click pulse** — expanding ring on click targets
- **Step checklist** — chat shows each step as it executes
  ("Opened CRM → Filled First Name → Submitting…")
- **Field resolution** (no hardcoded IDs): `label[for=]` → wrapping `<label>`
  text → `placeholder` → `name` → `aria-label`
- **Robustness**: `setTimeout`-driven async queue; each step guarded by
  try/catch; timeouts (e.g. 5 s) for DOM readiness; partial failure reports
  to chat instead of crashing

Apps can opt into precise targeting via `data-gb-field="first_name"` on
inputs — the orchestrator prefers these attributes, falling back to heuristics.

## 7. Files Changed

| File | Change |
|------|--------|
| `botlib/src/message_types.rs` | `UI_ACTION = 9` |
| `botserver/src/main_module/ui_plan.rs` | **New** — plan types + validation |
| `botserver/src/main_module/routes/unified_search.rs` | **New** — search endpoint |
| `botserver/src/core/bot/pipeline/exec.rs` | System prompt UI instruction |
| `botserver/src/core/bot/pipeline/llm.rs` | `__ui_plan__` interception |
| `botserver/src/main_module/routes/sub_router.rs` | Register search route |
| `botui/ui/suite/chat/chat-state.js` | `MessageType.UI_ACTION` |
| `botui/ui/suite/chat/chat-websocket.js` | Route type 9 → orchestrator |
| `botui/ui/suite/js/ui-orchestrator.js` | **New** — animated driver |
| `botui/ui/suite/js/gb-search.js` | **New** — universal search UI |
| `botui/ui/suite/desktop.html` | Search bar + scripts |
| `botui/ui/suite/chat/chat.html`, `partials/chat.html` | Include scripts |

## 8. Security

- Ops and app allowlists enforced server-side; the client never trusts the LLM
- Values are escaped before being written to DOM (`textContent`-safe writes)
- Search endpoint is auth-guarded (Bearer) and rate-limited
- No arbitrary `querySelector` from LLM input — only label/name/placeholder
  matching and `data-gb-field` attributes
