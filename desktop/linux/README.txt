General Bots desktop (Linux x86_64)
===================================

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
  - Linux x86_64 with a GTK/WebKitGTK 4.1 desktop (Ubuntu 22.04+ and
    equivalents ship it; `libwebkit2gtk-4.1-0` is the only required library)
  - About 4 GB of free disk space for the service stack
  - Internet access on first run: the setup wizard downloads the services
    (PostgreSQL, Valkey, MinIO, Qdrant, Vault, Zitadel and llama.cpp)

Install and run
---------------
  tar -xzf generalbots-*-linux-x86_64.tar.gz
  cd generalbots-*-linux-x86_64
  ./install.sh              # installs into ~/.local/opt/general-bots
  general-bots              # first run opens the setup wizard

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

Offline installation
--------------------
Put the component archives in an `installers/` directory next to start.sh
before the first run (the same layout as botserver-installers in the source
tree). When that directory has files, nothing is downloaded.

Troubleshooting
---------------
  ~/.local/share/generalbots/logs/botserver-sidecar.log   API output
  ~/.local/share/generalbots/logs/wizard-provision.log    provisioning output
  BOTSERVER_STACK_PATH=/tmp/gb stack ./start.sh           isolated test stack
