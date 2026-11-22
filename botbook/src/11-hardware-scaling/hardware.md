# Supported Platforms 🟡 BETA

"Supported" here means one thing only: `botserver/3rdparty.toml` publishes a
downloadable binary for that platform and the package manager can install it.
Anything without a component entry is unverified — do not assume it works.

## Server platforms

| Platform | Local LLM (`llm*`) | Drive (`drive`, `mc`) | Notes |
|---|---|---|---|
| Linux x86_64 | `llama-b7345-bin-ubuntu-x64.zip`, Vulkan variant available | ✅ | primary CI and production target |
| Linux arm64 | `llama-b12189-bin-ubuntu-arm64.tar.gz` | ✅ | arm64 uses a newer llama.cpp release; its checksum is not published upstream, so verification is skipped for this variant |
| Linux s390x | `llama-b7345-bin-ubuntu-s390x.zip` | ✅ | binary only; not exercised by CI |
| macOS arm64 | `llama-b7345-bin-macos-arm64.zip` | ✅ | desktop build path (`desktop/macos/`) |
| macOS x64 | `llama-b7345-bin-macos-x64.zip` | ✅ | Intel Macs |
| Windows x64 | CPU, CUDA 12.4, CUDA 13.1 and Vulkan variants | ✅ | `cudart_win_cuda*` ships the CUDA runtime |

## Single-board computers

There is **no SBC support statement in this repository**. A Raspberry Pi or
similar board is not a documented target: no component entry names one, and no
CI job builds for it. An arm64 Linux host can install the arm64 components, but
nothing here measures whether it serves a bot acceptably. Treat SBC deployment
as unsupported until a component and a build exist.

## What the installer needs

The provisioner verifies a SHA-256 per component after resolving the archive
(`botserver/3rdparty.toml`). Offline installs must ship the archives in the
cache directory it reads (`BOTSERVER_INSTALLERS_PATH`, default
`./botserver-installers/`); network downloads are refused when the
`desktop-local` feature is active.

## Host requirements

| Requirement | Why |
|---|---|
| 64-bit CPU | every published binary is 64-bit |
| ~4.5 GiB free RAM for a debug build | the workspace graph is large; `rustc` is killed below that |
| glibc-based Linux (deb/rpm/AppImage) | the desktop bundles target glibc; musl is not built |
| `libwebkit2gtk-4.1-0`, `libpq5`, OpenSSL 3 | desktop app runtime dependencies |

## Graphics support in the support bundle (#2481)

Settings → System → Diagnostics produces a support bundle. The bundle runs
the local probe helpers (`lspci -nn`, `vainfo` and `glxinfo -B`), redacts the
output and stores it in the `graphics` object. `graphics_support` is a pure
classification of those same strings (`graphics_support_status()` in
`botserver/crates/botsettings/src/os_appliance.rs`): the classifier itself
does no probing, contacts no GPU and measures no frame rate. A bundle value
describes only the host that produced it; it is not a measurement of any
other machine.

| Field | Values | Meaning |
|---|---|---|
| `graphics.gpu`, `graphics.vaapi`, `graphics.glx` | string or `null` | redacted `lspci -nn`, `vainfo` and `glxinfo -B` text; `null` means the probe tool is unavailable, failed or returned no output |
| `graphics_support.compositing` | `hardware`, `software`, `unknown` | `unknown` when no GLX text was collected (tool absent or empty output); `software` when the GLX text contains `llvmpipe`, `softpipe`, `swrast` or `software rasterizer` (case-insensitive); `hardware` otherwise |
| `graphics_support.software_rendering` | boolean | true when a software rasterizer marker appears in the GLX text; uses the same marker test as `compositing` |
| `graphics_support.gpu_detected` | boolean | true when the GPU probe text contains a line naming a VGA, Display or 3D controller; false when the probe is absent |
| `graphics_support.vaapi.available` | boolean | true when at least one `VAProfile*` line with a `VLD` or `ENC` entrypoint was parsed |
| `graphics_support.vaapi.decode_profiles` | array of strings | sorted, de-duplicated `VAProfile*` names whose entrypoint contains `VLD` (normally `VAEntrypointVLD`) |
| `graphics_support.vaapi.encode_profiles` | array of strings | sorted, de-duplicated `VAProfile*` names whose entrypoint contains `ENC` (normally `VAEntrypointEncSlice`) |
| `graphics_support.note` | string | states that compositing throughput and decode speed are not measured offline |

### Reading a `software` result

`compositing = "software"` with `software_rendering = true` is the expected
result inside QEMU or on a host without a DRM driver: Mesa falls back to the
llvmpipe software rasterizer. That is not evidence of a broken image and says
nothing about whether a different, real laptop has working hardware
acceleration. Conversely `compositing = "hardware"` only means the collected
GLX text named no software rasterizer; it is not a frame-rate measurement and
does not confirm VA-API decode on that host.

### Image manifest gate

`scripts/check_graphics_support.py` parses the `Packages=` block of
`desktop/os/mkosi.conf` and fails when the Mesa userspace, VA-API runtime, an
Intel VA driver, AMD/misc firmware or the graphics diagnostics helpers are
absent. It validates the manifest only; it does not boot the image.

### What still needs hardware

These acceptance steps cannot be satisfied by a text classifier or a
package-list check, and no offline pass reports them as verified: VA-API
decode profiles on the target laptop, the 1080p H.264 CPU threshold, the
WebKit hardware-compositing path, the idle battery-drain comparison, and the
actual image build, boot and runtime validation.

## See also

- [Local LLM with llama.cpp](./local-llm.md)
- [Desktop & Workstation Hardware](./desktop-hardware.md)
- [Installation](../01-getting-started/installation.md)
