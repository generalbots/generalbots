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
