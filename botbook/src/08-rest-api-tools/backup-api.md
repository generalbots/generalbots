# Data Protection API 🟡 PREVIEW

> Descriptor-only contract surface on the OS edition. **No archive, backup, snapshot or factory reset is produced by this build.**

## Status: descriptors only, no implementation

There is no `POST /api/backup/create`, no backup scheduler, no restore job and
no archive download in this product. Those endpoints were never implemented and
are not available. What ships instead is the OS Settings data-protection surface
documented here: four endpoints that report honestly what this build can and
cannot do, and return a bounded JSON descriptor instead of performing a
destructive action.

The authority is `botserver/crates/botsettings/src/os_data_protection.rs`, with
the static status payload in `botserver/crates/botsettings/src/os_data/status.rs`
and the Drive descriptor adapter in
`botserver/crates/botsettings/src/os_data/snapshot.rs`. The status response is
static — it performs no host probing — so it cannot drift from the shipped
behavior. The same surface is summarized in
[Chapter 9: Security](../09-security/README.md).

| Capability | Reality in this build |
|---|---|
| Disk encryption | Not implemented: no LUKS mapping, no TPM sealing, no recovery-key export. User data at rest is not encrypted by General Bots. |
| Backup / restore | Not implemented. `schedule` is `none`, `last_snapshot` is `null`, `restore_available` is `false`. Retention counts are a planned policy, not a running scheduler. |
| Export | Produces a deterministic JSON **manifest descriptor** only. No archive is created (`archive_created: false`, `bytes_read: 0`) and no data leaves the device. |
| Snapshot | A bounded **job descriptor** only. No bytes are copied and no source data is deleted. Fails closed with `503` when no backend is registered. |
| Factory reset | Never succeeds. Requires an exact typed token and then returns `501`; no wipe-and-wizard backend ships and nothing is erased. |

## Base URL

```
/api/os/data-protection
```

The routes are mounted by `configure_settings_routes()` in
`botserver/crates/botsettings/src/lib.rs`.

## Authentication and gating

All four endpoints require an authenticated user and pass through the same gate
(`gate()` → `authorize()` in `botserver/crates/botsettings/src/os.rs`):

- shell mode must be active (the panel is unavailable outside shell mode);
- the user must be an admin, a super admin, or a local `desktop_local` owner;
- the request's real `Host` header must be loopback (`localhost`, `*.localhost`
  or a loopback IP). `X-Forwarded-Host` is ignored to prevent spoofing.

A failed gate returns `403`. Every action — including refused and unimplemented
attempts — writes one `os_data_protection` audit row.

## Endpoints

| Method | Path | Purpose | Success status |
|---|---|---|---|
| GET | `/api/os/data-protection` | Static status + embedded endpoint contracts | `200` |
| POST | `/api/os/data-protection/export` | Stage an archive-manifest descriptor | `200` |
| POST | `/api/os/data-protection/snapshots` | Stage a snapshot job descriptor | `202` (or `503`) |
| POST | `/api/os/data-protection/reset` | Typed-confirmation reset request | `501` (never succeeds) |

### Status — `GET /api/os/data-protection`

Returns the static payload. No host probing is performed, so the answer cannot
drift from the capability table above. Notable fields:

```json
{
  "status": "not_implemented",
  "shell_mode_only": true,
  "data_root": "/var/lib/gbdesk",
  "encryption": { "implemented": false, "luks_mapping": false, "tpm_sealed": false, "recovery_key_export": false },
  "backup": {
    "implemented": false, "enabled": false,
    "schedule": "none", "last_snapshot": null, "restore_available": false,
    "retention_policy": { "daily_keep": 7, "weekly_keep": 4, "monthly_keep": 12,
                          "unit": "snapshots", "note": "Planned policy only. No snapshots are created in this build." },
    "descriptor_adapter": "drive-archive"
  },
  "export": { "implemented": false, "enabled": false, "descriptor_available": true },
  "reset": { "implemented": false, "enabled": false, "confirmation_token": "RESET THIS PC" }
}
```

Each of `backup`, `export` and `reset` also carries a `contract` object with the
method, request shape, path and the exact `result` semantics documented below.
The UI reads `reset.confirmation_token` from this response rather than
hardcoding it.

### Export manifest descriptor — `POST /api/os/data-protection/export`

Request body: `{"scope": "full-archive"}`.

- `200` — accepted scope. Returns a deterministic descriptor, e.g.:

  ```json
  {
    "manifest_id": "<64-hex digest>",
    "status": "descriptor_only",
    "generated_at": "<RFC 3339>",
    "source_root": "/var/lib/gbdesk",
    "scope": "full-archive",
    "format": "tar.zst",
    "includes": ["chat history", "documents", "bot workspace", "settings"],
    "excludes": ["credentials", "session tokens", "encryption keys"],
    "bytes_read": 0,
    "destructive": false,
    "archive_created": false
  }
  ```

  It lists exactly what a future export backend *would* include and exclude. It
  contains no user data and performs no file access.

- `400` — any other scope, with `{"error": "unsupported_export_scope"}`.

### Snapshot descriptor — `POST /api/os/data-protection/snapshots`

Request body: `{"reason": "<string>"}`. A valid reason is non-empty after
trimming, at most 200 bytes and contains no control characters. The raw reason
is never echoed: only a SHA-256 digest of it is kept in the descriptor.

- `400` — invalid reason, with `{"error": "invalid_reason"}`.

- `202` — a snapshot backend is registered. This happens only when Drive is
  configured (`AppState.drive` is `Some`), which registers the `drive-archive`
  descriptor adapter. The adapter stages a bounded, non-destructive job
  descriptor:

  ```json
  {
    "status": "staged",
    "adapter": "drive-archive",
    "medium": "drive-archive",
    "job_id": "snap-<12-hex>",
    "location": "os-snapshots/<digest>/<YYYYMMDD>/<digest>.tar.zst",
    "source_root": "/var/lib/gbdesk",
    "retention": { "daily_keep": 7, "weekly_keep": 4, "monthly_keep": 12 },
    "manifest": { "status": "descriptor_only", "archive_created": false,
                  "bytes_copied": 0, "destructive": false },
    "destructive": false
  }
  ```

  No bytes are read or written, no Drive call is made, and no source data is
  deleted. `restore` and `pruning` are separate typed-confirmation actions that
  the OS image would supply; this build does not ship them.

- `503` — no backend is registered (Drive is not configured and no btrfs
  backend ships in this build). The request fails closed with:

  ```json
  {
    "error": "os_backend_unavailable",
    "status": "not_implemented",
    "contract": "/api/os/data-protection/snapshots",
    "adapters": ["btrfs", "drive-archive"]
  }
  ```

  Nothing is staged, copied or deleted. The `btrfs` adapter is named in the
  contract but is not registered in this build.

### Reset request — `POST /api/os/data-protection/reset`

Request body: `{"confirmation": "<token>"}`. The UI keeps the button disabled
until the token read from the live status response (`RESET THIS PC`) is typed
verbatim.

- `400` — token missing or not an exact match, with
  `{"error": "confirmation_mismatch"}`.
- `501` — exact token supplied. Returns:

  ```json
  {
    "error": "reset_backend_absent",
    "status": "not_implemented",
    "message": "Factory reset is disabled: no wipe-and-wizard backend ships in this build. Nothing was erased.",
    "contract": "/api/os/data-protection/reset"
  }
  ```

  No build ships a wipe backend, so this action never succeeds and nothing is
  erased.
