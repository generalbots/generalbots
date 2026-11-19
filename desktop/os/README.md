# General Bots OS — kiosk appliance recipe + CI

This folder turns the **desktop edition into the whole operating system**:
a Debian 13 (trixie) image where the General Bots desktop shell (botapp +
botserver sidecar) is the only user interface. Built with **mkosi** — the
same tool IncusOS is built with.

## Design: kiosk option B (the secure one)

When comparing the two shell-replacement options, option **B** won:

| | A — display manager + dedicated X session | **B — no DM/X, cage kiosk** (this recipe) |
|---|---|---|
| Components | GDM/LightDM + Xorg + GNOME deps | systemd + **cage** (Wayland) + the GB app |
| Login/greeter | autologin X session still passes a DM | none — tty1 is owned by `generalbots-shell.service` |
| User shell | normal user account | **`gbdesk` system account with `nologin`** |
| Recovery shells | rescue/emergency available on console | **masked** — recovery = reprovision from a fresh image |
| Shell attack surface | X clients can grab/XTEST each other; multi-DE stack | one Wayland compositor, one known client, no ssh server, no terminal emulator |
| Crash model | DM respawns session | `Restart=always` — same semantics, fewer processes |

## Files (the build key artifacts)

| File | Purpose |
|---|---|
| `mkosi.conf` | Debian trixie appliance: hybrid ISO (El Torito), bootable, 16G grow-on-boot; build-time tools tree pinned to Debian sid for repart ≥ 261 |
| `mkosi.extra/opt/general-bots/` | the release tarball payload, unpacked by CI (gitignored) |
| `mkosi.extra/etc/sysusers.d/generalbots-kiosk.conf` | creates `gbdesk` (nologin, video/render groups) |
| `mkosi.extra/usr/lib/systemd/system/generalbots-shell.service` | the OS "shell": dbus run-session → cage → `/opt/general-bots/start.sh` |
| `mkosi.finalize.chroot` | validates the payload, masks `getty@tty1` + rescue/emergency, enables the units, creates the kiosk state dir |

Why the **tarball** and not the `.deb`: the distro package step in
`desktop-binaries.yml` is `continue-on-error` and has never produced a
`.deb`/`.rpm`/`.AppImage` asset (checked across all releases); the tarball
`generalbots-linux-x86_64.tar.gz` is the artifact that always exists.

## CI

`.github/workflows/desktop-os-iso.yml`:

1. resolves the newest release that actually carries
   `generalbots-linux-x86_64.tar.gz` (`latest` or an explicit tag) and
   unpacks it under `desktop/os/mkosi.extra/opt/general-bots/` — **no source
   build, binaries come from the release channel**;
2. runs `mkosi build` inside a **privileged `debian:sid` toolchain
   container** (mkosi 27.1 + a sid tools tree; the *image* stays trixie —
   El Torito needs systemd-repart ≥ 261 and trixie ships 257); the
   privileged container lifts the host's unprivileged-userns AppArmor
   restriction;
3. names the artifact `generalbots-os.iso` — a hybrid ISO via El Torito
   (mkosi 27.1 from the Debian pool plus a Debian **sid** build-time tools
   tree, since El Torito needs systemd-repart ≥ 261 and trixie ships 257;
   the produced system remains Debian 13 trixie);
4. runs the best-effort **QEMU boot smoke** (`mkosi vm`, text serial console
   — logs kernel, systemd journal and the kiosk shell startup; never blocks
   publication, log always uploaded);
5. publishes the image plus `SHA256SUMS-os` as workflow artifacts.

## Install / run

```bash
# from the artifact (hybrid ISO, UEFI bootable):
dd if=generalbots-os.iso of=/dev/sdX bs=4M oflag=direct status=progress
# or boot it in a VM without writing to a real disk:
qemu-system-x86_64 -m 4G -drive file=generalbots-os.iso,format=raw \
  -bios /usr/share/ovmf/OVMF.fd
```

First boot behaves exactly like the desktop edition's first run: the setup
wizard provisions the service stack (PostgreSQL, Valkey, MinIO, Qdrant,
Vault, Zitadel, llama.cpp) into `/var/lib/gbdesk` — internet required on
the first boot; the wizard then opens the suite fullscreen (the kiosk
shell). A crash respawns the shell; there is no console login on the box.

## Branding

The official `logo.svg` (repo root) ships in the image and is used on every
OS screen that can show a brand:

