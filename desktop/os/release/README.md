# General Bots OS — deterministic release packaging (#2440)

This directory holds the data-driven packaging slice for the **real** kiosk
appliance. `generalbots-os.iso` is reserved for the image built from
`desktop/os/mkosi.conf`; the GRUB/tarball carrier produced by
`scripts/build_iso.sh` is published separately as `generalbots-payload.iso`.

| File | Role |
|---|---|
| `appliance-manifest.tsv` | Expected OS identity, required paths, kiosk-shell unit, its enablement link (`unit.enabled_path`) and boot markers. Tab-separated; repeated keys are lists. No host paths, timestamps or secrets. |
| `verify-image.sh` | Reads those expectations and asserts them **inside** the image with read-only `guestfish`: root filesystem, `/etc/os-release`, `/opt/general-bots/start.sh`, the kiosk-shell unit **and its boot enablement link**, plus the GPT/ESP boot layout. Optional El Torito checks use `xorriso` when installed. |
| `build-release-image.sh` | Builds with mkosi (reusing `desktop/os/mkosi.conf`) or verifies a prebuilt `--image`, cross-checks the same image with `scripts/build_iso.sh --verify-appliance`, then publishes `generalbots-os.iso`, `SHA256SUMS-os`, `generalbots-os.manifest.txt` and the Secure Boot fallback runbook only after every gate passes. |

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

## Enabled unit and independent cross-check

A unit file that is merely installed does not boot. `verify-image.sh` reads
`unit.enabled_path` from the manifest and asserts that the path is a symlink
whose target is the kiosk-shell unit — the condition systemd needs to start
the shell. A missing/mistargeted link is a failed `unit.enabled` check; when
the root filesystem is not mountable, that group emits `SKIP` like the other
root-dependent groups and the run still fails via `root.mount`.

After the primary verifier passes, `build-release-image.sh` runs a second,
independently implemented gate on the same bytes:

```bash
bash scripts/build_iso.sh --verify-appliance <same-image>
```

The script path is resolved from the repository root relative to the release
script. The outcome is recorded in `generalbots-os.manifest.txt` as
`crosscheck=PASS`; if `guestfish` is unavailable, the cross-check prints an
explicit `SKIP` reason and records `crosscheck=SKIP` without weakening the
primary verifier. A failing cross-check aborts publication exactly like the
primary verifier, so no `generalbots-os.iso` is produced.

The wrapper step in `.github/workflows/desktop-os-iso.yml` runs the publish
path after the mkosi build, using a `debian:sid` container with
`libguestfs-tools` (and a kernel for supermin, as the existing build step
already requires). A real image, not this offline tree, is needed to exercise
a PASS run.
