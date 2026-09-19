# [VIBE] 1504 — Bot two-env model: `{bot}-test` twin, Run → TEST tab, Deploy → PROD tab

**Priority:** P0
**Kind:** feature (runtime + UI)
**Depends on:** 1500, 1502 · **Blocks:** nothing

## Problem

Websites already have the two-site model (`site_env.rs`: `{slug}` + `{slug}-test`, Run
lands on the test twin, only deploy pipeline writes the public slug). **Bots have no
equivalent**: a bot-kind Vibe project creates a single `bots` row (`ensure_vibe_bot_row`,
`projects_api.rs:544`) and every chat session — dev or not — talks to that one bot.
There is no way to test changes without touching the bot users are talking to.

The reform:

> "weba and bot have got the `-test` suffix for two projects to support run/deploy
> behaviour, so we will be working on `default-test` bot for the first time, deploy copy
> all to `default` bot… when a bot project is selected, browser button should be called
> **Bot**, opening the bot itself, two tabs **botname(TEST)** and **botname(PROD)**. Run
> will open test, and Deploy will open prod tab — instead of browser/new tab like website
> or app."

## Goal

One bot Vibe project owns **two bot identities**:

| Env | bots.slug | Origin | Chat reachable | Who edits it |
|-----|-----------|--------|----------------|--------------|
| TEST | `{bot}-test` | `vibe` | ✅ — Run/development sessions | Vibe workspace (git main / dev commits) |
| PROD | `{bot}` | `vibe` (or adopted `drive` for the default bot, issue 1500) | ✅ — end users | Deploy pipeline only |

Run compiles/loads the workspace into the TEST bot; Deploy copies ALL sources to the PROD
bot and reloads it — mirroring the website `websites/{slug}-test` → `websites/{slug}`
promote.

## Backend work

1. **Twin bot rows** — extend `ensure_vibe_bot_row` (`projects_api.rs:544`):
   - Ensure BOTH rows: `slug = bot_slug(name)` and `slug = format!("{bot_slug}-test")`,
     same `branch_id`/`org_id`, both `origin='vibe'` (test row flagged
     `payload.twin_of = {prod bot id}` or a `is_test_twin` boolean column — prefer a
     payload key, no migration).
   - Never steal slugs owned by another branch (keep the existing guard).
   - For the **default bot** (issue 1500 adopted row, `origin='drive'`): the PROD row is
     the adopted one; only create the `-test` twin. Test DB twin follows issue #1386
     `_dev` database semantics — TEST bot points at `bot_{branch}_{bot}_dev`.
2. **Bot compiler/runtime binding** — the git monitor (issue 1502) pulls into
   `work/{branch}/{bot}`; the TEST bot compiles from the project workspace/VM dev state,
   the PROD bot compiles from the deployed copy. Two compile targets per project:
   - `run` → write workspace sources to the TEST bot's load path → recompile `{bot}-test`.
   - `deploy` (pipeline `publish` stage, `pipeline.rs`) → copy ALL sources (`.gbdialog`,
     `.gbot`) from the pushed git state to the PROD bot load path → recompile `{bot}`.
   Record both compile stamps in the project payload (`test_compiled_at`, `prod_compiled_at`).
3. **Preview/env API** — `GET /api/vibe/projects/:id/preview?env=test|production`
   (`ops_api.rs`) must answer for bot projects too: `test` → `{bot}-test` chat URL/deep
   link, `production` → `{bot}`. Today website-only logic returns site URLs; add the bot
   branch.
4. **Deploy pipeline** — `pipeline.rs` PublishApp stage for bot kind: after deploy,
   `append_deployment` records `env: production` + the PROD bot slug (history/rollback
   works from git `release/deploy-<ts>` branches — already in `git_mode.rs`).

## UI work (`botui/ui/suite/vibe`)

5. **Toolbar button rename** — for bot-kind projects the **Browser** button becomes
   **Bot** (`vibe-shell/10_toolbar.js`): label "Bot", tooltip "Open the bot chat".
   Website/app keep Browser. The button host already branches on kind
   (`openProjectApp`/`openPreview` use `resolvedProjectKind()`).
6. **Chat window tabs** — the Chat window for a bot project shows TWO conversation tabs:
   `{NAME} (TEST)` and `{NAME} (PROD)`:
   - TEST tab → WS session bound to `bots.slug = {bot}-test` (deep-link `{ bot: slug }`,
     same mechanism as `openChat` today).
   - PROD tab → bound to `{bot}`.
   Tabs live inside the chat app window (tab bar), NOT two desktop windows.
7. **Run** (`openPreview` path for `isBot`) → starts dev VM, compiles into TEST bot,
   opens/activates the Chat window on the **TEST** tab.
8. **Deploy** (`vibe-dialog-deploy.js` `waitDeployFinished` → `openAppTab`) → for bot kind,
   instead of `window.open` of a production URL, activate the Chat window on the **PROD**
   tab (bot chat is in-suite; a new browser tab makes no sense).

## Code anchors

| What | Where |
|------|-------|
| Single-bot row creation today | `botserver/crates/botvibe/src/projects_api.rs:544` `ensure_vibe_bot_row` |
| Two-site model to mirror | `botserver/crates/botvibe/src/site_env.rs` |
| Chat deep-link for bot kind | `botui/ui/suite/vibe/vibe-shell/10_toolbar.js` `openChat()` |
| Run → chat for bot kind | `10_toolbar.js` `openPreview()` (`isBot` branch) |
| Deploy → new tab | `botui/ui/suite/vibe/vibe-dialog-deploy.js` `openAppTab()` |
| Publish pipeline stage | `botserver/crates/botvibe/src/pipeline.rs` (PublishApp) |
| Compile/load entry points | `botserver/src/main_module/drive_monitors.rs` (to be split per issue 1502) |

## Acceptance criteria

- [ ] Bot project bootstrap creates both `{bot}` and `{bot}-test` rows (default bot: PROD
      adopted, TEST twin created).
- [ ] Chat with `{bot}-test` works on TEST tab; end-user sessions on `{bot}` are unaffected
      while developing (regression: tenant isolation suite stays green).
- [ ] Run opens Chat window on TEST tab; Deploy compiles PROD and opens the PROD tab —
      no `window.open` for bot kind.
- [ ] Toolbar shows "Bot" (not Browser) only for bot projects.
- [ ] Deploy history records the production bot slug; rollback restores the previous
      PROD state from the release branch.
- [ ] Deployed PROD bot survives server restart (compiled from deployed copy, not dev VM).

## Non-goals

- More than two envs (staging) for bots.
- Channel bindings (WhatsApp) per env — PROD bot keeps existing bindings; TEST bot is
  chat-only.
