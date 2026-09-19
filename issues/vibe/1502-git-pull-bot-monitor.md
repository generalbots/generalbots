# [VIBE] 1502 — Drive monitor pulls bot sources from Forgejo into `work/` (no more Drive)

**Priority:** P0
**Kind:** feature (monitor rewrite)
**Depends on:** 1500, 1501 · **Blocks:** 1504

## Problem

`drive_monitors.rs` compiles and hot-reloads bots **from MinIO Drive**: every file change
in `{bot}.gbai/{bot}.gbdialog/*.bas` triggers recompile. After issue 1501, bot sources no
longer exist in Drive — they live in Forgejo repositories. The monitor must become a
**git pull monitor**: keep `work/{repo}` checkouts fresh from ALM, then compile/reload
from the local folder exactly as it did from Drive.

The user statement that defines this issue: *"drive monitor now will get `.gbdialog` and
`.gbot` from git pull into work folder not from drive anymore, cleanup this."*

## Goal

- New git monitor: for each `bots` row with `origin='vibe'` (or any bot with a resolved
  vibe project), ensure a checkout exists at `work/{branch}/{bot}` and `git pull` on
  change/interval.
- On pull that changes `.gbdialog`/`.gbot` content → same recompile + reload path the
  Drive monitor used (ETag-style dedup via HEAD commit hash instead of MinIO ETag).
- `bots` rows are still auto-created — but from **Forgejo repos of the branch org**, not
  from Drive object keys (`discover_and_create_bots` at `drive_monitors.rs:548`).
- Cleanup: remove the Drive-based `.gbdialog`/`.gbot` listing/compilation from the org
  bucket scan (`scan_org_bucket`, `start_org_bot_monitor`, `sync_tables_for_org_bot`
  drive reads).

## Design

```
Forgejo org {branch}                    work/                          runtime
┌─────────────────────┐   git pull    ┌──────────────────────┐       ┌──────────────┐
│ {branch}/{bot}.git  │ ────────────▶ │ work/{branch}/{bot}/ │ ────▶ │ compile .bas │
│  {bot}.gbdialog/    │  (monitor)    │  {bot}.gbdialog/     │       │ reload bot   │
│  {bot}.gbot/        │               │  {bot}.gbot/         │       │              │
└─────────────────────┘               └──────────────────────┘       └──────────────┘
```

- Monitor cadence: reuse the Drive monitor's periodic loop (same tick), replacing the S3
  LIST with `git fetch` + `git rev-parse HEAD..origin/main` compare (cheap). Per-bot
  webhook is a stretch goal, not required.
- Auth: clone/pull URL with token from `botcoresecrets::alm_config()` — same source as
  `git_mode.rs` (`add_token_to_url` helper is private there; extract to a shared util,
  do not duplicate).
- Slug resolution: `bots.slug` ↔ repo name via `VmLifecycle::alm_repo` (already used by
  `git_mode.rs` / `projects_api.rs::workspace_key`) — one canonical naming function.
- `tables.bas` schema sync (`sync_bot_tables`) must run from the pulled folder.

## Code anchors

| What | Where |
|------|-------|
| Current Drive-driven discovery + monitor loop | `botserver/src/main_module/drive_monitors.rs` (`scan_org_bucket`, `start_org_bot_monitor`, `discover_and_create_bots`, `sync_tables_for_org_bot`) |
| git plumbing to reuse | `botserver/crates/botvibe/src/git_mode.rs` (`add_token_to_url`, workspace `run("git", …)` pattern via `harness::cmd::run`) |
| Repo/org naming | `botserver/crates/botvibe/src/vm_lifecycle.rs` (`VmLifecycle::alm_org`, `alm_repo`) |
| ALM config from Vault | `botcoresecrets::alm_config()` |

## Implementation plan

1. Extract `git_mode::add_token_to_url` + repo naming into a shared `botvibe` util (pub).
2. New `botserver/src/main_module/git_bot_monitor.rs`:
   - `ensure_checkout(bot, branch) -> PathBuf` (clone if missing, auth URL).
   - `pull_if_changed(bot) -> Option<ChangedPaths>` (fetch + compare HEAD).
   - On change: call the existing compile/reload entry points previously fed by the Drive
     monitor (keep the same log tags `[drive_monitor]`-equivalent → use `[git_monitor]`).
3. Bot discovery: list Forgejo repos of `{branch}` org (ForgejoClient from `botdeployment`)
   instead of S3 LIST; upsert `bots` rows (keep `origin='drive'` rows untouched — only
   adopt vibe-backed bots into the git monitor).
4. Remove from `drive_monitors.rs` everything that reads `.gbdialog`/`.gbot` objects
   (discovery from object keys, file change compile, tables sync from Drive). Keep the
   `.gbkb` monitor untouched.
5. Bootstrap ordering: at boot, run issue-1501 import first, then start the git monitor —
   never both monitors compiling the same bot.

## Acceptance criteria

- [ ] Edit `.bas` in the Forgejo web UI → within the monitor tick, `work/` shows the change,
      bot recompiles, chat reflects the new behavior (browser test per AGENTS.md).
- [ ] New repo created in the branch org → bot row appears without Drive involvement.
- [ ] `restart.sh` cold start: bots compile from `work/` checkouts (no Drive sources needed).
- [ ] `.gbkb` changes still hot-reload via the Drive monitor.
- [ ] No double-compile races when a bot is open in Vibe (git monitor + agent commits) —
      document the lock (reuse `project_locks.rs` if applicable).
- [ ] `grep -r "gbdialog" botserver/src/main_module/drive_monitors.rs` returns only `.gbkb`
      or legacy-archival code.

## Non-goals

- Webhook-driven pull (interval tick is fine).
- Editing `.bas` inside Vibe code editor (Vibe works on the project workspace; bot
  projects already do via git mode).
