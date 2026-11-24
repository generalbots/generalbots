# General Bots OS — deterministic release packaging (#2440)

This directory holds the data-driven packaging slice for the **real** kiosk
appliance. `generalbots-os.iso` is reserved for the image built from
`desktop/os/mkosi.conf`; the GRUB/tarball carrier produced by
`scripts/build_iso.sh` is published separately as `generalbots-payload.iso`.

| File | Role |
|---|---|
| `appliance-manifest.tsv` | Expected OS identity, required paths, kiosk-shell unit and boot markers. Tab-separated; repeated keys are lists. No host paths, timestamps or secrets. |
| `verify-image.sh` | Reads those expectations and asserts them **inside** the image with read-only `guestfish`: root filesystem, `/etc/os-release`, `/opt/general-bots/start.sh`, the enabled `generalbots-shell.service`, and the GPT/ESP boot layout. Optional El Torito checks use `xorriso` when installed. |
| `build-release-image.sh` | Builds with mkosi (reusing `desktop/os/mkosi.conf`) or verifies a prebuilt `--image`, then publishes `generalbots-os.iso`, `SHA256SUMS-os` and `generalbots-os.manifest.txt` only after every check passes. |

## Usage

```bash
# Full build (needs mkosi, its tools tree and the staged payload in
# desktop/os/mkosi.extra/opt/general-bots):
bash desktop/os/release/build-release-image.sh --out-dir dist \
  --release v7.0.42

# Verify and publish an image built by another step (CI path):
bash desktop/os/release/build-release-image.sh \
  --image dist/generalbots-os.iso --out-dir dist/verified
```

Preflight is explicit: missing mkosi/recipe/payload inputs, or a missing image
in `--image` mode, print `SKIP: <reason>` and exit 0. A present image that
fails verification exits 1 and produces no final name. `build-release-image.sh
--help` lists all options. The generated manifest records the artifact SHA256,
size, pinned tag and per-check results without timestamps, so repeated runs on
the same bytes produce the same manifest.

The wrapper step in `.github/workflows/desktop-os-iso.yml` runs the publish
path after the mkosi build, using a `debian:sid` container with
`libguestfs-tools` (and a kernel for supermin, as the existing build step
already requires). A real image, not this offline tree, is needed to exercise
a PASS run.
