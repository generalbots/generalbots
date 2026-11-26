# Secure Boot: build modes, key handling and MOK enrolment (#2478)

Detailed runbook for the signing tooling in this directory. The short
fallback instructions for an unsigned image live in
[README.md](README.md#secure-boot-boot-failure-fallback-2478-offline-docs-slice).

The recipe has **two explicit build modes**. There is no implicit "maybe
signed" state:

| Mode | Command | UKI | Behaviour |
|---|---|---|---|
| Development | `desktop/os/build-image.sh --mode dev` (or plain `mkosi build`) | built, **unsigned** | boots in QEMU/OVMF with Secure Boot off; no key material needed |
| Release | `desktop/os/build-image.sh --mode release --key … --cert …` | built **and signed** | refuses to build if a signing key is missing, unreadable or not PEM; signs systemd-boot and the UKI |

The development default is what keeps the tree locally buildable — an
unsigned UKI is still a complete UKI, so a bootable kernel is produced. The
old failure (`SecureBoot=yes` with no key → "no bootable kernel") is now
impossible because the two states are separate.

## Generate build-time keys (development / CI)

```bash
desktop/os/generate-secure-boot-keys.sh --output-dir /tmp/gb-sb-keys \
  --common-name "General Bots OS (dev)"
# writes /tmp/gb-sb-keys/{db.key,db.crt,db.cer,env.sh}
```

`db.key` is mode 0600, lives outside the repository and is **never
committed** (`.gitignore` also blocks accidental `mkosi genkey` output in the
recipe directory). `db.cer` is the DER form of the certificate, intended for
`mokutil --import`. Production must replace this throwaway pair with an
offline-held key injected through `GB_SB_KEY` / `GB_SB_CERT`.

## Build and test the signed image

```bash
GB_SB_KEY=/secure/path/db.key GB_SB_CERT=/secure/path/db.crt \
  desktop/os/build-image.sh --mode release --output-dir /tmp/gb-os-signed
# OVMF with Secure Boot enabled and Microsoft keys + our MOK enrolled:
GB_SB_KEY=/secure/path/db.key GB_SB_CERT=/secure/path/db.crt \
  desktop/os/build-image.sh --mode release --output-dir /tmp/gb-os-signed --smoke
```

`--smoke` boots the signed output with `--firmware=uefi-secure-boot
--firmware-variables=microsoft-mok` and requires the "Started General Bots
kiosk shell" marker in `/tmp/gb-os-signed/secure-boot-smoke.log`. The
`microsoft-mok` firmware variable file needs the `virt-firmware` package
(`virt-fw-vars`) and OVMF with Secure Boot support in the build container.

The CI gate in `.github/workflows/desktop-os-iso.yml` performs exactly this
build with throwaway keys and uploads `db.cer` (`generalbots-os-MOK.cer`) and
the smoke log as the `generalbots-os-secure-boot` artifact.

Optional measured boot (signed TPM PCR values plus a `pcrs` JSON file for the
diagnostics bundle) is a second, independent profile:

```bash
desktop/os/build-image.sh --mode release -- --profile=secure-boot-measured
```

## Diagnostics and release artifacts (#2478, #2468)

Two plain-text artifacts next to this runbook make the fallback and measured
boot evidence shippable without adding a build dependency or a network call:

- `desktop/os/diagnostics/secure-boot-fallback.txt` is the release-note and
  recovery-screen copy of the fallback: "if the machine refuses to boot,
  disable Secure Boot", the firmware setup-key table for common vendors, and a
  pointer to this runbook for `sudo mokutil --import`. Copy it next to the ISO /
  release artifacts (the same directory as `appliance-manifest.tsv`) so the
  recovery screen (#2468) can display it when the shell cannot start. It holds
  no keys, credentials or host-specific data.
- `desktop/os/diagnostics/capture-measured-boot.sh` records measured-boot state
  into the diagnostics bundle. Run it on a machine with a TPM:

  ```bash
  desktop/os/diagnostics/capture-measured-boot.sh \
    --out /path/to/diagnostics/measured-boot.json
  desktop/os/diagnostics/capture-measured-boot.sh \
    --out /path/to/diagnostics/measured-boot.json --check
  ```

  It reads PCRs with `tpm2_pcrread`, falling back to a bounded parse of
  `/sys/kernel/security/tpm0/binary_bios_measurements`, and writes **only**
  sha256 digests of the raw PCR values, the PCR indices and a UTC timestamp.
  Raw values and event bytes are never logged or stored. With no TPM/tools it
  writes an explicit `status: skip` with a reason — no values are invented —
  and `--check` validates the JSON without exposing digests. Attach the
  resulting `measured-boot.json` to the diagnostics bundle alongside the
  `pcrs` artifact from the measured profile.

These are static artifacts: they make the fallback and the capture path real
and testable offline. Actually observing PCR values on a TPM-equipped machine,
enrolling a MOK in firmware, and seeing the recovery screen render the fallback
still require the runtime UAT listed below.

## MOK enrolment on a stock laptop

The signed image uses the distro's **Microsoft-signed shim** (`shim-signed`)
as the first stage; shim then trusts our key from its machine-owner list
(MokList). On a machine with Secure Boot enabled:

1. Copy `db.cer` (the DER certificate from key generation) to the machine,
   e.g. onto the same USB medium or via a trusted channel.
2. `sudo mokutil --import db.cer` and choose a one-time MOK password.
3. Reboot. At the blue **MokManager** screen choose
   *Enroll MOK → Continue → Yes → Enter password → Reboot*.
4. Boot General Bots OS. Firmware → shim (Microsoft key) → MokManager key →
   signed systemd-boot → signed UKI.

For a fleet deployment, replace the MOK path with enrolling the certificate
in firmware `db` (or `KEK`/`PK`) through your normal key-management tooling —
the image recipe does not and must not ship a private key. Firmware menu/key
names for the fallback are listed in
[README.md](README.md#secure-boot-boot-failure-fallback-2478-offline-docs-slice).
