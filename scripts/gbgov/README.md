# General Bots Lite agent runtime (Agent mode on Windows)

Build scripts for the files the desktop app downloads to run Agent mode on
Windows 10 (19041+) and 11. The workflow `.github/workflows/gbgov-agent-runtime.yml`
runs them on `ubuntu-latest` and publishes the results as assets of a
GitHub pre-release.

| File | What it is | Built by |
|------|------------|----------|
| `GeneralBotsLite-agent-rootfs-<v>-x86_64.tar.gz` | WSL2 root filesystem of the distro `GeneralBotsLite` | `build_wsl_rootfs.sh` |
| `GeneralBotsLite-agent-workspace-office-<v>-x86_64.tar.xz` | Incus unified image, alias `gb-workspace-office` | `../build_vm_image.sh --profile office --export` |
| `agent-runtime-manifest.json` | versions, sizes, SHA-256, image fingerprint | `agent_runtime_manifest.py merge` |
| `SHA256SUMS` | the same checksums for `sha256sum -c` (covers the manifest too) | `agent_runtime_manifest.py merge` |

## Releases

- `git tag gbgov-agent-runtime-v1.0.0 && git push origin gbgov-agent-runtime-v1.0.0`
  builds and publishes the pre-release `gbgov-agent-runtime-v1.0.0`. It is
  never marked Latest, and an existing release is never overwritten: a fix
  ships as a new version.
- A push to the branch `gbgov-agent-runtime-dryrun` builds the same files
  (version `0.0.0-dryrun.<sha7>`) and keeps them as workflow artifacts
  (`agent-runtime-rootfs`, `agent-runtime-workspace`, `agent-runtime-manifest`)
  for 30 days. Nothing is published.

The manifest URL of a release is
`https://github.com/generalbots/generalbots/releases/download/gbgov-agent-runtime-v<v>/agent-runtime-manifest.json`,
and every asset sits next to it under its `file` name.

## The WSL root filesystem

Debian 13 (trixie), `minbase` plus systemd, Incus (`incus-base` +
`incus-client` from Debian main, containers only, no QEMU), dnsmasq, nftables,
lxcfs and ca-certificates. Packages come from one snapshot.debian.org
timestamp (`DEFAULT_SNAPSHOT` in `build_wsl_rootfs.sh`); `SOURCE_DATE_EPOCH`
is that timestamp, so the same inputs give the same tarball. After import the
distro's apt points at deb.debian.org (trixie, trixie-updates,
trixie-security).

Inside the distro:

- `/etc/wsl.conf`: `systemd=true`, default user `root`,
  `appendWindowsPath=false`, automount on (the app imports the workspace
  image from `/mnt/c/...`).
- `gb-incus-firstboot.service` runs once:
  `incus admin init --preseed` with
  `/usr/local/share/generalbots/incus-preseed.yaml` (storage pool `default`
  on the `dir` driver, bridge `gblbr0` with IPv4 NAT and no IPv6, the
  default profile on both, no `core.https_address`, so the Incus API is only
  on the local unix socket). The bridge is named `gblbr0` instead of the
  Incus default `incusbr0` so the product does not collide with another Incus
  in a sibling WSL distro (all WSL2 distros share one kernel and network
  namespace). It then disables itself.
- `/var/lib/generalbots/incus-firstboot.status` holds one line:
  `running`, `ok` or `failed: <reason>`. After `ok` the file
  `/var/lib/generalbots/incus-firstboot.done` exists. A failure keeps the unit
  enabled; `systemctl start gb-incus-firstboot` retries.
- `/usr/lib/generalbots/agent-runtime-release` (KEY=value):
  `GB_AGENT_RUNTIME_VERSION`, `GB_DEBIAN_SUITE`, `GB_DEBIAN_SNAPSHOT`,
  `GB_INCUS_VERSION`. The app compares the version with the manifest to offer
  an update.

The tarball is gzip because every `wsl --import` accepts it; xz and zstd are
not accepted by older WSL releases.

## The workspace image

`scripts/build_vm_image.sh --profile office` launches `images:debian/13`
(pinned by fingerprint for the build), installs the office profile from
`botserver/crates/botagent/src/vm/profile/mod.rs`, creates the `runner` and
`claude` users, writes the marker `v7-office` to `/opt/vibe/.provisioned`,
clears caches, logs and the machine id, publishes it as `gb-workspace-office`
and exports it as one unified tarball (xz). Its Incus fingerprint is the
tarball's SHA-256. Agent CLIs (Claude Code, Codex, Antigravity) are not in the
image; they install from npm or their installers at first use, as on Linux.

Without the static Chromium cache (`chromium-headless.tar.zst`) the build
uses Debian's `chromium` package, roughly 230 MB larger installed. The
manifest's `chromium` field says which one an image carries.

