# [VIBE] 1507 — E2E test plan: branch default-bot bootstrap (1500) + backfill

**Priority:** P0
**Kind:** test (E2E, browser-verified per AGENTS.md)
**Covers:** issue #1500 / GH #1412

## Preconditions
- Dev stack up: botserver 8080 `/health` = 200, botui 3000/4000/5000 reachable.
- Binary rebuilt with reform code (check `git_monitor started` in botserver.log
  after boot — that line only exists in the new build).

## Cases
### Case A — signup bootstrap
1. Signup a fresh org at `http://localhost:5000/signup` (plan=free) —
   Playwright via CDP 9222, one tab per case, never close the browser.
2. Expect botserver.log: `[vibe_bootstrap] default project ensured for branch
   '<slug>' (bot kind, git mode)` within ~10s.
3. Expect DB: one `vibe_projects` row, `project_type='bot'`,
   `source_control='git'`, `payload.is_branch_default='true'`, payload has
   `test_bot_id` ≠ `bot_id`.
4. Expect `bots` table: original bot row untouched (same id/name), plus a
   `{bot}-test` row in the same branch.
5. Expect Forgejo (ALM :4747 dev): org `<branch-slug>` exists with repo
   `<default-bot-slug>` (153 backfills it at boot; signup creates it inline).

### Case B — boot backfill of pre-existing branches
1. Restart botserver with a branch that has NO vibe project yet.
2. Expect log `[vibe_backfill]` entries; re-run restart → no duplicates
   (idempotent).

### Case C — Drive-discovered org
1. Upload a `{org}.gborg/{branch}.gbai/{bot}.gbdialog/start.bas` via `mc`.
2. Expect drive_monitor creates org+bot, then the hook bootstraps the vibe
   project for that branch automatically.

## Acceptance
- All three paths converge on: default bot is a git-mode Vibe project with a
  TEST twin, selectable in the Vibe UI, chattable at `/chat/{bot}`.
