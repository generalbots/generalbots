# Third-party licences — General Bots Lite for Windows

This is the licence record for every third-party component that the Windows
installer of General Bots Lite ships or downloads on first run. The
`Desktop-Local Contribution Rules` in `AGENTS.md` require it before packaging.

**General Bots itself is MIT.** See [`LICENSE`](LICENSE); `botserver` and
`botapp` declare `license = "MIT"`.

Sources of truth:

- What the installer ships: the step "Stage the offline payload" and the step
  "PostgreSQL client libraries" in `.github/workflows/gbgov-release.yml`. The
  bundler copies `botserver/botserver-installers/*` to `installers/`
  (`botapp/tauri.gbgov.conf.json`).
- What is downloaded later: `botserver/3rdparty.toml`.
- The licence texts: `desktop/windows/licenses/` (texts and `SOURCE-OFFER.md`),
  `desktop/os/licenses/NOTICE` and this file. `botapp/tauri.gbgov.conf.json`
  copies them to `licenses/` in the install folder. Settings > About in the app
  shows the NOTICE and this record from `botui/ui/suite/about/licenses/`, a
  byte-identical copy that `scripts/check_third_party_licenses.py` enforces.

How the licences were checked (2026-11-26): the licence file of each upstream
repository at the pinned tag (GitHub licence API, then the text itself where
the API said `NOASSERTION`); the file list of each shipped archive, downloaded
from the pinned URL with its digest matching the pin; the model cards on
Hugging Face. **NOT VERIFIED** marks what could not be checked this way.

This record states what the licences say. It is not legal advice. Items marked
**Lawyer** need a legal opinion before a public release.

## 1. Shipped inside the installer

