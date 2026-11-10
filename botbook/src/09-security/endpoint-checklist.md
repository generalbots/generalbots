# Endpoint Security Checklist 🟡 BETA

A checklist to apply to every new route. Each item names the mechanism that
enforces it, so a reviewer can verify instead of trusting the description.

## Before the handler

| # | Requirement | Enforced by |
|---|---|---|
| 1 | Route is registered in a router that sits behind the auth layer | `botserver/src/main_module/server/run_axum_server.rs` |
| 2 | If it must be anonymous, the path is added explicitly | `add_anonymous_path` in `botserver/src/main_module/routes/security_setup.rs` (`/ws` is the only chat surface) |
| 3 | A rate-limit class is chosen (general 100 req/s, auth 10 req/s, API 50 req/s per token) | `governor` layers in the server module |
| 4 | State-changing verbs (POST/PUT/PATCH/DELETE) require CSRF protection unless the route is Bearer-authenticated (stateless) | server middleware |
| 5 | The route appears in the API command catalog if the LLM may call it | `botserver/src/core/bot/api_catalog/` |
| 6 | Admin-only capabilities are marked `admin_only` | same catalog; filtered by `resolve_user_role` |

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
