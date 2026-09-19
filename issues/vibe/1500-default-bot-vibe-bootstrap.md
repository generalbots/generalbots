# [VIBE] 1500 — Bootstrap the branch's default bot as a Vibe project

**Priority:** P0 (foundation — every other issue in this reform depends on it)
**Kind:** feature (vibe bootstrap + cloud signup hook)
**Depends on:** nothing · **Blocks:** 1501, 1502, 1503, 1504

## Problem

Today a bot-kind Vibe project is only created when the user clicks "New Project".
The branch's **default bot** — the one created at branch creation (`create_cloud_workspace_inner`
at `botserver/crates/botcloud/src/integration.rs:600`, or by `drive_monitors.rs:566` when the
org comes from Drive) — exists only as a `bots` row + Drive files. People cannot "vibe" the
base bot: open Vibe and extend the bot they already talk to.

The reform (issues 1501/1502) moves bot sources to Forgejo. That requires **at least one
project bot per branch**: the default bot must exist as a Vibe project so it gets an ALM
org/repository and its sources move from Drive to git.

## Goal

When a branch is created (signup or "New Workspace"), a Vibe project named after the branch
is created automatically:

- `vibe_projects` row: `project_type = 'bot'`, `source_control = 'git'`, `branch_id` = the
  branch, `name` = branch slug (e.g. workspace `cristo` → project `cristo`).
- `bots` row association: the **existing** default bot row is bound to the project (id/uuid
  recorded in the project payload as `bot_id`), NOT duplicated — `ensure_vibe_bot_row`
  (`projects_api.rs:544`) must be taught to adopt the existing default bot instead of
  inserting a second row for the same slug.
- ALM provisioning (issue 1503): Forgejo org `{branch}` + repository `{branch}` are created
  and the default bot's Drive sources are imported as the initial commit (issue 1502 step 1).
- First-open experience: when the user opens General Bots and selects the default bot
  project in Vibe, they are vibing the base bot — chat with it, extend it, deploy it.

## Current behaviour (code anchors)

| What | Where |
|------|-------|
| Workspace/branch creation on signup | `botserver/crates/botcloud/src/api.rs:435` → `integration::create_cloud_workspace_inner` |
| Default bot created for org buckets without bots | `botserver/src/main_module/drive_monitors.rs:566` (`bot_names.insert(branch_slug)`) |
| Vibe project creation (idempotent, unique `(branch_id, name)`) | `botserver/crates/botvibe/src/projects.rs` `create()` |
| Bot-kind project → bots row (always INSERT, never adopts) | `botserver/crates/botvibe/src/projects_api.rs:544` `ensure_vibe_bot_row` |
| Bot project kind | `botserver/crates/botvibe/src/projects.rs:52` `ProjectKind::Bot` |

## Implementation plan

1. **`botserver/crates/botcloud/src/integration.rs`** — extend
   `create_cloud_workspace_inner(conn, branch_id, name)`: after creating the workspace row,
   call a new `botvibe::bootstrap::ensure_branch_default_project(branch_id, org_id, name)`
   (feature-gated `saas`/`vibe`; on cold start botvibe may not be linked — return early
   with a warn instead of failing signup).
2. **New `botserver/crates/botvibe/src/bootstrap.rs`**:
   - `ensure_branch_default_project(pool, org_id, branch_id, name) -> Result<Uuid, String>`
   - Idempotent: look up `vibe_projects WHERE branch_id = $1 AND name = $2` first
     (`ProjectRegistry::create` is already idempotent — reuse it).
   - Look up the default `bots` row for `(branch_id, slug = bot_slug(name))`; if found,
     write `payload.bot_id = bots.id` and `payload.bot_origin = 'drive'` (adopted), so
     chat/WS identity keeps working unchanged. Only insert a bots row when none exists.
   - Trigger git-mode provisioning lazily (issue 1503) on first Vibe open, NOT inline in
     signup (signup must stay fast; Forgejo down must not break onboarding).
3. **`ensure_vibe_bot_row`** (`projects_api.rs:544`): when `payload.bot_id` is present,
   verify the bot exists and return early (no INSERT). Keeps `origin='vibe'` semantics for
   non-bootstrap projects.
4. **Drive-created orgs** (`drive_monitors.rs` `ensure_cloud_workspace` path): call the same
   bootstrap after `ensure_branch_exists` so orgs born from Drive also get the project.
   Guard with `LoadOnly`/LOAD_ONLY respect — do not bootstrap when the flag is set.
5. **Migration/backfill** (one-shot, safe to re-run): SQL or a `botvibe` startup hook that
   scans `cloud_workspaces` and creates the missing default project per branch. Log a
   summary (`[vibe-bootstrap] branch X → project Y (bot adopted|created)`).
6. **UI**: `botui/ui/suite/vibe` project list must show the bootstrap project with kind
   `bot`; the Chat window already binds via `{ bot: slug }` for bot-kind projects
   (`vibe-shell/10_toolbar.js` `openChat`) — verify the adopted default bot slug matches.

## Acceptance criteria

- [ ] Signup (free/shared) creates workspace + branch + default bot + **Vibe project** in one flow.
- [ ] Opening Vibe shows the default bot project without any manual "New Project".
- [ ] Chat with the default bot via Vibe Chat button works (WS binds to the adopted bots row).
- [ ] Re-running bootstrap (server restart, repeated signup replay) creates nothing twice.
- [ ] A branch created only by `drive_monitor` (`.gborg` uploaded via `mc`) also gets the project.
- [ ] `bottest` integration test: signup → `vibe_projects` contains 1 bot-kind project for the branch.

## Non-goals

- Moving the bot's `start.bas` content itself (issue 1502).
- Test/prod twin bots (issue 1504).
