# Container Deployment (Incus) 🟡 BETA

Containers are **Incus** instances. `lxc`/`lxd` are not a supported runtime for
this project: `AGENTS.md` states Incus is the only container layer, and the
bootstrap code calls `incus` directly
(`botserver/crates/botcore/src/package_manager/container/bootstrap_container_inner.rs`).

> ⚠️ All container commands run on the **host**, not inside a container.

## Prerequisites

```bash
sudo apt install incus
sudo incus admin init --auto
sudo usermod -aG incus-admin "$USER"   # then log out and back in
```

Membership of `incus-admin` is what lets a non-root user talk to the Incus
socket; without it every `incus` command fails with a permission error.

## Automatic setup

```bash
./botserver --container
```

`--container` selects container mode (`botserver/src/main_module/init.rs`,
`botserver/crates/botcorepkg/src/cli.rs`). The installer then, per component:

1. creates `{tenant}-{component}` as an Incus instance
   (`botserver/crates/botcore/src/package_manager/facade/install.rs`),
2. copies the component's binaries and configuration,
3. adds an `eth0` NIC and the port proxy devices
   (`bootstrap_container_inner.rs`),
4. starts the systemd unit inside the instance.

Images are tried in order: `ubuntu:24.04`, `ubuntu:22.04`, `images:debian/12`,
`images:debian/11` — the first that launches wins.

## Naming and mounts

| Instance | Component |
|---|---|
| `{tenant}-tables` | PostgreSQL |
| `{tenant}-drive` | object storage (MinIO) |
| `{tenant}-cache` | Valkey |
| `{tenant}-llm` | llama.cpp (optional) |
| `{tenant}-email` | mail server (optional) |

Data lives on host mounts, so deleting an instance does not delete data:

```
botserver-stack/tables/data/  → /opt/gbo/data/
botserver-stack/tables/conf/  → /opt/gbo/conf/
botserver-stack/tables/logs/  → /opt/gbo/logs/
```

## Common operations

```bash
incus list
incus exec default-tables -- psql -U gbuser botserver
incus exec default-tables -- journalctl -u tables
incus stop  default-tables
incus start default-tables
incus delete default-tables --force     # host mounts persist
```

## Limits and snapshots

```bash
incus config set default-tables limits.cpu 2
incus config set default-tables limits.memory 4GiB

incus snapshot create default-tables pre-upgrade-$(date +%Y%m%d)
incus info default-tables                       # lists snapshots
incus copy default-tables/pre-upgrade-20260101 default-tables-restored
```

Restore in place by copying the snapshot to a new instance and swapping names;
`incus` has no in-place `restore` for a running instance.

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `incus: command not found` | install Incus (above); `lxd` is not a substitute |
| Permission denied on the socket | user is not in `incus-admin` |
| Instance won't start | `incus info --show-log default-tables` |
| Port already bound on the host | check the proxy devices: `incus config device list <instance>` |
| Port reachable only from the host | NAT rules: `configure_iptables_nat.rs` writes them; DNAT must use the external interface |

## Runtime rules the build enforces

Two rules are checked mechanically (issue #2126,
`python3 scripts/check_container_runtime.py --check`) instead of in review:

| Rule | What fails the build |
|---|---|
| Incus is the only container layer | any spawn of `lxc`, `lxd` or `lxc-execute` — `SafeCommand` does not make LXC supported |
| Incus is spawned through `SafeCommand` | `std::process::Command::new("incus")` or `tokio::process::Command::new("incus")` |

`SafeCommand` (`botsecurity-core/src/command_guard.rs`) is what adds the binary
allowlist, argv validation and a scrubbed environment; a raw `Command` skips all
three. The one accepted exception is
`portable_pty::CommandBuilder::new("incus")` in the suite terminal
(`botserver/src/api/terminal/`), which builds an argv for the container shell
rather than a host command.

Container names are a third rule, enforced at each call site rather than by the
gate. A name is derived, never taken verbatim from user input:

- the installer uses `{tenant}-{component}`
  (`botcore/src/package_manager/facade/install.rs`);
- Vibe derives `{project}-{env}[-runner]` (`production` shortened to `prod`) and
  refuses the platform's own names (`is_protected_container_name`: `bot`,
  `system`, `tables`, `cache`, `drive`, `vault`, `directory`, `proxy`, `dns`,
  `email`, `webmail`, `meet`, `vectordb`, `llm`, `alm`, `alm-ci`,
  `table-editor`, `vibe`, `vibe-runner`, `host`, `incus`);
- the deployment gateway accepts only `[A-Za-z0-9_-]`, rejects a leading `-` and
  an empty name before it reaches the `incus` argv
  (`botdeployment/src/gateway_runtime.rs`).

Pre-existing `lxc` call sites are recorded in
`scripts/container_runtime_allowlist.txt`, which is shrink-only: fixing a file
means deleting its line, and a stale line is reported as a failure.

## Known gap in the installer

The legacy installer helpers still shell out to `lxc` and `lxd`
(`botserver/crates/botcore/src/package_manager/facade/safe_lxc.rs`), while the
bootstrap path uses `incus`. On a host that only has Incus, those legacy calls
fail and are ignored; container creation still happens through the Incus path.
Do not read the file name as support for LXC.

## Migration

```bash
# host → container
pg_dump botserver > /tmp/backup.sql
./botserver --container
incus exec default-tables -- psql -U gbuser botserver < /tmp/backup.sql

# container → host
incus exec default-tables -- pg_dump -U gbuser botserver > /tmp/backup.sql
./botserver uninstall tables
./botserver install tables --local
psql -U gbuser botserver < /tmp/backup.sql
```

## "Brother mode"

To let `botserver` manage Incus on the host from inside a container, expose the
host socket as a proxy device:

```bash
incus config device add default incus-sock proxy \
  connect=unix:/var/lib/incus/unix.socket \
  listen=unix:/tmp/incus.sock \
  bind=container uid=0 gid=0 mode=0660
```

## See also

- [Installation](../01-getting-started/installation.md)
- [Docker Deployment](./docker-deployment.md) — experimental, no Dockerfile
- [Infrastructure Design](./infrastructure.md)
