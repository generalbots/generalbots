# Chapter 8: REST API & Tools 🟡 BETA

HTTP API for integrating with botserver. The canonical port and prefix table
lives in [Network Contract](../02-architecture-packages/network-contract.md);
this chapter covers the endpoint families and their response shapes.

## Base URL

```
http://localhost:8080/api
```

Every local route starts with `/api`. The single exception is the legacy
sandbox endpoint `POST /api/v1/sandbox/exec`
(`botserver/crates/botagent/src/lib.rs`) — it is the only local `/api/v1` route
in the tree; do not treat `/api/v1` as a versioned base URL.

## Authentication

```bash
Authorization: Bearer <token>
```

The auth layer, the anonymous allow-list (`/ws`, health, public catalog) and
the rate-limit classes are applied in
`botserver/src/main_module/server/run_axum_server.rs` and
`botserver/src/main_module/routes/security_setup.rs`. Endpoints that need no
token are listed in the generated reference with `public`.

## Endpoint families

| Family | Prefix | Example routes |
|---|---|---|
| Users / profile | `/api/user/*`, `/api/users/*` | `GET /api/users` |
| Admin | `/api/admin/*` | `GET /api/admin/dashboard/stats` |
| Drive files | `/api/files/*` | `GET /api/files/list`, `POST /api/files/write` |
| Apps / catalog | `/api/apps/*` | `GET /api/apps/catalog` |
| CRM / people | `/api/crm/*`, `/api/contacts/*` | see generated reference |
| Products / billing | `/api/products/*`, `/api/billing/*` | see generated reference |
| Analytics | `/api/analytics/*` | `GET /api/analytics/usage` |
| Office events | `/api/events/*`, `/ws/events*` | `POST /api/events/publish`, `GET /ws/events/stream` |
| Approvals (issue #2501) | `/api/approvals/*` | `POST /api/approvals`, `POST /api/approvals/:id/decide` — see section below |

`/chat/*` is **not** a REST prefix: chat runs over the WebSocket pipeline at
`/ws` (see [WebSocket protocol](../07-user-interface/ws-protocol.md)). The only
`/api/chat*` routes are the history/session helpers in
`botserver/src/main_module/routes/chat_handlers.rs`.

The full, code-derived table is [generated](./generated/README.md).

## Response shapes are per family

There is **no universal envelope**. The claim `{"success", "data", "error"}`
for every endpoint is false. What the code actually returns:

| Family | Shape | Source |
|---|---|---|
| Typed handlers (most crates) | the resource JSON itself, e.g. a product object or an array | `botserver/crates/*/src/**/handlers.rs` |
| Handlers that report an outcome | `{"success": true, …}` plus family-specific keys | `botserver/crates/botanalytics/src/handlers.rs` |
| HTMX fragments (`/api/ui/*`) | HTML, not JSON | `botserver/crates/*/src/*_ui.rs` |
| Errors (all families) | `SafeErrorResponse` | `botserver/crates/botsecurity-core/src/error_sanitizer/mod.rs` |

Error body:

```json
{
  "error": "internal_error",
  "message": "…",
  "code": "…",
  "request_id": "…"
}
```

`code`, `request_id` and `details` are omitted when absent
(`skip_serializing_if`). Handlers must build this body with
`log_and_sanitize` / `log_and_sanitize_json` — never with `e.to_string()`.

## Status codes

| Code | Meaning in this server |
|---|---|
| 200 | success (JSON or HTML fragment) |
| 400 | validation failure |
| 401 | missing/invalid bearer token |
| 403 | RBAC role too low (`resolve_user_role`) |
| 404 | unknown route, or a feature not compiled into this build |
| 429 | rate limit (`governor`; general 100 req/s, auth 10 req/s, API 50 req/s) |
| 500 | sanitized internal error |

## Office event stream (#2503)

The Office event surface has two transports. Both are mounted in
`botserver/src/main_module/routes/feature_merge_platform/commerce.rs`.

### JSON endpoints (bearer token required)

These three routes are wrapped in `require_auth` middleware
(`commerce.rs:276-284`) and admitted by the default-deny RBAC catalogue
(`botserver/crates/botsecurity-auth/src/rbac_middleware/routes_user/app_routes.rs`).
A caller without a verified session receives 401; the catalogue grants these
routes to any authenticated caller, so no extra role is required.

| Method | Path | Purpose | Source |
|---|---|---|---|
| GET | `/api/events/dead-letters` | Privacy-safe dead-letter-queue summary for the admin UI | `commerce.rs:97-100` |
| POST | `/api/events/replay` | Exactly-once replay by idempotency key; body `{"idempotency_key": "..."}` | `commerce.rs:102-132` |
| POST | `/api/events/publish` | Emit one metadata-only event. Accepts `event_type`, optional `doc_id`, `workspace_id`, `actor_id` and `webhook_url`; clients never supply content or an event id | `commerce.rs:134-169` |

`POST /api/events/replay` answers 200 with the delivery receipt (including a
`deduped` flag), 404 for an unknown key, 409 when the subscription is gone and
a sanitized 502 when delivery fails. `POST /api/events/publish` answers 400 for
an unknown or invalid event type.

### Push streams (token in query parameter or cookie)

`EventSource` and `WebSocket` upgrades cannot set an `Authorization` header, so
both streams verify the credential inside the handler: the `?access_token=`
query parameter or the `gb-access-token` cookie, resolved by
`token_authenticated` (`commerce.rs:61-65`). A missing or invalid token answers
401 `authentication_required`. At the transport layer they are declared as
capabilities in
`botserver/crates/botsecurity-auth/src/auth_api/public_routes.rs:178-179`, so
the request reaches that check; the JSON API above is never anonymous.

| Method | Path | Transport | Source |
|---|---|---|---|
| GET | `/ws/events/stream` | Server-Sent Events (`event: native`), supports `?types=` filtering | `commerce.rs:171-222` |
| GET | `/ws/events` | WebSocket, same `?types=` filtering and token rules | `commerce.rs:224-241` |

The `sheet.exported` wire name is accepted as a singular alias for the
taxonomy's `sheets.exported` event (`commerce.rs:23-31`).

### Operator UAT checklist (#2503)

Use a local dev server and a **dev-only** token held in a shell variable.
`DEV_BEARER_TOKEN` below is a placeholder — supply your own dev credential in
the shell; never paste a production token into a terminal, shell history or
this book.

```bash
BASE="http://localhost:8080"   # dev API port
# DEV_BEARER_TOKEN is set in the dev shell, e.g. export DEV_BEARER_TOKEN='...'
```

1. **Dead letters (list).** `GET /api/events/dead-letters` → `200`, JSON array.
   Each entry has `idempotency_key` but **no** `body`, `payload` or `content`
   (privacy-safe summary). No/invalid token → `401`.
   ```bash
   curl -sS "$BASE/api/events/dead-letters" \
     -H "Authorization: Bearer ${DEV_BEARER_TOKEN}"
   ```

2. **Replay (exactly once).** Take an `idempotency_key` from step 1 and POST it.
   First call → `200`, `deduped:false`; identical repeat → `200`, `deduped:true`
   with the same `delivered_at`. An unknown key → `404`.
   ```bash
   curl -sS "$BASE/api/events/replay" \
     -H "Authorization: Bearer ${DEV_BEARER_TOKEN}" \
     -H 'Content-Type: application/json' \
     -d '{"idempotency_key":"<KEY_FROM_STEP_1>"}'
   ```

3. **Publish (metadata only).** `POST /api/events/publish` → `200` receipt with
   `event_id` and `dead_lettered`; unknown/invalid `event_type` → `400`. Clients
   never send content or an event id.
   ```bash
   curl -sS "$BASE/api/events/publish" \
     -H "Authorization: Bearer ${DEV_BEARER_TOKEN}" \
     -H 'Content-Type: application/json' \
     -d '{"event_type":"sheets.exported","doc_id":"doc-uat","workspace_id":"ws-uat","actor_id":"usr-uat"}'
   ```

4. **WebSocket push.** A WebSocket client cannot set an Authorization header,
   so pass the token as `?access_token=`. No token →
   `401 authentication_required`. With Python's `websocket-client`:
   ```python
   import os, websocket
   token = os.environ["DEV_BEARER_TOKEN"]
   ws = websocket.create_connection(
       f"ws://localhost:8080/ws/events?access_token={token}&types=sheets.exported"
   )
   print(ws.recv())  # native JSON envelope after a matching publish
   ws.close()
   ```
   Repeat the publish from step 3 while the socket is open to observe a pushed
   envelope; `?types=` filters the stream to the named event types.

## Approvals and e-signature API (issue #2501)

This is the hand-maintained surface for the typed approvals workflow. The
generated [REST reference](./generated/README.md) does **not** list these
routes: it derives the `consent` family from `botconsent/src/api.rs` only
(compare `generated/api-consent.md`, 9 routes), while the approvals router is a
separate module merged into it. Everything below is traced to source; no
runtime or browser evidence was collected for this documentation pass.

**Mount and gate.** The routes are declared in
`botserver/crates/botconsent/src/approvals/api.rs:97-110`, merged by
`botconsent/src/api.rs:159-160` into `configure_routes()`, and mounted on the API
router under `#[cfg(feature = "consent")]` in
`botserver/src/main_module/routes/feature_merge_office/ai_os.rs:79-98`. The
`consent` feature is part of `all-apps`, which is the default build
(`botserver/Cargo.toml:17,125,240`). Every handler calls `require_consent_user`
(`botconsent/src/api.rs:134-147`): a JWT bearer token or an opaque suite session
token is accepted, a missing one returns 401.

**Routes** (all under `http://localhost:8080`; `:id` is a UUID):

| Method | Path | Purpose | Source |
|---|---|---|---|
| POST | `/api/approvals` | Create a draft bound to `content`'s SHA-256 (`tenant?`, `doc_id`, `doc_class?`, `title?`, `owner`, `content`) | `approvals/api.rs:35-46,112-133` |
| GET | `/api/approvals` | List all, newest first; `?actor=` filters to that person's pending queue after delegations | `approvals/api.rs:135-148`, `approvals/store.rs:79-102` |
| GET | `/api/approvals/:id` | Read one record | `approvals/api.rs:150-161` |
| POST | `/api/approvals/:id/route` | Set the ordered `steps`, optional `message` and `due_at`; only writable before publish | `approvals/api.rs:48-55,163-175`, `approvals/engine.rs:53-83` |
| POST | `/api/approvals/:id/delegate` | `from` → `to`, optional `reason` and `expires_at` | `approvals/api.rs:57-65,177-195`, `approvals/types.rs:108-126` |
| POST | `/api/approvals/:id/decide` | `actor`, `decision` (`approve`/`reject`/`request_changes`), `reason?`, `comment?`, `version_hash?` | `approvals/api.rs:67-77,197-217`, `approvals/engine.rs:139-257` |
| POST | `/api/approvals/:id/sign` | `actor`, `comment?` — legal only while a `signature` step is active | `approvals/api.rs:79-84,219-231`, `approvals/store.rs:154-178` |
| POST | `/api/approvals/:id/update-content` | `content`; a changed hash invalidates an in-flight or signed record, a draft adopts it | `approvals/api.rs:86-89,233-244`, `approvals/engine.rs:335-352` |
| POST | `/api/approvals/:id/check-sla` | Read-only preview of due reminder/escalation events; optional `at` timestamp | `approvals/api.rs:91-95,246-262`, `approvals/engine.rs:259-307` |
| POST | `/api/approvals/sla-sweep` | Global sweep with optional `at`; marks each event delivered at most once per step | `approvals/api.rs:264-277`, `approvals/store.rs:193-218` |
| POST | `/api/approvals/:id/publish` | Signed → published; returns an immutable artefact and a `records://` entry | `approvals/api.rs:279-289`, `approvals/engine.rs:309-333` |
| GET | `/api/approvals/:id/verify` | Verify the HMAC chain of every decision in the trail | `approvals/api.rs:291-301`, `approvals/store.rs:226-250` |

Error mapping: 404 for an unknown id, 409 for `published approval is immutable`,
`already published`, `version_mismatch` or `invalidated`, 400 otherwise
(`approvals/api.rs:20-27`).

**Lifecycle and model.** Statuses are `draft`, `in_review`,
`awaiting_signature`, `approved`, `signed`, `published` and `invalidated`
(`approvals/types.rs:7-41`). Rejection and request-for-changes return the record
to `draft`. A step declares `mode` (`parallel`/`sequential`), `kind`
(`review`/`approval`/`signature`), `approvers`, `groups`, `sla_secs`,
`escalate_to` and `quorum` (`types.rs:43-90`); parallel quorum defaults to all
approvers, sequential to one (`types.rs:92-106`). Active delegations replace the
`from` assignee with `to` (`types.rs:208-241`). Each decision carries actor,
time, version hash, comment, `prev_hash` and an HMAC `signature`
(`types.rs:147-163`). The signature is `t=<unix>,v1=<hex>` HMAC-SHA256
(`approvals/signing.rs:83-101`), not a PAdES or PKI document signature.

### Acceptance criteria — honest status

| Criterion (issue #2501) | Status | Code evidence |
|---|---|---|
| Two parallel reviewers → CFO → signature, full trail visible | **Partial**: the engine supports parallel, sequential and signature steps and records a trail; the shipped UI hardcodes exactly this three-step shape. No runtime verification. | `engine.rs:53-83,220-257`; `botui/ui/suite/approvals/modules/04_events.js:44-65`; `approvals.html:48-58` |
| SLA breach triggers a reminder and escalation once | **Partial**: events are computed once per step and returned by the API, but there is no delivery mechanism and no scheduler; the UI uses manual sweep/breach buttons. | `engine.rs:14-18,259-307`; `store.rs:193-218`; `04_events.js:151-168` |
| Rejection requires a reason and returns to draft | **Implemented in code**: empty reason is rejected with `reason_required`; status resets to `draft`. | `engine.rs:177-179,220-229` |
| Approval binds to a version hash; later edits invalidate visibly | **Implemented in code**: SHA-256 content hash, 409 `version_mismatch` on a stale hash, `Invalidated` status on a changed hash. | `signing.rs:15-20`; `engine.rs:160-164,335-352` |
| "Approve the Q3 deck" from chat with a confirmation step | **Not implemented**: no approval command in the API command catalog; the BASIC `human_approval` helpers only read a separate `ApprovalRequest` model and register no chat approve/sign action or confirmation. | `botserver/src/core/bot/api_catalog/` (only an AutoTask stats string); `botbasic_ai/src/keywords/human_approval/to_dynamic.rs:78-133`; `human_approval/mod.rs:5-48` |
| Publishing produces an immutable, signed artefact and a records entry | **Partial**: the artefact sets `immutable: true`, an HMAC chain hash and a `records://{tenant}/{doc_id}/{version_hash}` URI string; there is no PDF/PPTX export, Drive filing or records-table row. | `engine.rs:309-333`; `store.rs:1-6` |
| Document locked for edits or tracks changes during approval | **Not implemented**: a content edit invalidates the record instead of being blocked or tracked. | `engine.rs:335-352` |

**Operational caveats.**

- Records live in an in-memory `RwLock<HashMap<…>>`; there is no migration or
  durable store, so all approvals are lost on process restart
  (`approvals/store.rs:1-6,21-24`).
- The signing key is `GB_APPROVALS_SIGNING_KEY` when set, otherwise a random
  per-process UUID; without the env key, `verify` fails after a restart
  (`store.rs:26-33,226-250`).
- With the default-deny RBAC layer enabled
  (`botserver/src/main_module/server/server_layers.rs:80-85`), the catalogue
  grants `/api/consent/**` but has **no** `/api/approvals/**` entry
  (`botsecurity-auth/src/rbac_middleware/routes_user/app_routes.rs:59-62`).
  Static analysis therefore indicates 403 before the handler runs; this was
  not verified against a running server in this pass.
- The suite app lives at `botui/ui/suite/approvals/` and is registered in the
  client window-manager catalog
  (`botui/ui/suite/js/window-manager/01_registry.js:26-27`). Its route builder
  uses a fixed three-step chain with a 3600 s default SLA, `chief-of-staff` as
  escalation target and `due_at: null` — there is no due-date field and no
  per-document-class workflow definitions (`04_events.js:44-65,84-90`).

## Quick example

```bash
curl -s http://localhost:8080/api/product \
  -H "Authorization: Bearer $TOKEN"
```

## Chapter contents

- [Generated REST reference](./generated/README.md) — every route, from the code
- [Files API](./files-api.md) — `/api/files/*`
- [Users API](./users-api.md) — user management
- [Groups API](./groups-api.md) — groups and membership
- [Conversations API](./conversations-api.md) — chat sessions
- [Admin API](./admin-api.md) — administration
- [AI API](./ai-api.md) — LLM endpoints
- [MCP format](./mcp-format.md) and [tool definitions](./tool-definition.md)
- [Example integrations](./examples.md) — code samples

## See also

- [Network Contract](../02-architecture-packages/network-contract.md)
- [Permissions Matrix](../09-security/permissions-matrix.md)
- [WebSocket protocol](../07-user-interface/ws-protocol.md)
