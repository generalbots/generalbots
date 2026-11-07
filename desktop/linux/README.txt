General Bots desktop (Linux x86_64 / aarch64)
=============================================

Contents
--------
  generalbots              desktop shell (Chat + Drive)
  botserver                local API + suite server started by the shell
  ui/suite, ui/themes      web suite served by botserver on 127.0.0.1:8080
  .product.desktop-local   desktop profile: Chat + Drive, single-user sign-in
  start.sh                 launcher (also the entry point of the desktop file)
  install.sh               per-user installer, no root required

Requirements
------------
  - Linux x86_64 (glibc 2.35+, e.g. Ubuntu 22.04+) or Linux aarch64
    (glibc 2.39+, e.g. Ubuntu 24.04+). The arm64 build is preview quality until
    the first-run stack is verified on real arm64 hardware.
  - Runtime libraries: WebKitGTK 4.1, GTK3, libpq5 and OpenSSL 3. The .deb and
    .rpm declare them and the package manager installs them; AppImage users
    install them first (see Runtime packages below)
  - About 4 GB of free disk space for the service stack
  - Internet access on first run for components that are not bundled; see
    Platform availability below. The MinIO/mc drive binaries are bundled.

Install and run
---------------
Choose ARCH=x86_64 on Intel/AMD or ARCH=aarch64 on arm64:

  ARCH=x86_64                 # or aarch64
  SUMS=SHA256SUMS-linux       # SHA256SUMS-linux-aarch64 on arm64
  curl -fL -o generalbots-linux-${ARCH}.tar.gz \
    https://github.com/generalbots/generalbots/releases/latest/download/generalbots-linux-${ARCH}.tar.gz
  curl -fL -o ${SUMS} \
    https://github.com/generalbots/generalbots/releases/latest/download/${SUMS}
  sha256sum -c <(grep " generalbots-linux-${ARCH}.tar.gz" ${SUMS})
  tar -xzf generalbots-linux-${ARCH}.tar.gz
  cd generalbots-*-linux-${ARCH}
  ./install.sh              # installs into ~/.local/opt/general-bots
  general-bots              # first run opens the setup wizard

Distro packages
---------------
The Tauri bundler also attempts to produce per-distro packages, but that step is
best effort and has never shipped a .deb/.rpm/.AppImage in a release. Do not
rely on these names being present; use the tarball or the single self-extracting
file instead. When a package is present it is used as follows:

  Debian/Ubuntu:  sudo apt install ./generalbots-linux-x86_64.deb
                  (arm64: generalbots-linux-aarch64.deb)
  Fedora/RHEL:    sudo dnf install ./generalbots-linux-x86_64.rpm
                  (arm64: generalbots-linux-aarch64.rpm)
  Any glibc:      chmod +x generalbots-linux-x86_64.AppImage
                  ./generalbots-linux-x86_64.AppImage
                  (arm64: generalbots-linux-aarch64.AppImage)
  macOS (arm64):  open generalbots-<tag>-macos-arm64.dmg (unsigned; see
                  desktop/macos/README.md for Gatekeeper and runtime caveats)

  Single file:    chmod +x generalbots && ./generalbots

Runtime packages
----------------
The .deb and .rpm declare their runtime libraries, so the package manager
installs them. The deb Depends are:

  libssl3 | libssl3t64      OpenSSL 3 (libssl3t64 on Debian 13/Ubuntu 24.04+)
  libwebkit2gtk-4.1-0       WebKitGTK 4.1 used by the Tauri shell
  libpq5                    PostgreSQL client used by botserver
  libgtk-3-0 | libgtk-3-0t64
                            GTK3 widgets (t64 on Debian 13/Ubuntu 24.04+)

The rpm declares openssl-libs, webkit2gtk4.1 and libpq.

The AppImage declares no dependencies; it uses the host WebKitGTK rather than
bundling it. Install the same libraries before running it:

  Debian 13:     sudo apt install libwebkit2gtk-4.1-0 libpq5 \
                   libgtk-3-0t64 libssl3t64 libfuse2t64
  Ubuntu 24.04:  sudo apt install libwebkit2gtk-4.1-0 libpq5 \
                   libgtk-3-0t64 libssl3t64 libfuse2t64
  Ubuntu 22.04:  sudo apt install libwebkit2gtk-4.1-0 libpq5 \
                   libgtk-3-0 libssl3 libfuse2
  Fedora/RHEL:   sudo dnf install webkit2gtk4.1 libpq openssl-libs fuse-libs

FUSE 2 is only needed to mount the AppImage. Where it is unavailable (for
example Debian 13 without libfuse2t64), either install the package or run the
file with:

  APPIMAGE_EXTRACT_AND_RUN=1 ./generalbots-linux-x86_64.AppImage

`--appimage-extract-and-run` is equivalent.