| Component | Version | Licence (SPDX) | Upstream licence read | Licence file in the shipped archive | SHA-256 (Windows x64) | Obligations |
|---|---|---|---|---|---|---|
| OpenBao (`bao.exe`, component `vault`) | 2.7.3 | MPL-2.0 | github.com/openbao/openbao/blob/v2.7.3/LICENSE | `LICENSE` (inside the zip) | 969e4872b21eeee7c9b4cd312e77027e624528ca7bc9dc2361311b97e34e6e90 | Keep the licence. MPL-2.0 §3.2: whoever distributes the executable must tell recipients how to get its source (the upstream tag is enough while it stays online). Unmodified, so no own files to publish |
| MinIO server (`minio.exe`) | RELEASE.2025-10-15T17-29-55Z, built from the unmodified source tag (fixes CVE-2025-62506) | AGPL-3.0-or-later | github.com/minio/minio/blob/RELEASE.2025-10-15T17-29-55Z/LICENSE (+ `NOTICE`) | none (single `.exe`) | e63f9c4ac8b8697ca679d4864497688c3486aa9ac9370c3569203420c25e5799 (release `drive-bin-v4` of generalbots/generalbots, built by `.github/workflows/drive-bin-v4.yml` on `main`) | See "AGPL" below. Ship the licence text and the upstream `NOTICE` |
| MinIO client (`mc.exe`) | RELEASE.2025-08-13T08-35-41Z | AGPL-3.0-or-later | github.com/minio/mc/blob/RELEASE.2025-08-13T08-35-41Z/LICENSE (+ `NOTICE`) | none (single `.exe`) | c8db13ebeda31497f354c0e950809db0ae9b2a2a69b8afee68c128c37300c157 | As MinIO server |
| Redis for Windows (Microsoft Open Technologies port) | 3.0.504 | BSD-3-Clause | github.com/microsoftarchive/redis/blob/win-3.0.504/license.txt | none | 5f761367601ca31f6c8969e427cacc0da4f428712954a66aab303f83e390566e | Reproduce both copyright lines (Salvatore Sanfilippo; Microsoft Open Technologies), the conditions and the disclaimer in the documentation. Do not use the name "Redis" to endorse the product. The sources also carry `deps/`: jemalloc and jemalloc-win (BSD-2-Clause), hiredis (BSD-3-Clause), Lua 5.1 (MIT), linenoise (BSD). Which of them are linked into `redis-server.exe`: NOT VERIFIED; ship their notices to be safe |
| Qdrant (`qdrant.exe`) | 1.20.0 | Apache-2.0 | github.com/qdrant/qdrant/blob/v1.20.0/LICENSE | none (zip holds only `qdrant.exe`) | 6d13553584fd0411f00637519e293331ff0aa1fb79bd9e30db45839bb75de2be | Ship the Apache-2.0 text. Upstream has no `NOTICE` at this tag. The notices of the Rust crates compiled into the binary are not published by upstream: NOT VERIFIED |
| Zitadel (`zitadel.exe`) | 4.20.1 | AGPL-3.0-only; `proto/`, `apps/docs/` Apache-2.0; `apps/login/`, `packages/zitadel-client/`, `packages/zitadel-proto/` MIT | github.com/zitadel/zitadel/blob/v4.20.1/LICENSE and `LICENSING.md` | `LICENSE` (inside the tar.gz) | dd38db24626a3f24f9e22244163f4ba50527378a4237cd19f0f6578b61c39405 | See "AGPL" below, plus the MIT notice for the login UI. Upstream `LICENSING.md` itself recommends legal counsel |
| Caddy (`caddy.exe`) | 2.11.9 | Apache-2.0 | github.com/caddyserver/caddy/blob/v2.11.9/LICENSE | `LICENSE` (inside the zip) | ad93112d335ee687100c4c9f11bced16ca590377976b7fe7883882b9e797e472 (the workflow checks the SHA-512 from upstream) | Keep the licence. No `NOTICE` upstream |
| PostgreSQL client `libpq.dll` (from the EDB 17.4-1 binaries zip) | 17.4-1 | PostgreSQL | postgresql.org/about/licence | taken from the EDB zip, digest 80bb49e80df0100dd52e7b3c647896f8d8cfde760b6473e5180819c8df4b23c4 | per file: NOT RECORDED | Keep the copyright notice and the licence paragraphs |
| OpenSSL `libcrypto-3-x64.dll`, `libssl-3-x64.dll` | 3.x (exact version NOT VERIFIED) | Apache-2.0 | github.com/openssl/openssl/blob/openssl-3.0.0/LICENSE.txt | same zip | — | Ship the Apache-2.0 text |
| GNU libiconv `libiconv-2.dll` | NOT VERIFIED | LGPL (gnu.org/software/libiconv: "the libiconv and libcharset libraries … are under LGPL"); exact version of the LGPL NOT VERIFIED | — | same zip | — | **Lawyer.** LGPL for a library distributed in object form: ship the LGPL text, keep the DLL replaceable (it is a separate DLL), and give the corresponding source of that exact build or a written offer |
| GNU gettext `libintl-9.dll` | NOT VERIFIED | LGPL-2.1-or-later (NOT VERIFIED: the gettext page answered HTTP 429) | — | same zip | — | As libiconv |
| mingw-w64 winpthreads `libwinpthread-1.dll` | NOT VERIFIED | MIT AND BSD-3-Clause | mingw-w64 `mingw-w64-libraries/winpthreads/COPYING` | same zip | — | Both copyright notices (mingw-w64 project; Lockless Inc.) and both texts |
| LZ4 `liblz4.dll` | NOT VERIFIED | BSD-2-Clause (the `lib/` directory) | github.com/lz4/lz4/blob/dev/lib/LICENSE | same zip | — | Copyright notice and licence text |
| Zstandard `libzstd.dll` | NOT VERIFIED | BSD-3-Clause OR GPL-2.0-only (we use the BSD option) | github.com/facebook/zstd (README "License") | same zip | — | Copyright notice and BSD text |
| zlib `zlib1.dll` | NOT VERIFIED | Zlib | github.com/madler/zlib/blob/develop/LICENSE | same zip | — | No notice duty in binaries; keep the text anyway |

Notes:

- The client DLLs come from EDB 17.4-1, but `3rdparty.toml` `[components.libpq_win]`
  names `postgresql-18.6-1-…zip` as `filename_win`, points `url_win` at 17.4-1
  and has an empty `sha256`. The record above follows the workflow.
