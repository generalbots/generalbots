# Suite Apps Status 🟡 BETA

The catalog and its counts are generated from `all_apps()`; do not maintain an
inventory by hand. Use the [generated app registry](./generated/registry.md) for
ids, titles, owners and visibility, and the [generated app cards](./generated/cards.md)
for the per-app summary.

| Fact derived from source | Value |
|---|---|
| Registry ids | **72** (36 in `entries.rs` + 36 in `entries_extra.rs`) |
| `AppStatus` | Supported **26** · Preview **44** · Hidden **1** (`research`) · Deprecated **1** (`clock`) |
| Launcher visibility | Always **25** · Preview **47** |
| Apps with a guide | **65** |
| Apps with a screen SVG | **65** |

Non-catalog surfaces documented elsewhere — the six `vibe-*` toolwindows, plus
`admin`, `biometry`, `sales` and `vdi` — are shell views or sub-pages, not
registry apps. They are excluded from these counts.

## Maturity markers

The badges on app pages are editorial maturity labels; `AppStatus` is the
support contract. An app is GA only when it has an owner, docs, i18n keys,
contract tests, a feature in every supported build and an on-call path. See the
[Supported Matrix](../app-support-matrix.md).
