# Office performance-budget gate (#2459, offline slice)

Machine-readable performance budgets plus a zero-dependency Python gate. This
is the **declarative + enforcement** half of #2459: it publishes per-app budgets
and fails a CI perf JSON report that breaks them. It does **not** collect
telemetry, emit metrics, render dashboards, fire alerts or talk to a running
server — those are separate slices of #2459 and remain out of scope here.

## Files

| File | Role |
|------|------|
| `perf_budgets.json` | Source of truth: per-app budgets and metric definitions. |
| `check_perf_budgets.py` | Schema validator and CI report gate; dispatches `--self-test`. |
| `perf_selftest.py` | Non-vacuity self-test (kept separate to keep each file small). |
| `README.md` | This document. |

Budgets mirror the targets already declared in `ops/office-slo.yaml` (#2459):

| Metric (id) | Direction | Budget | Issue target |
|-------------|-----------|--------|--------------|
| `open_p75_ms` | max | 1000 ms | 1 MB doc opens <1 s p75 |
| `keystroke_p95_ms` | max | 16 ms | keystroke <16 ms p95 |
| `scroll_fps` | min | 60 fps | scroll 60 fps |
| `export_pdf_25_pages_p95_s` | max | 5 s | 25-page PDF export <5 s |
| `memory_per_tab_mb` | max | 300 MB | memory <300 MB/tab |
| `coedit_propagation_50_editors_p95_ms` | max | 150 ms | 50 co-editors <150 ms propagation |

`min` means higher is better (scroll fps); every other metric is `max` (lower is
better). A value exactly at budget passes (inclusive comparison). The first
five metrics are listed in `core_metrics` as the task-mandated set; co-edit
propagation is included from the issue's own budget list.

## Config schema (`perf_budgets.json`)

- `schema_version` must be `1`, `issue` must be `2459`.
- `metrics`: non-empty map of `metric_id -> {unit, direction, label}` where
  `direction` is `max` or `min`.
- `core_metrics`: non-empty list of ids declared in `metrics`.
- `apps`: non-empty map of `app_id -> {budgets, note?}`; each `budgets` map must
  contain **exactly** the ids in `metrics`, each a positive finite number.
- `reference_profile`: object describing the reference device/network/window.

Any violation fails closed (exit 2), so a malformed table can never yield a
green gate.

## CI perf JSON report

The gate takes a report whose `apps` object holds one number per metric per
budgeted app. Percentiles are computed upstream by the perf job; this gate only
compares. Extra metadata (`profile`, `measured_at`, `trace_url`) is echoed in
the verdict but not required. Apps not present in the config are reported as
warnings and ignored.

```json
{
  "profile": "reference-laptop",
  "measured_at": "2026-11-26T00:00:00Z",
  "trace_url": "https://ci.example/trace/123",
  "apps": {
    "docs": {
      "open_p75_ms": 812,
      "keystroke_p95_ms": 12.4,
      "scroll_fps": 59.2,
      "export_pdf_25_pages_p95_s": 4.3,
      "memory_per_tab_mb": 245,
      "coedit_propagation_50_editors_p95_ms": 130
    },
    "slides": { "...": "..." },
    "sheets": { "...": "..." }
  }
}
```

All three budgeted apps (and all six metrics each) must be present; a missing
or non-finite value is a breach, never a silent pass.

## Usage

Run from anywhere; paths default to the files next to the script.

```bash
# 1. Validate the config schema (no report needed).
python3 bottest/tools/perf/check_perf_budgets.py --validate-config

# 2. Prove the gate is non-vacuous (no fixtures are written to the repo).
python3 bottest/tools/perf/check_perf_budgets.py --self-test

# 3. Gate a CI perf report (text or JSON verdict).
python3 bottest/tools/perf/check_perf_budgets.py --report /tmp/perf-report.json
python3 bottest/tools/perf/check_perf_budgets.py --report /tmp/perf-report.json --json

# Override the config if needed.
python3 bottest/tools/perf/check_perf_budgets.py --config /tmp/budgets.json --report /tmp/perf-report.json
```

Exit codes: `0` all checks met · `1` a budget breach or failed self-test ·
`2` the config or report is unusable. A CI step should treat any non-zero code
as a failure.

### What the self-test proves

- the shipped config validates;
- a report sitting exactly at every budget passes, and all app/metric pairs
  are evaluated;
- a 5% improvement (still inside budget) stays green — the gate is not
  "any change fails";
- a 10% regression on `docs.open_p75_ms` fails and names the exact app/metric;
- a 10% `sheets.scroll_fps` drop (higher-is-better) fails;
- an omitted metric fails closed and is reported as missing;
- a malformed config and a malformed report JSON are both rejected.

## Out of scope (needs a running server / CI hardware)

- Web Vitals + custom-mark collection, backend Prometheus metrics, RUM
  dashboards, per-tenant p75-doubling alerts.
- A perf CI job on the fidelity corpus with trace links and a baseline-driven
  10% regression threshold. This gate enforces the **absolute** budgets only.
- Measured reference-laptop / 200-page synthetic Docs evidence — budgets here
  are declared, not ratified by a hardware run.
