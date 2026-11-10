# Local LLM with llama.cpp 🟡 BETA

The local model server is a **llama.cpp** release binary fetched by the package
manager. There is no in-tree inference engine: the component is downloaded,
checksum-verified and launched as a separate process.

Authority: `botserver/3rdparty.toml` (`[components.llm*]`) and
`botserver/crates/botcore/src/package_manager/`.

## Pinned release

All variants below come from llama.cpp release **b7345**, except Linux arm64,
which uses **b12189** (b7345 publishes no `ubuntu-arm64` archive).

| Component | Platform | Archive |
|---|---|---|
| `llm` | Linux x64 (CPU) | `llama-b7345-bin-ubuntu-x64.zip` |
| `llm` (`url_arm64`) | Linux arm64 | `llama-b12189-bin-ubuntu-arm64.tar.gz` |
| `llm_linux_vulkan` | Linux x64 (Vulkan GPU) | `llama-b7345-bin-ubuntu-vulkan-x64.zip` |
| `llm_linux_s390x` | Linux s390x | `llama-b7345-bin-ubuntu-s390x.zip` |
| `llm_macos_arm64` | macOS Apple silicon | `llama-b7345-bin-macos-arm64.zip` |
| `llm_macos_x64` | macOS Intel | `llama-b7345-bin-macos-x64.zip` |
| `llm_win_cpu_x64` | Windows x64 | `llama-b7345-bin-win-cpu-x64.zip` |
| `llm_win_cpu_arm64` | Windows arm64 | `llama-b7345-bin-win-cpu-arm64.zip` |
| `llm_win_cuda12` / `llm_win_cuda13` | Windows + NVIDIA | `llama-b7345-bin-win-cuda-12.4-x64.zip`, `…-13.1-x64.zip` |
| `llm_win_vulkan` | Windows + Vulkan GPU | `llama-b7345-bin-win-vulkan-x64.zip` |
| `cudart_win_cuda12` / `cudart_win_cuda13` | Windows CUDA runtime | `cudart-llama-bin-win-cuda-*.zip` |

## Selection and fallback

`get_llama_cpp_url()` prefers the Vulkan build when the host exposes Vulkan
(`/usr/share/vulkan` or `VULKAN_SDK`) and otherwise falls back to the portable
CPU build — the fallback exists because a host without Vulkan previously left
the local model silently unavailable
(`botserver/crates/botcore/src/package_manager/facade/install_system_packages.rs`).

Checksums are enforced after download. The one exception is the Linux arm64
archive: b12189 publishes no `SHA256SUMS`, so `sha256_arm64` is empty and
verification is skipped for that variant instead of comparing it to the x64
hash (`botserver/3rdparty.toml`).

## Configuring the provider

A local model is a normal LLM provider: point the bot at the llama.cpp server
URL and model name in Vault (`llm-url`, `llm-model`, `llm-key`) as described in
`03-knowledge-ai/README.md`. Nothing is hardcoded in the binary.

## What this page does not claim

- No GPU model is required or recommended here — VRAM figures belong to
  [Desktop & Workstation Hardware](./desktop-hardware.md).
- No quantization, context-size or `-moe` flag is set by the product; the
  component ships the upstream server and its own defaults.
- Model files are not part of the installer. Offline installs must place them
  in the cache directory the provisioner reads
  (`BOTSERVER_INSTALLERS_PATH`).

## See also

- [Supported Platforms](./hardware.md)
- [Desktop & Workstation Hardware](./desktop-hardware.md)
- [Component versions](../12-ecosystem-reference/updating-components.md)
