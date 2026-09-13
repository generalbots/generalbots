# Chapter Verification Checklist — September 2026 sweep

Tracked checklist for the verify-or-remove sweep ([#1378](https://github.com/generalbots/generalbots/issues/1378)).
One row per chapter. A chapter is **Done** when every page in it has been read
against its authority (see `botbook/AGENTS.md` → "Where the truth lives") and
either left as verified, corrected, or stripped of unverifiable claims.

Counts are generated from the repository (`find botbook/src -name '*.md'`) —
regenerate them rather than editing by hand.

| Chapter | Pages | Status | Last verified | Notes |
|---|---|---|---|---|
| 01-getting-started | 7 | ✅ Done | 2026-09 | Install, first bot, channels — steps re-run against the current bootstrap |
| 02-architecture-packages | 44 | ✅ Done | 2026-09 | Crate layout audited against `botserver/crates/*/`; component diagrams regenerated |
| 03-knowledge-ai | 34 | ✅ Done | 2026-09 | `hybrid-search.md` rewritten from `kb_context/` + `botqdrant`; `whats-new.md` retired to a pointer; model tables de-staled |
| 04-basic-scripting | 144 | ✅ Done | 2026-09 | Every keyword cross-checked against `botserver/src/basic/keywords/`; fabricated keywords removed |
| 05-multi-agent | 8 | ✅ Done | 2026-09 | Orchestration docs match the shipped pipeline |
| 06-channels | 22 | ✅ Done | 2026-09 | Provider pages updated; model comparison refreshed (2026 landscape, verified-dated) |
| 07-user-interface | 105 | ✅ Done | 2026-09 | All 73 catalog apps aligned + maturity markers (#1373, #1375); SVGs re-shot (#1374) |
| 08-rest-api-tools | 68 | ✅ Done | 2026-09 | Routes verified against handlers |
| 09-security | 26 | ✅ Done | 2026-09 | Checklists match shipped guards (sql_guard, SafeCommand, rate limits) |
| 10-configuration-deployment | 16 | ✅ Done | 2026-09 | Vault paths + config keys verified against code |
| 11-hardware-scaling | 8 | ✅ Done | 2026-09 | Sizing pages updated |
| 12-ecosystem-reference | 45 | ✅ Done | 2026-09 | Comparison matrix refreshed: OpenClaw + OpenAI Frontier added (#1376); ROADMAP split shipped/planned to 2027 (#1377) |
| 13-ai-os | 8 | ✅ Done | 2026-09 | Roadmap-initiative content separated from reference content |
| Root pages (SUMMARY, README, features…) | 8 | ✅ Done | 2026-09 | — |
| **Total** | **545** | | | Baseline was 531 pages; the count grows with the book |

## Generated numbers — do not hand-edit

| Fact | Value | Source |
|---|---|---|
| Pages | 545 | `find botbook/src -name '*.md' \| wc -l` |
| Suite apps in catalog | 73 | `scripts/docs_suite_apps_status.py` |
| Apps documented / missing | 73 / 0 | same |
| Screen SVGs | 73 | same |

## Guards (must pass before any botbook commit)

```bash
python3 scripts/docs_link_check.py            # 0 broken/unpublished links
python3 scripts/docs_summary_sync.py --check  # SUMMARY.md complete
python3 scripts/docs_component_versions.py --check
python3 scripts/docs_suite_apps_status.py     # regenerates; expect zero diff
python3 scripts/docs_suite_svg_audit.py       # SVG version claims
```
