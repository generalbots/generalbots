# Third-party licences — General Bots Lite for Windows

This is the licence record for every third-party component that the Windows
installer of General Bots Lite ships or downloads, and for the outside
services the app can send text to. The `Desktop-Local Contribution Rules` in
`AGENTS.md` require it before packaging.

**General Bots itself is MIT.** See [`LICENSE`](LICENSE); `botserver` and
`botapp` declare `license = "MIT"`.

Sources of truth:

- What the installer ships: the steps "Stage the offline payload" and
  "PostgreSQL client libraries" of the Windows release workflow in
  `.github/workflows/`. The bundler copies `botserver/botserver-installers/*`
  to `installers/` (the desktop bundle configuration in `botapp/`).
- What is downloaded later: `botserver/3rdparty.toml` (programs) and
  `botserver/crates/botsettings/src/on_device_ai/catalog.rs` (local models).
- The licence texts: `desktop/windows/licenses/` (texts and `SOURCE-OFFER.md`),
  `desktop/os/licenses/NOTICE` and this file. The desktop bundle configuration
  copies them to `licenses/` in the install folder. Settings > About in the app
  shows the NOTICE and this record from `botui/ui/suite/about/licenses/`, a
  byte-identical copy that `scripts/check_third_party_licenses.py` enforces.

How the licences were checked (2026-11-26): the licence file of each upstream
repository at the pinned tag (GitHub licence API, then the text itself where
the API said `NOASSERTION`); the file list of each shipped archive, downloaded
from the pinned URL with its digest matching the pin; the model cards on
Hugging Face. **NOT VERIFIED** marks what could not be checked this way.

Checked again on 2026-11-27, for the parts that changed:

- The two EDB PostgreSQL zips, at their pinned URLs: the file list, EDB's
  licence files and single DLLs were read through HTTP range requests. Each
  file read matched the CRC-32 in the zip's own file list. The zips were not
  downloaded whole, so their SHA-256 pins were not computed again.
- GNU libiconv and GNU gettext: the release news on savannah.gnu.org, the
  announcement on lists.gnu.org, the download listing on ftp.gnu.org, and the
  licence files at the release tags of the GNU repositories. Savannah's own
  source browser answered HTTP 502 that day, so the tags were read through
  Software Heritage's archive of those repositories. An identifier written
  `swh:1:cnt:…` or `swh:1:rev:…` opens at
  `https://archive.softwareheritage.org/<identifier>`.
- Local models: the model card (`README.md`), the repository record and the
  download address of every catalogue entry on huggingface.co.
- The hosted free model: the service's terms, privacy and documentation
  pages, its public model list, and the upstream provider's terms.
- The NVIDIA CUDA licence page.

This record states what the licences say. It is not legal advice. Items marked
**Lawyer** need a legal opinion before a public release.

## 1. Shipped inside the installer

Since 2026-11-27 the installer no longer carries the Qdrant, Zitadel and Caddy
archives: this edition switches those services off and nothing in it starts
them. Their rows are now in section 2.2. Redis for Windows left earlier. The
PostgreSQL 18.6-1 server zip is now inside the installer; earlier releases
downloaded it at first run.

| Component | Version | Licence (SPDX) | Upstream licence read | Licence file in the shipped archive | SHA-256 (Windows x64) | Obligations |
|---|---|---|---|---|---|---|
| OpenBao (`bao.exe`, component `vault`) | 2.7.3 | MPL-2.0 | github.com/openbao/openbao/blob/v2.7.3/LICENSE | `LICENSE` (inside the zip) | 969e4872b21eeee7c9b4cd312e77027e624528ca7bc9dc2361311b97e34e6e90 | Keep the licence. MPL-2.0 §3.2: whoever distributes the executable must tell recipients how to get its source (the upstream tag is enough while it stays online). Unmodified, so no own files to publish |
| MinIO server (`minio.exe`) | RELEASE.2025-10-15T17-29-55Z, built from the unmodified source tag (fixes CVE-2025-62506) | AGPL-3.0-or-later | github.com/minio/minio/blob/RELEASE.2025-10-15T17-29-55Z/LICENSE (+ `NOTICE`) | none (single `.exe`) | e63f9c4ac8b8697ca679d4864497688c3486aa9ac9370c3569203420c25e5799 (release `drive-bin-v4` of generalbots/generalbots, built by `.github/workflows/drive-bin-v4.yml` on `main`) | See "AGPL" below. Ship the licence text and the upstream `NOTICE` |
| MinIO client (`mc.exe`) | RELEASE.2025-08-13T08-35-41Z | AGPL-3.0-or-later | github.com/minio/mc/blob/RELEASE.2025-08-13T08-35-41Z/LICENSE (+ `NOTICE`) | none (single `.exe`) | c8db13ebeda31497f354c0e950809db0ae9b2a2a69b8afee68c128c37300c157 | As MinIO server |
| ~~Redis for Windows~~ | — | — | — | — | — | Removed: Windows uses the built-in RESP cache inside the server; no external Redis binary is shipped or downloaded. Its licence texts stay in `licenses/` because installers before the removal carried it |
| PostgreSQL server: the whole EDB binaries zip `postgresql-18.6-1-windows-x64-binaries.zip` (component `tables`) | 18.6-1 | PostgreSQL for the server; the other parts of the zip are in 1.1 | postgresql.org/about/licence | `pgsql/server_license.txt` and four more licence files (1.1) | fbe23da234ee31547bf8a36d29dfd81e82b849df2d2b78d2eecb43d360252f8c | Keep the copyright notice and the licence paragraphs. Unpacked at first run. The zip is passed on whole, so everything in 1.1 is distributed with it. Whether EDB attaches its own terms to the zip: NOT VERIFIED, **Lawyer** |
| PostgreSQL client `libpq.dll` (from the EDB 17.4-1 binaries zip, digest 80bb49e80df0100dd52e7b3c647896f8d8cfde760b6473e5180819c8df4b23c4) | 17.4 (version stamp) | PostgreSQL | postgresql.org/about/licence | none in the 17.4-1 zip (its file list has no licence file at the top level) | 63f694976c10099dd8b815a10af479ded4f9cb8f09a00e37ec6b170843858d6e | Keep the copyright notice and the licence paragraphs |
| OpenSSL `libcrypto-3-x64.dll`, `libssl-3-x64.dll` | 3.0.16 (version stamps; "OpenSSL 3.0.16 11 Feb 2025") | Apache-2.0 | github.com/openssl/openssl/blob/openssl-3.0.0/LICENSE.txt | same zip | 46a8846bd395dfeeac857c26f789c0005591da4cf93661ded490ff60a2270dea / ca09015d28cd0a965b8184bc0f79c84ce334b5c0c191d03cb0d2112e34eb6417 | Ship the Apache-2.0 text |
| GNU libiconv `libiconv-2.dll` | 1.15 (version stamp) | LGPL-2.0-or-later (1.2) | 1.2 | same zip | 3ee9786ab3eb8dfd791bdbd17c7e791dbe025734befcded0ee4170e1089f79df | **Lawyer.** LGPL for a library distributed in object form: ship the LGPL text, keep the DLL replaceable (it is a separate DLL), and give the corresponding source of that exact build or a written offer. See 1.2 |
| GNU gettext `libintl-9.dll` | 0.19.8 (version stamp) | LGPL-2.1-or-later (1.2) | 1.2 | same zip | 1125ac8dc0c4f5c3ed4712e0d8ad29474099fcb55bb0e563a352ce9d03ef1d78 | As libiconv |
| mingw-w64 winpthreads `libwinpthread-1.dll` | stamp "1, 0, 0, 0", "POSIX WinThreads for Windows", MingW-W64 Project | MIT AND BSD-3-Clause | mingw-w64 `mingw-w64-libraries/winpthreads/COPYING` | same zip | ffe2d56375bb4e8bdee9037df6befc5016ddd8871d0d85027314dd5792f8fdc9 | Both copyright notices (mingw-w64 project; Lockless Inc.) and both texts. EDB's own list puts "pthreads" under the LGPL 2.1 text (1.2): which applies to EDB's build is NOT VERIFIED, **Lawyer** |
| LZ4 `liblz4.dll` | 1.10.0 (version stamp) | BSD-2-Clause (the `lib/` directory) | github.com/lz4/lz4/blob/dev/lib/LICENSE | same zip | 096af775241b3bd4b1c3d79c83b103bb8f02da54ec4cd76c5d49eef61a68ad01 | Copyright notice and licence text |
| Zstandard `libzstd.dll` | 1.5.6 (version stamp) | BSD-3-Clause OR GPL-2.0-only (we use the BSD option) | github.com/facebook/zstd (README "License") | same zip | 7ca219a719343f4a48acbcb2955dc2630b84a86214eedd107f8114455860ae8c | Copyright notice and BSD text |
| zlib `zlib1.dll` | 1.3.1 (version stamp) | Zlib | github.com/madler/zlib/blob/develop/LICENSE | same zip | 8af75452efadd51320a71cc1db155f5cbc6bbc860d4473940ebc6de5ac662e68 | No notice duty in binaries; keep the text anyway |

