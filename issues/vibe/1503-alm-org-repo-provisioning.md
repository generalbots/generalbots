# [VIBE] 1503 — Every project gets an ALM org/repository (org-branch provisioning)

**Priority:** P0
**Kind:** feature
**Depends on:** 1500 · **Blocks:** 1501, 1502, 1505

## Problem

Git-mode Vibe projects already create a Forgejo repo, but only when `source_control = "git"`
is explicitly chosen, and the org derivation (`VmLifecycle::alm_org(branch_id)`) is a
generic branch container — not the user-visible "org-branch organization" model. The reform
requires:

> "in ALM there will be an org and a repository for the default bot of that branch as well…
> if a new bot or website is created, the repository must go to the org-branch organization
> in Forgejo and a repository with same name of bot/web/app is created."

So the ALM layout becomes:

```
Forgejo
└── {branch-slug}/              ← one org per branch (the "org-branch organization")
    ├── {branch-slug}.git       ← default bot repo (issue 1500)
    ├── my-website.git         ← website project
    ├── my-app.git             ← app project
    └── other-bot.git          ← additional bot project
```

Repo name == project name == bot/site/app name. One org per branch, all projects inside.

## Goal

- `source_control = "git"` becomes the **default** for new projects (not opt-in).
- Org is derived from the branch slug (readable, stable), not from the branch UUID.
- Repo creation is automatic on project create; failures are surfaced in the project
  status, never silently swallowed.

## Code anchors

| What | Where |
|------|-------|
| Org/repo derivation today | `botserver/crates/botvibe/src/vm_lifecycle.rs` — `VmLifecycle::alm_org(branch_id)`, `alm_repo(name)` |
| Repo creation flow | `botserver/crates/botvibe/src/git_mode.rs` `ensure_git_repo` (create → init → commit → push) |
| Default source_control today | `botserver/crates/botvibe/src/projects.rs` `create()` — `_ => "native"` |
| Forgejo client | `botdeployment::ForgejoClient::create_repository` |
| ALM config | `botcoresecrets::alm_config()` (Vault `secret/gbo/alm`) |
| Delete path already removes the repo | `botserver/crates/botvibe/src/projects_api.rs:~300` (delete handler) |

## Implementation plan

1. **Branch-slug orgs:** add `branch_slug` lookup (branches table) with a
   `sanitize_project_id`-style slugifier; fall back to `alm_org(branch_id)` when the branch
   row has no slug. Cache per-process (slug never changes per boot).
2. **Org auto-create:** `ForgejoClient` may only create repos; ensure the org exists first
   (`POST /api/v1/orgs` — idempotent 409-tolerant). Extend `ForgejoClient` with
   `ensure_org(name)`.
3. **Default to git:** in `ProjectRegistry::create`, when `source_control` is absent →
   `"git"` for kinds `bot`/`website`/`apps` (keep `"github"` import path untouched).
   Update the API docs + `tool_executor.rs` schema descriptions accordingly.
4. **Wire provisioning into create flow:** after `create()`, call
   `git_mode::ensure_git_repo(project)` (already async + idempotent) from
   `projects_api::create_project`; set `status='ready'` on success, `status='provision_failed'`
   + payload error on failure (retry button / agent can re-run provisioning).
5. **Migrate legacy projects:** a backfill pass (same pattern as issue 1500 bootstrap) —
   for every `vibe_projects` row with `source_control='native'`, run `ensure_git_repo`
   once, record `payload.git_provisioned_at`.
6. **Permissions:** Forgejo repo visibility private by default; branch admins get owner
   access (org team provisioning is a stretch goal — document if deferred).

## Acceptance criteria

- [ ] Creating a bot/website/app project (UI or agent `create/project`) auto-creates
      `{branch}/{project}.git` in Forgejo without any git checkbox.
- [ ] The default bot of a branch has its own repo (issue 1500 wiring).
- [ ] Deleting the project deletes the repo (existing path — verify it uses the same org
      derivation, otherwise repos leak).
- [ ] Forgejo down → project creation still succeeds (status `provision_failed`, error in
      payload, retry works).
- [ ] Legacy `native` projects get provisioned by the backfill.
- [ ] Repo name collisions across branches are impossible (org per branch).

## Non-goals

- Per-user Forgejo accounts / SSO into Forgejo.
- CI workflows inside Forgejo for vibe projects (deployment stays via the deployment API).
