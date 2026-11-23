# Chapter 9: Security 🟡 BETA

This chapter covers advanced security topics for General Bots.

## Overview

General Bots implements a comprehensive security model designed for enterprise deployments:

- **Multi-tenant Architecture**: Support for multiple organizations with complete data isolation
- **Role-Based Access Control (RBAC)**: Fine-grained permissions at every level
- **Knowledge Base Security**: Folder-level permissions with Qdrant vector search integration
- **SOC 2 Type II Compliance**: Enterprise-grade security controls and audit logging

## Security bulletin program

The authoritative advisory channel is the [security bulletin](https://generalbots.org/security/),
not the community forum. For the public track record and recognition:

- [Status and incident history](https://generalbots.org/security/status.html) — resolved advisories with dates, severity and affected versions.
- [Hall of fame](https://generalbots.org/security/hall-of-fame.html) — opt-in credit for reporters; no bug bounty, reward budget 0.

Process, roles, approval authority and escalation to CTIR Gov / CERT.br are in
[Incident Response and Publication Authority](./incident-response.md).

## Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Organization Layer                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    Bot Layer                         │    │
│  │  ┌─────────────────────────────────────────────┐    │    │
│  │  │              App Layer                       │    │    │
│  │  │  ┌─────────────────────────────────────┐    │    │    │
│  │  │  │        Resource Layer                │    │    │    │
│  │  │  │  (KB folders, files, data)          │    │    │    │
│  │  │  └─────────────────────────────────────┘    │    │    │
│  │  └─────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Organizations

Organizations are the top-level tenant in General Bots. Each organization has:

- Its own subscription and billing
- Isolated user base and permissions
- Separate bots and knowledge bases
- Independent quota management

Users can belong to multiple organizations and switch between them.

### Roles and Permissions

General Bots uses a role-based model with predefined roles:

| Role | Description |
|------|-------------|
| Global Admin | Full access to all resources |
| Billing Admin | Manage subscriptions and payments |
| User Admin | Manage users, groups, and role assignments |
| Bot Admin | Create and configure bots |
| KB Manager | Manage knowledge bases and permissions |
| App Developer | Create and publish apps (Forms, Sites, Projects) |
| Editor | Edit content and use apps |
| Viewer | Read-only access |

### Knowledge Base Security

KB folders can have individual permission settings:

- **Public**: Anyone can access
- **Authenticated**: Logged-in users only
- **Role-based**: Users with specific roles
- **Group-based**: Users in specific groups
- **User-based**: Named individual users

These permissions are enforced during vector search, ensuring users only see content they're authorized to access.

## In This Chapter

- [RBAC & Security Design](./rbac-design.md) - Complete RBAC architecture and security matrix
- [Role-Based Tool Access](./role-based-tools.md) - Restrict bot tools to admins via the `role` runtime variable
- [Organization Multi-Tenancy](./organizations.md) - Multi-organization support and switching
- [Multi-Tenant Data Isolation](./multi-tenant-isolation.md) - branch_id scoping rules for all data apps
- [SOC 2 Compliance](./soc2-compliance.md) - Enterprise compliance controls
- [Security Matrix Reference](./security-matrix.md) - Complete permission reference tables
- [Peer Network Privacy Model](../13-ai-os/peer-network.md) - What the opt-in P2P mesh carries, what helpers can observe, and the no-retention guarantees

## Quick Links

- [Authentication & Permissions](../09-security/README.md) - Basic auth setup
- [Subscription & Billing](../12-ecosystem-reference/billing.md) - Plan-based access control

## Best Practices

1. **Principle of Least Privilege**: Assign the minimum permissions necessary
2. **Use Groups**: Manage permissions through groups rather than individual users
3. **Regular Audits**: Review permissions and access logs periodically
4. **Secure KB by Default**: Set restrictive default permissions on sensitive folders
5. **Enable Audit Logging**: Track all permission changes and access attempts
## Media capture indicator (#2504)

The desktop WebView grants microphone and camera access only to origins the
shell owns; every other origin is denied and the denial is logged. Each grant
publishes a `gb-media-grant` event carrying the origin and device kinds. The
shell renders that event as a red tray dot (#2504): red means the microphone
or camera is currently granted, matching the hardware-style LED convention
users expect from a laptop — the dot is the software equivalent of the camera
LED. The origin is included in the accessible label and in the server-side
audit event; no credentials or internal addresses are logged.

## Kiosk shell shortcut map (#2450)

In the OS edition (`edition=os`, the appliance kiosk) the Tauri bundle is built
without the `devtools` feature, so no WebKit inspector or developer console is
available in release builds. The main window is undecorated and fullscreen; a
native close request is intercepted and opens the shell power menu rather than
exiting to a black screen. A crashed or restarting shell shows a branded
"Restarting…" surface instead of a TTY.

The only system-level shortcuts exposed to the kiosk user are:

| Shortcut | Action | Locks first? |
|---|---|---|
| Power button / tray → Power | Opens the shell power menu | Yes for suspend/restart/shut down |
| `Ctrl+K` | Opens the app command palette (no OS access) | n/a |
| `?` | Opens the in-app shortcuts help | n/a |
| `Esc` | Closes the current dialog or menu | n/a |

No shortcut opens a terminal, a file manager, a WebView inspector, or any host
shell. Power actions that can interrupt the session route through the lock
screen and the authenticated `/api/system/power` endpoint; they never bypass
the lock. The escape hatch at boot is deliberately absent — recovery is by
reprovisioning (see the appliance diagnostics documentation).

### Firewall UAT (#2477)

The image ships a default-deny nftables input chain. The whole provisioned
stack (8080, 3000, 4000, 5000) binds to loopback in shell mode; the only way
to open it is the **Allow other devices on my network** switch in
Settings → Network, which adds the source to `gb_lan_allowed` for 15 minutes
and then closes it automatically. `desktop/os/check-bind-policy.sh` asserts
the bind half of this contract at smoke-test time.

To verify the host half from a second machine on the same network:

```sh
# Default state — the ports must be closed or filtered.
nmap -Pn -p 8080 <laptop-hostname>    # expect: closed|filtered

# Turn on Settings -> Network -> "Allow other devices on my network" on the
# laptop, then immediately repeat from the second machine:
nmap -Pn -p 8080 <laptop-hostname>    # expect: open

# After 15 minutes the switch turns itself off; the port is closed again.
```

Never hardcode addresses in documentation or scripts — resolve the laptop by
hostname. A failed "closed" assertion means the bind policy or nftables rule
regressed and must be fixed before release.

### Keyboard / Fn map (#2462)

The Settings → Keyboard & Region page owns layout switching (Alt+Shift) and
the Settings → Peripherals page owns touchpad, Bluetooth and printers. Common
laptop Fn combinations are handled by the compositor / power daemon in the
image; the shell mirrors them with these DOM events so the UI updates without
a native helper:

| Key | Effect in the shell |
|---|---|
| Fn + Brightness up/down | `gb:display-change` → Display slider |
| Fn + Volume up/down / mute | `gb:sound-key` → Sound slider/mute |
| Fn + Airplane mode | `rfkill` Bluetooth via Peripheral API |
| Fn + Media keys | standard MPRIS; no suite shortcut |

No shortcut opens a terminal or bypasses the lock screen.

### Session bus surface (#2502)

The kiosk's session bus is not a general power or networking API. Polkit
(`/etc/polkit-1/rules.d/50-general-bots-kiosk.rules`) denies login1 power and
NetworkManager system-modify actions to the `gbdesk` user; the only power path
is the audited, admin+loopback `/api/system/power` endpoint and its fixed
root helper. The allow-list of session-bus interfaces the shell may use is the
deny-list's complement in that rule file plus the portal interfaces used by
the file picker.

Screencast/screenshot capture requires an explicit "Share screen" gesture:
`screen-capture-guard.js` wraps `getDisplayMedia` and rejects with
`NotAllowedError` unless the user has just activated the action; the tray
shows a red capture indicator for the duration of the stream.

Known limitation: the WebView shares the kiosk user's session bus, so a page
that already has code execution in the shell is inside the same trust domain.
The appliance does not claim otherwise; the WebKit web process is sandboxed
(#2500) and the session-bus restrictions above are defense in depth, not a
per-page sandbox.

### HTTP error envelope (#2414)

New handlers return `botsecurity_core::api_error::ApiError`, which implements
`IntoResponse` and always produces the same JSON shape:

```json
{"error":"internal_error","message":"Sanitized, user-safe text","code":"stable_code","request_id":"abc123"}
```

The original driver/IO error is passed to the shared sanitizer and stays in the
server log; `message`, `code` and `request_id` are the only fields clients
should read. Handlers still returning `(StatusCode, String)` are legacy; their
count is ratcheted down by `scripts/check_raw_error_strings.py`, a blocking CI
guard. This envelope is the contract referenced by the API command catalog for
LLM-driven calls.

## Office imports fail closed without a malware scanner (#2522, #2606)

The shared hostile-file gate in `botsecurity-core` protects every zip-based
Office import (Docs `.docx`/`.odt`, Sheets `.xlsx`, Slides `.pptx`). It checks
the archive *inside* the container — macros, encrypted and nested parts, zip
bombs, DOCTYPE entities and SSRF-bearing relationships — and finishes with a
malware-scan decision. That decision is honest: with no scanner registered it
is `NotConfigured`, never a fabricated `Clean`.

Since #2522 the shared `HostileFilePolicy::default()` sets
`require_malware_scanner` to `true`. A zip import is therefore **refused** with
the typed `MalwareScannerMissing` error when no scanner has been installed,
instead of being accepted unscanned. This applies both to the central
`validate_upload` gate and to the explicit import-boundary audits in Docs,
Sheets and Slides; all three use the same default policy.

#2606 removed the temporary escape hatches: `GB_ALLOW_UNSCANNED_IMPORT` and the
Slides-only `GB_ALLOW_UNSCANNED_SLIDES_IMPORT` alias are no longer read, and no
configuration path can lower the scanner requirement. A registered scanner is
the only way an Office package can be imported. Register one with
`set_malware_scanner`; until then, imports fail closed rather than being treated
as clean. The decision is recorded on the import report as `NotConfigured`,
`Clean`, `Infected` or `Failed` — never fabricated as `Clean` by the gate itself.

## Data-protection honesty on the OS appliance (#2470)

On the OS edition (`edition=os`) the Settings → Storage & Reset panel exposes a
data-protection surface whose labels state what this build actually does. The
authority is `botserver/crates/botsettings/src/os_data_protection.rs`: its status
response is static and performs no host probing, so the panel cannot drift from
the shipped behavior.

| Action | Endpoint | What this build does |
|---|---|---|
| Disk encryption | — | Not implemented: no LUKS mapping, no TPM sealing and no recovery-key export. General Bots does not encrypt user data at rest in this image. |
| Snapshots / backup | `POST /api/os/data-protection/snapshots` | Adapter contract only. No btrfs or Drive backend is registered, so a valid request fails closed with `503` and no snapshot is created. If a backend is registered it stages a job descriptor (`202`); restore and pruning are separate typed-confirmation actions supplied by the OS image. Retention counts are a planned policy, not a running scheduler. |
| Export my data | `POST /api/os/data-protection/export` | Returns `200` with a deterministic, descriptor-only archive manifest (`tar.zst`, `full-archive`). Its `bytes_read` is `0`, `archive_created` is `false` and no data leaves the device. |
| Reset this PC | `POST /api/os/data-protection/reset` | Requires the exact typed confirmation token (`RESET THIS PC`); the button stays disabled until that token is read from the live status response and entered verbatim. A confirmed request returns `501` — no wipe-and-wizard backend ships and nothing is erased. |

Every action is gated (shell mode + admin/owner + loopback) and written to the
audit log, including refused and unimplemented attempts. The same text is
localized in `botlib/locales/*/settings-os-appliance.ftl` under the `os-data-*`
keys, so all eight catalogs describe the deployed behavior instead of a future
promise.

## Runtime mTLS for inter-service traffic (#2590)

In a containerized (Incus) install the backing services live in separate
containers on a private bridge. On that hop, reachability plus a password or
token is the only authentication, so a compromised container can impersonate
another service. Mutual TLS closes that gap: both ends present a certificate
issued by the install's internal CA and verify the peer against that CA.

This is an **opt-in** for containerized installs. The authority for the
configuration is `botserver/crates/botcoresecrets/src/mtls/` (`config.rs`,
`runtime.rs`, `status.rs`); the data clients that consume it are
`botserver/src/main_module/cache.rs` (cache),
`botserver/crates/botcore/src/shared/utils/create_conn.rs` (Postgres and
internal HTTP) and `botserver/crates/botintegrations/src/http.rs` (internal
service HTTP). The resolved flag is reported as `mtls_enabled` in the security
overview (`botserver/crates/botsettings/src/security_admin.rs`).

### Enabling it: configuration keys and environment variables

Configuration holds **paths and mode/stage selectors only — never certificate
or key material**. A non-blank environment value wins over a non-blank Vault
field; a blank value means "not set" and falls through to the next source. The
configuration is resolved **once** at startup and cached, so changing the
environment later cannot half-enable a transport — a restart is required.

| Setting | Environment variable | Vault field (path `secret/gbo/system/security`) |
|---|---|---|
| Mode | `GB_MTLS_MODE` | `mtls_mode` |
| Rollout stage | `GB_MTLS_STAGE` | `mtls_stage` |
| Internal CA bundle (PEM) | `GB_MTLS_CA_CERT` | `mtls_ca_cert` |
| Server certificate (PEM) | `GB_MTLS_SERVER_CERT` | `mtls_server_cert` |
| Server private key (PEM) | `GB_MTLS_SERVER_KEY` | `mtls_server_key` |
| Client certificate (PEM) | `GB_MTLS_CLIENT_CERT` | `mtls_client_cert` |
| Client private key (PEM) | `GB_MTLS_CLIENT_KEY` | `mtls_client_key` |

- **Mode** — `mtls` enables it (also accepted: `mutual`, `mutual_tls`,
  `mutual-tls`, `required`, `secure`, `true`, `1`). Any other or blank value —
  including a typo — leaves plaintext enabled, so a typo cannot silently turn
  on a half-configured secure mode.
- **Stage** — `cert-provisioning` (default, stage 0), `server-tls` (1),
  `client-cert-verification` (2), `plaintext-disabled` (3). A transport change
  requires the mode to be `mtls` **and** the stage to be at least `server-tls`;
  at stage 0 nothing changes even with the mode set.

### Plaintext is the default

The default transport mode is `plaintext` and the default stage is
`cert-provisioning`. The mode is never inferred from the presence of
certificates. Single-host and `desktop-local` installs keep their loopback
threat model and are explicitly **out of scope**: the opt-in is off for them
unless an operator sets it, and their URLs and clients are byte-for-byte
unchanged.

### What each transport does when the opt-in is active

- **PostgreSQL** — the connection URL gains `sslmode=verify-full` and
  `sslrootcert=<CA path>`. At stage `client-cert-verification` or later it also
  gains `sslcert=<client cert path>` and `sslkey=<client key path>`. If the
  configuration cannot be resolved, the database connection is refused (a
  fatal error) — a plaintext fallback would defeat the opt-in.
- **Cache (Redis/Valkey)** — each candidate URL is rewritten from `redis://`
  to `rediss://`; no plaintext fallback is kept. The workspace `redis` dependency
  does not enable `tls-rustls`, so a `rediss://` URL cannot be opened: the client
  fails and cache functions are **disabled (fail closed)** rather than silently
  downgrading to plaintext. A configuration resolution failure likewise disables
  the cache. No Redis TLS connection is established or verified in this build.
- **Internal HTTP** — service base URLs are rewritten from `http://` to
  `https://` (scheme only; host, port and path are preserved). The client
  verifies the peer against the configured CA and presents this service's
  client certificate and key. Current limit: if client-identity setup fails the
  client degrades to CA-verification-only (with a warning); if configuration
  resolution fails, the default stack CA is used. Neither case re-enables
  plaintext transport.
- **Diagnostics** — startup logs `configuration loaded: mode=… stage=…` with
  the `[mtls]` prefix. Missing-setting diagnostics name the item only; the
  helpers carry paths but never read or log certificate/key bytes.

### Staged rollout

| Stage | Name | Effect |
|---|---|---|
| 0 | `cert-provisioning` | CA and per-service certificates provisioned; no behaviour change. |
| 1 | `server-tls` | Servers present certificates; clients verify server identity. |
| 2 | `client-cert-verification` | Clients present certificates; servers verify them. |
| 3 | `plaintext-disabled` | Plaintext listeners removed or firewalled after stage 2 is green. |

Each stage is a prerequisite of the next; do not advance while the previous
stage is not verified green.

### Operator verification checklist

1. Before enabling, confirm the security overview reports `mtls_enabled =
   false` and the startup log shows `mode=plaintext` (the default).
2. At stage 0, provision the internal CA and per-service certificates; confirm
   no client URL or transport has changed.
3. Set `GB_MTLS_MODE=mtls` and `GB_MTLS_STAGE=server-tls` (or their Vault
   equivalents), then restart botserver and confirm the startup log shows
   `mode=mtls stage=server-tls`.
4. Verify the Postgres URL reports `sslmode=verify-full` with `sslrootcert`,
   and that `sslcert`/`sslkey` are **not** yet present.
5. Verify cache URLs are `rediss://` and internal service URLs are `https://`;
   confirm there is no plaintext fallback (a TLS-less cache build disables the
   cache rather than reconnecting over `redis://`).
6. Advance to `client-cert-verification` and confirm the Postgres URL gains
   `sslcert`/`sslkey`, servers accept the client certificate, and all clients
   still connect.
7. Only after stage 2 is green, advance to `plaintext-disabled` and confirm the
   plaintext listeners are gone.
8. Confirm diagnostic logs contain no certificate paths or key material.

### Rollback path

The configuration is resolved once at startup, so every change below requires
a **botserver restart** to take effect. Environment variables override Vault,
so clear both sources where applicable.

- **Full rollback:** set `GB_MTLS_MODE=plaintext` (or clear/blank it) and set
  the Vault field `mtls_mode` to `plaintext` or blank it, then restart.
  Transports revert to their plaintext/loopback URLs and clients. Setting
  `GB_MTLS_STAGE=cert-provisioning` as well returns to the no-op stage 0.
- **Step back one stage:** lower `GB_MTLS_STAGE` to the previous stage and
  restart. For example, return from `client-cert-verification` to `server-tls`
  to stop presenting client certificates while keeping verified server TLS.
- **Misconfiguration at stage 0:** because stage 0 changes no behaviour,
  returning the mode to plaintext (or leaving it blank) restores the original
  loopback behaviour without any transport migration.

## Enterprise sharing and access control (#2452)

This build ships **resource-level sharing for collaboration documents**, not a
general enterprise ACL/DRM stack. The authority is
`botserver/src/main_module/routes/collab_routes/` and
`botserver/crates/botdocs/src/shared_link.rs`.

### What is implemented

| Capability | Authority | Behaviour |
|---|---|---|
| Per-resource grants | `collab_routes/permissions.rs:12-111`, `resource_permissions` table | Owner (or admin) grants `viewer` / `commenter` / `editor` to a `user`, `group` or `domain` (`dto.rs:363-369`); the first sharer becomes owner; revoke and ownership transfer are owner-only. |
| Effective-role resolution | `collab_routes/helpers.rs:129-195` | An explicit `user` grant is checked first (owner wins), then a `domain` grant matched against the user's email domain. `GET /api/collab/permissions/me` (`permissions.rs:219-241`) lets Docs/Slides force the editor into read-only mode for viewers. |
| Share links | `collab_routes/permissions.rs:257-407`, `resource_links` table | The owner mints an unguessable token (two concatenated UUIDv4 values), with optional expiry (`expires_in_hours`), a role and a stored owner namespace; links can be listed and revoked. |
| Read-only document link | `botdocs/src/shared_link.rs:97-142`, `botsecurity-auth/src/auth_api/public_routes.rs:130-134,420-425` | `GET /api/docs/shared/:token` resolves exactly one document and returns `read_only: true`; the token is not echoed and mutating methods on that path are not anonymous. |
| Anonymous drive download links | `api_setup.rs:271-283`, `agent_drive.rs:286-309` | `/api/files/share-link` creates a token, `/api/files/public/:token` serves it without a session, and a revoke route removes it. |
| Activity trail | `collab_routes/helpers.rs:70-98` (`collab_activity`) | Share, revoke, transfer, comment and version events are appended; a failed audit write is logged but never aborts the primary write. |

Two limits are worth stating plainly. The RBAC crate's in-memory `ResourceAcl`
(`botsecurity-auth/src/rbac_middleware/types.rs:232-337`, managed in
`manager.rs:140-219`) supports owner, per-user, per-group and public grants, but
no production caller invokes `check_resource_access` or `set_resource_acl` — only
unit tests do — so it does **not** enforce live requests. Its
`RbacService::check_permission` implementation is a stub returning
`!default_deny` without consulting any ACL (`manager.rs:345-354`). Likewise,
`effective_role` resolves only `user` and `domain` grants (`helpers.rs:147-192`),
so a `group` grant accepted by `valid_grantee_type` is not honoured by that
lookup.

### What is not implemented

| Capability | Status |
|---|---|
| External guest identities / invitations | No `guest` principal exists: grantee types are limited to `user`, `group` and `domain` (`dto.rs:367-369`). Token links permit anonymous *viewing*, but they are bearer capabilities, not guest accounts with an identity. |
| Watermark / DRM | No document watermark, copy/print restriction or DRM enforcement. The only `brand_watermark` field is a cosmetic option on the generated image-card `CARD` keyword (`botserver/src/basic/keywords/card.rs:155-166`). |
| Full access audit | `collab_activity` records sharing and editing actions, and a generic `access_logs` table is used by cleanup and SOC 2 evidence queries, but a share-link read (`handle_shared_document`) writes no per-read record. There is no complete "who read what" audit for shared resources. |
