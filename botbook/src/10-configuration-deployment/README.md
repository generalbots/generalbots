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
for four components changed together and now read as follows:

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