Notes:

- The nine client DLLs come from EDB 17.4-1; `3rdparty.toml`
  `[components.libpq_win]` names `postgresql-17.4-1-…zip` for all variants
  with the workflow's verified checksum `80bb49…23c4`. The per-file SHA-256
  values above are of the files inside that pinned zip, read on 2026-11-27.
  The copies in a built installer were not read.
- The CUDA 13.1 llama.cpp archive (b7345) and the two model files that an
  earlier draft listed as shipped are not in the release workflow. They are
  downloaded (section 2).
- `incus.exe` (agent workspace on WSL) is built from `desktop/windows/incus-wsl`
  and shipped as `installers/incus.exe`. The source is MIT (see its Cargo.toml).

### 1.1 Inside the PostgreSQL 18.6-1 zip

Read from the zip's file list (21,968 entries) and from the files named
here. Nothing else in the zip was opened.

| Part | Version, and how it was read | Licence | Note |
|---|---|---|---|
| Server and client programs (`pgsql/bin/*.exe`, `pgsql/lib`, `pgsql/share`) | 18.6-1 (the zip's name) | PostgreSQL | `pgsql/server_license.txt`: "Portions Copyright (c) 1996-2025, The PostgreSQL Global Development Group" |
| `pgsql/bin/libiconv-2.dll` | 1.19 (version stamp) | LGPL-2.1-or-later (1.2) | SHA-256 6095b3fad19fff040aeae05865c28fdeb455526fca109c3216f6ca737af60249. A different build from the 1.15 DLL shipped loose |
| `pgsql/bin/libintl-9.dll` | 0.19.8 (version stamp) | LGPL-2.1-or-later (1.2) | The same file as the one shipped loose (equal SHA-256) |
| `pgsql/bin/libcrypto-3-x64.dll`, `libssl-3-x64.dll` | OpenSSL 3.5.7 (version stamps; "OpenSSL 3.5.7 9 Jun 2026") | Apache-2.0 | SHA-256 e864c2b31452d84892e885adfc07f151964bc9065134f7254b47194db253eaf7 / 45041ed336e6658ac796424431eb8b22c08e5a51982155bf94d727a5442dfba7 |
| `pgsql/bin/libwinpthread-1.dll`, `liblz4.dll` | the same files as the ones shipped loose (equal size and CRC-32 in both file lists) | as in section 1 | — |
| `pgsql/bin/libzstd.dll` | 1.5.7 (version stamp) | BSD-3-Clause OR GPL-2.0-only | SHA-256 287e7e474961e8886a3d961e2c07da647a7ac8d60d97023545f3a3ecf604daba |
| `pgsql/bin/zlib1.dll` | 1.3.2 (version stamp) | Zlib | SHA-256 890afa7a17fb66308e0026631070409138b157ef2773c0a41d22a76943f7aedf |
| ICU (`icudt77.dll`, `icuin77.dll`, `icuio77.dll`, `icutu77.dll`, `icuuc77.dll`), `libcurl.dll`, `libxml2.dll`, `libxslt.dll` | NOT VERIFIED | NOT VERIFIED. EDB's `pgsql/commandlinetools_3rd_party_licenses.txt` has a "libxml2, libxslt license" section; it has no section for ICU or curl | — |
| StackBuilder (`pgsql/bin/stackbuilder.exe`, eight `wx…3211u…` DLLs, `pgsql/StackBuilder`) | NOT VERIFIED | NOT VERIFIED. `pgsql/StackBuilder_3rd_party_licenses.txt` holds a curl notice only | Carried in the zip. Whether the app ever starts it: NOT VERIFIED |
| pgAdmin 4 (`pgsql/pgAdmin 4`: 17,503 of the entries, with its own Python and a browser runtime) | NOT VERIFIED | `pgsql/pgAdmin_license.txt`: "This software is released under the PostgreSQL licence." Third parties are in `pgsql/pgAdmin_3rd_party_licenses.txt`; its headings include "QT, gettext, libiconv, pthreads license" (LGPL 2.1 text), "xulrunner license" and "python license" | Carried in the zip. Whether the app ever starts it: NOT VERIFIED. **Lawyer:** passing the zip on whole also passes on pgAdmin 4 and what it bundles |

The zip's five licence files are `pgsql/server_license.txt`,
`pgsql/commandlinetools_3rd_party_licenses.txt`, `pgsql/pgAdmin_license.txt`,
`pgsql/pgAdmin_3rd_party_licenses.txt` and
`pgsql/StackBuilder_3rd_party_licenses.txt`.

### 1.2 LGPL libraries: version, licence and source

Two GNU libraries reach the user's machine in object form, in two places: as
loose DLLs in `installers/` (from the EDB 17.4-1 zip) and inside the 18.6-1
zip (`pgsql/bin`).

**GNU libiconv 1.15** (`libiconv-2.dll`, loose). Licence: LGPL-2.0-or-later,
that is the GNU Library General Public License, version 2 or later.

- Release news of libiconv 1.17,
  https://savannah.gnu.org/forum/forum.php?forum_id=10175 : "The libiconv
  library is now licensed under the LGPL version 2.1, instead of the LGPL
  version 2.0." Version 1.15 is older than 1.17.
- Tag `v1.15` of https://git.savannah.gnu.org/git/libiconv.git
  (swh:1:rev:941f63afe17adcb01e150e29ddb50c6fd40f1291):
  - `README` (swh:1:cnt:19f8370ab3453b2a742b76fb9e2babaef89aac39): "The
    libiconv and libcharset _libraries_ and their header files are under
    LGPL, see file COPYING.LIB."
  - `COPYING.LIB` (swh:1:cnt:778d0bb5b2a459f6f28ccb2e8aad409c2b497e50)
    begins "GNU LIBRARY GENERAL PUBLIC LICENSE Version 2, June 1991".
  - `lib/iconv.c` (swh:1:cnt:31853a7f1c47871221189dbf597473a16d8a8da7):
    "either version 2 of the License, or (at your option) any later version".
- **Lawyer.** In the same tag, `woe32dll/export.h`
  (swh:1:cnt:23eae9ab8e184308f918deec434673e545a8ad16) and
  `woe32dll/iconv-exports.c`
  (swh:1:cnt:9db49b7df263bee1219c00e20fa27a43fbcbd7e0), which some Windows
  DLL builds compile in, carry a GPL header ("either version 3 of the
  License, or (at your option) any later version"). The maintainer answered a
  report about version 1.15 on 2019-05-07: "Oops, this was an oversight. I've
  fixed it through this commit now"
  (https://lists.gnu.org/archive/html/bug-gnu-libiconv/2019-05/msg00003.html).
  Whether EDB's build of this DLL compiled those two files: NOT VERIFIED.
- Source of that version:
  https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz (5,264,188 bytes,
  listed 2017-02-02), signature
  https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz.sig . SHA-256: not
  hashed. GNU publishes the signature only; the release news
  (https://savannah.gnu.org/news/?id=8754) gives no checksum, and the tarball
  was not downloaded.

**GNU libiconv 1.19** (`libiconv-2.dll` inside the 18.6-1 zip). Licence:
LGPL-2.1-or-later.

- Tag `v1.19` of the same repository
  (swh:1:rev:a8850dc314a906b7bbe49a7c60424c7b10f57236): `COPYING.LIB`
  (swh:1:cnt:f6683e74e0f0130f246dbf7054f4cacc88ca84b0) begins "GNU LESSER
  GENERAL PUBLIC LICENSE Version 2.1, February 1999"; `lib/iconv.c`
  (swh:1:cnt:ec704cfcbe72e8807b37627a8546f5ab5d39c6e1): "either version 2.1
  of the License, or (at your option) any later version". In this tag
  `woe32dll/export.h` (swh:1:cnt:034c73bc28718b7b6a9d26d6e7974659a2b1d4ff)
  is under the Lesser GPL too.
- Source of that version:
  https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz (5,921,103 bytes;
  the address https://www.gnu.org/software/libiconv/ gives), signature
  https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz.sig . SHA-256: not
  hashed (no checksum page was found; not downloaded).

**GNU gettext libintl 0.19.8** (`libintl-9.dll`, loose and inside the 18.6-1
zip: the same file). Licence: LGPL-2.1-or-later.

- Tag `v0.19.8` of https://git.savannah.gnu.org/git/gettext.git
  (swh:1:rev:5a0f9fee95295945d7769b5724e3845dd1aa31cc):
  - `gettext-runtime/COPYING`
    (swh:1:cnt:9045d4cd3fdc2f192776f6663ff5411ded29f8a1): "The following
    parts are under the LGPL, see file intl/COPYING.LIB: - the libintl and
    libasprintf libraries and their header files".
  - `gettext-runtime/intl/COPYING.LIB`
    (swh:1:cnt:c2c87d69256345eb6a287aaac5645fc6185ea885) begins "GNU LESSER
    GENERAL PUBLIC LICENSE Version 2.1, February 1999".
  - `gettext-runtime/intl/gettext.c`
    (swh:1:cnt:e2c46755324dd7edd5a70972286573b4a66daf36): "either version 2.1
    of the License, or (at your option) any later version".
- Source of that version:
  https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz (19,977,131
  bytes, listed 2016-06-09), signature
  https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz.sig . The release
  announcement
  (https://lists.gnu.org/archive/html/info-gnu/2016-06/msg00003.html)
  publishes MD5 7ad5c90e32ac6828de955a0432ab4a7c and SHA-1
  4b3dc947a729fac8a14bde473454730919382b23 for that file. SHA-256: not hashed
  (GNU published none; not downloaded).

Texts in `licenses/`: `LGPL-2.0.txt` (libiconv 1.15) and `LGPL-2.1.txt`
(libiconv 1.19, libintl 0.19.8). Both were compared on 2026-11-27 with
https://www.gnu.org/licenses/old-licenses/lgpl-2.0.txt and
https://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt : the same text, line
endings aside.

EDB's build. EDB's list inside the 18.6-1 zip,
`pgsql/commandlinetools_3rd_party_licenses.txt`, has one section headed
"gettext, libiconv, pthreads license" with the LGPL 2.1 text. It gives no
version, no source address and no written offer. The 17.4-1 zip has no such
file at its top level. The exact source and build recipe EDB used for these
DLLs: NOT VERIFIED. The upstream tarballs above are the source of the
versions the DLLs are stamped with, not proof of what EDB compiled.
**Lawyer:** whether that meets the LGPL for a redistributor.

## 2. Downloaded, not in the installer

### 2.1 When the person uses a local model, or when Windows lacks a part

| Component | Version | Licence (SPDX) | Upstream licence read | When | SHA-256 (Windows x64) | Obligations |
|---|---|---|---|---|---|---|
| llama.cpp, CPU x64 | b12483 | MIT | github.com/ggml-org/llama.cpp/blob/b12483/LICENSE | local model mode | 7c3ec4ff5cdddc272cb1d6b52a05c8751dc566c7105750c10ed3688bbdbf8fa8 | "Copyright (c) 2023-2026 The ggml authors" and the MIT text |
| llama.cpp, Vulkan x64 | b12483 | MIT | as above | when selected | 7859e96ba91201a41e2769d4456fe86e8968ba93c6794eeb4fe49b7537a17628 | as above |
| llama.cpp, CUDA 12.4 / 13.5 x64 | b12483 | MIT | as above | when selected | 89dcd32dcd0d0b99daaa5150ff0107cb2f47a6dc5a269e23c2c10e86ba4cc9f2 / 3467a6cf27418a5265428e3eaea1249daa7aba40ca75b594dca47083104cbe3e | as above |
| llama.cpp, CPU ARM64 | b12483 | MIT | as above | ARM64 machines | bbf6f781a01fb0f3ce0adc17a96ebda8200477dc58b8f6c0367ab7317db1b5b8 | as above |
| LLVM OpenMP runtime (inside the llama.cpp zips, per the earlier draft) | — | Apache-2.0 WITH LLVM-exception | github.com/llvm/llvm-project/blob/main/openmp/LICENSE.TXT | with llama.cpp | — | Keep the licence file. Presence in the zips NOT VERIFIED |
| NVIDIA CUDA runtime, from the llama.cpp release (`cudart-llama-bin-win-cuda-12.4-x64.zip`, 391,443,627 bytes, holding `cublas64_12.dll`, `cublasLt64_12.dll`, `cudart64_12.dll` and no licence file; `…-13.5-x64.zip`, 426,314,199 bytes, file list not read) | 12.4 / 13.5 | LicenseRef-NVIDIA-CUDA-EULA (proprietary) | docs.nvidia.com/cuda/eula (Attachment A lists them as redistributable) | with a CUDA build only | 8c79a9b226de4b3cacfd1f83d24f962d0773be79f1e7b75c6af4ded7e32ae1d6 / c2bf4cc202bfbaaa7a0b75f5082457c3811326947bb216059b72839d844e9cba | EULA terms for distributors: the application must add material functionality; only the application may use the libraries; our terms must be consistent with the EULA. We do not ship them; the machine fetches them from the ggml-org release. The notice to show before that download is in section 9. **Lawyer:** whether that counts as our distribution |
| Microsoft Edge WebView2 Runtime | Evergreen | LicenseRef-Microsoft-WebView2-Runtime (proprietary) | learn.microsoft.com/microsoft-edge/webview2/concepts/distribution | only if missing | — | Part of Windows 11. The desktop bundle configuration sets no `webviewInstallMode`, so the Tauri default applies (download the bootstrapper from Microsoft); NOT VERIFIED in a built installer. The runtime comes from Microsoft under Microsoft's terms; we do not redistribute it |

### 2.2 Only if a switched-off service is switched on

The product file switches the services `cache`, `vector_db` and `directory`
off. A switched-off service is never installed or started. Naming it in the
environment variable `GB_SERVICES_ON` (for example `GB_SERVICES_ON=directory`),
or removing it from `services_off` in the product file, switches it on again;
the server then fetches the archive from the address pinned in
`3rdparty.toml`, unless `GB_DESKTOP_OFFLINE=1` forbids downloads. Read in
`botserver/crates/botcore/src/product/services.rs` and
`botapp/src/desktop/sidecar.rs`; the download itself was not tried.
Installers before 2026-11-27 carried these three archives, and the product
file says that binaries an earlier version left on disk stay untouched.

| Component | Version | Licence (SPDX) | Upstream licence read | When | SHA-256 (Windows x64) | Obligations |
|---|---|---|---|---|---|---|
| Qdrant (`qdrant.exe`, service `vector_db`) | 1.20.0 | Apache-2.0 | github.com/qdrant/qdrant/blob/v1.20.0/LICENSE | only if `vector_db` is switched on | 6d13553584fd0411f00637519e293331ff0aa1fb79bd9e30db45839bb75de2be | Ship the Apache-2.0 text. Upstream has no `NOTICE` at this tag. The notices of the Rust crates compiled into the binary are not published by upstream: NOT VERIFIED |
| Zitadel (`zitadel.exe`, service `directory`) | 4.20.1 | AGPL-3.0-only; `proto/`, `apps/docs/` Apache-2.0; `apps/login/`, `packages/zitadel-client/`, `packages/zitadel-proto/` MIT | github.com/zitadel/zitadel/blob/v4.20.1/LICENSE and `LICENSING.md` | only if `directory` is switched on | dd38db24626a3f24f9e22244163f4ba50527378a4237cd19f0f6578b61c39405 | See "AGPL" below, plus the MIT notice for the login UI. Upstream `LICENSING.md` itself recommends legal counsel. It stays in `SOURCE-OFFER.md` because the app can still fetch exactly this version. **Lawyer:** whether a download the app starts from the upstream release counts as our distribution |
| Caddy (`caddy.exe`, components `caddy` and `proxy`) | 2.11.9 | Apache-2.0 | github.com/caddyserver/caddy/blob/v2.11.9/LICENSE | never by itself: it is not in the list of components the desktop start-up installs, and it is not one of the three switchable services. Only the server's `install` command for that component would fetch it (not tried) | ad93112d335ee687100c4c9f11bced16ca590377976b7fe7883882b9e797e472 | Keep the licence. No `NOTICE` upstream |

## 3. Models

The weights are downloaded only when the person chooses a local model. None
is in the installer. Licences are those on the model cards; the files are
quantised conversions of the original models.

**Catalogue as shipped since 2026-11-27.** The app offers five files, each with
its exact name, size and SHA-256 pinned in the catalogue:
DeepSeek-R1-Distill-Qwen-7B Q4_K_M, Qwen3.5-9B Q4_K_M and Qwen3.8-27B in
UD-Q3_K_XL, UD-Q4_K_XL and UD-Q6_K. GLM-4-9B, Qwen3.5-14B and "DeepSeek V3
(bundled)" are no longer offered; the flags and rows about them below are
kept as the record of why, and the GLM-4 licence text is no longer shipped.
A model file a person already has keeps working and is listed as not in the
catalogue.

**Flags for a product offered to government offices** (2026-11-27):

- **GLM-4-9B is not under a permissive licence.** Its card names "The
  glm-4-9b License", a licence of its own: free for academic research,
  commercial use only after registration with the author, a "Built with
  glm-4" display and a copy of the licence required, no military use,
  revocable, under Chinese law. Details in the table. **Lawyer** before it is
  offered.
- **"DeepSeek V3 (bundled)" is labelled MIT in the catalogue, and the model
  card says otherwise.** The card of deepseek-ai/DeepSeek-V3 puts the weights
  under a separate "Model License" with use restrictions. The catalogue entry
  names no repository, so which file it means is NOT VERIFIED.
- **Qwen3.5-14B: NOT VERIFIED.** Hugging Face answered HTTP 401 for the card,
  the repository record and the file (the page shown is its "404" page), and
  the same for Qwen/Qwen3.5-14B.
- Two catalogue files do not exist under the name the catalogue gives
  (`Qwen3.8-27B-Q3_K_XL.gguf`, `Qwen3.8-27B-Q6_K.gguf`: HTTP 404), and two
  sizes in the catalogue differ from the published size. These are not
  licence matters; they are in the Status column because a download that
  fails cannot be licensed either.
- The other entries (DeepSeek-R1-Distill-Qwen-7B, Qwen3.5-9B, Qwen3.8-27B)
  are MIT or Apache-2.0 on their cards: no limit on the field of use, no
  acceptable-use policy; the licence and copyright notices must stay with
  copies.

### 3.1 The local model catalogue (Settings > Model)

One row per entry of `catalog.rs`. The app downloads
`https://huggingface.co/<repository>/resolve/main/<file>`.

| Catalogue entry | File the catalogue names (repository) | Licence on the model card | Card read | SHA-256 and size published by Hugging Face | Does the licence restrict use? | Status on 2026-11-27 |
|---|---|---|---|---|---|---|
| `deepseek-r1-distill-qwen-7b-q4km` DeepSeek-R1-Distill-Qwen-7B, Q4_K_M | `DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf` (bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF) | MIT. The GGUF card states no licence in its header; its repository holds a `LICENSE` file equal to DeepSeek's MIT text ("Copyright (c) 2023 DeepSeek"). The original card says `license: mit` and "This code repository and the model weights are licensed under the MIT License." Base model per that card: Qwen2.5-Math-7B, `license: apache-2.0` on its card | https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF ; https://huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-7B ; https://huggingface.co/Qwen/Qwen2.5-Math-7B | 731ece8d06dc7eda6f6572997feb9ee1258db0784827e642909d9b565641937b ; 4,683,073,504 bytes | No. MIT, and Apache-2.0 for the base: keep the notices | File exists. Digest and size equal the catalogue's |
| `qwen3.5-9b-q4km` Qwen3.5-9B, Q4_K_M | `Qwen3.5-9B-Q4_K_M.gguf` (unsloth/Qwen3.5-9B-GGUF) | Apache-2.0 (`license: apache-2.0`; licence link to the Qwen/Qwen3.5-9B `LICENSE`, the Apache-2.0 text with "Copyright 2026 Alibaba Cloud"). Base model: Qwen/Qwen3.5-9B, `license: apache-2.0` | https://huggingface.co/unsloth/Qwen3.5-9B-GGUF ; https://huggingface.co/Qwen/Qwen3.5-9B | 03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8 ; 5,680,522,464 bytes | No. Apache-2.0: keep the licence and notices | File exists. No digest in the catalogue. The catalogue's size (5,905,580,032) differs from the published one |
| `glm-4-9b-chat-q4km` GLM-4-9B, Q4_K_M | `glm-4-9b-chat-Q4_K_M.gguf` (bartowski/glm-4-9b-chat-GGUF) | LicenseRef-GLM-4 (`license: other`, `license_name: glm-4`), "The glm-4-9b License". Base model: THUDM/glm-4-9b-chat, which Hugging Face now serves as zai-org/glm-4-9b-chat, same licence | https://huggingface.co/bartowski/glm-4-9b-chat-GGUF ; https://huggingface.co/THUDM/glm-4-9b-chat ; https://huggingface.co/THUDM/glm-4-9b-chat/blob/main/LICENSE | aa6cb8f5ef0a70399bdbf92eef566c26c6017ee0b131a424b65b5c757f1c81a2 ; 6,250,926,848 bytes | **Yes.** From the licence text: "use all open source models in this repository for free for academic research"; commercial use only after registration at https://open.bigmodel.cn/mla/form ; whoever provides the model or "products or services that use" it must "Provide a copy of this Agreement" and "Prominently display "Built with glm-4" on the relevant website, user interface, blog post, related page or product documentation"; no use "for any military or illegal purposes", and no "behavior that endangers national security and unity"; the grant is "non-transferable, non-sublicensable, revocable"; governed by "the laws of People's Republic of China", disputes go to the "Haidian District People's Court in Beijing". **Lawyer** | File exists. Digest and size equal the catalogue's. The licence text is in `licenses/glm-4-9b-LICENSE.txt`; the app does not show "Built with glm-4" |
| `qwen3.5-14b-q4km` Qwen3.5-14B, Q4_K_M | `Qwen3.5-14B-Q4_K_M.gguf` (unsloth/Qwen3.5-14B-GGUF) | NOT VERIFIED. The catalogue says Apache-2.0 | https://huggingface.co/unsloth/Qwen3.5-14B-GGUF answered HTTP 401; so did https://huggingface.co/Qwen/Qwen3.5-14B | NOT VERIFIED | NOT VERIFIED | The download address answers HTTP 401: the file cannot be fetched without an account, or the repository does not exist |
| `qwen3.8-27b-q3kxl` Qwen3.8-27B, Q3_K_XL | `Qwen3.8-27B-Q3_K_XL.gguf` (unsloth/Qwen3.8-27B-GGUF) | Apache-2.0 (`license: apache-2.0`). Base model: Qwen/Qwen3.8-27B, `license: apache-2.0`, `LICENSE` equal to the Qwen3.5-9B one | https://huggingface.co/unsloth/Qwen3.8-27B-GGUF ; https://huggingface.co/Qwen/Qwen3.8-27B | none: no such file | No. Apache-2.0: keep the licence and notices | The download address answers HTTP 404. The repository has `Qwen3.8-27B-UD-Q3_K_XL.gguf` (7b20f0679da6f5c6ec7971e241630f44fe1e123e5c99bfea90c433268b355723 ; 13,146,393,504 bytes) |
| `qwen3.8-27b-udq4kxl` Qwen3.8-27B, UD-Q4_K_XL | `Qwen3.8-27B-UD-Q4_K_XL.gguf` (same repository) | as above | as above | 2a5f3565f39cf885103da591182c1266da64745737b42b452fd3e675369f1211 ; 17,659,407,264 bytes | No | File exists. No digest in the catalogue. The catalogue's size (19,240,000,000) differs from the published one |
| `qwen3.8-27b-q6` Qwen3.8-27B, Q6_K | `Qwen3.8-27B-Q6_K.gguf` (same repository) | as above | as above | none: no such file | No | The download address answers HTTP 404. The repository has `Qwen3.8-27B-UD-Q6_K.gguf` (6cac55f876015d6d4b94b1656d23f79f424ac9b583652aedf0633dba116576a8 ; 21,980,441,504 bytes) |
| `deepseek-v3-bundled` "DeepSeek V3 (bundled)", server profile only | `deepseek-v3-q4.gguf` (no repository in the catalogue; not in the installer; no download) | NOT VERIFIED for that file. The catalogue says MIT. The card of deepseek-ai/DeepSeek-V3 says: "This code repository is licensed under the MIT License. The use of DeepSeek-V3 Base/Chat models is subject to the Model License. DeepSeek-V3 series (including Base and Chat) supports commercial use." | https://huggingface.co/deepseek-ai/DeepSeek-V3 ; https://huggingface.co/deepseek-ai/DeepSeek-V3/blob/main/LICENSE-MODEL | NOT VERIFIED | **Yes, by the card's Model License** ("DEEPSEEK LICENSE AGREEMENT Version 1.0, 23 October 2023"): use restrictions that must be passed on to users, among them no use "For military use in any way" and none "For fully automated decision making that adversely impacts an individual's legal rights"; a copy of the licence must go with the model; "governed and construed under PRC laws". **Lawyer** | The entry cannot be installed from the app. Its "MIT" label disagrees with the card |

### 3.2 Models named in `3rdparty.toml`, and candidates

| Model | File (bytes) | Licence | Upstream | SHA-256 (matches the Hugging Face LFS digest) | Obligations | Status |
|---|---|---|---|---|---|---|
| DeepSeek-R1-Distill-Qwen-1.5B, GGUF by bartowski | DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf (924,456,032) | MIT (deepseek-ai card). Base model Qwen2.5-Math-1.5B: Apache-2.0, itself from Qwen2.5-1.5B: Apache-2.0. The bartowski repository declares no licence of its own | huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B | b1094879ee7e222cba5be013e6de2d0658c64f91aef5d3c9bff0db5cefbfa020 (not pinned in `3rdparty.toml`; size and digest seen again on 2026-11-27) | MIT text; Apache-2.0 text for the Qwen base; say the file is a quantised derivative | In use (local mode) |
| bge-small-en-v1.5, GGUF by CompendiumLabs | bge-small-en-v1.5-f32.gguf (133,609,568) | MIT (both cards) | huggingface.co/BAAI/bge-small-en-v1.5 | bf40c42ad7d89382e9ba7376d5c4b73f6b556cb541fab37aaa1da9c320149b65 (not pinned; size and digest seen again on 2026-11-27) | MIT text | In use (local mode) |
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
| `cache` | Valkey Cache (Linux/macOS; Windows uses a built-in RESP cache, no external binary) | 9.1.3 | BSD-3-Clause | https://download.valkey.io/ | not used on Windows (built-in cache) |
| `vector_db` | Qdrant Vector Database | 1.20.0 | Apache-2.0 | https://github.com/qdrant/qdrant | `licenses/qdrant-LICENSE.txt`; Edition: not in the installer; switched off; downloaded only if switched on |
| `directory` | Zitadel Identity Provider | 4.20.1 | AGPL-3.0-only | https://github.com/zitadel/zitadel | `licenses/zitadel-LICENSE.txt`, `zitadel-LICENSING.md`, `zitadel-login-LICENSE.txt`; Edition: not in the installer; switched off; downloaded only if switched on |
| `caddy` | Caddy Web Server | 2.11.9 | Apache-2.0 | https://github.com/caddyserver/caddy | `licenses/caddy-LICENSE.txt`; Edition: not in the installer; not installed or started |
| `proxy` | Caddy (same archive as `caddy`) | 2.11.9 | Apache-2.0 | https://github.com/caddyserver/caddy | `licenses/caddy-LICENSE.txt`; Edition: not in the installer; not installed or started |
| `tables` | PostgreSQL Database (EDB binaries on Windows) | 18.6-1 | PostgreSQL | https://github.com/postgres/postgres | `licenses/postgresql-COPYRIGHT.txt`; inside the zip `pgsql/server_license.txt`; the other parts of the zip: see section 1.1; Edition: shipped (the whole zip is in the installer) |
| `libpq_win` | PostgreSQL client libraries | 17.4-1 | PostgreSQL | https://github.com/postgres/postgres | `licenses/postgresql-COPYRIGHT.txt`; the other DLLs of the zip: see section 1; Edition: shipped |
| `llm_win_cpu_x64` | llama.cpp (Windows x64 CPU) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | `licenses/llama.cpp-LICENSE.txt`; Edition: downloaded |
| `llm_win_cpu_arm64` | llama.cpp (Windows ARM64 CPU) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_cuda12` | llama.cpp (Windows CUDA 12.4) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_cuda13` | llama.cpp (Windows CUDA 13.5) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `llm_win_vulkan` | llama.cpp (Windows Vulkan) | b12483 | MIT | https://github.com/ggml-org/llama.cpp | as above |
| `cudart_win_cuda12` | CUDA runtime (CUDA 12.4) | 12.4 | LicenseRef-NVIDIA-CUDA-EULA | https://docs.nvidia.com/cuda/eula/ | EULA; Edition: downloaded with a CUDA build; notice in section 9 |
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

- **AGPL-3.0** (MinIO, mc; Zitadel when it is fetched, section 2.2). Shipping
  the binary is "conveying object code" (§6). That needs the Corresponding
  Source by one of §6a–6e. §6d allows a network location, including someone
  else's server, with clear directions next to the binary, and the distributor
  stays responsible for it staying available. §13 (offer source to remote
  network users) applies to modified versions; these binaries are unmodified
  and listen on the local machine only. Ship the licence text and the upstream
  `NOTICE`. MinIO and mc upstream repositories are archived (read-only); the
  tags remain. **Lawyer:** whether a link to the upstream tag is enough for
  §6d, and whether "Corresponding Source" must include the Go modules of that
  build.
- **MPL-2.0** (OpenBao). Ship the licence and say where the source of v2.7.3 is.
- **LGPL** (libiconv 1.15 under LGPL-2.0-or-later; libiconv 1.19 and libintl
  0.19.8 under LGPL-2.1-or-later). Ship both texts, keep the DLLs replaceable,
  give or offer the source of the exact build. Versions, licence and upstream
  source are in section 1.2; the source of EDB's own build is not located.
- **BSD / MIT / Zlib** (winpthreads, lz4, zstd, zlib, llama.cpp). Reproduce
  the copyright lines and texts in the documentation.
- **Apache-2.0** (OpenSSL, the Qwen models and bases; Qdrant and Caddy when
  they are fetched). Ship the text and any upstream `NOTICE` (none found for
  Qdrant, Caddy).
- **Model licences with conditions** (GLM-4-9B; the DeepSeek-V3 Model
  License). See section 3: a copy of the licence, a "Built with glm-4"
  display, registration for commercial use, and limits on the field of use.
- **Proprietary** (CUDA runtime, WebView2 Runtime). Downloaded from the vendor
  or from the llama.cpp release, not redistributed by us; see section 2. The
  CUDA notice is in section 9.
- **Terms of an outside service** (the hosted free model). Not a software
  licence: section 8.

## 7. Open before a public release

1. Licence texts travel with the installer: `licenses/` holds the NOTICE,
   this record, `SOURCE-OFFER.md` and the texts fetched from each upstream tag
   on 2026-11-26. The release workflow now has a step that fails when the
   built installer lacks this record, the NOTICE, `SOURCE-OFFER.md` or every
   licence text (read in the workflow file; a run of it was not seen). Still
   missing: the notice list of the Rust crates compiled into `botserver.exe`
   and the desktop shell (section 5), and a licence file for
   `bge-small-en-v1.5` (upstream publishes none; the card says MIT).
2. `SOURCE-OFFER.md` says where the source of MinIO, mc, Zitadel, OpenBao and
   the LGPL libraries is. The versions of libiconv and libintl are settled
   (section 1.2). Not settled: the exact source of EDB's build of those DLLs;
   the GPL-headed files in libiconv 1.15's Windows build; and what else the
   PostgreSQL 18.6-1 zip passes on (pgAdmin 4, StackBuilder, ICU, curl,
   libxml2, libxslt: section 1.1). A payload with the server programs only
   would shorten that list. **Lawyer:** whether links to upstream tags satisfy
   AGPL-3.0 §6 and MPL-2.0 §3.2.
3. MinIO now ships RELEASE.2025-10-15T17-29-55Z (CVE-2025-62506 fixed), built
   from source. Redis 3.0.504 removed from the Windows payload.
4. `libpq_win` in `3rdparty.toml` agrees with the workflow (17.4-1) and
   carries the verified SHA-256. The two models of `3rdparty.toml` and six of
   the eight catalogue entries have no pinned SHA-256; section 3 gives the
   digests Hugging Face publishes for the files that exist.
5. The catalogue offers three files that cannot be downloaded as named
   (Qwen3.5-14B, Qwen3.8-27B Q3_K_XL and Q6_K) and gives two sizes that differ
   from the published ones (section 3.1).
6. PostgreSQL is in the installer now. The release still downloads llama.cpp,
   the CUDA runtime, the models and possibly WebView2 when they are first
   needed. That contradicts "no downloads at first boot" in `AGENTS.md` for the
   local model and fails on an offline machine.
7. **Lawyer:** the hosted free model. Its terms call it "Trial use only", say
   "do not submit personal or confidential data", forbid "governmental
   information" and allow the provider to use what is sent to improve its
   models (section 8). Whether a government office may use it at all, and what
   the app must show first.
8. **Lawyer:** GLM-4-9B in the catalogue (section 3): keep it with the
   licence's conditions met, or remove it. The same question for the "MIT"
   label of the "DeepSeek V3 (bundled)" entry.
9. The app shows no notice about NVIDIA's licence before it downloads the
   CUDA runtime (no such text was found in the interface or the server on
   2026-11-27). The text to show is in section 9.

## 8. Hosted free model: what leaves the machine

**What the app uses.** The "Modelo gratuito / Free model" choice of the
first-run wizard and of Settings > Model sends the conversation to the Kilo AI
Gateway, with no key. `botserver/crates/botsettings/src/model_config/mod.rs`
sets the address `https://api.kilo.ai/api/gateway/chat/completions`, the
model `nvidia/nemotron-3-ultra-550b-a55b:free` and the provider name `kilo`.
The gateway's public model list (https://api.kilo.ai/api/gateway/models) gives
`nvidia` as the company that runs that model. When the gateway answers that
the model no longer exists, the app picks another model whose name ends in
`:free` from that list (`botserver/src/core/bot/pipeline/llm/free_model_fallback.rs`),
so the company that runs the model can change without an update of the app.

**What is sent.** Each request carries the instructions the app gives the
model, the conversation so far, the new message, and the text the app read
from attached documents. Images are not sent to this address
(`botserver/crates/botllm/src/media_parts.rs`). The service sees the public
internet address of the machine or of the office network.

**What the pages say** (read 2026-11-27):

| Question | What the pages say | Source |
|---|---|---|
| Who receives the text? | Kilo Code Inc. and the company that runs the model. The privacy page lists among its service providers "OpenRouter - Routing layer that forwards your request to the AI provider you select per request" and "Anthropic, OpenAI, Google, and other AI providers - Generate AI responses to your prompts." Its part about the Kilo mobile app says: "Prompts and conversation content are sent to the AI provider you select per request, routed through OpenRouter." The documentation page carries the line "Kilo has been acquired by Anaconda." | https://kilo.ai/privacy (last updated May 29, 2026); https://kilo.ai/docs/gateway |
| Are prompts logged? | Yes, by the company that runs the model. "For NVIDIA free endpoints (Super/Ultra/etc): Trial use only - do not submit personal or confidential data. Your use is logged for security purposes and to improve NVIDIA products and services. The logged session data for improvement purposes is not linked to your identity or any persistent identifier." NVIDIA's terms, 3.3: "Your use of the API Services will be logged for security, fraud or abuse monitoring and shared with third party service providers for this purpose." Kilo: "Anonymous requests are identified by IP address". Whether Kilo itself keeps the text of requests made without an account: NOT VERIFIED, the pages do not say | https://kilo.ai/docs/gateway/models-and-providers ; https://assets.ngc.nvidia.com/products/api-catalog/legal/NVIDIA%20API%20Trial%20Terms%20of%20Service.pdf (v. September 19, 2025); https://kilo.ai/docs/gateway/authentication |
| For how long? | No period is stated for these free endpoints. NVIDIA's terms, 2.3: "Except as stated below in Section 2.4 or unless expressly disclosed to you for an API Service, NVIDIA will not store or use User Content or Generated Content at the end of each API Service session"; for these endpoints the logging quoted above is what is disclosed. NVIDIA's privacy policy: "We retain your personal data for as long as our engagement with you continues" and "We erase customer and enterprise personal data if there has been no engagement with you for more than five years." Kilo: "Data already shared with third-party AI providers to generate responses is governed by those providers' retention policies" | the NVIDIA terms above; https://www.nvidia.com/en-us/about-nvidia/privacy-policy/ (effective September 22, 2025); https://kilo.ai/privacy |
| Used for training? | Yes, it may be. The model list marks this model `"mayTrainOnYourPrompts": true`, and all 13 free models were marked so that day. NVIDIA's terms, 3.3: NVIDIA collects "User Content and Generated Content to improve NVIDIA products and services, including AI models." Kilo's terms: "You agree that any Customer Data provided to any AI Model through the Services will be used by the applicable AI Model in accordance with the applicable AI Model Terms. If you elect not to grant a license to any AI Model to use your Customer Data for training purposes, you may not be able to use certain AI Models." Kilo also takes "a perpetual, irrevocable, fully-paid, royalty-free, worldwide, sublicensable, transferable right and license to use such Customer Data to provide and improve the Services and Kilo's other products and services" | https://api.kilo.ai/api/gateway/models ; the NVIDIA terms above; https://kilo.ai/terms (last updated November 2, 2026) |
| Where is it processed? | In the United States, as far as the pages say. Kilo: "The Services are hosted in the United States and the personal information we collect will be stored and processed on our servers in the United States." NVIDIA: "NVIDIA is located in California, and in most cases we need to securely transfer and store your information in the United States." and "We are subject to U.S. law, and we may be required to share data with U.S. law enforcement agencies." Where the machines that run this model are: NOT VERIFIED, not stated. For another free model the app may fall back to: NOT VERIFIED | https://kilo.ai/privacy ; https://www.nvidia.com/en-us/about-nvidia/privacy-policy/ |
| What may not be sent? | Personal, confidential and government data. NVIDIA's terms, 1.2: access is "for limited trial purposes only and without use of the API Service or Generated Content in production"; 2.6(a): no "confidential information, controlled or sensitive data, including protected health information, personal data (unless expressly permitted by an API Service)"; 4.3: "you will not upload any personal information relating to an identifiable individual, financial, health or governmental information, or any other information which may be subject to data privacy or data security laws, rules, or regulations." Kilo's terms forbid sending "data or information considered to be sensitive or otherwise subject to specific protections under applicable laws" | the NVIDIA terms above; https://kilo.ai/terms |
| What does the service ask of the app? | "If you use the Kilo Gateway API to make the Services available through your application or other service ("Your Service"), you understand and agree that you are solely responsible for ensuring that end users of Your Service understand and comply with these Terms with respect to their use of the Services, including any usage limitations or restrictions." | https://kilo.ai/terms |
| Is there a use limit? | "The gateway allows unauthenticated access for free models only. Anonymous requests are identified by IP address and are subject to rate limiting (200 requests per hour per IP)." An office behind one public address shares that limit | https://kilo.ai/docs/gateway/authentication |

**What the app shows today.** The interface asks for confirmation before it
switches to the free model and calls the destination "um provedor externo /
an external provider" (`botui/ui/suite/assistant/modules/10_settings_model.js`
and `05b_model_picker.js`). It names neither the service nor its terms.

**Lawyer:**

- whether a government office may use a service whose terms say "Trial use
  only", "not in production", and forbid "governmental information" and
  personal data;
- whether the app must have each person accept Kilo's terms and NVIDIA's
  trial terms before the first message, since Kilo's terms make the
  application responsible for its users understanding them;
- whether sending a citizen's personal data to servers in the United States
  meets the data-protection law that applies to the office.

**Proposed notice in the app** (for the interface team to place: before the
free model is first used, in the wizard step and in the confirmation that
already exists, with links to https://kilo.ai/terms , https://kilo.ai/privacy
and NVIDIA's trial terms above). It names the provider as "today NVIDIA"
because the app can fall back to another free model.

- pt-BR: "O modelo gratuito roda fora deste computador: sua mensagem, a
  conversa e o texto dos arquivos anexados vão pela internet para o serviço
  Kilo, nos Estados Unidos, e para a empresa que opera o modelo (hoje a
  NVIDIA), que podem registrar o conteúdo e usá-lo para melhorar os próprios
  modelos e serviços. O serviço é só para teste: não envie dados pessoais,
  sigilosos ou de governo; para esses casos, use o modelo local."
- en: "The free model runs outside this computer: your message, the
  conversation and the text of attached files go over the internet to the
  Kilo service, in the United States, and to the company that runs the model
  (today NVIDIA), which may log the content and use it to improve their own
  models and services. The service is for trial use only: do not send
  personal, confidential or government data; for those, use the local model."

## 9. NVIDIA CUDA runtime: the notice before the download

**What happens.** When the app picks a CUDA build of llama.cpp for the local
model, it also downloads NVIDIA's runtime libraries to go next to it
(`botserver/crates/botcorepkg/src/installer/llama.rs`):
`cudart-llama-bin-win-cuda-12.4-x64.zip` (391,443,627 bytes) or
`cudart-llama-bin-win-cuda-13.5-x64.zip` (426,314,199 bytes), from
`https://github.com/ggml-org/llama.cpp/releases/download/b12483/`, the
release page of the llama.cpp project on GitHub. The 12.4 archive holds
`cublas64_12.dll`, `cublasLt64_12.dll` and `cudart64_12.dll`, and no licence
file. The installer does not carry them.

**What NVIDIA's licence page says**
(https://docs.nvidia.com/cuda/eula/index.html , "Last updated: January 26,
2026"):

- Acceptance: "If you don't have the required age or authority to accept
  this Agreement, or if you don't accept all the terms and conditions of this
  Agreement, do not download, install or use the SDK." And: "If you are
  entering into this Agreement on behalf of a company or other legal entity,
  you represent that you have the legal authority to bind the entity to this
  Agreement".
- 1.1.2, Distribution Requirements: "Your application must have material
  additional functionality, beyond the included portions of the SDK." "The
  distributable portions of the SDK shall only be accessed by your
  application." "The terms under which you distribute your application must
  be consistent with the terms of this Agreement, including (without
  limitation) terms relating to the license grant and license restrictions
  and protection of NVIDIA's intellectual property rights. Additionally, you
  agree that you will protect the privacy, security and legal rights of your
  application users." "You agree to notify NVIDIA in writing of any known or
  suspected distribution or use of the SDK not in compliance with the
  requirements of this Agreement".
- Wording NVIDIA requires to be shown: the page prescribes one sentence, and
  only for sample source code: "The following notice shall be included in
  modifications and derivative works of sample source code distributed: "This
  software contains source code provided by NVIDIA Corporation."" The app
  ships no NVIDIA sample code. No wording is prescribed for an application
  that uses the runtime libraries. The page also says: "You may not reverse
  engineer, decompile or disassemble, or remove copyright or other
  proprietary notices from any portion of the SDK or copies of the SDK."
- Attachment A lists `cudart`, `cublas` and `cublasLt` among the files that
  "may be distributed with applications developed by you".

**Lawyer:** whether the person at a government office who clicks has the
"legal authority to bind the entity"; and whether a download the app starts
from the llama.cpp release counts as our distribution (section 2.1).

**The notice the app should show before that download** (for the interface
team to place; the archive name and size come from the build that was
chosen):

- pt-BR: "Para usar a placa NVIDIA, o General Bots Lite vai baixar as
  bibliotecas de execução NVIDIA CUDA (arquivo
  cudart-llama-bin-win-cuda-12.4-x64.zip, cerca de 390 MB) da página de
  versões do projeto llama.cpp no GitHub
  (github.com/ggml-org/llama.cpp/releases). Essas bibliotecas são software da
  NVIDIA, não do General Bots, e estão sob a licença da NVIDIA, o Contrato de
  Licença de Usuário Final do CUDA Toolkit: https://docs.nvidia.com/cuda/eula/
  . Ao continuar, você declara que leu e aceita esses termos; se não aceitar,
  cancele e nada será baixado."
- en: "To use the NVIDIA card, General Bots Lite will download the NVIDIA
  CUDA runtime libraries (file cudart-llama-bin-win-cuda-12.4-x64.zip, about
  390 MB) from the release page of the llama.cpp project on GitHub
  (github.com/ggml-org/llama.cpp/releases). These libraries are NVIDIA
  software, not part of General Bots, and come under NVIDIA's licence, the
  CUDA Toolkit End User License Agreement: https://docs.nvidia.com/cuda/eula/
  . By continuing you confirm that you have read and accept those terms; if
  you do not, cancel and nothing will be downloaded."
