# [VIBE] Reform — Git-controlled bots & two-environment projects (issues 1500–1506)

Reform goal: **Vibe code coordinates the bot**. Every branch gets a default bot as a Vibe
project backed by a Forgejo repository; Drive (MinIO) stops storing bot sources; bots and
websites gain the same TEST/PROD two-environment model; the editor gets a GitHub-style
source control view.

## Read order (dependency order)

| # | Issue | Summary | Priority |
|---|-------|---------|----------|
| 1 | [1500 — Default bot Vibe bootstrap](1500-default-bot-vibe-bootstrap.md) | Branch creation auto-creates the default bot as a Vibe project (people vibe the base bot) | P0 |
| 2 | [1501 — Remove bot sources from Drive](1501-remove-bot-sources-from-drive.md) | `.gbot`/`.gbdialog` move to git; Drive keeps only `.gbkb`/`.gbdrive`; archival migration | P0 |
| 3 | [1502 — Git-pull bot monitor](1502-git-pull-bot-monitor.md) | Drive monitor replaced by Forgejo git-pull into `work/` + compile/reload | P0 |
| 4 | [1503 — ALM org/repo provisioning](1503-alm-org-repo-provisioning.md) | One Forgejo org per branch; repo per project, named after the bot/web/app; git becomes default | P0 |
| 5 | [1504 — Bot two-env model](1504-bot-two-env-test-prod.md) | `{bot}-test` twin bot; toolbar **Bot** button; Chat window with `NAME (TEST)` / `NAME (PROD)` tabs; Run→TEST, Deploy→PROD | P0 |
| 6 | [1505 — Websites git-controlled](1505-website-git-controlled-deploy.md) | Run/Deploy materialize proxy payloads from git revisions/tags; `{slug}-test.{domain}` keeps working | P1 |
| 7 | [1506 — Editor GitHub-style source control](1506-editor-github-style-source-control.md) | Repo tree center (GitHub-like), manual commit right, M/U/D diff icons, modern toolbar | P1 |

Dependency graph:

```
1500 ──▶ 1501 ──▶ 1502 ──▶ 1504
  │        └───────▶ 1502
  └──────▶ 1503 ──▶ 1505
           1503 ──▶ 1506
```

(1504 also consumes 1503's repos; 1505/1506 are independent of the bot monitor path.)

## Key invariants (all issues)

- Manual commit stays (user requirement, issue 1506).
- Two-env rule: test first, production only via the deploy pipeline (mirrors `site_env.rs`).
- Drive keeps serving `.gbkb` (knowledge) and `.gbdrive` (documents) — only bot **sources** move.
- No panics/`unwrap` in new server code; files ≤ 450 lines; local assets only; browser-test
  all UI flows via Chrome CDP 9222 (AGENTS.md).
- Naming is canonical through `VmLifecycle::alm_org`/`alm_repo` (issue 1503 extends to
  branch slugs) — one naming function, no divergent derivations.
