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

`.github/workflows/desktop-os-iso.yml` builds the ONLY image published as
`generalbots-os.iso`. It is a reusable workflow called by `release.yml` with
the pinned tag being released, and it can be dispatched manually with an
explicit tag. There is no `github.server_url` gate: GitHub and ALM both build
the same pinned image.

1. requires a **pinned tag** (`vMAJOR.MINOR.PATCH`) — `latest` is rejected.
   When `release.yml` calls it (that release is only published at the end),
   the tarball comes from the same run's `linux-installer` artifact and its
   name and `SHA256SUMS-linux` must match the pinned tag; manual dispatch
   downloads the stable tarball and checksums from the pinned tag's release
   assets (the optional `tarball_sha256` input is an extra pin);
2. unpacks it under `desktop/os/mkosi.extra/opt/general-bots/` — **no source
   build, binaries come from the pinned build**;
3. runs `mkosi build` inside a **privileged `debian:sid` toolchain
   container** (mkosi 27.1 + a sid tools tree; the *image* stays trixie —
   El Torito needs systemd-repart ≥ 261 and trixie ships 257); the
   privileged container lifts the host's unprivileged-userns AppArmor
   restriction;
4. names the hybrid ISO (El Torito, UEFI bootable) `generalbots-os.iso` —
   the produced system remains Debian 13 trixie;
5. runs a **REQUIRED QEMU boot smoke** (`mkosi vm`, text serial console) and
   fails unless `Started General Bots kiosk shell` appears in the log; the
   log is uploaded as `vm-smoke.log` either way;
6. asserts with `scripts/build_iso.sh --verify-appliance` that the image
   really is the OS: `/etc/os-release` (`ID=general-bots-os`),
   `/opt/general-bots/start.sh` and
   `/usr/lib/systemd/system/generalbots-shell.service` are read from inside
   the root filesystem, not from a directory listing;
7. uploads `generalbots-os.iso`, `SHA256SUMS-os` and `vm-smoke.log` as the
   `generalbots-os-appliance` artifact. `release.yml` attaches the ISO and
   its checksum file to the pinned release, next to
   `generalbots-payload.iso` — a GRUB data medium built by
   `scripts/build_iso.sh` that is explicitly **not** a bootable OS.

To assert the identity of a built appliance locally (needs `guestfish` from
`libguestfs-tools`):

```bash
scripts/build_iso.sh --verify-appliance dist/generalbots-os.iso
```

The carrier has its own checker (`scripts/build_iso.sh --verify
dist/generalbots-payload.iso`); it cannot pass the appliance assertions
because it has no root filesystem at all.

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

#### Secret-store boundary (#2498)

The Vault root token, unseal keys and scoped sidecar token no longer live in the
kiosk-owned stack. Provisioning keeps the root token in memory (init only),
creates a scoped `gb-sidecar` Vault token and stores it with the unseal keys in
`/etc/generalbots/credentials` — `root:gbsvc`, mode `0750`. The kiosk user
(`gbdesk`) has no access to that directory; only the service account and root
can. `.env` contains a `VAULT_TOKEN_FILE` reference instead of a token value,
and Vault is reached over TLS with a client certificate rather than plain HTTP.

The root-owned helper `/usr/local/libexec/gb-secrets` is the only way to write
or read the store (sudoers grants it to `gbsvc` only). It confines source and
destination paths to `/tmp/gb-secret-*`, and logs an `authpriv.warn` audit event
whenever an open is attempted by a UID other than the sidecar or root. Legacy
`vault-unseal-keys` and `admin-credentials.json` files are removed from the
appliance at image build and on boot; `.env`/`init.json` are generated without
`VAULT_TOKEN=`, `root_token` or unseal-key literals. A build gate in
`mkosi.finalize.chroot` and a unit test in `installer_vault/secret_store_tests.rs`
fail if those literals reappear.

