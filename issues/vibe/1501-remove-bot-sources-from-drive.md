# [VIBE] 1501 — Remove `.gbot`/`.gbdialog` from Drive: git becomes the only bot source

**Priority:** P0
**Kind:** refactor (source of truth migration)
**Depends on:** 1500 · **Blocks:** 1502, 1504

## Problem

Bot sources (`.gbot`, `.gbdialog/*.bas`, `tables.bas`) currently live **exclusively** in
MinIO Drive buckets (`{bot}.gbai` / `{org}.gborg/{branch}.gbai/`). `drive_monitors.rs`
compiles and syncs them from Drive; `mc` is the only way to edit them. Meanwhile Vibe
projects are git-backed (issue #1271, `git_mode.rs`). Two sources of truth for bots:

1. Agents/users edit files in the Vibe workspace → git → deploy, but the running botserver
   still reads Drive.
2. Files pushed via `mc` to Drive are invisible to Vibe/git.

The reform: **vibe code coordinates the bot**. Drive stops being the bot source store.

## Goal

- `.gbot` and `.gbdialog` artifacts are **removed from Drive** (MinIO buckets keep only
  `.gbkb` knowledge, `.gbdrive` documents/uploads — runtime data, not source).
- Bot sources live in Forgejo: one repository per bot inside the branch org (issue 1503).
- `drive_monitor` no longer compiles bots from Drive; it **git-pulls** sources into
  `work/` (issue 1502).

## Migration plan (ordered, reversible per step)

### Step 1 — Import (must be safe to re-run)
- New `botvibe::bootstrap::import_drive_bot_sources(project)` (or a `botlib` helper used by
  bootstrap, issue 1500): for each bot of the branch, list
  `{bucket}/{bot}.gbdialog/*` and `{bot}.gbot/` via the existing S3 client
  (`state.drive`), copy content into the git workspace, initial commit + push `main`
  (reuse `git_mode::ensure_git_repo` — it already creates repo + pushes).
- Record `payload.source_imported_at` in the vibe project to make it idempotent.
- ONLY `.gbdialog/` and `.gbot` are imported. `.gbkb/`, `.gbdrive/` stay in Drive.

### Step 2 — Stop syncing bot sources from Drive
- `drive_monitors.rs`: skip `sync_tables_for_org_bot` and `.gbdialog`/`.gbot` compilation
  for bots whose `vibe_projects.payload.source_imported_at` is set (query by branch+slug).
  Keep monitoring `.gbkb` (KB hot reload stays Drive-driven).
- Remove (or gate behind a `drive-bot-sources` feature flag, default off) the code paths
  that pull `.bas` from Drive for compile. **Never delete code by commenting it out** —
  either gate it properly or delete it (AGENTS.md).
- `create_bot_from_drive` keeps creating the `bots` row (identity stays in Postgres), but
  no longer expects `.gbdialog` objects to exist.

### Step 3 — Cleanup Drive
- `botserver/src/main_module/drive_monitors.rs` cleanup pass, run once at bootstrap after
  import verification: move `{bot}.gbdialog/` and `{bot}.gbot/` prefixes to
  `{bucket}/archive/{bot}-{ts}/` (do NOT hard-delete first; archive is cheap, reversible).
  After one release cycle, a second switch hard-deletes archives.
- Keep `default.gbai` used by local reset (`botserver-stack`) — the reset script seeds
  from `work/` now (update `restart.sh`/reset docs), since Drive no longer carries sources.

### Step 4 — Compile path
- Wherever `ScriptService::run`/compile previously read from Drive (`drive_monitors.rs` /
  bot loading), the compile input becomes `work/{bot}.gbdialog/*.ast` + `.bas` produced by
  the git pull monitor (issue 1502). Verify TOOL_EXEC (message type 6) still resolves the
  tool from the pulled folder.

## Code anchors

| What | Where |
|------|-------|
| Drive monitors — the only bot source path today | `botserver/src/main_module/drive_monitors.rs` (984 lines) |
| git workspace + push machinery to reuse | `botserver/crates/botvibe/src/git_mode.rs` |
| Workspace root (`work/`) | `botvibe::harness::workspace_root` |
| Bots loaded from Drive statement in AGENTS.md | `AGENTS.md` → "Bot Source Rules" section — **update it** (sources from git via work/, `.bas` still never committed for PROD bots; repo now holds them by design) |

## Acceptance criteria

- [ ] Fresh signup → bot sources exist ONLY in Forgejo (`.gbai` bucket has no `.gbdialog`/`.gbot`).
- [ ] Legacy org (Drive-born) → sources imported to git and Drive copies archived on first boot.
- [ ] Bot chat/TOOL_EXEC works end-to-end with Drive compile path disabled.
- [ ] `.gbkb` hot reload still works from Drive.
- [ ] AGENTS.md "Bot Source Rules" updated to the new reality (work/ from git; `.bas` allowed in work/ as runtime artifact, repo sources canonical).
- [ ] Rollback path documented: restore archive prefix → restart → old behavior (until hard-delete switch).

## Non-goals

- `.gbkb`/`.gbdrive` migration (stays in Drive).
- Deleting buckets or rewriting bucket layout (issue 1502 handles the work/ layout).