Packaging checks
----------------
desktop/linux/test-deb-install.sh installs a .deb in clean debian:13 and
ubuntu:24.04 containers and fails when any dependency cannot be resolved:

  desktop/linux/test-deb-install.sh generalbots-linux-x86_64.deb
  docker run --rm -v "$PWD:/w" debian:13 \
    bash -c 'apt-get update && apt-get install -y /w/<file>.deb'

Without Docker the script falls back to `dpkg-deb --info` plus a
dependency-resolution check on the host; that is a smoke test, not proof for
both distributions.

Updating
--------
There is no in-app auto-updater in this build. Supported upgrades:

  AppImage  Download the new AppImage, chmod +x it and run it. Data lives in
            ~/.local/share/generalbots, so it survives; the old AppImage can
            be deleted after the new one starts.
  deb       sudo apt install ./generalbots-linux-x86_64.deb (or apt upgrade
            from a repository); the package manager replaces the files.
  rpm       sudo dnf upgrade ./generalbots-linux-x86_64.rpm (or dnf upgrade
            from a repository).
  tarball   Run ./install.sh again; it replaces ~/.local/opt/general-bots and
            leaves ~/.local/share/generalbots untouched.
  macOS     Replace the .app with the one from the new dmg; see
            desktop/macos/README.md.

On arm64 replace x86_64 with aarch64 in the file names above.

The running version is reported by the local API:

  curl -s http://127.0.0.1:8080/api/health
  {"status":"healthy","service":"botserver","version":"6.7.0",...}

Data (stack, logs, workspace, cached installers) lives in
~/.local/share/generalbots (override with BOTSERVER_STACK_PATH). A
Settings -> About panel showing the version and data directory is follow-up UI
work; until it exists, /api/health and the path above are the supported
surface.

A truncated download extracts without `generalbots` and `install.sh`; start.sh
and install.sh refuse to run in that case and print the re-download command.

Running ./start.sh directly from the extracted directory works too: nothing is
written next to the binaries. Runtime state lives in
~/.local/share/generalbots (override with BOTSERVER_STACK_PATH).

Headless provisioning
---------------------
  ./start.sh --provision

Runs the same component sequence the wizard runs, without a display, and prints
one GBWIZARD:{...} progress line per component. Useful on a server or in a CI
test; the wizard UI is not involved.

First run
---------
  1. The shell starts the local botserver.
  2. The wizard asks for a display name, a model provider and an API key.
  3. Services are provisioned into the stack; progress is shown per component
     and the full log is written to ~/.local/share/generalbots/logs.
  4. When /api/product answers, the suite opens with Chat and Drive.

Platform availability (runtime components)
------------------------------------------
The first-run wizard provisions the stack from botserver/3rdparty.toml. The
component matrix is not identical on every platform:

  Linux x86_64   PostgreSQL, Valkey, MinIO/mc, Qdrant, Vault, Zitadel,
                 llama.cpp and the BotModels pythonbox are available.
  Linux aarch64  PostgreSQL, Valkey, MinIO/mc, Qdrant, Vault, Zitadel and
                 llama.cpp have arm64 builds. There is no aarch64 BotModels
                 pythonbox, so vision/STT/TTS are reported unavailable. The
                 arm64 desktop uses llama.cpp b12189 (the x86_64 pin b7345 has
                 no ubuntu-arm64 asset); that release publishes no checksum, so
                 its download is not hash-verified.
  Windows x64    MinIO/mc, Redis, Qdrant, Zitadel and Caddy are bundled offline.
                 PostgreSQL, Vault and llama.cpp (plus models) are downloaded
                 once. There is no Windows BotModels pythonbox, so media stays
                 unavailable; /api/product reports the reason.

Optional add-ons that are not part of the first-run stack (Caddy, Stalwart,
InfluxDB, Vector, Forgejo/runner, NocoDB, LiveKit, CoreDNS) still have x86_64
URLs only on Linux. They are not installed by the desktop wizard.

Offline installation
--------------------
Put the component archives in an `installers/` directory next to start.sh
before the first run (the same layout as botserver-installers in the source
tree). Archives found there are preferred over the network. Set
GB_DESKTOP_OFFLINE=1 to refuse any download at all.

The download already bundles MinIO (object store) because upstream archived it
and removed the binaries from dl.min.io. The Linux tarballs carry the matching
x86_64 or arm64 MinIO/mc pair. Those files are AGPL-3.0; see
https://github.com/minio/minio and https://github.com/minio/mc for the source.

Troubleshooting
---------------
  ~/.local/share/generalbots/logs/botserver-sidecar.log   API output
  ~/.local/share/generalbots/logs/wizard-provision.log    provisioning output
  BOTSERVER_STACK_PATH=/tmp/gb stack ./start.sh           isolated test stack