| Screen | Artifact |
|---|---|
| systemd-boot / UKI splash (boot menu + early boot) | `Splash=/usr/share/generalbots/logo.png` (white knockout so it reads on the black boot screen), `UnifiedKernelImages=unsigned` |
| Plymouth boot splash (root switch → kiosk shell) | theme `generalbots` under `/usr/share/plymouth/themes/generalbots/` (script + logo + progress bar), selected via `/etc/plymouth/plymouthd.conf`; `splash` on the kernel command line |
| ISO 9660 volume label (firmware boot menus) | `ElToritoVolume=GeneralBotsOS` |
| OS identity (`hostnamectl`, settings, icon lookups) | branded `/etc/os-release` + `/usr/share/pixmaps/general-bots.svg` (`LOGO=general-bots`) |
| Suite / About window | shipped `logo.svg` served from `/usr/share/generalbots/` (consumers can reference it; see issue #1842 for the in-app artwork pass) |

The PNGs are exported 512 px from the same SVG, with the white variant
produced by recoloring (alpha preserved) — no third-party art.

## Service hardening (`generalbots-shell.service`)

The kiosk is started by `generalbots-shell.service` (installed from
`mkosi.extra/usr/lib/systemd/system/generalbots-shell.service`). Hardening is
applied there with inline comments in the unit; this section is the human
reference and the place to re-check regressions.

### Directive table (intent and notes)

| Directive | Value / effect | Notes / exceptions |
|---|---|---|
| `NoNewPrivileges=` | `yes` | No setuid/setgid privilege gain. |
| `RestrictSUIDSGID=` | `yes` | No new SUID/SGID files in the unit’s context. |
| `CapabilityBoundingSet=` | empty | All capabilities dropped (service runs as unprivileged `gbdesk`). |
| `ProtectSystem=` | `strict` | `/usr`, `/boot`, `/etc` read-only; writes only under `ReadWritePaths`. |
| `ReadWritePaths=` | `/var/lib/gbdesk`, `/run/user/%U` | Mutable stack/state under `/var/lib/gbdesk`; session runtime dir for XDG/Wayland/dbus artifacts. |
| `PrivateTmp=` | `yes` | Isolated `/tmp`, `/var/tmp`. |
| `ProtectHome=` | `tmpfs` | Hides `/home`, `/root`, `/run/user` behind tmpfs; `BindPaths=/run/user/%U` restores the user runtime dir. `HOME=/var/lib/gbdesk` stays outside `/home`. |
| `BindPaths=` | `/run/user/%U` | Required for `XDG_RUNTIME_DIR`, Wayland sockets, libseat/logind session state, and `dbus-run-session` artifacts. |
| `PrivateDevices=` | `yes` | Most device nodes hidden. |
| `DevicePolicy=` | `closed` | Only explicitly allowed devices. |
| `DeviceAllow=` | `char-drm rw`, `/dev/dri rw`, `/dev/dri/card0 rw`, `/dev/dri/renderD128 rw`, `/dev/tty0 rw`, `/dev/tty1 rw` | **DRM/render (exception):** Wayland compositor (`cage`/wlroots) and GPU accel need DRM/KMS nodes; `gbdesk` is in `video`/`render` groups (`sysusers`). Adjust device names if hardware differs (e.g. other `renderD*`). |
| `ProtectKernelTunables=` | `yes` | No writes to `/proc/sys`, `/sys` tunables. |
| `ProtectKernelModules=` | `yes` | No loading modules. |
| `ProtectKernelLogs=` | `yes` | No access to kernel ring buffer. |
| `ProtectControlGroups=` | `yes` | cgroup tree read-only. |
| `ProtectClock=` | `yes` | Clock changes blocked. |
| `ProtectHostname=` | `yes` | Hostname changes blocked. |
| `RestrictNamespaces=` | `yes` | Namespace creation restricted. |
| `LockPersonality=` | `yes` | No personality changes. |
| `RestrictRealtime=` | `yes` | No realtime scheduling abuse. |
| `RestrictAddressFamilies=` | `AF_UNIX AF_INET AF_INET6 AF_NETLINK` | **Session/network exceptions:** Unix sockets (dbus, Wayland), TCP/UDP (provisioning/app traffic), netlink (routing/udev/logind-adjacent). Tighten only with measured failures and document why. |
| `SystemCallFilter=` | `@system-service` | Baseline allowlist for typical system services (cage, dbus-run-session, GTK/WebKit). May need explicit additions if tracing shows denials; record changes in git history and this table. |
| `SystemCallErrorNumber=` | `EPERM` | Denied syscalls return `EPERM` (clearer than default in practice). |
| `UMask=` | `0077` | Restrictive file creation mask. |

### Documented exceptions (do not remove without a replacement plan)

1. **DRM/render devices for the Wayland compositor (cage/wlroots) and GPU acceleration.**  
   Required: `/dev/dri/*`, `char-drm`, and the VT nodes used by the session (`/dev/tty0`, `/dev/tty1`). Removing these prevents graphical output.
2. **Session bus / runtime directory.**  
   `pam_systemd` + `logind` provide `/run/user/%U` (`XDG_RUNTIME_DIR`). `dbus-run-session` runs a session bus for the launcher and children; GTK/WebKit and Wayland clients depend on it. The unit bind-mounts `/run/user/%U` back under `ProtectHome=tmpfs`.
3. **WebKit JIT (no `MemoryDenyWriteExecute`).**  
   WebKitGTK uses JIT and requires writable+executable memory in practice (W^X trade-off). `MemoryDenyWriteExecute=yes` is **not** set. If the webview moves to a sandboxed helper with a JIT-free path, consider enabling MDE in that narrower domain instead.
4. **Network access allowed (`PrivateNetwork` not set).**  
   First-boot provisioning and normal app traffic need network (subject to offline flags elsewhere). Keep unless the deployment model guarantees fully offline operation and the unit is redesigned accordingly.
5. **TTY ownership.**  
   `TTYPath=/dev/tty1`, `PAMName=login`, `UtmpIdentifier=tty1` keep the kiosk VT bound to this service.

### Security domain boundary: sidecar vs webview

Today the **botserver sidecar** and the **desktop/webview UI** run in the **same security domain**: the `generalbots-shell.service` unit starts `cage` which runs `/opt/general-bots/start.sh`; that launcher starts the desktop app process (`generalbots`) which, per the desktop sidecar design, manages the botserver sidecar in the same user session/context (see `botapp/src/desktop/sidecar.rs` and `desktop/linux/start.sh`). There is **no separate systemd unit** isolating botserver from the webview process tree in this appliance image.

**Intended direction (documented):** split into distinct security domains, for example:
- a dedicated service (or hardening profile) for the botserver sidecar with its own `Protect*`, syscall/address-family policy, and no access to the user’s Wayland/DRM seat beyond what is strictly required (ideally none), and
- a separate service for the compositor + webview with DRM/session-bus access, running with tighter filesystem access.

Until that split lands, treat compromises of the webview/renderer as potentially compromising the sidecar process under the same UID/session constraints of this unit. Changes to enforce separation belong in `desktop/linux/start.sh`, `botapp/**` (sidecar lifecycle), and additional `desktop/os/**` unit files — out of scope for the current hardening-only edit except this documentation.

### Verify and re-run `systemd-analyze security`

On a booted appliance (or in the image environment after install), capture evidence:

```bash
# Show the unit’s effective properties (includes hardening fields)
systemctl show generalbots-shell.service > /var/log/generalbots-shell.systemctl-show.txt 2>&1 || true

# Score / analysis (requires systemd-analyze on the target)
systemd-analyze security generalbots-shell.service | tee /var/log/generalbots-shell.security.txt
```

If `systemd-analyze` is unavailable in the checked environment, record that in validation notes (e.g. “not present in CI container”). When present, **do not regress** past the agreed target in CI: the `desktop-os-hardening` workflow runs this check and fails if the numeric exposure score is worse than **≤ 4.0** (see `.github/workflows/desktop-os-hardening.yml`, `GB_SHELL_SECURITY_MAX_SCORE`).

To iterate locally against the unit file in the repo:

```bash
systemd-analyze security desktop/os/mkosi.extra/usr/lib/systemd/system/generalbots-shell.service
```


## Runtime stack (what the image actually provides)

The appliance is a laptop, not only a kiosk: the package list in `mkosi.conf`
and the configuration written by `mkosi.finalize.chroot` cover the surfaces a
laptop user expects. Each block names the issue it answers.

| Surface | What is in the image |
|---|---|
| Networking | NetworkManager + `wpasupplicant` + `firmware-iwlwifi` + `wireless-regdb` + `rfkill`; `systemd-networkd` disabled; DNS via `systemd-resolved` (#2453) |
| Firewall | `nftables` default-deny inbound, loaded at boot; loopback-only stack; LAN exposure is opt-in through `GB_ALLOW_LAN=1` plus the `gb_lan_allowed` set — there is no UI switch yet (#2477) |
| Sound | PipeWire + WirePlumber + `pipewire-alsa` + ALSA UCM + GStreamer plugins; `/dev/snd` allowed in the unit (#2455) |
| Time/locale | `systemd-timesyncd`, `tzdata`, `locales`, `console-setup`, `keyboard-configuration`; `/etc/locale.gen`, `/etc/default/keyboard`, `/etc/timezone` written at build time (#2458) |
| Removable media | `udisks2` + `gvfs` + `exfatprogs`/`ntfs-3g`/`dosfstools` (#2460) |
| Session | `dbus-user-session`; `/usr/local/bin/gb-session` waits for the user manager's bus before exec'ing cage; user services (PipeWire, portals, gvfs) enabled via `default.target.wants` (#2449) |
| Sandbox | `bubblewrap` + `libseccomp2`, `WEBKIT_FORCE_SANDBOX=1`, `LIBSEAT_BACKEND=logind` (#2500) |
| Power | `/usr/local/libexec/gb-power` (fixed action list) + `sudoers.d/gb-power`, logind `HandlePowerKey=ignore`, polkit rule denying logind/NetworkManager power and network actions to `gbdesk` (#2444, #2502) |

## Next hardening steps (mkosi supports all of them)

1. `--secure-boot` signed kernels + TPM 2.0 LUKS (IncusOS-style A/B updates
   via systemd-sysupdate once the image ships as a real kernel).
2. Swap the "newest release carrying the tarball" resolution for a **pinned
   tag + checksum** for reproducible image builds.
3. Drop in the vendored installers (`installers/` in the tarball directory)
   for an offline-first appliance that needs no internet on first boot.
