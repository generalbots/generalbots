# Multi-Tenant Data Isolation 🟢 STABLE

This guide documents the tenant-scoping rules that must be applied to every
data-owning app crate. It was introduced by issue #734 (cross-branch data
exposure) and is a hard requirement for any new feature that persists rows.

## Tenancy Model

There are three scoping levels. Every table that stores per-tenant business
data carries a `branch_id` column:

| Level | Table column | Description |
|-------|--------------|-------------|
| Organization (`.gborg`) | `organization_id` / `org_id` | The top-level tenant that owns the workspace |
| Workspace (branch) | `branch_id` | The workspace branch; **this is the tenant boundary** |
| Bot | `bot_id` | Legacy column; kept for compatibility, NOT the tenant boundary |

The authoritative tenant is the server-minted JWT `branch_id` claim. It is set
at login/signup from the verified user→org binding and must never be derived
from client input (headers, query params, or JSON bodies).

## The Canonical Resolver

`botserver/crates/botsecurity-core/src/tenant.rs` provides the shared,
dependency-light resolver used by all DataOwners.

```rust
pub fn branch_from_claims(headers: &HeaderMap) -> Option<Uuid> // branch (workspace)
pub fn org_from_claims(headers: &HeaderMap) -> Option<Uuid>    // owning tenant org
pub fn email_from_claims(headers: &HeaderMap) -> Option<String>
```

`botcore::shared` re-exports it as `botcore::shared::tenant`, which is the path
used by crates that already depend on `botcore`. Crates that do not depend on
`botcore` (e.g. `botinventory`, `botgl`, `botsearch`, `bottasks`) must depend on
`botsecurity-core` directly and call `botsecurity_core::tenant::...`.

### The resolver verifies the signature before reading a claim

`branch_from_claims` is both a claim reader **and** the verifier. It extracts the
token (`Bearer` or the `Basic` password field), then hands it to
`botsecurity_core::verified_jwt::verify_saas_jwt`, the same function the
`SaasJwtAuthProvider` uses on the authentication path. That function:

1. requires exactly three dot-separated segments;
2. requires the header to declare `HS256` — `none` (any casing), a missing
   algorithm and asymmetric algorithms (`RS256`, `ES256`, `PS512`, `EdDSA`) are
   refused, so algorithm confusion cannot downgrade the check;
3. recomputes the HMAC-SHA256 over `header.payload` with the persisted
   `saas_jwt_secret` and compares it with `Mac::verify_slice`, which is
   constant time;
4. requires a non-empty `sub` and an `exp` in the future, and rejects an `nbf`
   that is more than 60 seconds ahead (clock-skew leeway).

Only then does `VerifiedJwt` exist, and only `VerifiedJwt` exposes `branch_id`,
`org_id`, `sub` and `email`.

```text
Authorization: Bearer <token>
        │
        ├─ verify_hs256_jwt(token, saas_jwt_secret)   ← alg + HMAC + exp
        │      ├─ error → None  (no claim is ever returned)
        │      └─ VerifiedJwt
        └─ verified.branch_id → Option<Uuid>
```

Consequences:

- A forged, tampered, `alg=none`, wrong-key or expired token yields `None` for
  every `*_from_claims` helper, and the handler falls back to its crate default
  (`Uuid::nil()`) — it never sees the payload.
- The `saas_jwt_secret` is the secret the login path mints with
  (`directory_config.json` → `SAAS_JWT_SECRET` → `JWT_SECRET` → development
  default), so mint and verify cannot drift.
- Verification happens *inside* the resolver. A route still has to be mounted in
  the authenticated scope to know *who* the caller is; the resolver only
  guarantees that the tenant claim it returns was signed by this server.

### One verifier for the whole tree

