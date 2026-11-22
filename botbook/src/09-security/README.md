# Chapter 9: Security 🟡 BETA

This chapter covers advanced security topics for General Bots.

## Overview

General Bots implements a comprehensive security model designed for enterprise deployments:

- **Multi-tenant Architecture**: Support for multiple organizations with complete data isolation
- **Role-Based Access Control (RBAC)**: Fine-grained permissions at every level
- **Knowledge Base Security**: Folder-level permissions with Qdrant vector search integration
- **SOC 2 Type II Compliance**: Enterprise-grade security controls and audit logging

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

## Office imports fail closed without a malware scanner (#2522)

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
Sheets and Slides. Slides additionally owns its own boundary switch,
`GB_ALLOW_UNSCANNED_SLIDES_IMPORT`, which still governs that app's override.

### Opt-out and rollback path

A deployment that cannot deploy a scanner yet can temporarily restore the
permissive behaviour with one explicit environment switch:

```sh
GB_ALLOW_UNSCANNED_IMPORT=1   # also accepts true / yes / on, case-insensitive
```

Only `1`, `true`, `yes` or `on` (whitespace trimmed, case-insensitive) opt out;
an unset variable or any other value keeps the fail-closed default. The switch
is read by `HostileFilePolicy::default()` through
`botsecurity_core::upload_hostile::opt_out`. The scan decision is still recorded
as `NotConfigured` — the opt-out lowers the *requirement*, it does not claim a
clean scan. To roll back the change, remove the variable and restart botserver;
imports then fail closed again.

The switch is an environment variable so it can be injected by the deployment,
including from Vault-backed configuration. A caller that reads the same flag
from Vault (or another configuration store) reaches the identical decision by
passing the value to `botsecurity_core::upload_hostile::malware_scanner_required_from`,
which interprets env- and Vault-sourced values with the same grammar:

```rust
use botsecurity_core::upload_hostile::{HostileFilePolicy, malware_scanner_required_from};

let vault_value = std::env::var("GB_ALLOW_UNSCANNED_IMPORT").ok(); // or a Vault read
let policy = HostileFilePolicy {
    require_malware_scanner: malware_scanner_required_from(vault_value.as_deref()),
    ..HostileFilePolicy::default()
};
```

Leaving the opt-out enabled in a production deployment accepts unscanned Office
packages; it is intended only as a temporary rollback and should be removed as
soon as a scanner is registered with `set_malware_scanner`.

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
