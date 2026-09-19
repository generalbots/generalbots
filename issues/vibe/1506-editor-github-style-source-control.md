# [VIBE] 1506 — Editor: GitHub-style source control (repo tree center, manual commit right, diff icons)

**Priority:** P1
**Kind:** frontend (editor layout rework)
**Depends on:** 1503 · **Blocks:** nothing

## Problem

The current Source Control dialog (`vibe-dialog-git.js`) is a flat two-pane dialog: a
sidebar with a commit box + a flat changed-file list, and a main area with a commit log /
diff toolbar. The user wants a **GitHub-like review experience**:

> "review editor layout to use modern toolbar style — i.e. source control, keep manual
> commit OK, but focus in show a github view like, folders and files on center like github
> and manual commit on right, put diff icons on changed files/new."

## Goal

Rework the editor (`editor.html` + `vibe-dialog-git.js`) into a three-zone layout:

```
┌──────────────────────────────────────────────────────────────────────┐
│ Toolbar: branch selector ▾ | Fetch · Pull · Push | ±N changed | ✚ Commit │
├───────────────────────────────┬──────────────────────────────────────┤
│ CENTER — repository tree      │ RIGHT — manual commit panel          │
│ (GitHub "Files" view)         │  commit message (Ctrl+Enter)         │
│  📁 gbdialog/                 │  [ Commit ] [ Commit & Push ]        │
│    📄 start.bas        M      │  ── staged/changed list (checkboxes) │
│    📄 tables.bas       M      │  branch bar · ahead/behind · sync    │
│  📄 README.md          U      │                                      │
│  (click file → diff view)     │                                      │
└───────────────────────────────┴──────────────────────────────────────┘
```

- **Center:** full repo tree (folders + files, like the GitHub file browser), not just
  changed files. Status letters with color coding at the right of each changed row:
  `M` modified (amber), `U`/`A` untracked/added (green), `D` deleted (red), `R` renamed.
  Unchanged files show no badge.
- **Right:** manual commit stays (keep manual commit OK): message, staged selection
  (checkbox per changed file, stage-all default), commit, commit & push.
- **Toolbar:** modern top toolbar — branch selector (dropdown switching branches),
  fetch/pull/push buttons with ahead/behind counters, changed-count chip, commit CTA.
- Clicking a changed file opens the diff (`/api/git/diff/:file` — exists) in the center
  pane replacing the tree, with a breadcrumb to go back to the tree.

## Code anchors

| What | Where |
|------|-------|
| Dialog to rework | `botui/ui/suite/vibe/vibe-dialog-git.js` (264 lines — sidebar/main split today) |
| Status → color mapping (reuse) | `statusClass()` in `vibe-dialog-git.js:79` |
| APIs already available | `/api/git/status`, `/api/git/log`, `/api/git/branches`, `/api/git/commit`, `/api/git/diff/:file` (see `loadStatus`/`loadLog`/`loadDiff`) |
| Tree API | needs backend: extend git API with `/api/git/tree?repo=&ref=&path=` (list dir entries + per-file status) — reuse the git plumbing in `botvibe` harness `git_tools.rs` |
| Editor shell hosting the dialog | `botui/ui/suite/editor.html` |
| Dialog CSS conventions | `vibe-dialogs.p1.css` / `vibe-dialogs.p2.css` (`vibe-dialog-sidebar`, `vibe-dialog-main`, `vibe-list`, `vibe-status`) |

## Implementation plan

1. **Backend first:** add `GET /api/git/tree` (path-listing with recursive status map) to
   the git routes (same auth/RBAC as existing git endpoints). One call returns entries +
   status, so the frontend renders the tree without N status calls.
2. **Frontend structure:** keep `vibe-dialog-git.js` as the entry (D.register("git", …))
   but split the view builders under `botui/ui/suite/vibe/modules/` only if the file
   exceeds the 450-line limit — prefer one cohesive file if it fits (currently 264; budget
   ~200 more lines is fine).
3. **Tree renderer:** recursive `<ul>` from `/api/git/tree`, folders collapsible
   (persist open state in sessionStorage), status badge column, click → diff pane.
4. **Commit panel:** staged checkboxes wired to `/api/git/commit` (extend payload with
   `files: []` — backend currently commits everything; add pathspec support in the git
   commit handler).
5. **Branch selector:** dropdown fed by `/api/git/branches`; switching runs checkout +
   reloads tree/log. Pull/push buttons call the existing push endpoint (and a new pull —
   small backend addition mirroring push).
6. **Diff view:** reuse `loadDiff`; add +/- line highlighting (green/red background rows)
   if the API returns raw patch text (style, no library).
7. **Toolbar styling:** consistent with `vibe-shell/10_toolbar.js` modern look (no new
   frameworks, local assets only — NO CDN per AGENTS.md).

## Acceptance criteria

- [ ] Center pane shows the full folder/file tree of the repo (like GitHub), changed files
      carry M/U/D badges with color coding.
- [ ] Right panel: manual commit with per-file staging + commit & push.
- [ ] Toolbar: branch dropdown (switch works), pull/push with ahead/behind, changed chip.
- [ ] Clicking a changed file shows the diff; breadcrumb returns to the tree.
- [ ] Works for every git-mode project (bot, website, app) — same dialog, project-scoped
      repo name (repoName()).
- [ ] No file in this change exceeds 450 lines; no CDN assets; keyboard: Ctrl+Enter commit.

## Non-goals

- Inline editing from the tree (Code dialog already edits files).
- PR/review workflows (Forgejo-side concern).
- Merge-conflict resolution UI.
