# Service Level Objectives (SLOs) 📊 STABLE

"High performance" is only a claim until it is measured. This chapter defines the
service level objectives for the conversational path and the HTTP surface, the
exact point where each number is measured, and the CI gate that fails a build
when a budget is breached.

The machine-readable copy of every number below lives in `ops/slo.yaml`. That
file — not this page — is what `scripts/check_slo.py` reads, so a target can
never drift between the documentation and the gate.

## Status: provisional targets

The targets below are the ones the performance program (#2115) ratified to start
measuring against. They are **provisional** until a 7-day baseline exists for the
profile in question: the first ratified baseline is committed under
`perf/baselines/`, and a target may only move through a reviewed pull request.
A target never rises silently to hide a regression.

## The SLO table

| SLO | Class | Statistic | Target | Window | Measured at |
|-----|-------|-----------|--------|--------|-------------|
| First visible token | chat | p95 / p99 | ≤ 2000 ms / ≤ 4000 ms | 7 days | `chat.first_token_ms` — WS receive → first streamed chunk |
| Complete answer (300 tokens) | chat | p95 / p99 | ≤ 15000 ms / ≤ 30000 ms | 7 days | `chat.complete_ms` — WS receive → stream end |
| Tool call | chat | p95 / p99 | ≤ 3000 ms / ≤ 6000 ms | 7 days | `chat.tool_call_ms` — TOOL_EXEC dispatch → result persisted |
| REST API (non-LLM) | api | p95 / p99 | ≤ 300 ms / ≤ 900 ms | 7 days | `rest.api_ms` — request received → response written |
| WebSocket stability | ws | rate | ≤ 0.5 % unexpected disconnects per 1 000 sessions | 7 days | `ws.unexpected_disconnect_pct` |
| Turn error rate | chat | rate | ≤ 1 % of turns end in an unhandled error | 7 days | `turn.error_pct` |
| Queue / backlog wait | chat | p95 | ≤ 2000 ms at rated load | 7 days | `queue.wait_ms` — admitted → worker picked up |
| Provider errors | llm | rate | ≤ 2 % 5xx/429 from the provider | 7 days | `llm.provider_error_pct` |
| Provider retries | llm | rate | ≤ 5 % of calls retried | 7 days | `llm.provider_retry_pct` |
| Database query | dependency | p95 | ≤ 50 ms | 7 days | `dep.db_ms` |
| Cache (Valkey) | dependency | p95 | ≤ 10 ms | 7 days | `dep.cache_ms` |
| Drive (MinIO/S3) | dependency | p95 | ≤ 100 ms | 7 days | `dep.drive_ms` |
| Cost per conversation | cost | mean | ≤ USD 0.010 | 1 day rolling | `cost.conversation_usd` — computed from tokens in/out |
| Cost per tenant per day | cost | total | ≤ USD 10.00 hard quota, alert at 80 % | 1 day rolling | `cost.tenant_day_usd` |
| Soak RSS growth | soak | relative | ≤ 10 % over a 60-minute soak | 60-minute run | `soak.rss_growth_pct` |
| Soak file-descriptor growth | soak | relative | ≤ 5 % over a 60-minute soak | 60-minute run | `soak.fd_growth_pct` |

The chat targets are split by profile on purpose. On the `local-mock` profile
(`scripts/mock_llm.py`) the provider is not in the loop, so the first-token
target measures **our** path: WebSocket receive, classification, knowledge-base
lookup, prompt assembly and the first streamed chunk. A real provider's latency
is measured separately and never gates a pull request, because it is not ours to
fix and it is not deterministic.

## Measurement points

Every turn is one trace with the phases below. The spans are emitted through the
existing monitoring crate — no new vendor, no new agent:

| Phase | Span / histogram | Labels |
|-------|------------------|--------|
| WebSocket receive | `ws.receive` | `channel`, `bot` (hashed) |
| Classification | `chat.classify` | `channel`, `intent` |
| Knowledge base | `kb.lookup` | `bot` (hashed) |
| LLM call | `llm.request` + `governance_llm_latency_ms` | `provider`, `model` |
| Tool execution | `tool.execute` | `tool` |
| Stream | `chat.first_token_ms`, `chat.complete_ms` | `provider`, `model`, `channel` |
| Persist | `chat.persist` | `channel` |

Counters: `turns_total`, `turns_failed_total{phase}`, `llm_errors_total{status}`,
`llm_retries_total`, `rate_limit_hits_total`, `fallbacks_total`. Labels carry the
bot, org, provider, model and channel — **never** message text, a phone number or
an e-mail address. Bot and org identifiers are hashed before they leave the
process.

## Cost model

Cost per conversation is derived from the tokens the provider reports, not from
an estimate:

```
cost_usd = tokens_in / 1_000_000 * price_in + tokens_out / 1_000_000 * price_out
```

`price_in` / `price_out` come from the `pricing.models` table in `ops/slo.yaml`,
which is the single price source for the gate. A model that is not in the table
is a configuration error, not a free call: the checker refuses to guess. The
mock and the free gateway model are priced at zero, so a CI run can never accrue
a bill.

Cost per tenant per day is the sum of that value over the tenant's conversations
in the rolling 24-hour window. The quota is a hard limit — the alert fires at
80 % of it (`cost_budgets.alert_at_pct`) so the tenant owner has time to react.

## Load and soak profiles

| Profile | Concurrency | Duration | Where |
|---------|-------------|----------|-------|
| smoke | 10 sessions | 30 s | every pull request, mock LLM |
| load | 50 / 200 sessions, mixed text, tool and streaming | 10 min | nightly |
| soak | 50 sessions | 60 min | nightly, RSS/FD/queue must stay flat |

The HTTP classes are driven by the k6 scripts in `scripts/load/`
(`k6_http.js` for REST, `k6_ws.js` for session stability and first token). The
in-process budgets — locale resolution, translation, the regex memo — are
asserted by `bottest/tests/slo_budget_test.rs`, which needs no server and runs on
every pull request:

| Hot path | p95 budget |
|----------|------------|
| `Locale::new` on a BCP-47 tag | ≤ 20 µs |
| `Locale::negotiate` over the catalogue | ≤ 10 µs |
| Catalogue lookup plus placeholder interpolation | ≤ 60 µs |
| Memoised runtime regex lookup | ≤ 30 µs |

A load run never calls a paid provider: the mock (`scripts/mock_llm.py`) and the
free gateway model are priced at zero in `ops/slo.yaml`, and the smoke profile
runs against anonymous endpoints only.

The k6 summary is not the snapshot format, so `scripts/load/k6_to_snapshot.py`
is the one adapter between them:

```bash
k6 run -e SCENARIO=smoke --summary-export=/tmp/k6.json scripts/load/k6_http.js
python3 scripts/load/k6_to_snapshot.py --summary /tmp/k6.json --out /tmp/slo.json --class api
python3 scripts/check_slo.py --snapshot /tmp/slo.json --class api
```

## How the gate works

`scripts/check_slo.py` reads a metrics snapshot (JSON, from a file or standard
input), resolves each SLO from `ops/slo.yaml`, and exits non-zero when a budget
is breached:

```bash
# Validate a snapshot produced by a load run
python3 scripts/check_slo.py --snapshot /tmp/slo-snapshot.json

# Machine-readable output for a dashboard or a nightly report
python3 scripts/check_slo.py --snapshot /tmp/slo-snapshot.json --json

# Regression gate: fail when a statistic grows more than 20 % over the baseline
python3 scripts/check_slo.py --snapshot /tmp/run.json --baseline perf/baselines/local-mock.json

# Prove the checker itself still fails closed
python3 scripts/check_slo.py --self-test
```

CI runs the checker against the committed baseline and its self-test on every
pull request. A breach fails the build; a baseline may only be refreshed by a
reviewed pull request that also updates this page when a target changes.

## What a breach means

| Symptom | First place to look |
|---------|---------------------|
| First token over budget, full answer normal | prompt assembly, knowledge-base lookup, streaming chunk size |
| Full answer over budget, first token normal | provider tokens/second, retries, context size |
| REST p95 over budget | a handler doing synchronous I/O, missing index, cache miss storm |
| Turn error rate over budget | `turns_failed_total{phase}` — the phase label names the failing stage |
| Cost per conversation over budget | context growth (history limit), tool loops, a model change |
| Soak RSS or FD growth | a cache without eviction, a leaked connection pool, a task that never ends |

## Related

- [Database Optimization](./database-optimization.md) — indexes and query shapes
- [Sharding Architecture](./sharding.md) — horizontal scaling and tenant placement
- [Monitoring Dashboard](../07-user-interface/monitoring.md) — the live panels
