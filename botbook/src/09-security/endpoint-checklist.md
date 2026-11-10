# Endpoint Security Checklist 🟡 BETA

A checklist to apply to every new route. Each item names the mechanism that
enforces it, so a reviewer can verify instead of trusting the description.

## Before the handler

| # | Requirement | Enforced by |
|---|---|---|
| 1 | Route is registered in a router that sits behind the auth layer | `botserver/src/main_module/server/run_axum_server.rs` |
| 2 | If it must be anonymous, the exact method+path is declared explicitly | `platform_public_routes()` in `botserver/crates/botsecurity-auth/src/auth_api/public_routes.rs` — an exact `(method, path)`, a `:param` or a declared `/*`; never a bare prefix. `platform_private_routes()` removes a route that a broader public pattern would otherwise match (#2133) |
| 3 | A rate-limit class is chosen (general 100 req/s, auth 10 req/s, API 50 req/s per token) | `governor` layers in the server module |
| 4 | State-changing verbs (POST/PUT/PATCH/DELETE) require CSRF protection unless the route is Bearer-authenticated (stateless) | server middleware |
| 5 | The route appears in the API command catalog if the LLM may call it | `botserver/src/core/bot/api_catalog/` |
| 6 | Admin-only capabilities are marked `admin_only` | same catalog; filtered by `resolve_user_role` |

### Identity on the internal hop

`api.exec` — the executor behind the API command catalog the LLM can call —
performs a real HTTP request back into the running server, so the call passes
through the same auth, RBAC and CSRF layers as a browser request. Two rules make
that hop attributable (#2122):

| Rule | Implementation |
|---|---|
| `X-User-ID` is **not** a credential | the auth extractor only logs it, truncated, as a correlation id (`botsecurity-auth/src/auth_api/auth_api_utils/extracted.rs`) |
| the hop proves itself with a signed assertion | `botserver/src/core/bot/api_exec_assertion.rs` mints an HS256 JWT and sends it as `Authorization: Bearer <assertion>` |

The assertion is signed with the same `saas_jwt_secret` the login flow mints
with, lives 120 seconds, and carries `sub` (session user), `email` (tenant and
drive scoping), `bot_id` when the call is bot-scoped, `gb_aud`/`scope`
(`"api-exec"`, a marker of origin — no verifier enforces an audience today),
plus `iat` and `exp`. The middleware verifies signature and expiry before any
handler runs, so the assertion cannot be forged and cannot be replayed after it
expires.

Two consequences worth knowing when writing a route:

- CSRF is skipped only for the **verified** internal service identity (the auth
  middleware inserts an `AuthenticatedUser` with `Role::Service` and username
  `service:internal` after matching the configured internal token). A bare
  `X-User-ID` or `X-Internal-Token` string never exempts a request.
- The per-user quota is keyed on the peer address
  (`rate_limit_middleware` in `botsecurity-auth/src/rate_limiter.rs`), not on a
  request header, so a caller cannot buy a fresh quota by inventing a user id.

### Identity on the generic CRUD surface

`/api/db/{table}` is table-driven CRUD: the caller names a table, and the
handlers evaluate the table's `read_roles`/`write_roles` and per-field
restrictions from `dynamic_table_definitions`. Those rules were applied to a
`UserRoles` built from the `X-User-Id` and `X-User-Roles` headers, so a caller
could claim any identity and grant itself `admin` (#2122). Identity now comes
from the `AuthenticatedUser` extension or from a Bearer credential verified by
`botsecurity_core::verified_jwt`, and the role set is the verified principal's
roles plus the caller's active `rbac_groups` rows
(`botserver/crates/botbasic_data/src/keywords/db_api/verified_caller.rs`). A
request with neither credential is answered `401` before the table is touched —
there is no default identity and no default role. The `api.exec` loopback hop
reaches the same handlers with its signed assertion, not with headers.

## Inside the handler

| # | Requirement | Enforced by |
|---|---|---|
| 7 | No `unwrap`/`expect`/`panic!` | `cargo clippy` in review; error paths return `Result` |
| 8 | Tenant scope comes from the session, never from a query parameter | `bottest/tests/integration/tenant_isolation.rs` guards drive/CRM/products scoping |
| 9 | Role check uses `resolve_user_role` (fails closed to `user`) | `botserver/src/security/user_role.rs` |
| 10 | SQL identifiers go through `sanitize_identifier` / `validate_table_name` | `botserver/crates/botsecurity-core/src/sql_guard.rs` |
| 11 | External commands go through `SafeCommand`, never `Command::new` | `botserver/crates/botsecurity-core/src/command_guard.rs` |
| 12 | Errors are returned as `SafeErrorResponse` built by `log_and_sanitize` | `botserver/crates/botsecurity-core/src/error_sanitizer/` |
| 13 | File paths stay inside the bot's work dir or Drive prefix | drive bridge / `botdrive` user scoping |

## Responses

| # | Requirement |
|---|---|
| 14 | The error body is `{error, message, code?, request_id?, details?}` — never a raw `e.to_string()` |
| 15 | A 404 means "unknown route" or "feature not compiled in this build", never "forbidden" |
| 16 | HTML fragments are served only under `/api/ui/*` and never with an auth bypass |

## Verification commands

```bash
grep -rn "Command::new(" botserver/src --include=*.rs     # must be empty outside tests
grep -rn "\.unwrap()" botserver/src/main_module/routes --include=*.rs | head
grep -rn "e.to_string()" botserver/src/main_module/routes --include=*.rs | head
```

## See also

- [Security Matrix Reference](./security-matrix.md)
- [Permissions Matrix](./permissions-matrix.md)
- [Generated REST reference](../08-rest-api-tools/generated/README.md)
