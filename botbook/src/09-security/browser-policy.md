# Browser Policy and Budgets 🟡 BETA

Agentic browsing is controlled by the `botbrowserpolicy` crate (Cargo feature
`browser-policy`). The crate does **not** drive a browser. It is a control
plane: an external driver service reports each completed step, and the crate
decides whether that step may be recorded, charges it against a budget, and
persists what the page said.

Authority: `botserver/crates/botbrowserpolicy/src/` — `policy.rs`, `budget.rs`,
`tasks.rs`, `memory.rs`, `api.rs`.

## Task lifecycle

```text
POST /api/browser/tasks                → queued
  POST …/:id/pause                     → paused   (only from running)
  POST …/:id/resume                    → running  (only from paused)
  POST …/:id/cancel                    → cancelled (not from a terminal state)
  POST …/:id/advance                   → records one step (only while running)
  POST …/:id/finish                    → finished, stores result + citations
  GET  …/:id/events                    → SSE stream of new steps
GET  /api/browser/tasks                → the caller's recent tasks
GET  /api/browser/memory               → browsing facts
DELETE /api/browser/memory             → purge the caller's facts
GET|PUT /api/browser/policy            → platform admin policy document
```

States are the string constants `queued`, `running`, `paused`, `finished`,
`failed`, `cancelled`. A policy or budget violation during `advance` marks the
task `failed` and stores the reason in `error`, so the failure is visible after
the fact rather than only in the response.

`GET …/:id/events` is an SSE endpoint that polls the task once per second, up to
3,600 ticks. Because `EventSource` cannot set headers, the suite passes the
bearer token as `?token=`; a header wins when both are present.

## Policy document

The per-task policy is embedded in the task's `plan` JSONB under key `policy`
(`PolicyConfig::from_plan`). When absent, `PolicyConfig::default()` applies.

| Field | Default | Meaning |
|---|---|---|
| `allowed_domains` | `[]` | Empty means "allow everything not denied"; otherwise exact host or dot-suffix subdomain |
| `denied_domains` | `[]` | Always wins over the allow list |
| `max_steps` | `60` | Step cap for the task |
| `max_cost_units` | `10_000` | Cost cap, in milli-units as reported by the driver |
| `allow_credential_pages` | `false` | Disables the credential-phishing guard |

Domain matching (`host_matches`) trims a trailing dot and accepts `host ==
domain` or `host.ends_with(".{domain}")`. `notexample.com` therefore does not
match `example.com`.

Only `http` and `https` are accepted (`url_allowed`). `file://`,
`javascript:`, `ftp://`, a malformed URL and an empty string are all rejected.
The parser is hand-written (`parse_url_parts`), not a URL crate: it splits on
`://`, drops userinfo after the last `@`, and takes the host before the first
`:`. That is sufficient for scheme and host checks but does not normalise
punycode, trailing dots beyond the one trimmed, or percent-encoded hosts.

### Credential-page guard

`credential_guard` is deliberately narrow: for tasks whose kind is `checkout`,
any URL containing the substring `login` (case-insensitive, path and query
included) is refused. The block is lifted by `allow_credential_pages` or by a
task kind other than `checkout`. It is substring-based, so it also blocks
`/blog/login-guide`; the crate accepts that over-blocking as the cost of a
simple rule. It is not a phishing detector.

## Budgets

`BudgetTracker` enforces two independent caps:

- **steps** — `charge_step()` increments a counter and fails when it would
  exceed `max_steps`;
- **cost** — `charge_cost(milli)` fails when the cumulative milli-units would
  exceed `max_cost_units`.

The tracker is process-local, but `record_step` rehydrates it from the persisted
progress document on every call (`BudgetTracker::with_usage`), so a restart does
not reset consumption. `create_task` clamps the requested `budget_steps` into
`1..=10_000`; the value is stored on the task, while the enforced cap comes from
the policy document.

Cost is whatever the driver reports in `cost_milli`. The crate does not price
anything itself, so a driver that reports zero is never stopped by the cost cap.

## Page facts

When a step succeeds and carries `facts`, `memory::upsert_page_fact` stores them
per user, keyed by URL, with a visit count and last-seen timestamp. `top_facts`
ranks them with a recency-weighted score (`fact_score`) that decays with page
age in hours, so frequently visited pages stay near the top and stale ones sink.
`purge_user` deletes a user's facts; `start_browse_session` and
`end_browse_session` bracket a browsing session.

Facts are only written on `ok: true` steps. A failed step records its note in
the progress document but contributes no memory.

## Enforcement: server vs client

Everything above is enforced **server-side**, inside `advance_task`, before the
step is appended to progress. There is no client-side policy in this crate and
no client bypass: a driver that skips the endpoint records nothing.

| Control | Enforced where |
|---|---|
| Scheme and host allow/deny | `policy::url_allowed`, called from `record_step` |
| Credential guard | `policy::credential_guard`, called from `record_step` |
| Step and cost caps | `budget::BudgetTracker`, called from `record_step` |
| Task state transitions | `tasks::{start,pause,resume,cancel,finish}` |
| Admin policy edit | `is_super_admin` (`botsecurity_auth::platform_admin`) |
| Ownership of a task | **Not enforced** — see below |

## What is not implemented

| Gap | Detail |
|---|---|
| Global policy is stored but not applied | `PUT /api/browser/policy` writes the document into `user_preferences` under the nil user and `browser_policy`, and `GET` reads it back; `create_task` and `advance_task` never read it. Only the per-task `policy` in `plan` (or the defaults) is enforced. |
| Task ownership is not checked | `pause`, `resume`, `cancel`, `advance` and `finish` resolve the caller but act on any task id. `list_user_tasks` does filter by user. Any authenticated user who knows a UUID can advance or finish that task. |
| `domains` on the task is informational | `create_task` stores the requested domains in `tasks.domains`, but validation uses `PolicyConfig`, not that column. |
| No driver in this crate | There is no Chromium or Playwright code here; execution lives in the external driver that calls `/advance`. |
| No pricing | Cost caps depend entirely on driver-reported `cost_milli`. |

## Related pages

- [Browser - Web Browser](../07-user-interface/apps/browser.md) — the suite app.
- [Security Features](./security-features.md) — the wider guardrail set.
- [System Limits & Rate Limiting](./system-limits.md) — request-level limits.
- [Generated REST reference](../08-rest-api-tools/generated/api-browser.md) —
  every browser route with parameters.
