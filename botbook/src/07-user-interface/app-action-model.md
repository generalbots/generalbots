# App Action Model 🟢 GA

Every action the assistant can take inside a suite application comes from one
declarative table in the server. This page describes that table, the two
protocols that reach it, and the checks that decide whether a given user is
allowed to run an action.

Sources of truth:

| Claim | Authority |
|---|---|
| Which apps exist | `botserver/src/apps/registry/mod.rs` (`all_apps`), entries in `entries.rs` + `entries_extra.rs` |
| Which verbs exist | `botserver/src/apps/commands/app_command.rs` (`AppCommand`), tables in `deep_link_params_for_app.rs` (`ALL_COMMANDS_CORE`) and `app_commands_extra.rs` (`ALL_COMMANDS_EXTRA`) |
| Verb → REST binding | `botserver/src/apps/commands/verb_endpoints.rs` (`VERB_ENDPOINTS`) |
| Registered routes | `botserver/src/core/bot/endpoint_inventory.rs` and its four sibling chunks |
| Chat execution | `botserver/src/core/bot/api_catalog/execute_command.rs`, `botserver/src/core/bot/api_exec.rs` |
| UI execution | `botserver/src/main_module/ui_plan/`, `botui/ui/suite/js/ui-orchestrator/` |

Measured today: **68 apps** in the registry, **312 curated verbs**, **74
verb→endpoint bindings**, and **1,335 entries** in the route inventory.

## What a verb is

A verb is an `AppCommand` record. It is not a route and not a BASIC keyword; it
is the named, user-facing action that both the chat prompt and the command
palette agree on.

```rust
pub struct AppCommand {
    pub app: &'static str,        // owning app id, must exist in the registry
    pub name: &'static str,       // "docs.export"
    pub label: &'static str,      // palette label
    pub summary: &'static str,    // one line injected into the LLM prompt
    pub params: &'static [(&'static str, &'static str)],
    pub deep_link: Option<&'static str>, // "app://docs?file={file_id}"
    pub admin_only: bool,         // admin role required to execute
}
```

Names follow `app.action`, for example `docs.create`, `sheet.cells.set`,
`crm.people.search`, `recycle.empty`. Three build-time tests in
`botserver/src/apps/commands/mod.rs` fail the build if an app has no verb, if a
verb names an app the registry does not know, or if two verbs share a name.
That is why the registry and the verb table cannot drift apart.

### The read / write / admin distinction

| Distinction | Where it lives | Effect |
|---|---|---|
| Read vs write | The HTTP method of the bound endpoint | `GET` verbs return data; `POST`/`PUT`/`PATCH`/`DELETE` change state |
| Destructive | Verb summary wording plus `admin_only` | `recycle.empty` says "Confirm first" and is admin-only, but it is navigation-only today: chat returns a deep link to the app rather than deleting |
| Admin-only | `admin_only: true` on the verb, **and** a row in `rbac_api_permissions` for the endpoint prefix | Filtered out of `api.find` and refused by `execute_command` |
| Navigation-only | `deep_link` present, no endpoint binding | Opens the app at a record; performs no server mutation |

There are **24 verbs with `admin_only: true`** (13 in the core table, 11 in the
extra table), including `recycle.empty`, `store.product.publish` and
`o365.mail.send`.

## Surface 1 — chat and WhatsApp: `__api_call__`

On every channel the model is taught a two-step protocol in the system prompt
(`api_command_instructions` in
`botserver/src/core/bot/api_catalog/commands_list.rs`):

1. Discover with `api.find` and `{query}`. It matches verb names and summaries,
   and also harvested endpoint names derived from the route inventory
   (`botserver/src/core/bot/commands_derived.rs`).
2. Execute with `api.exec` and `{method, path, params}`, or call the curated
   verb name returned by the search.

The call is a single JSON block as the **first line** of the reply:

```json
{"__api_call__": {"name": "docs.export", "params": {"file_id": "...", "format": "pdf"}, "compose": true}}
```

`compose: true` asks the backend to turn the result into prose for the user.

Execution is in-process. `execute_command` dispatches to a curated handler when
one exists (tax, banking, CRM, docs, database verbs), otherwise to the generic
loopback executor:

```text
verb name ──► curated handler (in-process)
          └─► verb_endpoint(name) ──► api_exec::exec_endpoint(method, path, params)
                                    └─► no binding ──► deep_link response, or a
                                        "not executable from chat" error
```

`api_exec::exec_endpoint` (`botserver/src/core/bot/api_exec.rs`) is the boundary
that makes this safe:

- the method must be one of `GET`, `POST`, `PUT`, `PATCH`, `DELETE`;
- the path must match a registered route in `endpoint_inventory::all_routes()`;
- `:id`-style path parameters are substituted only from a fixed key allowlist
  (`ALLOWED_PARAM_KEYS`), never from arbitrary keys;
- `is_admin_only_endpoint` consults `rbac_api_permissions` for
  `group_name = 'admin'` and refuses non-admin users;
- with the `consent` feature the action class (`read`/`create`/`update`/`delete`)
  goes through `consent_gate::check`;
