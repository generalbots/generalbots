# Chapter 10: Configuration & Deployment 🟡 BETA

This chapter covers bot configuration and deployment options. Each bot's behavior is controlled by a simple CSV configuration file in its `.gbot` package.

## Configuration System

botserver uses a straightforward name-value CSV format for configuration:

```csv
name,value
setting_name,setting_value
another_setting,another_value
```

## File Location

```
mybot.gbai/
└── mybot.gbot/
    └── config.csv
```

## Configuration Categories

### Server Settings
- Web server binding and ports
- Site generation paths
- Service endpoints

### LLM Configuration
- Model paths (local GGUF files)
- Service URLs
- Cache settings
- Server parameters (when embedded)

### Prompt Management
- Context compaction levels
- History retention
- Token management

### Email Integration
- SMTP server settings
- Authentication credentials
- Sender configuration

### Theme Customization
- Color schemes
- Logo URLs
- Bot titles

### Custom Database
- External database connections
- Authentication details

## Key Features

### Simple Format
- Plain CSV with name-value pairs
- No complex syntax
- Human-readable

### Flexible Structure
- Empty rows for visual grouping
- Optional settings with defaults
- Extensible for custom needs

### Local-First
- Designed for local LLM models
- Self-hosted services
- No cloud dependency by default

## Example Configurations

