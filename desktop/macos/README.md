# General Bots desktop - macOS

**Status: best effort, unsigned.** GitHub Actions builds an arm64 `.app` and
`.dmg` on a `macos-14` runner with
[`.github/workflows/desktop-macos.yml`](../../.github/workflows/desktop-macos.yml).
The job is `continue-on-error`: a macOS toolchain gap or a missing darwin
component must not hide the Linux installer from a release.

## Bundle configuration

| File | Purpose |
|---|---|
| `botapp/tauri.conf.json` | Base config; on macOS it contributes the app windows, UI assets and `bundle.macOS.minimumSystemVersion = "11.0"`. |
| `botapp/tauri.desktop-local.conf.json` | Desktop-local override: `externalBin: ["binaries/botserver"]`, the `.product.desktop-local` and `installers/` resources, and the Linux/Windows installer settings. |
| `botapp/tauri.macos.conf.json` | macOS override: `bundle.targets = ["dmg", "app"]`, the ICNS icon and `bundle.macOS.minimumSystemVersion`. |

The workflow passes both overrides, in this order:

```bash
cd botapp
cargo tauri build \
  --features desktop-local,custom-protocol \
  --config tauri.desktop-local.conf.json \
  --config tauri.macos.conf.json \
  --bundles dmg,app
```

The second `--config` wins on conflicting keys, so macOS builds only the
`.dmg`/`.app` even though the desktop-local config also lists NSIS/MSI/deb/rpm.
`bundle.externalBin` resolves `binaries/botserver` to the target-triple suffix,
so the build stages `botapp/binaries/botserver-aarch64-apple-darwin`.

`desktop/macos/stage-sidecar.sh [target-triple]` performs that staging and
refuses a non-Mach-O or wrong-architecture file. It also keeps Linux stack
archives out of `botserver/botserver-installers`; only the botserver sidecar,
the suite UI and the desktop-local resources are bundled.

## Artifacts

For a `vMAJOR.MINOR.PATCH` product tag the workflow publishes a
`vMAJOR.MINOR.PATCH-macos` pre-release (it never takes `latest`) with:

- `generalbots-<version>-macos-arm64.dmg`
- `generalbots-<version>-macos-arm64.app.zip` (the raw `.app`, zipped for release)
- `SHA256SUMS-macos`

The artifact is named `-macos-arm64`; the workflow asserts that the runner and
Rust host are `aarch64-apple-darwin` before building, so it cannot silently
publish an Intel build under the arm64 name.

## Install (unsigned build)

1. Open the `.dmg` and drag **General Bots** to `/Applications`.
2. Gatekeeper blocks the first launch because the build has no Developer ID.
   Right-click the app and choose **Open**, or clear the quarantine flag once:

   ```bash
   xattr -dr com.apple.quarantine "/Applications/General Bots.app"
   ```

3. Updates replace the app: open the new `.dmg`, drag the new app over the old
   one, and delete the old download. User data lives in
   `~/.local/share/generalbots` and survives the replacement.

## Signing and notarization (future work)

Signing and notarization are **not implemented** and are deliberately left as
follow-up work. The current artifact is unsigned; the CI log prints the
`codesign`/`spctl` result to make that explicit.

The future path needs a paid Apple Developer account and these repository
secrets:

| Secret | Purpose |
|---|---|
| `APPLE_CERTIFICATE` | base64-encoded Developer ID `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | password for the `.p12` |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: ...` (optional) |
| `APPLE_ID` | Apple account used for notarization |
| `APPLE_TEAM_ID` | Apple team identifier |
| `APPLE_APP_SPECIFIC_PASSWORD` | app-specific password for notarization |

To enable signing, export them from the packaging step. tauri-cli reads
`APPLE_PASSWORD`, so map `APPLE_APP_SPECIFIC_PASSWORD` to `APPLE_PASSWORD`
(the other names match). Do not export empty values: an empty
`APPLE_CERTIFICATE` is treated as a certificate to import. Hardened runtime
and notarization should be enabled together with the certificate, then the
Gatekeeper bypass above can be removed.

## Runtime caveats

The macOS bundle cannot provision the Linux service stack natively. The wizard
must report these components as unavailable instead of failing provisioning:

- **Service stack:** PostgreSQL, Valkey, MinIO and `mc`, Qdrant, Vault,
  Zitadel and llama.cpp have no darwin builds. Run them in a Linux VM or on a
  remote Incus host (`incus remote add`) and point botserver at it.
- **BotModels:** the published pythonboxes are
  `botmodels-pythonbox-linux-x86_64.tar.gz` and
  `botmodels-pythonbox-windows-x86_64.zip`; the darwin pythonbox is not
  published yet, so vision/STT/TTS stay unavailable.
- **Agent VM:** Incus cannot run containers natively on macOS. Agent mode
  needs a remote Incus endpoint or a Linux VM.

Chat and Drive still open and work against whatever backend the user
configures; the missing pieces are the local stack and the agent/media tools.

The running version is reported by `GET http://127.0.0.1:8080/api/health`
(`{"service":"botserver","version":"6.7.0",...}`), and the data directory is
`~/.local/share/generalbots`. A Settings -> About surface is follow-up UI work.

## Build locally

```bash
rustup target add aarch64-apple-darwin
cargo build -p botserver --release --no-default-features --features desktop-local
cargo build -p botui --release
cargo build -p botapp --release --features desktop-local,custom-protocol
desktop/macos/stage-sidecar.sh aarch64-apple-darwin
cd botapp
cargo tauri build \
  --features desktop-local,custom-protocol \
  --config tauri.desktop-local.conf.json \
  --config tauri.macos.conf.json \
  --bundles dmg,app
```

The result is under `target/release/bundle/dmg/` (workspace target directory)
or `botapp/target/release/bundle/dmg/` when building outside the workspace.

## Prerequisite check

```bash
./PROD-DEPENDENCIES.sh --check     # reports the state; exits 2 on macOS
./PROD-DEPENDENCIES.sh --install   # prints guidance, installs nothing
```

The check is read-only. It reports the macOS version, the dmg target/CI state
and that the agent backend needs a remote Incus or a Linux VM. It never
downloads or installs anything.

## Alternatives

- When the caveats above block your workflow, use the Linux x86_64 or
  Windows x64 desktop bundle.
- For development on macOS, build and run `botserver`/`botui` from source. That
  is a source-tree development setup, not a fully provisioned desktop edition.

The platform prerequisite matrix and exit codes are documented in
[`botbook/src/07-user-interface/first-run-wizard.md`](../../botbook/src/07-user-interface/first-run-wizard.md).