- The CUDA 13.1 llama.cpp archive (b7345) and the two model files that an
  earlier draft listed as shipped are not in the release workflow. They are
  downloaded (section 2).
- The workflow skips `incus.exe` (agent workspace on WSL) while its source is not
  in the repository. Nothing to record yet.

## 2. Downloaded on first run

| Component | Version | Licence (SPDX) | Upstream licence read | When | SHA-256 (Windows x64) | Obligations |
|---|---|---|---|---|---|---|
| PostgreSQL server (EDB binaries zip) | 18.6-1 | PostgreSQL | postgresql.org/about/licence | first run | fbe23da234ee31547bf8a36d29dfd81e82b849df2d2b78d2eecb43d360252f8c | As libpq. The zip also carries pgAdmin 4, StackBuilder and their own libraries; contents NOT VERIFIED. Whether EDB attaches its own terms to the zip: NOT VERIFIED, **Lawyer** |
| llama.cpp, CPU x64 | b12483 | MIT | github.com/ggml-org/llama.cpp/blob/b12483/LICENSE | local model mode | 7c3ec4ff5cdddc272cb1d6b52a05c8751dc566c7105750c10ed3688bbdbf8fa8 | "Copyright (c) 2023-2026 The ggml authors" and the MIT text |
| llama.cpp, Vulkan x64 | b12483 | MIT | as above | when selected | 7859e96ba91201a41e2769d4456fe86e8968ba93c6794eeb4fe49b7537a17628 | as above |
| llama.cpp, CUDA 12.4 / 13.5 x64 | b12483 | MIT | as above | when selected | 89dcd32dcd0d0b99daaa5150ff0107cb2f47a6dc5a269e23c2c10e86ba4cc9f2 / 3467a6cf27418a5265428e3eaea1249daa7aba40ca75b594dca47083104cbe3e | as above |
| llama.cpp, CPU ARM64 | b12483 | MIT | as above | ARM64 machines | bbf6f781a01fb0f3ce0adc17a96ebda8200477dc58b8f6c0367ab7317db1b5b8 | as above |
| LLVM OpenMP runtime (inside the llama.cpp zips, per the earlier draft) | — | Apache-2.0 WITH LLVM-exception | github.com/llvm/llvm-project/blob/main/openmp/LICENSE.TXT | with llama.cpp | — | Keep the licence file. Presence in the zips NOT VERIFIED |
| NVIDIA CUDA runtime (cudart, cuBLAS, cuBLASLt), from the llama.cpp release | 12.4 / 13.5 | LicenseRef-NVIDIA-CUDA-EULA (proprietary) | docs.nvidia.com/cuda/eula (Attachment A lists them as redistributable) | with a CUDA build only | 8c79a9b226de4b3cacfd1f83d24f962d0773be79f1e7b75c6af4ded7e32ae1d6 / c2bf4cc202bfbaaa7a0b75f5082457c3811326947bb216059b72839d844e9cba | EULA terms for distributors: the application must add material functionality; only the application may use the libraries; our terms must be consistent with the EULA. We do not ship them; the machine fetches them from the ggml-org release. **Lawyer:** whether that counts as our distribution |
| Microsoft Edge WebView2 Runtime | Evergreen | LicenseRef-Microsoft-WebView2-Runtime (proprietary) | learn.microsoft.com/microsoft-edge/webview2/concepts/distribution | only if missing | — | Part of Windows 11. `tauri.gbgov.conf.json` sets no `webviewInstallMode`, so the Tauri default applies (download the bootstrapper from Microsoft); NOT VERIFIED in a built installer. The runtime comes from Microsoft under Microsoft's terms; we do not redistribute it |

## 3. Models

Downloaded when the person chooses the local model (`[models.*]` in
`3rdparty.toml`). They are not in the release payload. Licences are those on
the model cards; the weights are quantised conversions of the original models.

