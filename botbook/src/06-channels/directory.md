# Directory Services 🟡 BETA

The directory (identity) layer is the `botcoredirectory` crate plus the Zitadel
service it talks to. It owns users, groups, organisations and SCIM provisioning;
it does **not** own bot sessions or RBAC role resolution (those are
`botserver/src/core/session/` and `botserver/src/security/user_role.rs`).

Authority: `botserver/crates/botcoredirectory/src/`.

## Module map

| Path | Responsibility |
|---|---|
| `client/` | HTTP client for the Zitadel v2 API (sessions, users, grants) |
| `auth_routes/` | login/session routes the server mounts |
| `users/`, `groups/` | user and group operations |
| `organizations.rs` | organisation records and their Zitadel mapping |
| `scim/` | SCIM 2.0 provisioning endpoints |
| `provisioning.rs` | first-boot provisioning of the directory |
| `router.rs`, `api.rs` | route assembly and the JSON API surface |
| `ui_api.rs`, `ui_fragments/` | HTMX fragments for the admin directory pages |

## Protocols

| Protocol | Where |
|---|---|
| Zitadel v2 REST (sessions, users, grants) | `client/` |
| OIDC / OAuth | `botserver/src/core/oauth/` with `botserver/crates/botcoreoauth` |
| SCIM 2.0 | `scim/` |

## What the directory is not

- It is not the bot database: bots, sessions and messages live in PostgreSQL
  (`botschema`).
- It is not the permission model: a directory group only becomes a role through
  `rbac_user_groups` + `rbac_groups` (`resolve_user_role`), where a group name
  containing `admin` maps to the `admin` role.
- Login is served only on port 5000 (`login.*`); cloud on 4000 does not serve
  `/login` or `/signup`.

## Configuration

| Setting | Source |
|---|---|
| Directory base URL | Vault `secret/gbo/directory` (`DIRECTORY_URL`) |
| Admin credentials | Vault, never `.env` |
| SaaS JWT secret | persisted in `conf/system/directory_config.json` (`resolve_saas_jwt_secret`) so tokens survive restarts |

## Verification

```bash
curl -s http://localhost:8080/api/directory/groups -H "Authorization: Bearer $TOKEN"
# every /api/directory/* route is listed in the generated reference
cargo test -p botcoredirectory
```

## See also

- [Security Matrix](../09-security/security-matrix.md)
- [Network Contract](../02-architecture-packages/network-contract.md)
- [User Auth](../09-security/user-auth.md)
