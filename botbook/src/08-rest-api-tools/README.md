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
