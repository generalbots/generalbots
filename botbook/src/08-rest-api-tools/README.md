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
`botserver/crates/botsecurity-auth/src/auth_api/public_routes.rs:173-183`, so
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
