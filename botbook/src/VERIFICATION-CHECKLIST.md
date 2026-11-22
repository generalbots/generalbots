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
| Suite apps in catalog | 68 | `botserver/src/apps/registry/entries*.rs` (`all_apps()`) |
| Apps documented / missing | 73 / 0 | same |
| Screen SVGs | 73 | same |

## Live acceptance gates (not claimable offline)

Two acceptance gates are tracked as GitHub issues rather than on this page:
[UAT-GATE-2 — 39 capability apps (#2178)](https://github.com/generalbots/generalbots/issues/2178)
and [the 100-scenario acceptance script (#2031)](https://github.com/generalbots/generalbots/issues/2031).
Both are **live, browser-driven runs**: #2178 drives chat cases against the 39
apps declared in `botui/ui/suite/js/app-capabilities.js` (one child issue per
app, #2183–#2221); #2031 walks the ten verticals of `UAT.md` (ten phrases per
vertical). Neither can be satisfied by reading source, running an offline
harness or inspecting a build. A code-at-HEAD review, a green `node --check`
or a passing scan is **not** a substitute for a per-case browser pass.

This page deliberately records **no PASS/FAIL verdict** for either gate. A PASS
asserted without browser evidence is invalid (the #2178 rubric states this
explicitly), and an unfinished capability is a failed acceptance, not an
exemption (#2031). Current status, run tables and screenshots live in the two
issues; the blank templates below are the standard a run must meet before an
issue may be closed.

### Per-capability-app case template

Copy this block once per app (or per scenario). Leave every cell empty until
the phrase has actually been sent in a browser on a live stack.

| Field | Value |
|---|---|
| App / scenario id | `<app-id>` or `<SCENARIO-ID>` |
| Child issue | `#NNNN` |
| Phrase (sent verbatim) | … |
| Session / conversation id | … |
| Screenshot path | `/tmp/opencode/uat<issue>_case<k>.png` |
| Verdict | PASS / FAIL / BLOCKED (fill only with evidence) |

### Per-criterion evidence table (required in the closing comment)

| Criterion | Observed evidence | Verdict |
|---|---|---|
| … | … | PASS / FAIL |

Every row must name the artifact that proves it (screenshot path, console or
network capture, DOM state, command output). "Looks correct" is not evidence.

### Gate #2178 criteria (all six per case)

- [ ] Correct app window opens side by side with chat (`window-<id>` visible).
- [ ] No raw JSON, tool payload or plan fragments in chat (`planLeak` false).
- [ ] Content complete — no TODO/lorem/empty grid — and language matches the phrase.
- [ ] Deep-link buttons, when promised, exist and open the right record.
- [ ] No uncaught console errors during the flow.
- [ ] Latency recorded (first reaction < 60 s, completion < 180 s).

Exit: every one of the 39 child issues carries a PASS/FAIL comment with
screenshot paths; every FAIL has a linked bug issue; any registry difference is
folded back into `app-capabilities.js`; the epic is closed with a final table.

### Gate #2031 criteria

- [ ] Each of the 100 phrases sent exactly as written, one at a time.
- [ ] Every step recorded PASS / FAIL; a failure carries a screenshot plus
      console/network evidence.
- [ ] Each vertical is green only when all ten of its scenarios pass.
- [ ] The release is green only when all ten verticals are green.
- [ ] One issue per failed scenario (`UAT <ID>: <symptom>`); a missing feature
      is filed as `requirement`, and a vertical with an open gap stays red.
- [ ] Environment recorded per run: botserver commit/build date, botui bundle
      date, browser version, desktop versus remote, and downloaded media opened.

### Screenshot and evidence conventions

- Screenshots go under `/tmp/` — `/tmp/opencode/uat<issue>_case<k>.png` is the
  shared convention; #2178 uses `/tmp/uat_<APP>_after.png` per app. Never in
  the repository.
- One tab per active case; one short CDP script per action, printing state and
  stopping. Never batch navigation, send and assertions into one script.
- A PASS based only on chat text is invalid. A retry after a slow-model timeout
  is a new observation, not a carry-over of an earlier verdict.
- Offline harness output (for example the `webtest-*` scans) is a pre-check, not
  a gate result. It may point at defects to fix before the run; it never
  closes a case.

See [End-to-End Testing](./12-ecosystem-reference/e2e-testing.md) for the
automated tests, which are a separate surface from these manual gates.

## Guards (must pass before any botbook commit)

```bash
python3 scripts/docs_link_check.py            # 0 broken/unpublished links
python3 scripts/docs_summary_sync.py --check  # SUMMARY.md complete
python3 scripts/docs_component_versions.py --check
python3 scripts/docs_suite_apps_status.py     # regenerates; expect zero diff
python3 scripts/docs_suite_svg_audit.py       # SVG version claims
```
