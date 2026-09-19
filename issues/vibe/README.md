# Vibe Reform Issues — Index

## Wave 2 — Reform (implemented, commit bc66df4c2)

| # | Title | Priority | Depends on |
|---|-------|----------|------------|
| [1500](1500-default-bot-vibe-bootstrap.md) | Branch default bot becomes a Vibe project | P0 | — |
| [1501](1501-remove-bot-sources-from-drive.md) | Remove `.gbot`/`.gbdialog` from Drive (git is source) | P0 | 1500, 1503 |
| [1502](1502-git-pull-bot-monitor.md) | Git-pull bot monitor into `work/` | P0 | 1500, 1503 |
| [1503](1503-alm-org-repo-provisioning.md) | ALM org/repo provisioning, git default | P0 | 1500 |
| [1504](1504-bot-two-env-test-prod.md) | Bot TEST/PROD twins, Run/Deploy, chat tabs | P0 | 1500 |
| [1505](1505-website-git-controlled-deploy.md) | Website deploys from git revisions | P1 | 1503 |
| [1506](1506-editor-github-style-source-control.md) | GitHub-style source control dialog | P1 | 1503 |

## Wave 2 — E2E verification (this wave's test backlog)

> **Run these only after botserver is rebuilt with the reform code** — the
> log lines cited (`git_monitor started`, `[git_import]`, `[vibe_bootstrap]`)
> only exist in the new binary.

| # | Title | Priority | Verifies |
|---|-------|----------|----------|
| [1507](1507-e2e-default-bot-bootstrap.md) | E2E: default-bot bootstrap (signup/backfill/drive) | P0 | 1500 |
| [1508](1508-e2e-git-owned-bots.md) | E2E: git-owned bots (import→pull→compile→chat) | P0 | 1501, 1502 |
| [1509](1509-e2e-alm-provisioning.md) | E2E: ALM org/repo provisioning + git default | P0 | 1503 |
| [1510](1510-e2e-bot-two-env.md) | E2E: bot TEST/PROD Run/Deploy + toolbar/tabs | P0 | 1504 |
| [1511](1511-e2e-website-git-deploy.md) | E2E: website git-revision deploy + rollback | P1 | 1505 |
| [1512](1512-e2e-editor-source-control.md) | E2E: GitHub-style source control dialog | P1 | 1506 |

**Execution order:** 1507 → 1509 → 1508 → 1510 → 1511 → 1512.

**Invariants across all cases:**
- Manual commit stays (per-file staging) — no auto-commit-only flows.
- Test-first deploy: agents land on `-test`; PROD writes are pipeline/
  admin-sanctioned only.
- Browser testing via Chrome CDP 9222, one tab per case, never close.
- Bots compile from `work/` (git-pulled), never from Drive object keys.
