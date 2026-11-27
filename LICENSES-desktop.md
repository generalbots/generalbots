# Third-party licenses — desktop and appliance edition

This is the license record for every binary General Bots ships or downloads.
`botserver/3rdparty.toml` is the machine-readable provenance (version, URL,
SHA-256); this file is the human-readable license record that has to travel
with it.

`scripts/check_third_party_licenses.py` keeps the two in sync and runs in the
appliance-image workflow: a component without a row here fails the build, and a
row whose license, source or text location is empty fails too. See #2480.

**General Bots itself is MIT** — see [`LICENSE`](LICENSE). The appliance image
ships that file at `/usr/share/doc/general-bots/licenses/LICENSE`.

## Components

Source URLs point at the upstream repository, not at the download mirror: for
the AGPL/BUSL entries that is where the corresponding source is obtained.
Rows whose key ends in `_cached` are legacy offline-installer copies of the same
upstream binary and carry the same license as their non-cached counterpart.

| component | name | version | license | source | text |
|---|---|---|---|---|---|
| `drive` | OtterIO Object Storage | RELEASE.2026-11-25T20-30-00Z | Apache-2.0 | https://github.com/soulteary/otterio | payload `LICENSE` |
| `mc` | OC CLI (oc) | RELEASE.2026-11-25T20-30-01Z | Apache-2.0 | https://github.com/soulteary/oc | payload `LICENSE` |
| `clamav` | ClamAV Antivirus Engine | 1.5.5 | GPL-2.0-or-later | https://github.com/Cisco-Talos/clamav | payload `copyright` |
| `tables` | PostgreSQL Database | 18.6.1 | PostgreSQL | https://github.com/postgres/postgres | payload `COPYRIGHT` |
| `tables_17_2_0_cached` | PostgreSQL Database (bundled cache copy) | 17.2.0 | PostgreSQL | https://github.com/postgres/postgres | payload `COPYRIGHT` |
| `cache` | Valkey Cache (Redis-compatible) | 9.1.3 | BSD-3-Clause | https://github.com/valkey-io/valkey | payload `COPYING` |
| `cache_8_1_5_cached` | Valkey Cache (bundled cache copy) | 8.1.5 | BSD-3-Clause | https://github.com/valkey-io/valkey | payload `COPYING` |
| `caddy` | Caddy Web Server | 2.11.9 | Apache-2.0 | https://github.com/caddyserver/caddy | payload `LICENSE` |
| `email` | Stalwart Mail Server | 1.0.3 | AGPL-3.0-only | https://github.com/stalwartlabs/mail-server | payload `LICENSE-AGPL` |
| `directory` | Zitadel Identity Provider | 4.20.1 | Apache-2.0 | https://github.com/zitadel/zitadel | payload `LICENSE` |
| `alm` | Forgejo Git Server | 17.0.1 | GPL-3.0-or-later | https://codeberg.org/forgejo/forgejo | payload `LICENSE` |
| `alm_10_0_1_cached` | Forgejo Git Server (bundled cache copy) | 10.0.1 | GPL-3.0-or-later | https://codeberg.org/forgejo/forgejo | payload `LICENSE` |
| `alm_ci` | Forgejo Actions Runner | 13.2.1 | GPL-3.0-or-later | https://code.forgejo.org/forgejo/runner | payload `LICENSE` |
| `alm_ci_6_0_0_cached` | Forgejo Actions Runner (bundled cache copy) | 6.0.0 | GPL-3.0-or-later | https://code.forgejo.org/forgejo/runner | payload `LICENSE` |
| `dns` | CoreDNS Server | 1.15.0 | Apache-2.0 | https://github.com/coredns/coredns | payload `LICENSE` |
| `webmail` | Roundcube Webmail | 1.7.5 | GPL-3.0-or-later | https://github.com/roundcube/roundcubemail | payload `LICENSE` |
| `meet` | LiveKit Media Server | 1.13.11 | Apache-2.0 | https://github.com/livekit/livekit | payload `LICENSE` |
| `libpq_win` | PostgreSQL Client Libraries (libpq) | 18.6-1 | PostgreSQL | https://github.com/postgres/postgres | payload `COPYRIGHT` |
| `vector_db` | Qdrant Vector Database | 1.20.0 | Apache-2.0 | https://github.com/qdrant/qdrant | payload `LICENSE` |
| `vector_db_1_12_0_cached` | Qdrant Vector Database (bundled cache copy) | 1.12.0 | Apache-2.0 | https://github.com/qdrant/qdrant | payload `LICENSE` |
| `timeseries_db` | InfluxDB Time Series Database | 2.7.10 | MIT | https://github.com/influxdata/influxdb | payload `LICENSE` |
| `openbao` | OpenBao (secrets server) | 2.7.3 | MPL-2.0 | https://github.com/openbao/openbao | payload `LICENSE` |
| `vault_1_18_2_cached` | HashiCorp Vault (bundled cache copy) | 1.18.2 | BUSL-1.1 | https://github.com/hashicorp/vault | payload `LICENSE` |
| `observability` | Vector Log Aggregator | 0.61.0 | MPL-2.0 | https://github.com/vectordotdev/vector | payload `LICENSE` |
| `botmodels` | BotModels Python Service (pythonbox) | release tag | MIT | https://github.com/generalbots/generalbots | payload `LICENSE` |
| `llm` | Llama.cpp Server (Linux x64 CPU) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_linux_vulkan` | Llama.cpp Server (Linux Vulkan) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_linux_s390x` | Llama.cpp Server (Linux s390x) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_macos_arm64` | Llama.cpp Server (macOS ARM64) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_macos_x64` | Llama.cpp Server (macOS x64) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_win_cpu_x64` | Llama.cpp Server (Windows x64 CPU) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_win_cpu_arm64` | Llama.cpp Server (Windows ARM64 CPU) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_win_cuda12` | Llama.cpp Server (Windows CUDA 12.4) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_win_cuda13` | Llama.cpp Server (Windows CUDA 13.1) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `llm_win_vulkan` | Llama.cpp Server (Windows Vulkan) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | payload `LICENSE` |
| `cudart_win_cuda12` | CUDA Runtime (Windows CUDA 12.4) | b7345 | LicenseRef-NVIDIA-CUDA | https://developer.nvidia.com/cuda-toolkit | Windows installer payload only |
| `cudart_win_cuda13` | CUDA Runtime (Windows CUDA 13.1) | b7345 | LicenseRef-NVIDIA-CUDA | https://developer.nvidia.com/cuda-toolkit | Windows installer payload only |