### Minimal Setup
Just the essentials to run a bot:
```csv
name,value
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### Production Setup
Full configuration with all services:
```csv
name,value
,
server_host,0.0.0.0
server_port,8080
,
llm-url,http://localhost:8081
llm-model,../../../../data/llm/production-model.gguf
llm-cache,true
,
email-server,smtp.company.com
email-from,bot@company.com
,
theme-title,Company Assistant
```

## Configuration Philosophy

1. **Defaults Work**: Most settings have sensible defaults
2. **Local First**: Assumes local services, not cloud APIs
3. **Simple Values**: All values are strings, parsed as needed
4. **No Magic**: What you see is what you get

## Installer-Managed Components (Local / Stage)

For a local or stage install, the package manager is the authority on which
components are provisioned. The registry is assembled in
`botserver/crates/botcorepkg/src/installer/manager_construction.rs`
(`PackageManager::all_components`); each entry is defined under
`botserver/crates/botcorepkg/src/installer_regs/`. Naming and platform details
for the components touched by this cleanup now read as follows:

| Component | Current behavior | Source |
|---|---|---|
| `timeseries_db` | Runs the upstream `influxd` binary but stores everything under generic `timeseries` directories: `{{DATA_PATH}}/timeseries` (containing `engine/` and `influxd.bolt`) and `{{CONF_PATH}}/timeseries`. The `INFLUXD_ENGINE_PATH` and `INFLUXD_BOLT_PATH` variables point at those paths. The HTTP API listens on 8086 and RPC on 8083. | `installer_regs/register_databases.rs::register_timeseries_db` |
| `observability` | Depends on `timeseries_db`, runs `vector` with `{{CONF_PATH}}/monitoring/vector.toml` and checks its `/health` endpoint on 8686. | `installer_regs/register_observability.rs::register_observability` |
| `host` | Host preparation uses Incus only (the sole supported container layer): `incus admin init`, `incus storage`, `incus profile device`, and a proxy that connects `unix:/var/lib/incus/unix.socket` to `unix:/tmp/incus.sock` inside containers. It shells out to no legacy container daemon and mounts only the Incus socket. | `installer_regs/register_observability.rs::register_host` |
| `remote_desktop` | The former `remote_terminal` key was renamed to `remote_desktop`. It installs Xvfb, XRDP and XFCE and serves an RDP desktop session on 3389. The registry contains only the new key; there is no compatibility alias. | `installer_regs/register_workspace.rs::register_remote_desktop` |
| `table_editor` | **Not managed by botserver.** Out of scope and pending deletion (#2582), it is intentionally absent from `PackageManager::all_components`, so it does not appear in `botserver install` or component listings and cannot be started or stopped by the package manager. Provision it outside botserver. The registrar still exists in source but is deliberately not called. | `installer_regs/register_workspace.rs::register_table_editor` (registrar retained, not wired) |

`devtools` was a no-op placeholder and is no longer registered, so it does not
appear in `botserver install` or component listings.

### Existing `timeseries_db` data

The directory rename changed only the parent folder name — the `engine/` and
`influxd.bolt` file names are unchanged. For stage and local installs the
supported path is a clean install: the component's pre-install commands create
a fresh `timeseries` directory.

To keep metrics from a stack that used the old `influxdb` directories:

1. Stop `timeseries_db` before moving any files.
2. Rename `{{DATA_PATH}}/influxdb` to `{{DATA_PATH}}/timeseries` and
   `{{CONF_PATH}}/influxdb` to `{{CONF_PATH}}/timeseries` (on Windows, rename the
   matching `...\influxdb` directories). Do not copy the old tree next to a
   newly created `timeseries` tree: `influxd` is pointed only at the
   `timeseries` paths.
3. Start `timeseries_db`, confirm its `/health` check passes, then start
   `observability`, which depends on it.

## Install acceptance harness

The component install UAT is a fragmented shell harness. `scripts/uat_install.sh`
is a dispatcher: each `step` runs one small script from `desktop/os/tests/`,
prints the state it observed, records one result line and stops. `plan` prints
the recommended one-command-at-a-time order without running anything; `report`
aggregates the results into a verdict.

This is a **live** harness and the evidence source for the full-install,
service-health, login and regression items of #2582. Its live steps and `report`
never report PASS from an offline or development session: every live step first
requires `UAT_TARGET_HOST=1`, an `incus` CLI and a `botserver` binary under test,
otherwise it records `UNVERIFIED` and exits 3. Containers are handled only by
`incus`. The separate `--self-test` mode (below) can print `verdict: PASS`, but
only from synthetic stub rows and never as a live-host result.

### Offline self-test (`--self-test`)

`scripts/uat_install.sh --self-test` (alias `selftest`) checks the verdict logic
without a host. It deliberately prints `verdict: PASS` from a full set of stub
rows, but that PASS is explicitly synthetic proof that the verdict logic behaved
as asserted on those rows — **not** a result from a real target host.

The self-test never contacts a live target: it clears `UAT_TARGET_HOST`,
`UAT_BOTSERVER`, `UAT_LOGIN_USER`, `UAT_LOGIN_PASSWORD` and `UAT_SOURCE_ROOT`,
never invokes `incus` or `botserver`, and writes only to a throwaway results
directory under `/tmp/opencode` (`UAT_SELFTEST_DIR`, default
`/tmp/opencode/uat-install-selftest`, mode `0700`; a directory outside
`/tmp/opencode` is refused with exit 2). It builds the stub `results.tsv` from
the required ids parsed directly out of `step_99_report.sh`, so the stub cannot
drift from the verdict it is meant to satisfy.

It runs **14 assertions** (14 `ok` lines on success), all of which must hold —
14/14 — for exit 0:

| # | Assertion | Expected exit |
|---|---|---|
| 1–3 | A live step run offline records `UNVERIFIED` (not PASS), prints `UNVERIFIED` and exits 3 | 3 |
| 4–5 | `report` with no evidence prints `UNVERIFIED: no results recorded` and exits 3 | 3 |
| 6–8 | `report` with one injected `FAIL` row prints `verdict: FAIL`, names the failed step (`health:vault`) and exits 1 | 1 |
| 9–10 | `report` with full stub evidence prints `verdict: PASS` and exits 0 | 0 |
| 11–12 | `report` with one required row missing (`status:vault`) prints `verdict: UNVERIFIED`, names it and exits 3 | 3 |
| 13–14 | A non-PASS row (`health:cache` set to `UNVERIFIED`) is not evidence and keeps verdict `UNVERIFIED`, exiting 3 | 3 |

Exit codes for `--self-test`: 0 = all 14 assertions held on stub evidence,
1 = at least one assertion failed, 2 = precondition failure (the results
directory is outside `/tmp/opencode` or could not be created). The final banner
restates that `SELF-TEST OK` is not a real host PASS and that no live target was
contacted.

### Preconditions

| Requirement | How to supply it |
|---|---|
| Explicit target-host opt-in | `UAT_TARGET_HOST=1` |
| Container layer | `incus` on `PATH`, already initialized on the host |
| Binary under test | `UAT_BOTSERVER=/path/to/botserver`, or `botserver` on `PATH` |
| Login credentials | `UAT_LOGIN_USER` and `UAT_LOGIN_PASSWORD` in the operator's environment; never stored in the harness or its results |
| Source checkout for step 22 | `UAT_SOURCE_ROOT`, defaulting to the repository containing the harness |
| Evidence directory | `UAT_RESULTS_DIR`, default `/tmp/opencode/uat-install` (created mode `0700`); use a fresh directory per target host |
| Clean stage host | No pre-existing `<tenant>-<component>` Incus containers; `UAT_TENANT` defaults to `default` |

The plan installs `vault` and `tables` before their dependents and
`timeseries_db` before `observability`. `botserver install` also resolves
missing dependencies itself (`botcore/src/package_manager/facade/install.rs`,
`install`). `table_editor`, `webmail` and `news` are out of scope and are never
installed; step 24 asserts that `table_editor` has no registry entry and no
container.

### Running one action at a time

```bash
scripts/uat_install.sh plan                       # print order, no side effects
scripts/uat_install.sh preflight                  # target-host preconditions
scripts/uat_install.sh step 01                    # live component registry
scripts/uat_install.sh step 02 vault              # install ONE component
scripts/uat_install.sh step 03 vault              # status of ONE component
scripts/uat_install.sh step 04 vault 8200         # HTTP or TCP health probe
scripts/uat_install.sh step 05                    # start all installed components
scripts/uat_install.sh step 10                    # login page GET
scripts/uat_install.sh step 11                    # authenticated login POST
scripts/uat_install.sh step 20                    # remote_desktop regression
scripts/uat_install.sh step 21                    # timeseries path regression
scripts/uat_install.sh step 22                    # Incus-only host regression
scripts/uat_install.sh step 23                    # devtools absence
scripts/uat_install.sh step 24                    # table_editor absence
scripts/uat_install.sh report                     # aggregate recorded evidence
scripts/uat_install.sh --self-test                # offline verdict-logic self-test (14 assertions)
```

Steps use loopback URLs (`http://localhost`) and never hardcode an address or a
credential. Step 11 reads `UAT_LOGIN_USER`/`UAT_LOGIN_PASSWORD` from the
environment, posts them on stdin and records only the HTTP status and the
`success` flag — no token is printed or written to a result file.

