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
may turn a token into claims (issue #2123). Its callable surface:

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

**Rule: claims exist only as `VerifiedJwt`.** A verifying path never hands claims
around as a `serde_json::Value`, a `(sub, email)` tuple or a locally decoded
payload: the type is produced by the two `verify_*` functions and nowhere else.
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
shrink-only: the cloud SSO handler (`cloud_sso_handler/handlers.rs`) verifies
the HMAC itself in constant time before decoding, and six files still read a
payload without a signature (`botcloud` payment cards and vouchers, `botlearn`,
`botsecurity-auth` platform admin and the Zitadel request authenticator,
`ai_accounts_chatgpt`). The unverified readers each name the follow-up that must
remove them; the gate described in
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

### Writes never fall back to the default branch (#2128)

The default branch is a **read** convenience for suite sessions only. A write
handler must resolve the branch from the verified identity and answer `401`
when it cannot. `botproducts` used to call
`branch_from_jwt_pool(..).unwrap_or_else(|| get_bot_context(..))` in
`create_product` and `adjust_stock`, so an anonymous `POST /api/products/items`
silently inserted into the default branch — another tenant's catalogue. Writes
now resolve through `verified_branch`
(`botserver/crates/botproducts/src/write_scope.rs`), which has no default
parameter, and the whole `/api/products/*` namespace is absent from the
anonymous allowlist. The public storefront catalogue is the exact-path,
GET-only `/api/catalog/products` route.

### Object storage namespaces carry the verified owner (#2135)

Apps that store files instead of rows scope the object key, not a SQL column.
The Docs app (`botdocs`) used to build every key from the constant
`default-user`, so all accounts of a bot shared `users/default-user/docs` and
any signed-in user could list, read, edit, export and delete every other user's
documents.

Ownership now comes from the `DocUser` request extension that the host
middleware (`botserver/src/docs/user_middleware.rs`) derives from the verified
`AuthenticatedUser`:

| Profile | Owner segment |
|---------|---------------|
| directory / cloud user | the verified user id (UUID) |
| desktop-local (`no_security`) | a UUID derived from the installation (`botdocs::local_owner::install_owner_id`, same seed as the Sheet middleware; `GB_DESKTOP_LOCAL_USER_ID` overrides) |
| anonymous, service, bot | none — the handler answers `401` |

Key layout: `users/{owner}/docs/{doc-id}.html` (plus `.meta.json`, `.docx`,
`.docx.source.json`) inside the bot bucket, so the bucket is the tenant boundary
and the owner segment is the per-user boundary. There is no default and no
fallback: a Docs handler takes `user: DocUser` or it does not compile.

Pre-#2135 objects under `users/default-user/docs` depend on whether an owner can
be derived. A **single-user** install (`no_security`, desktop edition) has
exactly one possible author, so `botdocs::legacy::adopt_legacy_namespace` copies
those objects into the install owner's folder at boot and keeps the originals as
a backup. A **multi-user** deployment has no derivable owner — assigning the
prefix to any account would hand one user's documents to another — so the
objects stay orphaned and `botdocs::legacy::audit_legacy_namespace` logs their
count at boot for an operator to move into `users/{user-id}/docs`.

### Slides and Paper persist the verified owner (#2137)

The office apps had the same shared-namespace defect in two shapes. Slides
built every deck key from the constant `default-user`
(`botslides::storage::get_current_user_id`), so all callers listed, opened,
overwrote and deleted one collection. Paper resolved a caller from a
client-supplied `x-session-id` header or cookie and, when there was none,
**found or created** a shared `anonymous@local` account — every
unauthenticated caller wrote into that one store.

Both crates now take a verified owner extension from the host, exactly like
Docs:

| App | Extension | Host bridge | Key layout |
|-----|-----------|-------------|------------|
| Slides | `botslides::auth::SlidesUser` | `botserver/src/slides/user_middleware.rs` | `users/{owner}/presentations/{id}.json` (+ `.pptx`) |
| Paper | `botpaper::auth::PaperUser` | `botserver/src/paper/user_middleware.rs` | `users/{owner}/papers/{current,named}/…` |

Owner resolution is shared (`botserver/src/security/office_owner.rs`):

| Profile | Owner segment |
|---------|---------------|
| directory / cloud user | the verified user id (UUID) |
| desktop-local (`no_security`) | `local-{install-uuid}` generated once per install and persisted as `office_local_owner` in `conf/system/directory_config.json` |
| anonymous, service, bot | none — the handler answers `401` |

The extractor is the only way a handler reads the owner: a Slides or Paper
handler takes `user: SlidesUser` / `user: PaperUser`, so a request without a
verified identity never reaches storage. `SlidesUser::verified` /
`PaperUser::verified` additionally refuse the legacy shared ids, so the old
prefix can never be re-created.

Behavior changes that come with it:

- Paper no longer reads `x-session-id` at all; the platform middleware resolves
  the cookie or `Authorization` credential. A forged `x-session-id` header
  authenticates nothing.
- `delete` verifies the object exists in the caller's own prefix and answers
  `404` otherwise (it used to report success for any id). `get` answers `404`
  for a document/deck that is not under the caller's prefix.
- The stored `owner_id` is checked on every load: a deck copied into the wrong
  prefix is not served.
- Pre-#2137 objects (`users/default-user/presentations`,
  `users/anonymous@local/papers`, and Paper's old per-email prefixes) are
  **quarantined in place**: still on Drive, no longer reachable by any caller.
  Move them into the install owner's prefix (or `legacy/`) with `mc` before
  deleting them.

Tests: `botslides/tests/owner_isolation.rs` and
`botpaper/tests/owner_isolation.rs` cover list/load/delete isolation, the
mismatched-owner read, the `401` on create without identity, and the verified
owner persisted on create.

### Plans are owned and the collaboration socket is authenticated (#2136)

`botplan` kept every plan in one process-wide map keyed by a **client-supplied**
id, and `GET /api/plan/:plan_id` created the plan when the id was unknown — with
`owner_id: "anonymous"`. The REST routes sat behind the global auth middleware
but no handler read the caller, so any signed-in user who learned an id could
read, edit and delete that plan; `/ws/plan/:plan_id` matched the anonymous `/ws`
allowlist and invented a guest identity when no token was present.

The crate now has a single authorization module,
`botserver/crates/botplan/src/auth.rs`:

| Concern | Rule |
|---------|------|
| Identity | `botsecurity_core::verified_jwt::verify_saas_jwt` (HS256 pinned, constant-time HMAC, `exp`/`nbf`); opaque `gb_*` suite tokens resolve through the session cache, a malformed JWT never falls back to it |
| Plan ids | minted by `POST /api/plan` (`Uuid::new_v4`); `GET` never creates |
| Read / write | owner **or** member (`Plan::members`), otherwise `404` |
| Membership | owner only, otherwise `403` |
| Tenant | `Plan::branch_id` must equal the verified `branch_id` claim; a plan with no branch is reachable only by a branchless (suite session) caller |
| Legacy `owner_id: "anonymous"` | owns nothing — no caller inherits it |
| WebSocket | the upgrade is refused with `401` (`x-auth-reason`) for a missing/invalid token and `404` for a plan the caller cannot see; a revocation between handshake and subscription closes the socket with `4403` |

`owner_id`, `org_id` and `branch_id` are persisted with each snapshot
(`migrations/6.8.18-plan-ownership` projects them out of the JSONB payload into
columns). `POST /api/plan` is the only creation path, so the client no longer
chooses a plan id; the Plan app (`botui/ui/suite/plan/modules/`) creates the
server plan on boot and sends the suite token on every call.

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