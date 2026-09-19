# [VIBE] 1511 — E2E test plan: website git-controlled deploy (1505)

**Priority:** P1
**Kind:** test (E2E, browser + proxy)
**Covers:** issue #1505 / GH #1417

## Cases
### Case A — deploy ships committed state only
1. In a git-mode website project: commit `index.html` v1, Run (TEST shows v1
   at `{slug}-test.{published-domain}`).
2. Edit `index.html` to v2 in the workspace WITHOUT committing.
3. Deploy → expect the PROD host `{slug}.{published-domain}` to serve **v1**
   (the deploy auto-commits per #1504 contract — if it serves v2, verify the
   auto-commit landed on `main` first; either way PROD must equal `origin/
   main` HEAD, never unpushed state).

### Case B — explicit revision deploy
1. Tag v1 as `site/test-<ts>` (or keep the auto `release/deploy-<ts>`
   snapshot branch created by gitflow).
2. Commit v3, deploy → PROD = v3.
3. Rollback to the recorded deployment (ops `…/deployments/:index/rollback`)
   → PROD = the earlier revision, re-materialized from the git tag/branch.

### Case C — python site payload from git
1. Python project with `app.py` committed; add `requirements.txt` uncommitted.
2. Deploy → service boots from committed `app.py`; `DATABASE_URL` still
   injected (production DB, `_dev` for test).

### Case D — serveability guard still applies
1. Remove `index.html` from the repo and deploy → expect a clean failure
   ("static publish requires an index.html"), not a blank site.

## Acceptance
- PROD content is always a materialization of a git revision; rollback works
  from history without workspace state.