The systemd unit that runs the sidecar as `gbsvc` is a separate change owned by
the OS-shell workstream; this image only provisions the account and the
credential directory it needs.

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
| Graphics/compositing (#2481, offline slice) | Mesa userspace (`libgl1-mesa-dri`, EGL/GBM), VA-API/VDPAU/Vulkan stacks (`mesa-va-drivers`, `mesa-vdpau-drivers`, `mesa-vulkan-drivers`, `intel-media-va-driver`, `i965-va-driver`, `gstreamer1.0-vaapi`, `vainfo`/`vdpauinfo`) and common Intel/AMD firmware blobs so WebKitGTK/media can use hardware acceleration on supported laptops; `llvmpipe` remains the documented QEMU/software fallback. Runtime/hardware verification (vainfo profiles, CPU threshold, compositing path, battery comparison, diagnostics strings) is CI/hardware-gated follow-up. |
| Firmware/power/memory (#2464, offline slice) | `fwupd` (device firmware), `power-profiles-daemon` and `thermald` (profiles/thermal), `intel-microcode`/`amd64-microcode` (early CPU updates via initramfs when supported), `systemd-zram-generator` with `/etc/systemd/zram-generator.conf` (RAM-sized compressed swap) and `systemd-oomd` with slice/service drop-ins that prefer reclaiming workloads before the kiosk shell/compositor. Settings/Firmware UI, tray power-profile selector, diagnostics telemetry, microcode/firmware status UI, and 4 GB soak/runtime verification are CI/hardware-gated follow-ups. |
| Peripherals (#2462, offline slice) | Bluetooth stack (`bluez`), printing/discovery groundwork (`cups`, `cups-client`, `cups-filters`, `printer-driver-all`, `avahi-daemon`, `libnss-mdns`) and input/layout tooling (`libinput-tools`, `xkb-data`) installed; `bluetooth.service`, `cups.service`, and `avahi-daemon.service` enabled in finalize. Touchpad Settings/live preview, tray BT/pairing UI, print dialogs/app integration, runtime layout switching/Fn-key handling, and hardware tests remain deferred. |
| OS accessibility (#2469, offline slice) | AT-SPI, screen reader and on-screen keyboard packages (`at-spi2-core`, `orca`, `squeekboard`) installed as groundwork. Lock/wizard/on-demand OSK integration, shell screen-reader wiring, magnifier/high-contrast/sticky-slow-bounce keys runtime behavior/persistence, and screen-recorded verification remain deferred. |
| Display/sensors (#2456, offline slice) | `brightnessctl`, `ddcutil`, and `iio-sensor-proxy` installed/enabled as groundwork for backlight/external-display/rotation handling. Settings UI, hot-plug/output arrangement, night-light, saved brightness-at-boot, and hardware/QEMU verification remain deferred. |
| Data at rest / recovery (#2470, honest state) | **Not implemented in this image recipe:** no LUKS mapping, no TPM-sealed disk key, no `systemd-cryptenroll` provisioning flow, and no scheduled snapshot/backup or "Export my data"/"Reset this PC" implementation. Previous false claims were removed from `mkosi.conf.d/20-secure-boot.conf`; implementation and verification remain follow-up work. |
| Time/locale | `systemd-timesyncd`, `tzdata`, `locales`, `console-setup`, `keyboard-configuration`; `/etc/locale.gen`, `/etc/default/keyboard`, `/etc/timezone` written at build time (#2458) |
| Removable media | `udisks2` + `gvfs` + `exfatprogs`/`ntfs-3g`/`dosfstools` (#2460) |
| Session | `dbus-user-session`; `/usr/local/bin/gb-session` waits for the user manager's bus before exec'ing cage; user services (PipeWire, portals, gvfs) enabled via `default.target.wants` (#2449) |
| Sandbox | `bubblewrap` + `libseccomp2`, `WEBKIT_FORCE_SANDBOX=1`, `LIBSEAT_BACKEND=logind` (#2500) |
| Power | `/usr/local/libexec/gb-power` (fixed action list) + `sudoers.d/gb-power`, logind `HandlePowerKey=ignore`, polkit rule denying logind/NetworkManager power and network actions to `gbdesk` (#2444, #2502) |


## Secure Boot boot failure fallback (#2478, offline docs slice)

The current appliance image is **unsigned** (`SecureBoot=no`,
`UnifiedKernelImages=unsigned`). On laptops with Secure Boot enabled by default,
firmware may refuse to boot it. Until signed boot artifacts and release
enforcement land, the documented fallback is:

1. Power off the machine and open firmware/UEFI setup.
2. Find **Secure Boot** and set it to **Disabled** (or clear/unenroll keys only
   if your firmware requires that to disable enforcement — prefer the explicit
   Disable toggle when available).
3. Save changes and boot the General Bots OS medium again.

Common firmware keys/menu names (vary by vendor/model/firmware revision):

| Vendor / firmware | Setup key(s) | Secure Boot location (typical names) |
|---|---|---|
| Dell | `F2` (Setup), `F12` (Boot menu) | `Boot Configuration` / `Secure Boot` / `Secure Boot Enable` |
| HP | `Esc` then `F10` (BIOS Setup) | `Security` / `Secure Boot Configuration` / `Secure Boot` |
| Lenovo (ThinkPad/IdeaPad) | `F1` or `Enter`+`F1` (ThinkPad), `F2`/`Fn+F2` (IdeaPad) | `Security` / `Secure Boot` |
| ASUS | `F2` or `Del` | `Boot` or `Security` / `Secure Boot` / `OS Type` (`Other OS`) |
| Acer | `F2` (sometimes requires Supervisor password first) | `Security` / `Secure Boot` / `Set Supervisor Password` then disable |
| MSI | `Del` | `Settings` / `Security` / `Secure Boot` |
| Microsoft Surface | hold `Volume Up` while pressing Power | `Security` / `Secure Boot` / change certificate/profile |
| Generic UEFI/AMI/Award/Phoenix | `Del`, `F2`, `Esc`, `F10`, or `F12` | `Boot`, `Security`, or `Authentication` / `Secure Boot` |

Notes: keep UEFI mode enabled unless your hardware requires legacy/CSM for a
separate reason; do not enable CSM as part of this fallback. This is a
temporary workaround for the unsigned development/current image, not a security
recommendation. Signed UKI/bootloader, MOK enrolment for unsigned-developer
flows, release-build key enforcement, QEMU/OVMF Secure Boot CI tests, and
recovery-screen integration are tracked/follow-up work (#2478, #2468).

## Next hardening steps (mkosi supports all of them)

1. Signed boot chain (Secure Boot UKI/bootloader) — tracked by #2478. When
   implemented and verified, at-rest encryption with documented unlock/recovery
   (LUKS/TPM) — tracked by #2470. **Not implemented in this image recipe yet**
   (no signed UKI, no LUKS mapping, no TPM-sealed root/data flow).
2. A/B or slot-based OS updates with rollback — tracked by #2443.
3. Drop in the vendored installers (`installers/` in the tarball directory)
   for an offline-first appliance that needs no internet on first boot —
   tracked by #2441.

The "newest release carrying the tarball" resolution is gone: CI now requires
a pinned tag plus the checksums of that same pin (see the build steps above).

The "newest release carrying the tarball" resolution is gone: CI now requires
a pinned tag plus the checksums of that same pin (see step 1 above).
