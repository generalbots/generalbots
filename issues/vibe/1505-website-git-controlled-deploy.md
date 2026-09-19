# [VIBE] 1505 — Websites fully git-controlled: Run/Deploy to both proxy hosts (`{slug}-test` / `{slug}`)

**Priority:** P1
**Kind:** feature (deploy pipeline + proxy)
**Depends on:** 1503 · **Blocks:** nothing

## Problem

Website projects already publish to two hosts (`site_env.rs`: `{slug}-test.{domain}` test
twin + `{slug}.{domain}` public) — but the deployed payload is assembled from the local
workspace (`collect_workspace_files`) and pushed straight into the proxy container. The
site is **hosted only in the proxy**, with no git artifact of what is live:

> "the website itself is two env too, so now is git controlled not hosted only in proxy,
> so you need to git control artifacts during run/deploy to those proxy addresses and
> `-test` slug so person can still see it in browser."

If the proxy payload is lost or edited out-of-band, there is no record; rollback replays
old payload dirs (`{dir}.prev-N`) instead of a source revision; two people publishing race.

## Goal

Every Run/Deploy for a website (and python) project produces a **git-controlled artifact**:

- The workspace is committed and pushed to `{branch}/{project}.git` (issue 1503 repo).
- Deploy builds/collects from a **known git revision** (the pushed commit), tags it
  (`site/{env}/{ts}` lightweight tag or `release/site-{env}-<ts>` branch), and materializes
  the proxy payload from that revision.
- Both hosts keep working in the browser: test host `{slug}-test.{domain}`, public
  `{slug}.{domain}`.

## Design

```
workspace ──commit/push──▶ {branch}/{project}.git ──tag site/test-<ts>──┐
                                     │ (archive at revision)             │
                                     ▼                                   ▼
                      proxy payload websites/{slug}-test/ ◀── materialize ──┘
                      proxy payload websites/{slug}/     ◀── promote from test ring head
```

- **Run** → commit+push (if dirty) → materialize `websites/{slug}-test` from the revision
  → Caddy block for the test host (already handled by `proxy_sites.rs` + `site_env.rs`).
- **Deploy (pipeline)** → snapshot branch (`release/deploy-<ts>`, already implemented in
  `git_mode::snapshot_deploy_branch`) → materialize test payload → promote to production
  target (`site_env.rs` both targets) → tag `site/prod-<ts>`.
- Rollback (`ops_api.rs::rollback`) → re-materialize payload from the recorded tag instead
  of replaying `{dir}.prev-N` copies.

## Code anchors

| What | Where |
|------|-------|
| Publish tool + `do_publish` (env default test) | `botserver/crates/botvibe/src/publish.rs` |
| Site targets per env | `botserver/crates/botvibe/src/site_env.rs` |
| Proxy payload assembly today (workspace, not git) | `botvibe::publish::collect_workspace_files` + `proxy_sites.rs` (`MAX_FILES`/`MAX_TOTAL_BYTES` guards) |
| Deploy snapshot branch | `botserver/crates/botvibe/src/git_mode.rs` `snapshot_deploy_branch` |
| Rollback | `botserver/crates/botvibe/src/ops_api.rs` (`rollback`), `ops_tools.rs` (`publish/rollback`) |
| Pipeline publish stage | `botserver/crates/botvibe/src/pipeline.rs` |
| Website Run reuses publish path | `botserver/crates/botvibe/src/projects_api.rs:~1022` (`run_project_site`) |

## Implementation plan

1. **Archive-at-revision helper** (`git_mode.rs` or new `site_release.rs`):
   `materialize_site_payload(project, revision, target_dir)` — `git archive {rev} | tar -x`
   into a temp dir, apply the existing `proxy_sites` file/size guards, then swap into the
   proxy payload dir (atomic rename, keep `.prev-N` ring as today).
2. **Commit-before-publish gate:** `do_publish` for git-mode sites: if the workspace is
   dirty → commit (as the agent's identity, reuse `GIT_USER_NAME`/`EMAIL` from git_mode)
   + push. Publish proceeds from HEAD. Refuse to publish when push fails (payload would
   not be reproducible) — surface the error, do not fall back to workspace copy.
3. **Record revision in deployment history:** `append_deployment` record gains
   `git_revision` + `git_tag`. Rollback resolves the tag from history and re-materializes.
4. **Run keeps test-host default:** `run_project_site` already publishes to test env —
   switch its payload source to `materialize_site_payload(HEAD)`.
5. **Python sites:** same flow; the python service deploy in the proxy container runs the
   materialized revision (keep DATABASE_URL per-env injection from #1386 untouched).
6. **UI:** deploy dialog history list (`vibe-dialog-deploy.js`) shows the revision/tag per
   entry; browser preview URLs unchanged (`{slug}-test.{domain}` still reachable — the
   test slug keeps working).

## Acceptance criteria

- [ ] Run → test host updated from a pushed revision; `{slug}-test.{domain}` serves it.
- [ ] Deploy pipeline → public host updated from a tagged revision; release branch + tag exist.
- [ ] `git log`/tags in the Forgejo repo reproduce exactly what both hosts serve.
- [ ] Rollback re-materializes from the recorded tag (not dir copies) and both hosts serve it.
- [ ] Publishing with a failing push is impossible (clear error in Run Dock/deploy dialog).
- [ ] Proxy guards (`MAX_FILES`, `MAX_TOTAL_BYTES`) still enforced on the materialized payload.

## Non-goals

- Migrating legacy sites (pre-reform) — they keep working; new deploys adopt the flow.
- Custom domain binding changes (`domains.rs` untouched).
