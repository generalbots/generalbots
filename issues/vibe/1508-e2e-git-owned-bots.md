# [VIBE] 1508 — E2E test plan: git-owned bots (1501+1502) — import, pull, compile, edit→run

**Priority:** P0
**Kind:** test (E2E, browser + mc + Forgejo)
**Covers:** issues #1501 / GH #1413 and #1502 / GH #1414

## Preconditions
1507 passing for at least one branch.

## Cases
### Case A — one-shot Drive import
1. Seed the branch bucket via `mc` with `{bot}.gbdialog/start.bas` + one tool
   `.bas` (Vault creds per AGENTS.md canonical block).
2. Restart botserver → expect `[git_import] {bot}: N file(s) imported from
   {bucket} → git and pushed` and `payload.source_imported_at` set.
3. Expect the Forgejo repo `org/{bot-slug}` to contain `.gbdialog/*.bas` on
   `main`.
4. Re-run restart → no duplicate import commits.

### Case B — drive_compiler refuses git-owned bots
1. Bump an etag on `{branch}.gbai/{bot}.gbdialog/start.bas` in drive_files.
2. Expect NO compile for the bot from the Drive path (log shows the skip);
   the only compile source is the git monitor.

### Case C — git pull → work/ materialization
1. Edit `start.bas` in the Forgejo repo (web UI) on `main` (change TALK text).
2. Within `GB_VIBE_GIT_MONITOR_SECS` (default 15s) + compile latency, expect:
   `[git_monitor] {bot} pulled to …` and `[git_monitor] queued N file(s) for
   compile`.
3. Expect the new TALK text in chat at `http://localhost:3000/{bot}`.

### Case D — Run → TEST twin
1. In Vibe UI, select the bot project, press Run.
2. Expect `POST /api/vibe/projects/{id}/bot/run-test` → 200, and the Chat
   window opens on tab `NAME (TEST)` showing the new text (materialized
   `{bot}-test` layout, `_dev` DB twin per #1386).

## Acceptance
- Drive is out of the bot-source business; a Forgejo edit reaches both
  `{bot}` (PROD) and `{bot}-test` chat windows without any `mc` push.