`botserver/crates/botsecurity-core/src/verified_jwt.rs` is the only module that
may turn a token into claims (issue #2123). It exposes four items:

| Item | Purpose |
|---|---|
| `verify_hs256_jwt(token, secret)` | verifies against an explicit secret |
| `verify_saas_jwt(token)` | verifies against the process secret, resolved once (`OnceLock`) |
| `bearer_token(headers)` | extracts `Bearer <token>` (first entry if a proxy appended more) |
| `resolve_saas_jwt_secret()` | the secret chain: `directory_config.json` → `saas_jwt_secret` → `SAAS_JWT_SECRET` → `JWT_SECRET` → development default |

Every refusal is a distinct `JwtVerifyError`, and none of them falls back to the
payload:

| Refusal | Condition |
|---|---|
| `Malformed` | not exactly three dot-separated segments |
| `UnsupportedAlgorithm` | header `alg` is anything but `HS256` — `none`, `RS256`, `ES256` and `EdDSA` included |
| `InvalidSignature` | HMAC-SHA256 over `header.payload` does not match (`Mac::verify_slice`, constant time) |
| `MissingSecret` | empty signing secret |
| `MissingSubject` | no non-empty `sub` (legacy `user_id` accepted) |
| `MissingExpiry` / `Expired` | no `exp`, or `exp` in the past |
| `NotYetValid` | `nbf` more than 60 s ahead (clock-skew leeway) |

**Rule: claims exist only as `VerifiedJwt`.** No handler, crate or helper keeps
claims as a `serde_json::Value`, a `(sub, email)` tuple or a locally decoded
payload; the type is produced by the two `verify_*` functions and nowhere else.
Role claims (`role`/`roles`) are read only after the signature checks pass, so
an authorization decision that reads `VerifiedJwt::roles` cannot be driven by a
payload the caller wrote.

Consumers, all through the same module:

| Consumer | Uses it for |
|---|---|
| `botsecurity-core/src/tenant.rs` | `branch_id` / `org_id` / `sub` / `email` for every tenant-scoped handler |
| `botsecurity-core/src/scope.rs` | `branch_from_jwt` — claim first, CRM contact second |
| `botsecurity-auth/src/saas_jwt_auth.rs` | `SaasJwtAuthProvider` on the authentication path |
| `botcloud/src/branch_scope.rs` | cloud branch resolution |
| `botemail/src/models.rs` | mail/imap caller identity |
| `botmarketplace/src/auth.rs` | publisher identity and the admin role |
| `botbrowserpolicy/src/jwt.rs` | task ownership and the admin policy gate |
| `botmemory/src/api.rs` | memory caller (`user_id`, `org_id`, `branch_id`) |
| `botslides/src/collaboration/ws.rs` | WebSocket collaboration identity |
| `botsettings/src/settings_api/saas_jwt_secret.rs` | settings scope |
| `botserver/src/main_module/directory_setup.rs` | persists the secret the others verify with |

Known exceptions live in `scripts/jwt_hardening_allowlist.txt` and are
shrink-only: `cloud_sso_handler.rs` verifies the HMAC itself in constant time
before decoding, and six files still read a payload without a signature
(`botcloud` payment cards and vouchers, `botlearn`, `botsecurity-auth`
platform admin and the Zitadel request authenticator, `ai_accounts_chatgpt`).
Each carries the follow-up that must remove it; the gate described in
[Security Features](./security-features.md#mechanical-gates-in-ci) fails on any
new occurrence.

### Fallback rule

When no verified JWT claim is present (anonymous/system callers), the resolver
returns `None` and callers substitute `Uuid::nil()`. Requests must still include
`branch_id = ?` in every query; the nil value then matches the "legacy global"
rows, so a tenant can never see another tenant's rows.

## Mandatory Query Scoping (ALL CRUD paths)

For every table that owns tenant data:

1. `storage.rs` / schema sync MUST create the `branch_id` column:
   ```sql
   branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
   ```
   (Or use the consolidated migration `6.5.50-multitenant-scope` for existing
   databases.)
2. Every handler calls the resolver once and constrains all queries:
   ```rust
   let branch = botcore::shared::tenant::branch_from_claims(&headers)
       .unwrap_or_else(Uuid::nil);
   ```
3. `SELECT` → `WHERE branch_id = $1` (or `AND branch_id = $N`).
4. `INSERT` → include `branch_id` in the column list + bind.
5. `UPDATE` / `DELETE` → add `branch_id = $N` to the predicate; if 0 rows are
   affected the resource is either not yours or does not exist → `NOT_FOUND`.

A handler that reads *or* writes tenant data MUST take a
`headers: axum::http::HeaderMap` extractor as its first parameter so the
resolver has access to the JWT.

## Object-Level Authorization (IDOR)

Handlers that address a single resource by primary key (`get`/`update`/`delete`
by `:id`) must add the tenant predicate, not just the PK:

```sql
SELECT ... WHERE products.id = $1 AND products.branch_id = $2
UPDATE products SET ... WHERE products.id = $1 AND products.branch_id = $2
```

This prevents a caller from mutating another tenant's row whose id they guess
(broken object level authorization).

## Never trust client-supplied tenant

- The `X-Organization-Id` header and `org_id`/`branch_id` query/body parameters
  are NOT authoritative. The tenant comes from the JWT claim only.
- The `organization_context_middleware` (botcore) already rejects requests
  whose header org contradicts the token — the header may only confirm scope.

## Suite-mode default branch

Suite (desktop/cookie auth) requests without a JWT fall back to the default
branch resolved by `resolve_default_branch` in
`botserver/src/main_module/routes/feature_routers.rs`:

```sql
SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE LIMIT 1
```

When no bot is flagged default, `Uuid::nil()` is used (legacy global scope).

## Covered Apps

The following crates are branch-scoped (all queries carry `branch_id`):

- `bothr` (hr), `bottax`, `botvision`, `boterp`, `botintegrations`
- `botsales`, `botminutes`, `bottemplates`, `botitsm`, `botpos`
- `bothandoff`, `botkyc`, `bottimeclock`, `botbanking`
- `botm365`, `botinventory`, `botgl`, `botsearch` (org-scoped)
- `bottasks` (IDOR), `botautotask`, `botresearch`
- `botproducts` and `botcontacts` IDOR (get/update/delete by id) were already
  branch-filtered.
- `misc_handlers` contacts search was already branch-scoped.

Migration: `botserver/migrations/6.5.50-multitenant-scope/up.sql` adds the
`branch_id` column + index to 56 tables. It is idempotent
(`ADD COLUMN IF NOT EXISTS`).

## Adding a New Data App: Checklist

1. Add `branch_id UUID NOT NULL DEFAULT '<nil uuid>'` to every new table.
2. Depend on `botcore` (existing) or add `botsecurity-core = { workspace = true }`.
3. Import the resolver (`branch_from_claims` + fallback to `Uuid::nil`).
4. Take `headers: HeaderMap` in every handler doing SQL.
5. Constrain every SELECT/INSERT/UPDATE/DELETE by `branch_id`.
6. IDOR: add the branch predicate to every by-`id` get/update/delete.
7. Run the audit grep: no `FROM {table}` without `branch_id` in the crate.