### Health probes used by the plan

| Component | Probe | Registry source |
|---|---|---|
| `vault` | TCP 8200 | `installer_regs/register_secrets.rs` |
| `tables` | TCP 5432 | `installer_regs/register_drive.rs` |
| `cache` | TCP 6379 | `installer_regs/register_drive.rs` |
| `drive` | TCP 9100 | `installer_regs/register_drive.rs` |
| `llm` | TCP 8081 | `installer_regs/register_drive.rs` |
| `email` | TCP 8025 | `installer_regs/register_drive.rs` |
| `proxy` | TCP 80 | `installer_regs/register_drive.rs` |
| `directory` | TCP 8300 | `installer_regs/register_directory.rs` |
| `alm` | TCP 4747 | `installer_regs/register_directory.rs` |
| `alm-ci`, `host` | status only (no listening port) | `installer_regs/register_directory.rs`, `register_observability.rs` |
| `dns` | TCP 53 | `installer_regs/register_directory.rs` |
| `meet` | TCP 7880 | `installer_regs/register_directory.rs` |
| `vector_db` | TCP 6334 | `installer_regs/register_databases.rs` |
| `timeseries_db` | HTTP `8086/health` | `installer_regs/register_databases.rs` |
| `botmodels` | TCP 8085 | `installer_regs/register_botmodels.rs` |
| `observability` | HTTP `8686/health` | `installer_regs/register_observability.rs` |
| `remote_desktop` | TCP 3389 | `installer_regs/register_workspace.rs` |

All registry paths are under `botserver/crates/botcorepkg/src/`. The probe
numbers come from each registration's `ports` field and `check_cmd`.

### Verdict and exit codes

| Exit | Verdict | Meaning |
|---|---|---|
| 0 | `PASS` | Every required step has a live PASS row: install, status and health for all components, login, start and all five regressions |
| 1 | `FAIL` | At least one live step recorded a failed assertion |
| 3 | `UNVERIFIED` | Offline/dev session, missing precondition or missing evidence for at least one required step |
| 2 | usage error | Unknown step or missing argument |

A `FAIL` takes precedence over missing evidence. A static source inspection is
not a substitute for a live step: step 22 combines its source assertion with a
live Incus check and cannot pass offline.

`--self-test` reuses exit codes 0 and 1 for its own result: 0 means all 14
assertions held on stub evidence, 1 means at least one assertion failed, and 2
means its own precondition failed (the results directory is outside
`/tmp/opencode` or could not be created). Its `verdict: PASS` is synthetic — the
verdict logic behaved correctly on stub rows — and is never a live-host result.

### Live-evidence requirements

These items are established only by running the steps on the target host; a
source-only review produces no evidence for them: per-component install,
start and health; login page and authenticated API access; `remote_desktop`
registered and running with no `remote_terminal` key; generic
`timeseries` paths with no `influxdb` directories in the installed container;
Incus-only host preparation; and the absence of `devtools` and `table_editor`
from the live CLI and container lists. The `--self-test` mode establishes none
of these: it uses synthetic stub rows and no live target is contacted.

## See Also

- [config.csv Format](./config-csv.md) - Complete reference
- [LLM Configuration](./llm-config.md) - Language model settings
- [Parameters](./parameters.md) - All available parameters
- [System Limits](./system-limits.md) - Rate limits, package sizes, and configurable limits
- [Sources Sync Strategy](./sources-sync-strategy.md) - Gmail, Outlook, Drive sync to vector DB
- [Secrets Management](./secrets-management.md) - Secure credential storage
---

<div align="center">
  <img src="../assets/gb-logo.svg" alt="General Bots" width="120">
</div>
