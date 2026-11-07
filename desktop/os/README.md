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
| `mkosi.conf` | Debian trixie appliance: GPT disk image, bootable, 16G grow-on-boot |
| `mkosi.conf.d/10-eltorito.conf` | auto-upgrades the artifact to a **real hybrid .iso** (El Torito) when the build env's systemd-repart is ≥ 261; skipped cleanly otherwise |
| `mkosi.extra/etc/sysusers.d/generalbots-kiosk.conf` | creates `gbdesk` (nologin, video/render groups) |
| `mkosi.extra/usr/lib/systemd/system/generalbots-shell.service` | the OS "shell": dbus run-session → cage → `/usr/bin/generalbots` |
| `mkosi.finalize.chroot` | masks `getty@tty1` + rescue/emergency, enables the service units, resolves the shell binary path defensively |
| `deb/` | CI drops the release `.deb` here (never committed, see `.gitignore`) |

## CI

`.github/workflows/desktop-os-iso.yml`:

1. downloads `generalbots-linux-x86_64.deb` from a GitHub release (`latest` or a tag) — **no source build, binaries come from the release channel**;
2. runs `mkosi build` inside a **privileged `debian:trixie` container** (mkosi 27.1 straight from Debian's repos, full toolchain self-consistent; the privileged container lifts the host's unprivileged-userns AppArmor restriction);
3. names the artifact `generalbots-os.iso` (hybrid ISO via El Torito, when trixie's systemd-repart ≥ 261 — the current case) or `generalbots-os.img` (UEFI bootable GPT image otherwise), brackets to the same gate as `mkosi.conf.d/10-eltorito.conf`;
4. runs the best-effort **QEMU boot smoke** (`mkosi vm`, text serial console — logs kernel, systemd journal and the kiosk shell startup; never blocks publication, log always uploaded);
5. publishes the image plus `SHA256SUMS-os` as workflow artifacts.

## Install / run

```bash
# from a release artifact — hybrid ISO when the build env had systemd ≥ 261:
dd if=generalbots-os.iso of=/dev/sdX bs=4M oflag=direct status=progress
# older build env → GPT disk image, same content:
dd if=generalbots-os.img of=/dev/sdX bs=4M oflag=direct status=progress
# or boot it in a VM without writing to a real disk:
qemu-system-x86_64 -m 4G -drive file=generalbots-os.iso,format=raw \
  -bios /usr/share/ovmf/OVMF.fd
```

First boot behaves exactly like the desktop edition's first run: the setup
wizard provisions the service stack (PostgreSQL, Valkey, MinIO, Qdrant,
Vault, Zitadel, llama.cpp) into `/var/lib/gbdesk` — internet required on
the first boot; the wizard then opens the suite fullscreen (the kiosk
shell). A crash respawns the shell; there is no console login on the box.

## Next hardening steps (mkosi supports all of them)

1. `--secure-boot` signed kernels + TPM 2.0 LUKS (IncusOS-style A/B updates
   via systemd-sysupdate once the image ships as a real kernel).
2. Swap the `deb/` fetch for a **pinned tag** + checksum for reproducible
   image builds.
3. `mkosi.burn`/`mkosiios` wrappers for real appliances.
