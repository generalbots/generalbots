# Feature Editions 🟡 BETA

Build profiles are **Cargo features** in `botserver/Cargo.toml`. There is no
separate "edition" layer: an edition is just a named bundle of features, and a
bundle that is not in the manifest cannot be built.

Authority: `[features]` in `botserver/Cargo.toml` (121 entries) and
`APP_FEATURES` in `scripts/product_profile_check.py`.

## Bundles that exist

| Bundle | Expands to | Use |
|---|---|---|
| `default` | `all-apps`, `botcore/default` | every application compiled; `.product` decides visibility |
| `minimal` | `chat` | smallest build that answers chat |
| `minimal-chat` | `chat`, `automation`, `cache` | chat with no security layer |
| `lightweight` | `chat`, `tasks`, `people` | chat + tasks + CRM |
| `full` | `chat`, `people`, `mail`, `tasks`, `calendar`, `drive`, `docs`, `llm`, `cache`, `compliance`, `tax` | the wide server profile |
| `chat-drive` | `chat`, `drive`, `agent-vm` + office workbench | lean product profile |
| `desktop-local` | `chat`, `drive`, `security`, `directory`, `llm`, `vectordb`, `settings`, `no-security`, `kb-extraction`, `botcore/desktop-local` | desktop installer |
| `all-apps` | every app feature | used by `default` |

```bash
cargo build -p botserver --no-default-features --features minimal
cargo build -p botserver --no-default-features --features lightweight
cargo build -p botserver --no-default-features --features full
cargo build -p botserver --no-default-features --features desktop-local
```

## Bundles that do not exist

These names were documented in earlier editions of this book and are **not**
Cargo features — `cargo build --features <name>` fails:

`core`, `standard`, `enterprise`, `productivity`, `communications`.

Use the closest real bundle from the table above, or compose individual
features.

## Individual features

Every feature maps to a crate (`dep:<crate>`) and, usually, a `botcore`
sub-feature. A representative slice:

| Area | Features |
|---|---|
| Core transport | `chat`, `scripting`, `automation`, `cache`, `no-security` |
| Storage | `drive`, `kb-extraction`, `docs`, `sheet`, `slides`, `paper`, `player`, `canvas` |
| People / business | `people`, `tickets`, `billing`, `workspaces`, `calendar`, `tasks`, `project`, `plan`, `goals` |
| Channels | `mail`, `whatsapp`, `telegram`, `instagram`, `social`, `msteams`, `meet` |
| AI | `llm`, `vectordb`, `research`, `web-search`, `agent-vm`, `memory-os`, `connectors`, `consent`, `browser-policy` |
| Platform | `security`, `directory`, `rbac`, `settings`, `monitoring`, `analytics`, `dashboards`, `marketplace`, `git`, `database`, `editor` |

Feature names are case-sensitive and hyphenated (`web-search`, `agent-vm`,
`memory-os`).

## Composing a custom build

```bash
cargo build -p botserver --no-default-features --features "chat,drive,tasks"
```

Dependencies are declared in the manifest, so asking for `mail` also brings
`automation`, `drive`, `cache`, `digest` and the email crates. Enabling a
feature never disables another; use `--no-default-features` to drop `all-apps`.

## Verifying a profile

`scripts/product_profile_check.py` holds the mapping between Cargo features and
the apps they gate (`APP_FEATURES`). Apps without a feature gate (notes, photos,
timer, weather) ship with every build and are hidden through the `.product`
file instead.

```bash
python3 scripts/product_profile_check.py
```

## Related

- [Module Structure](../02-architecture-packages/crates.md) — the 123-member workspace
- [Building from Source](../02-architecture-packages/building.md)
- [Suite apps status](../07-user-interface/apps/suite-apps-status.md)
