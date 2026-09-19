# [VIBE] 1512 — E2E test plan: GitHub-style source control in the editor (1506)

**Priority:** P1
**Kind:** test (E2E, browser CDP — visual evidence mandatory)
**Covers:** issue #1506 / GH #1418

## Cases
### Case A — repo tree (center pane)
1. Open Source Control for a git-mode project with ≥2 folders and ≥5 files.
2. Expect a GitHub-like tree: folders + files in the center, sorted dirs
   first; clicking a file opens it; clicking a CHANGED file opens the diff
   view with a breadcrumb back to the tree.
3. Screenshot `/tmp/{project}_source_control_tree.png`.

### Case B — change badges
1. Create a file (untracked), edit one, delete one via the editor.
2. Expect **U**, **M**, **D** badges (correct colors) on the right rows and
   the ±N chip in the toolbar.

### Case C — manual commit panel (right pane)
1. Stage only SOME files (per-file staging kept — manual commit stays).
2. Write a message (Ctrl+Enter submits), commit.
3. Expect only staged files in the commit; Forgejo `main` shows them; the
   badges for committed rows disappear.

### Case D — pull/push with ahead/behind
1. Commit a change in Forgejo web UI (behind).
2. Open Source Control → expect "behind 1"; Pull → workspace updated, badge
   clears.
3. Make a local commit ahead → expect "ahead 1"; Push → Forgejo shows it.

### Case E — backend endpoints
- `GET /api/git/tree?path=` returns GitHub-shaped `{ tree: [...] }` entries
  (path, type, status M/U/A/D/R, staged).
- `POST /api/git/commit` with a pathspec subset commits only those paths.

## Acceptance
- The dialog reads like GitHub: tree center, commit right, branch/pull/push
  toolbar — with manual commit preserved.
