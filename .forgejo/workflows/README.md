# Forgejo (ALM) does not run these workflows on purpose

This directory deliberately exists and must not be deleted.

Forgejo only falls back to `.github/workflows` **when `.forgejo/workflows` is
absent**. Adding this file makes the ALM instance ignore the GitHub-only
workflows in `.github/workflows`, which ask for `ubuntu-latest` /
`ubuntu-22.04` runners that the ALM runner does not have. Without this marker,
every push to `main` queues `ci.yml`, and every `v*.*.*` tag queues
`release.yml` (then `release-check.yml`) as *waiting* jobs and the Actions page
shows "no runner matches the labels" forever.

Division of work:

| Where | What runs |
|-------|-----------|
| GitHub (`github.com/generalbots/generalbots`) | `.github/workflows/*` — `ci.yml` on pushes/PRs, `security.yml` on schedule/call, and `release.yml` on version tags to compile the binaries and publish the product release. |
| ALM (`alm.pragmatismo.com.br`, repo `generalbots/generalbots`) | **nothing** — this repository only stores the source. |
| ALM deploy repo (`generalbots/gb`) | `.github/workflows/botserver.yaml`, `botui.yaml`, `botmodels.yaml` on the `gbo` runner: downloads the release published above, verifies its `sha256` and deploys it. |

If an ALM-runnable workflow is ever needed in this repository, put the real
`.yaml` files next to this README — they will be picked up by the `gbo` runner.
The `malware-clamav` job in `.github/workflows/ci.yml` (#2602) deliberately has
no `github.server_url` guard, so it can be copied verbatim into such a workflow
and will run the real ClamAV upload gate there too. Likewise, all six audit
jobs in `.github/workflows/security.yml` (#2744) are host-neutral; only SARIF
uploads to GitHub's Security tab remain GitHub-only. The guard
`scripts/check_security_workflow_host_neutral.py` enforces this.

## Why `ubuntu-latest` is not mapped to the `gbo` runner

Changing `runs-on` to `gbo` looks like the obvious fix and does not work:

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

Each job therefore carries `if: github.server_url == 'https://github.com'` as a
second guard: even if this README is deleted, ALM marks those jobs *skipped*
instead of queueing them forever. The deploy path stays `generalbots/gb` → `gbo`.

