# Forgejo (ALM) does not run these workflows on purpose

This directory deliberately exists and must not be deleted.

Forgejo only falls back to `.github/workflows` **when `.forgejo/workflows` is
absent**. Adding this file makes the ALM instance ignore the GitHub-only
workflows in `.github/workflows`, which ask for `ubuntu-latest` /
`ubuntu-22.04` runners that the ALM runner does not have. Without this marker,
every push to `main` queues `alm-release.yml`, `boteval.yml`,
`botmodels-pythonbox.yml` and `desktop-binaries.yml` as *waiting* jobs and the
Actions page shows "no runner matches the labels" forever.

Division of work:

| Where | What runs |
|-------|-----------|
| GitHub (`github.com/generalbots/generalbots`) | `.github/workflows/*` — compiles the binaries and publishes the immutable `alm-<stamp>-<sha>` release (`alm-release.yml`). |
| ALM (`alm.pragmatismo.com.br`, repo `generalbots/generalbots`) | **nothing** — this repository only stores the source. |
| ALM deploy repo (`generalbots/gb`) | `.github/workflows/botserver.yaml`, `botui.yaml`, `botmodels.yaml` on the `gbo` runner: downloads the release published above, verifies its `sha256` and deploys it. |

If an ALM-runnable workflow is ever needed in this repository, put the real
`.yaml` files next to this README — they will be picked up by the `gbo` runner.
