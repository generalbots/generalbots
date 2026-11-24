# OS graphics / VA-API runtime profile slice (#2481)

This directory holds the **runtime** half of the #2481 graphics work. The
**image** half already lives in `../mkosi.conf` (`Packages=` pins Mesa
GL/EGL/GBM, VA-API/VDPAU/Vulkan userspace, `vainfo`/`vdpauinfo`, Intel/AMD VA
drivers, firmware blobs and `gstreamer1.0-vaapi`). This slice chooses *which*
of those drivers a particular host actually needs, applies the matching
WebKitGTK/GBM environment, and records the software fallback explicitly.

Nothing here claims hardware acceleration has been verified. All runtime /
hardware evidence (vainfo profile, CPU threshold, compositing path, battery
drain, diagnostics strings) remains CI/hardware-gated — see "What still needs
real hardware" below.

## Files

| Path | Purpose |
|------|---------|
| `extra/usr/local/libexec/gb-graphics/gpu-driver-profiles.psv` | Documented data file: architecture + driver class -> Mesa / VA / firmware packages and environment policy. |
| `extra/usr/local/libexec/gb-graphics/detect-graphics.sh` | sysfs-only detector + idempotent applier. No secrets, network, CDN or `eval`. |
| `extra/usr/lib/systemd/system/gb-graphics-detect.service` | Oneshot that runs the detector before the kiosk shell at boot. |
| `extra/usr/lib/systemd/system-preset/90-gb-graphics.preset` | Enables the unit at image build (`preset-all`). |
| `../mkosi.conf.d/45-graphics.conf` | Drop-in that ships `graphics/extra/` into the image via `ExtraTrees`. |

## Data file format (`gpu-driver-profiles.psv`)

Pipe-separated, one profile per line, `#` comments allowed. Fields:

```
ARCH|DRIVER|VENDOR_IDS|KMODS|MESA_PKGS|VA_PKGS|FW_PKGS|LIBGL_SW|LIBVA_DRIVER|DESC
```

- `ARCH` — `uname -m` normalised to `x86_64` / `aarch64`, or `any` (last resort).
- `DRIVER` — `intel`, `amd`, `nvidia`, `virt`, `software`.
- `VENDOR_IDS` — PCI vendor IDs (lower hex, no `0x`), e.g. `8086` Intel,
  `1002` AMD, `10de` NVIDIA, `1af4` virtio, `1234` QEMU, `15ad` VMware.
- `KMODS` — kernel driver names from sysfs (`i915`, `amdgpu`, `radeon`,
  `virtio_gpu`, `qxl`, `vmwgfx`, ...).
- `MESA_PKGS` / `VA_PKGS` / `FW_PKGS` — space-separated Debian package names.
- `LIBGL_SW` — `0` lets Mesa choose a hardware/virtio driver; `1` forces
  `LIBGL_ALWAYS_SOFTWARE=1` (llvmpipe/swrast).
- `LIBVA_DRIVER` — value for `LIBVA_DRIVER_NAME`, or `-` for auto-detection.
- `DESC` — human-readable label (last field, no `|`).

Selection order: exact `ARCH|DRIVER` from sysfs evidence, then `ARCH|software`,
then `any|software`.

## Detector behaviour

Evidence is read **only** from sysfs (plus `uname` and `/dev/dri`):
`/sys/bus/pci/devices/*/vendor`, `/sys/class/drm/card*/device/{vendor,device}`,
driver symlinks, `/sys/class/dmi/id/*`, `/sys/class/video4linux/video*`,
`/dev/dri/renderD*`.

Modes:

- **apply** (default): detect GPU class, install only the missing packages of
  the matched profile when `apt-get`/`apt` is available (root only), then write
  `/etc/environment.d/60-gb-graphics.conf`.
- **`--audit`**: print a sysfs-only capability report (DRM cards, render nodes,
  V4L2 devices, PCI vendors, kernel drivers, virtual-firmware hint, matched
  profile, VA-API verdict). Makes **no changes** and does not run `vainfo`.
- **`--dry-run`** (or `DRY_RUN=1`): print the exact package install and the
  rendered environment file. Makes **no changes**.
- **`GB_GRAPHICS_SKIP=1`**: exit 0 immediately with a single log line; used by
  non-target/CI hosts. A systemd drop-in can set it without editing the unit.

Idempotency: a second apply run detects already-installed packages with
`dpkg-query` and replaces the environment file only when its rendered content
differs (`cmp`). Logs go to stderr; audit output on stdout stays parseable.

Environment written for the session (`60-gb-graphics.conf` loads after the
image's `50-general-bots.conf`):

- `WEBKIT_DISABLE_COMPOSITING_MODE=0` — do not force WebKit's software path.
- `GBM_BACKEND=drm` — explicit GBM backend hint for the cage/wlroots session
  (ignored by compositors that do not read it).
- `LIBGL_ALWAYS_SOFTWARE=0|1` — hardware/virtio path vs. explicit llvmpipe.
- `LIBVA_DRIVER_NAME=<driver>` when the profile names one (Intel `iHD`, AMD
  `radeonsi`); omitted otherwise so VA auto-detection stays intact.

When no GPU, DRM card or render node is present, the detector selects the
software profile and logs an explicit fallback line, e.g.:

```
[gb-graphics] no GPU or DRM render node detected (cards=0 render_nodes=0) — falling back to SOFTWARE rendering (llvmpipe/swrast)
```

## Validation performed in the offline slice

- `bash -n` on `detect-graphics.sh`.
- `detect-graphics.sh --audit` and `--dry-run` run on the development host;
  output captured in the driver's report.
- grep for forbidden patterns (secrets, CDN URLs, `eval`, `curl`/`wget`,
  `Command::new`, `panic!`, `#![allow`).

## What still needs real hardware

These acceptance criteria cannot be satisfied in an offline/source-only pass
and are deliberately **not** claimed here:

1. `vainfo` shows the expected decode profile (H.264/HEVC) on a target laptop.
2. 1080p H.264 playback CPU stays below a documented threshold.
3. The WebKit web process reports hardware compositing (not llvmpipe) on real
   hardware; QEMU/software fallback documented and verified separately.
4. GPU/driver strings surfaced in the diagnostics bundle (#2468) — owned by
   that issue, not this slice.
5. 30-minute idle battery-drain comparison before/after on one laptop.
6. Booting the built image (QEMU first, then laptop) to confirm the unit runs
   and `60-gb-graphics.conf` reaches the WebKit session.

`llvmpipe`/`swrast` remains the documented QEMU fallback; a software result in
`--audit` is a correct, expected outcome on a VM without GPU pass-through.
