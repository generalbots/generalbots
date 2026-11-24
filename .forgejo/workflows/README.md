# Forgejo (ALM) workflows

This directory deliberately exists and must not be deleted.

Forgejo only falls back to `.github/workflows` **when `.forgejo/workflows` is
absent**. Keeping this marker makes the ALM instance ignore the GitHub-only
workflows in `.github/workflows`, which ask for `ubuntu-latest` /
`ubuntu-22.04` runners that the ALM runner does not have. Without a file here,
every push to `main` queues `ci.yml`, and every `v*.*.*` tag queues
`release.yml` (then `release-check.yml`) as *waiting* jobs and the Actions page
shows "no runner matches the labels" forever.

Since #2744 this directory also contains a real workflow, `security.yml`, which
runs the **supply-chain audits** on the `gbo` host executor. Since #2602 it also
contains `malware-clamav.yml`, the mirror of the GitHub `ci.yml` job that runs a
**real ClamAV scan through the upload handler** (no `github.server_url` guard).
Their presence does not change the fallback rule above: Forgejo runs the
workflows in this directory and still ignores `.github/workflows`.

Division of work:

| Where | What runs |
|-------|-----------|
| GitHub (`github.com/generalbots/generalbots`) | `.github/workflows/*` — `ci.yml` on pushes/PRs (it calls `security.yml`), `security.yml` on schedule/call with SARIF upload, and `release.yml` on version tags to compile the binaries and publish the product release. |
| ALM (`alm.pragmatismo.com.br`, repo `generalbots/generalbots`) | `.forgejo/workflows/security.yml` — the same blocking supply-chain audits as GitHub (cargo-deny, OSV allowlists incl. npm, gitleaks, pip-audit, SBOM, workflow guards) on the `gbo` runner. No SARIF / Security-tab upload, no artifact upload, no advisory header probe. `.forgejo/workflows/malware-clamav.yml` — the real ClamAV EICAR upload gate, identical to the GitHub `ci.yml` job. |
| ALM deploy repo (`generalbots/gb`) | `.github/workflows/botserver.yaml`, `botui.yaml`, `botmodels.yaml` on the `gbo` runner: downloads the release published above, verifies its `sha256` and deploys it. |

`scripts/check_supply_chain_gates.py` (run in the `workflow-lint` job of **both**
manifests) enforces the parity: it fails if an audit job is gated on
`github.server_url`, if a SARIF upload loses its GitHub-only guard, if the
Forgejo mirror is missing an audit job or command, or if either manifest
references repository secrets. Advisory failures therefore block the ALM
pipeline exactly as they block GitHub.

`scripts/check_malware_clamav_gates.py` does the same for the #2602 gate: it is
run as a step of `malware-clamav` in **both** manifests and fails if either host
loses the ClamAV/EICAR upload check, if a `github.server_url` skip appears, if
the Docs/Sheets/Slides corpora drift apart, or if secrets / boteval creep in.

## Why `ubuntu-latest` is not mapped to the `gbo` runner

Changing `runs-on` to `gbo` in `.github/workflows` looks like the obvious fix
and does not work:

- `gbo` is a **host executor** on the production host (sudo for `incus`). A
  `cargo build -p botserver` there competes with the live botserver/botui for
  30–60 minutes per push and duplicates a build GitHub already made.
- The `publish` job calls `gh release create … --repo generalbots/generalbots`
  with `github.token`. On Forgejo that token belongs to the ALM instance, so the
  GitHub release is never created and the job fails.
- The `evaluate` job in `ci.yml` needs `secrets.BOTEVAL_ENDPOINT` /
  `secrets.BOTEVAL_API_KEY`, which only exist on GitHub.
- `actions/upload-artifact@v4` / `download-artifact@v4` are only supported by
  Forgejo as v3 or patched v4.
- `release.yml` needs `windows-latest`, which ALM can never provide.

The supply-chain audits are different: they download checksum-pinned static
binaries and run plain scripts, so they are host-independent and safe to run on
`gbo`. They are therefore mirrored here instead of changing the other jobs'
runners. The remaining GitHub-only jobs still carry
`if: github.server_url == 'https://github.com'` as a second guard: even if this
marker were deleted, ALM marks them *skipped* instead of queueing them forever.
The deploy path stays `generalbots/gb` → `gbo`.