## Where the texts land

The license text ships next to the component it belongs to, not inside this
document: each distribution already carries its own `LICENSE`/`COPYING`, and a
copy taken from the shipped artifact cannot drift from what is actually
distributed. Packaging copies it to
`/usr/share/doc/general-bots/licenses/<component>/`.

Components that ship no license file get a text under `desktop/os/licenses/`;
the `text` column names it. The CUDA runtime is Windows-only, so the appliance
image never distributes it — its EULA travels with the Windows installer.

The image also carries [`NOTICE`](desktop/os/licenses/NOTICE), which points at
this record and at the per-component texts.

## Obligations that follow from the licenses

- **AGPL-3.0** (`email`): the image distributes this binary, so the
  corresponding source must be reachable from the source URLs above and the
  offer must be documented next to the license texts. Do not describe our own
  code as AGPL — only these notices are.
- **Apache-2.0** (`drive`, `mc`): the image distributes these binaries with the
  upstream license and NOTICE texts; attribution is retained in the payload.
- **GPL-3.0** (`alm`, `alm_ci`, `webmail`): same source-availability rule.
- **BUSL-1.1** (`vault_1_18_2_cached`): the legacy cached HashiCorp Vault copy
  is not an open-source license. Redistribution is allowed for non-production
  use; a production deployment needs a HashiCorp license. The shipped secrets
  server is OpenBao (`openbao`, MPL-2.0).
- **LicenseRef-NVIDIA-CUDA** (`cudart_win_*`): proprietary, redistributable
  under the CUDA Toolkit EULA. Windows payload only; not in the Linux image.

## Debian packages

The appliance also installs distribution packages
(`linux-image-amd64`, `dracut`, `systemd-boot`, `cage`, `dbus`, `plymouth`,
`fonts-dejavu-core`, `ca-certificates`, `libpq5`, `libwebkit2gtk-4.1-0`,
`libssl3t64`, `bash`). Debian policy requires each of them to ship its own
copyright and license files under `/usr/share/doc/<package>/copyright`, which
the image carries verbatim — they are not duplicated here and are not part of
`3rdparty.toml`.
