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

## See also

- [Local LLM with llama.cpp](./local-llm.md)
- [Desktop & Workstation Hardware](./desktop-hardware.md)
- [Installation](../01-getting-started/installation.md)