| Model | File (bytes) | Licence | Upstream | SHA-256 (matches the Hugging Face LFS digest) | Obligations | Status |
|---|---|---|---|---|---|---|
| DeepSeek-R1-Distill-Qwen-1.5B, GGUF by bartowski | DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf (924,456,032) | MIT (deepseek-ai card). Base model Qwen2.5-Math-1.5B: Apache-2.0, itself from Qwen2.5-1.5B: Apache-2.0. The bartowski repository declares no licence of its own | huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B | b1094879ee7e222cba5be013e6de2d0658c64f91aef5d3c9bff0db5cefbfa020 (not pinned in `3rdparty.toml`) | MIT text; Apache-2.0 text for the Qwen base; say the file is a quantised derivative | In use (local mode) |
| bge-small-en-v1.5, GGUF by CompendiumLabs | bge-small-en-v1.5-f32.gguf (133,609,568) | MIT (both cards) | huggingface.co/BAAI/bge-small-en-v1.5 | bf40c42ad7d89382e9ba7376d5c4b73f6b556cb541fab37aaa1da9c320149b65 (not pinned) | MIT text | In use (local mode) |
| Candidate, not used yet: S5-nano | S5-nano-Q4_K_M.gguf | Apache-2.0 (card of maritaca-ai/S5-nano-GGUF) | huggingface.co/maritaca-ai/S5-nano-GGUF | c7040c0770c427d06eb22643e41f061a1bc30e267f07dbcad5a9861b4564003b (earlier draft; not re-checked) | Apache-2.0 text | Candidate |
| Candidate, not used yet: BGE-M3 Q8_0 | bge-m3-q8_0.gguf | MIT (cards of ggml-org/bge-m3-Q8_0-GGUF and BAAI/bge-m3) | huggingface.co/ggml-org/bge-m3-Q8_0-GGUF | aa473d51f451a22f0fcf39ba3330c14bed38a385712b1113440f69df4047a173 (earlier draft; not re-checked) | MIT text | Candidate |

A model card licence covers the weights. It does not settle who owns what the
model writes, or whether the training data allows a given use. **Lawyer** for
government use.

## 4. Component register (one row per `[components.*]` in `botserver/3rdparty.toml`)

Kept in the six-column form that `scripts/check_third_party_licenses.py` on
`main` reads. "Edition" says what General Bots Lite on Windows does with it.