- the loopback request authenticates with a short-lived HS256 service assertion
  (`api_exec_assertion.rs`) signed by the server secret — `sub` is the session
  user, `exp` is 120 s — so the auth middleware verifies the caller instead of
  trusting a header, then re-applies RBAC and CSRF rules. `X-User-ID` is never
  an identity (#2122);
- response bodies are truncated to 60,000 bytes before reaching the model.

## Surface 2 — web UI: `__ui_plan__` and message type 9

When the user asks for something that is a *UI operation* rather than a data
operation, the model answers with a plan instead of a call:

```json
{"__ui_plan__": {"app": "crm", "steps": [
  {"op": "open", "app": "crm"},
  {"op": "click", "label": "New Lead"},
  {"op": "fill", "field": "First Name", "value": "Jeff"},
  {"op": "submit"}
]}}
```

The server validates the plan before any client sees it
(`botserver/src/main_module/ui_plan/sanitize_value.rs`,
`mod.rs`):

| Limit | Value | Constant |
|---|---|---|
| Steps per plan | 200 | `MAX_UI_STEPS` |
| Length of any string value | 500 | `MAX_VALUE_LEN` |
| Allowed ops | `open`, `click`, `fill`, `select`, `submit`, `wait`, `cell`, `menu`, `download` | `UiOp` |
| App id | must match a registry id | `validate_ui_plan` |

Values are sanitised (markup is stripped) and unknown ops are rejected. The
validated plan leaves the server as a WebSocket frame with
`"message_type": 9` (`MessageType::UI_ACTION` in `botlib/src/message_types.rs`),
emitted from `botserver/src/core/bot/pipeline/llm/stream.rs`.

The frontend consumes it in two places:

- `botui/ui/suite/chat/chat-websocket.js` — on `UI_ACTION` with a `plan`, calls
  `window.GBUiOrchestrator.executePlan(data.plan)`;
- `botui/ui/suite/js/ui-orchestrator/99_execute.js` — refuses any app not in
  `window.APPS_REGISTRY`, runs one plan at a time, chains steps, and aborts the
  remaining steps after a hard failure.

The prompt that teaches the plan format is built by
`ui_automation_instructions()` in
`botserver/src/main_module/ui_plan/ui_automation_instructions.rs`, which lists
only the apps enabled by the product profile, their deep-link keys, and the
per-app sequence hints from `UI_SEQUENCE_HINTS`.

## Why a verb can disappear for a role

Two independent filters run, and both must agree:

1. **Discovery filter.** `find_api_entries` drops verbs with `admin_only: true`
   when `resolve_user_role` is not `admin`, and drops derived endpoint commands
   when `is_admin_only_endpoint` matches the method and path.
2. **Execution filter.** `execute_command` re-checks the verb's `admin_only`
   flag, and `exec_endpoint` re-checks the endpoint matrix. A model that
   hallucinates an admin verb therefore gets an error, not a result.

Role resolution itself is in `botserver/src/security/user_role.rs`: a user is
`admin` when any active group in `rbac_groups` they belong to has a name
containing `admin`. Note the fallback in `resolve_user_role_with_email`, which
matches the canonical `users` row by email when the derived id has no groups.

`session_tool_associations` is a **different** surface: it gates `.bas` tools
that a session may run (see `botserver/src/core/bot/tool_context.rs`), not
verbs. A tool can be unavailable while its app's verbs still work, and the
reverse is also true.

## End-to-end example

Request: *"export the Q3 report as PDF"* on the web channel.

1. The system prompt carries the `__api_call__` contract with a short list of
   common commands (not all 312), and, on web, the `__ui_plan__` contract with
   the enabled-app catalog.
2. The model emits `{"__api_call__": {"name": "docs.export", "params":
   {"file_id": "<id>", "format": "pdf"}}}`.
3. `handle_api_call` in `botserver/src/core/bot/pipeline/api_call/handle.rs`
   detects the trigger, then `execute_command` looks up `docs.export`.
4. `docs.export` is a curated handler (`api_catalog/docs_verbs.rs`), because a
   single verb spans several routes; it resolves the format through
   `docs_export_path("pdf")` → `/api/docs/export/pdf`.
5. `exec_endpoint` validates the route, the admin matrix and consent, then calls
   the running server over loopback as the same user.
6. The JSON result is truncated, humanised (`pipeline/api_call/humanize.rs`) and
   streamed back as `message_type: 2`.

For a UI-only request the same turn would instead produce a `__ui_plan__` with
`open` and `menu` steps (`["Download", "PDF"]`), and the browser driver would
perform the clicks.

## Coverage and gaps

| Area | State |
|---|---|
| Apps with at least one verb | 68 / 68 — enforced by `every_registered_app_has_at_least_one_command` |
| Verbs with a `VERB_ENDPOINTS` binding | 74 / 312 |
| Verbs without a binding | The remainder — a curated handler (docs, database, CRM, tax, banking, vibe) executes some; the rest answer with a deep link |
| Verbs with `admin_only` | 24 |
| Routes discoverable through `api.find` | 1,335 inventory entries, filtered per role |
| Generated REST pages | See [Generated REST reference](../08-rest-api-tools/generated/README.md) |

The honest reading of the numbers: the *surface* is complete for navigation and
for the 74 generic bindings, and the rest of the 312 verbs are either handled by
a curated in-process handler or answered with a deep link. A verb with a deep
link answers `{"navigate": true, "app": …, "deep_link": …}` and the UI opens the
app; a verb with neither a binding nor a deep link answers
`command '<name>' is not executable from chat; open the <app> app instead`.
Extending coverage means adding a `bind(...)` entry in `verb_endpoints.rs`
naming a route that already exists in the inventory — never inventing a path.

## Related pages

- [WebSocket Protocol](./ws-protocol.md) — message types 0–9 and frame shapes.
- [Agentic UI Orchestration & Universal Search](./agentic-ui-orchestration.md) —
  the plan/execute loop from the UI side.
- [Suite Apps](./apps/README.md) — per-app surfaces.
- [RBAC Overview](../09-security/rbac-overview.md) and
  [Role-Based Tool Access](../09-security/role-based-tools.md) — the role model.
