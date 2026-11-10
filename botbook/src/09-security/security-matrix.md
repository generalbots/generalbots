# Security Matrix Reference 🟡 BETA

How a request is authorised, which tables hold the decision, and where each
check lives in the code. Tables are the real `table!` names in
`botschema/src/core/rbac_roles_tables.rs`.

## RBAC tables

| Table | Holds |
|---|---|
| `rbac_roles` | role definitions |
| `rbac_groups` | groups (`name`, `is_active`) |
| `rbac_permissions` | permission definitions |
| `rbac_role_permissions` | role → permission |
| `rbac_user_roles` | user → role |
| `rbac_user_groups` | user → group |
| `rbac_group_roles` | group → role |
| `rbac_api_permissions` | endpoint prefix → permission (created by migration `6.5.41-rbac-api-permissions`; not modelled in `botschema`) |

## How a role is resolved

`botserver/src/security/user_role.rs` (`resolve_user_role`):

1. Load the user's groups from `rbac_user_groups`.
2. No groups → role `user`.
3. Load the active groups' names from `rbac_groups`; any name containing
   `admin` (case-insensitive) → role `admin`.
4. Otherwise role `user`.

The function fails closed: a pool error or a load error returns `user`, never
`admin`. The chat session exposes the result to BASIC as `role`/`ROLE`, which is
why scripts gate admin tools with `IF role = "admin" THEN …`.

## Where the checks happen

| Check | Location |
|---|---|
| Bearer-token authentication | `botserver/src/main_module/server/run_axum_server.rs` |
| Anonymous allow-list (`/ws`, health, public catalog) | `botserver/src/main_module/routes/security_setup.rs` |
| Role resolution per request | `botserver/src/security/rbac_role_resolver.rs` |
| Admin-only API command filtering | `botserver/src/core/bot/api_catalog/` (`admin_only` entries filtered by `resolve_user_role`) |
| Admin-only chat tools and suggestions | `start.bas` gate on `role`, plus session-scoped tool execution |
| Tenant scoping (drive, CRM, products) | `bottest/tests/integration/tenant_isolation.rs` regression suite |

## Rate limits

Applied with `governor` in the server layer:

| Class | Limit |
|---|---|
| General | 100 req/s (global) |
| Auth endpoints | 10 req/s |
| API per token | 50 req/s |
| WebSocket messages | 10 msgs/s |

## Data-handling rules that are enforced in code

| Rule | Mechanism |
|---|---|
| No internal detail in an HTTP error | `log_and_sanitize` → `SafeErrorResponse` (`botserver/crates/botsecurity-core/src/error_sanitizer/`) |
| No shell string interpolation | `SafeCommand` (`botserver/crates/botsecurity-core/src/command_guard.rs`) |
| No identifier concatenation into SQL | `sql_guard` (`sanitize_identifier`, `validate_table_name`) — `botserver/crates/botsecurity-core/src/sql_guard.rs` |
| Passwords | Argon2 hashing; never stored or logged in clear |
| Secrets | Vault only; `.env` may carry `VAULT_*` alone |

## Verification

```bash
cargo test -p bottest --features integration tenant_isolation
```

## See also

- [Permissions Matrix](./permissions-matrix.md)
- [RBAC Design](./rbac-design.md)
- [Endpoint Security Checklist](./endpoint-checklist.md)