| component | name | version | license | source | text |
|---|---|---|---|---|---|
| `openbao` | OpenBao secrets server (component id `vault`) | 2.7.3 | MPL-2.0 | https://github.com/openbao/openbao | `licenses/openbao-LICENSE.txt`; Edition: shipped, started |
| `drive` | MinIO Object Storage | RELEASE.2025-10-15T17-29-55Z (built from source; Windows) | AGPL-3.0-or-later | https://github.com/minio/minio | `licenses/minio-LICENSE.txt`, `minio-NOTICE.txt`; Edition: shipped, started |
| `mc` | MinIO Client (mc) | RELEASE.2025-08-13T08-35-41Z | AGPL-3.0-or-later | https://github.com/minio/mc | `licenses/mc-LICENSE.txt`, `mc-NOTICE.txt`; Edition: shipped |
| `cache` | Redis for Windows (Windows `url_win`; Valkey 9.1.3 elsewhere) | 3.0.504 | BSD-3-Clause | https://github.com/microsoftarchive/redis | `licenses/redis-license.txt` and `redis-*` deps; Edition: shipped |
| `vector_db` | Qdrant Vector Database | 1.20.0 | Apache-2.0 | https://github.com/qdrant/qdrant | `licenses/qdrant-LICENSE.txt`; Edition: shipped |
| `directory` | Zitadel Identity Provider | 4.20.1 | AGPL-3.0-only | https://github.com/zitadel/zitadel | `licenses/zitadel-LICENSE.txt`, `zitadel-LICENSING.md`, `zitadel-login-LICENSE.txt`; Edition: shipped |
| `caddy` | Caddy Web Server | 2.11.9 | Apache-2.0 | https://github.com/caddyserver/caddy | `licenses/caddy-LICENSE.txt`; Edition: shipped |
| `proxy` | Caddy (same archive as `caddy`) | 2.11.9 | Apache-2.0 | https://github.com/caddyserver/caddy | `licenses/caddy-LICENSE.txt` |
| `tables` | PostgreSQL Database (EDB binaries on Windows) | 18.6-1 | PostgreSQL | https://github.com/postgres/postgres | zip `COPYRIGHT` (NOT VERIFIED); Edition: downloaded |
| `libpq_win` | PostgreSQL client libraries | 17.4-1 (workflow) | PostgreSQL | https://github.com/postgres/postgres | `licenses/postgresql-COPYRIGHT.txt`; the other DLLs of the zip: see section 1; Edition: shipped |
| `llm_win_cpu_x64` | llama.cpp (Windows x64 CPU) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | `licenses/llama.cpp-LICENSE.txt`; Edition: downloaded |
| `llm_win_cpu_arm64` | llama.cpp (Windows ARM64 CPU) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_cuda12` | llama.cpp (Windows CUDA 12.4) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_cuda13` | llama.cpp (Windows CUDA 13.5) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_vulkan` | llama.cpp (Windows Vulkan) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `cudart_win_cuda12` | CUDA runtime (CUDA 12.4) | 12.4 | LicenseRef-NVIDIA-CUDA-EULA | https://docs.nvidia.com/cuda/eula/ | EULA; Edition: downloaded with a CUDA build |
| `cudart_win_cuda13` | CUDA runtime (CUDA 13.5) | 13.5 | LicenseRef-NVIDIA-CUDA-EULA | https://docs.nvidia.com/cuda/eula/ | as above |
| `botmodels` | BotModels pythonbox | release tag | MIT (own code); CPython and wheel licences NOT VERIFIED | https://github.com/generalbots/generalbots | Edition: not in the release payload; use NOT VERIFIED |
| `llm` | llama.cpp (Linux x64 CPU) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | not used on Windows |
| `llm_linux_vulkan` | llama.cpp (Linux Vulkan) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | not used on Windows |
| `llm_linux_s390x` | llama.cpp (Linux s390x) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | not used on Windows |
| `llm_macos_arm64` | llama.cpp (macOS ARM64) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | not used on Windows |
| `llm_macos_x64` | llama.cpp (macOS x64) | b7345 | MIT | https://github.com/ggml-org/llama.cpp | not used on Windows |
| `email` | Stalwart Mail Server | 1.0.3 | AGPL-3.0-only (README badge "AGPL v3"; exact form NOT VERIFIED) | https://github.com/stalwartlabs/stalwart | mail is not compiled into this edition |
| `webmail` | Roundcube Webmail | 1.7.5 | GPL-3.0-or-later (NOT VERIFIED) | https://github.com/roundcube/roundcubemail | no Windows asset |
| `meet` | LiveKit Media Server | 1.13.11 | Apache-2.0 | https://github.com/livekit/livekit | voice/meet not compiled into this edition |
| `dns` | CoreDNS | 1.15.0 | Apache-2.0 | https://github.com/coredns/coredns | use by this edition NOT VERIFIED |
| `timeseries_db` | InfluxDB | 2.7.10 | MIT | https://github.com/influxdata/influxdb | use by this edition NOT VERIFIED |
| `observability` | Vector Log Aggregator | 0.61.0 | MPL-2.0 | https://github.com/vectordotdev/vector | use by this edition NOT VERIFIED |
| `alm` | Forgejo | 17.0.1 | GPL-3.0-or-later (NOT VERIFIED) | https://codeberg.org/forgejo/forgejo | refused on Windows |
| `alm_ci` | Forgejo Runner | 13.2.1 | GPL-3.0-or-later (NOT VERIFIED) | https://code.forgejo.org/forgejo/runner | refused on Windows |
| `table_editor` | NocoDB | unpinned | AGPL-3.0-or-later (NOT VERIFIED) | https://github.com/nocodb/nocodb | refused on Windows |

## 5. Bundled in the interface and the program

From the earlier draft; not re-read on 2026-11-26.

| Component | Licence | Location | Obligations |
|---|---|---|---|
| pdf.js 6.5.438 | Apache-2.0 | `botui/ui/suite/vendor/pdfjs` (`LICENSE`) | Keep the licence |
| pdf.js WASM decoders | JBIG2 (PDFium) BSD-3-Clause; OpenJPEG BSD-2-Clause; qcms MIT | `vendor/pdfjs/wasm/LICENSE_*` | Keep the licence files |
| pdf.js standard fonts | Foxit (BSD-style); Liberation 1.x GPL-2.0 with font exception | `vendor/pdfjs/standard_fonts/LICENSE_*` | Keep the licence files |
| Fira Sans, Inter, Geist, Geist Mono, Open Sans | OFL-1.1 | `botui/ui/suite/identity/fonts/*-LICENSE.txt`, `*-OFL.txt` | Keep the OFL texts; do not sell the fonts on their own |
| Rust crates in `botserver.exe` and the desktop shell | mostly MIT and/or Apache-2.0; also Unicode-3.0, MPL-2.0, BSD, ISC, Zlib, 0BSD, Unlicense, CC0-1.0 (draft count over the whole workspace) | `Cargo.lock`; the workflow writes a CycloneDX SBOM | Generate the notice list for the exact feature set (`cargo about` or similar) and ship it |
| NSIS installer stub, WebView2 loader linked by the shell | NOT VERIFIED | — | Check before release |

## 6. What the licences require of this installer

- **AGPL-3.0** (MinIO, mc, Zitadel). Shipping the binary is "conveying object
  code" (§6). That needs the Corresponding Source by one of §6a–6e. §6d allows a
  network location, including someone else's server, with clear directions next
  to the binary, and the distributor stays responsible for it staying available.
  §13 (offer source to remote network users) applies to modified versions; these
  binaries are unmodified and listen on the local machine only. Ship the licence
  text and the upstream `NOTICE`. MinIO and mc upstream repositories are
  archived (read-only); the tags remain. **Lawyer:** whether a link to the
  upstream tag is enough for §6d, and whether "Corresponding Source" must
  include the Go modules of that build.
- **MPL-2.0** (OpenBao). Ship the licence and say where the source of v2.7.3 is.
- **LGPL** (libiconv, libintl). Ship the text, keep the DLLs replaceable, give
  or offer the source of the exact build. Versions not yet identified.
- **BSD / MIT / Zlib** (Redis and its deps, winpthreads, lz4, zstd, zlib,
  llama.cpp). Reproduce the copyright lines and texts in the documentation.
- **Apache-2.0** (Qdrant, Caddy, OpenSSL, the Qwen base). Ship the text and
  any upstream `NOTICE` (none found for Qdrant, Caddy).
- **Proprietary** (CUDA runtime, WebView2 Runtime). Downloaded from the vendor
  or from the llama.cpp release, not redistributed by us; see section 2.

## 7. Open before a public release

1. Licence texts now travel with the installer: `licenses/` holds the NOTICE,
   this record, `SOURCE-OFFER.md` and the texts fetched from each upstream tag
   on 2026-11-26. That a built installer really contains the folder: NOT
   CHECKED. Still missing: the notice list of the Rust crates compiled into
   `botserver.exe` and the desktop shell (section 5), and a licence file for
   `bge-small-en-v1.5` (upstream publishes none; the card says MIT).
2. `SOURCE-OFFER.md` says where the source of MinIO, mc, Zitadel and OpenBao
   is. Not settled: the versions and the exact source of the LGPL DLLs
   (libiconv, libintl) in the EDB zip. **Lawyer:** whether links to upstream
   tags satisfy AGPL-3.0 §6 and MPL-2.0 §3.2.
3. MinIO now ships RELEASE.2025-10-15T17-29-55Z (CVE-2025-62506 fixed), built
   from source. Redis 3.0.504 is from 2016, archived, with unpatched CVEs.
4. Pin the SHA-256 of both models and of the `libpq_win` archive in
   `3rdparty.toml`, and make `libpq_win` agree with the workflow (17.4-1).
5. The release downloads PostgreSQL, llama.cpp, the models and possibly
   WebView2 on first run. That contradicts "no downloads at first boot" in
   `AGENTS.md` and fails on an offline machine.
