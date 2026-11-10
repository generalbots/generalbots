# Supported Matrix 🟢 GA

68 apps and more than a hundred Cargo features multiply every test, doc,
security review, translation string and support obligation. "Everything is
preview" is another way of saying nothing is supported, so the product declares
one matrix: which build tier supports which app, on which platforms, owned by
which team.

| Claim | Authority |
|---|---|
| The product decision | `botserver/.product.matrix` (machine-readable, one `[tier …]` section per build) |
| Owner and status of each app | `botserver/src/apps/registry/ownership.rs` (`OWNERSHIP`, projected onto `AppDefinition.owner` / `.status`) |
| Per-tier app classification | `botserver/.product` (cloud) and `botserver/.product.desktop-local` (desktop/OS) |
| Per-app complexity ceiling | `scripts/app_complexity_budget.toml` |
| Enforcement | `scripts/product_profile_check.py`, `scripts/app_complexity_budget.py`, registry guard tests |

## The three tiers

| Build tier | Profile | Platforms | Support level | Owner | Build root |
|---|---|---|---|---|---|
| Desktop-local | `.product.desktop-local` | Linux, Windows, macOS | **GA** | `desktop` | `desktop-local` |
| Cloud | `.product` | Linux | **Supported** | `platform` | `default` |
| Everything else | — | any | **Preview / best effort** | `community` | — |

The blessed path is the desktop-local build plus the cloud server build. They
are the only tiers in the release and test matrix; anything else is community
or experimental and carries no SLA.

## Apps per tier

| Status | Apps | Meaning |
|---|---|---|
| Supported | 14 — `chat`, `drive`, `clock-widget`, `jukebox`, `sheet`, `player`, `calculator`, `notes`, `photos`, `timer`, `weather`, `integrations`, `about`, `settings` | In `apps=` of a tier; covered by the matrix SLA |
| Preview | 52 — every other registry app | Ships, excluded from the SLA, withheld until Preview mode is on |
| Hidden | `research` | Compiled and routed, never advertised (#1626) |
| Deprecated | `clock` (superseded by `clock-widget`) | Announced for removal; never in `apps=` |

An app is **GA** only when it has an owner, docs, i18n keys, contract tests, a
feature in every supported build and an on-call path. Missing any of those
demotes it to Preview. An unowned app is automatically Preview and can never
appear in `apps=`.

## Complexity budget

`scripts/app_complexity_budget.toml` declares a ceiling per app for three
metrics:

| Metric | Measured as |
|---|---|
| `routes` | REST endpoints the app owns, via `ENDPOINT_PREFIX_APPS` over the route inventory |
| `js_modules` | JavaScript files in the app's suite surface |
| `loc` | Lines of UI code (JS/HTML/CSS) in that surface |

The surface of an app is `botui/ui/suite/<app>/` plus the suite-root files whose
name is the app id or starts with `<app>-` (longest registry id wins, so
`clock-widget.css` belongs to `clock-widget`, not `clock`).

Current totals: **1,098 routes**, **513 JS modules**, **200,168 UI lines**. The
largest surfaces are the ones a change must be careful with:

| App | Routes | JS modules | UI lines |
|---|---|---|---|
| `desktop` (shell) | 4 | 62 | 27,938 |
| `chat` | 30 | 74 | 21,896 |
| `tasks` | 3 | 23 | 17,364 |
| `vibe` | 31 | 64 | 14,156 |
| `settings` | 153 | 23 | 10,391 |

The budget is a ceiling, not a snapshot: a metric may stay equal or fall, never
rise. A PR that grows one must justify the new ceiling in the same change or
split/delete code. `--update` re-baselines.

## Ownership and deprecation

Every registry entry carries `owner` (team) and `status` (`supported`,
`preview`, `hidden`, `deprecated`). The owner is responsible for the build,
tests, docs, i18n, security review and support escalation of that app.

New surface requires a matrix row before code:

- **New app** — matrix row, feature flag, route contract, i18n keys, owner and
  a deletion plan.
- **New crate** — ADR with owner, boundary, dependency direction, test and doc
  plan; no ad-hoc `botfoo` crate.
- **New Cargo feature** — must map to an app or a documented deployment
  profile; no "just in case" features.

Deprecation runs announce → hide from the launcher → keep the API for one
release → remove, with a data export path, a changelog entry and a "removed"
note in the docs. `clock` is the first app in that pipeline.

Every quarter the maintainers review crates and apps with zero tests, zero
docs, zero usage or no owner, rank the surface by usage × support cost and
delete, merge or hide the bottom decile.

## Running the guards

```bash
python3 scripts/product_profile_check.py     # matrix vs profiles vs owners
python3 scripts/app_complexity_budget.py     # per-app ceilings
python3 scripts/adequacy_guard.py            # structural rules, 450-line limit
```

`product_profile_check.py` prints a matrix diff when a tier, a profile and the
ownership table disagree, so the fix is mechanical: the profile, the matrix or
the owner row is wrong, and the diff names which.