Import on the Windows side (inside `GeneralBotsLite`):
`incus image import /mnt/c/.../GeneralBotsLite-agent-workspace-office-<v>-x86_64.tar.xz --alias gb-workspace-office`,
then check that `incus image info gb-workspace-office` reports the manifest's
`incus_fingerprint`.

## Manifest schema (`schema_version` 1)

`agent_runtime_manifest.py` (`validate()`) is the executable form of this
schema; `--self-test` exercises it.

```json
{
  "schema_version": 1,
  "runtime_version": "1.0.0",
  "release_tag": "gbgov-agent-runtime-v1.0.0",
  "commit": "<40 hex>",
  "built_at": "2026-11-26T12:00:00Z",
  "debian_snapshot": "20261120T000000Z",
  "assets": {
    "wsl_rootfs": {
      "kind": "wsl-rootfs",
      "file": "GeneralBotsLite-agent-rootfs-1.0.0-x86_64.tar.gz",
      "size": 123456789,
      "sha256": "<64 hex>",
      "arch": "x86_64",
      "distro_name": "GeneralBotsLite",
      "compression": "gzip",
      "debian_suite": "trixie",
      "debian_snapshot": "20261120T000000Z",
      "incus_version": "6.0.4-2+deb13u12",
      "systemd_version": "<dpkg version>",
      "installed_bytes": 345678901,
      "first_boot_unit": "gb-incus-firstboot.service",
      "first_boot_status": "/var/lib/generalbots/incus-firstboot.status"
    },
    "workspace_image": {
      "kind": "incus-image",
      "file": "GeneralBotsLite-agent-workspace-office-1.0.0-x86_64.tar.xz",
      "size": 345678901,
      "sha256": "<64 hex>",
      "arch": "x86_64",
      "format": "unified",
      "compression": "xz",
      "incus_fingerprint": "<64 hex, equal to sha256>",
      "alias": "gb-workspace-office",
      "profile": "office",
      "provisioned_marker": "v7-office",
      "base_image": "images:debian/13",
      "base_image_fingerprint": "<64 hex>",
      "chromium": "distro",
      "builder_incus_version": "6.0.0"
    }
  }
}
```

Required, and checked by `validate()`:

| Field | Rule |
|-------|------|
| `schema_version` | `1`. A breaking change bumps it; the app refuses a version it does not know. |
| `runtime_version` | `major.minor.patch[-prerelease]`, the tag without `gbgov-agent-runtime-v`. Dry runs use `0.0.0-dryrun.<sha7>`. |
| `release_tag` | the tag; empty string for a dry run. |
| `commit` | 40 lowercase hex. |
| `built_at` | commit time, UTC, `YYYY-MM-DDTHH:MM:SSZ`. |
| `debian_snapshot` | `YYYYMMDDTHHMMSSZ`. |
| `assets` | object keyed by role; `wsl_rootfs` and `workspace_image` are both required. Unknown roles are ignored. |
| `assets.*.kind` | `wsl-rootfs` / `incus-image`. |
| `assets.*.file` | plain file name (`[A-Za-z0-9][A-Za-z0-9._+-]*`), no path; resolve it next to the manifest. |
| `assets.*.size` | bytes, positive integer. |
| `assets.*.sha256` | 64 lowercase hex. |
| `assets.*.arch` | `x86_64` (the only architecture built today). |
| `wsl_rootfs.distro_name` | `GeneralBotsLite`. |
| `wsl_rootfs.compression` | `gzip`. |
| `wsl_rootfs.debian_suite`, `incus_version` | non-empty strings. |
| `workspace_image.format` | `unified`. |
| `workspace_image.incus_fingerprint` | 64 lowercase hex, equal to `sha256`. |
| `workspace_image.alias`, `profile`, `provisioned_marker`, `compression` | non-empty strings. |

Every other field is informational: readers ignore fields they do not know,
and new informational fields do not bump `schema_version`.

How the app should use it: download the manifest from the pinned release
tag; for each asset download `file`, check `size` and then `sha256` before
using it (resume a partial download, never import an unverified file); after
`incus image import`, confirm `incus_fingerprint`. `SHA256SUMS` carries the
same digests plus the manifest's own, for people verifying by hand.

## Local checks (no WSL, Incus or Docker needed)

```sh
bash -n scripts/gbgov/build_wsl_rootfs.sh scripts/build_vm_image.sh
python3 scripts/gbgov/agent_runtime_manifest.py --self-test
bash scripts/build_vm_image.sh --profile office --verify
bash scripts/gbgov/build_wsl_rootfs.sh --version 1.0.0 --out dist/rootfs.tar.gz --dry-run
```
