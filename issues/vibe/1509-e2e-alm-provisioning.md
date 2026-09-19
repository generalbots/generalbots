# [VIBE] 1509 — E2E test plan: ALM org/repo provisioning + git default (1503)

**Priority:** P0
**Kind:** test (E2E, Forgejo + API)
**Covers:** issue #1503 / GH #1415

## Cases
### Case A — new project defaults to git
1. Create a website project via Vibe UI (New Project).
2. Expect `source_control='git'` without any explicit source-control choice,
   and a Forgejo repo `{branch-org}/{project-slug}` created.

### Case B — org per branch
1. For two branches in the same org, create one project each.
2. Expect both repos to live under the branch-derived org
   (`alm_org_from_slug(branch_slug)`), not a shared/global org.

### Case C — legacy native backfill
1. Insert a legacy `source_control='native'` project row (or use one that
   predates the reform).
2. Restart botserver → expect `[vibe_backfill]` to provision the repo and
   flip the row to `git` with a pushed initial commit.

### Case D — protected names
1. Try creating a project named `tables` / `proxy` / `bot`.
2. Expect rejection (container-name guard, `is_protected_container_name`).

## Acceptance
- Every project in the branch has its own Forgejo repo under the branch org;
  no `native` rows remain after one boot cycle.
