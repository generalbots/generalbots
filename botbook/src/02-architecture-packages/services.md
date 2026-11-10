# Service Layer 🟡 BETA

The service layer is the set of in-process modules `botserver` mounts at boot.
It is **not** a list of crates: crates live in `botserver/crates/*` (see
[Module Structure](./crates.md)) and are reached through Cargo features.

Authority for this page: `botserver/src/main_module/mod.rs`,
`botserver/src/core/mod.rs` and the directories they declare.

## `main_module/` — process wiring

Declared in `botserver/src/main_module/mod.rs`:

| Module | Responsibility |
|---|---|
| `init` / `init_tasks` | bootstrap sequence, task scheduler start-up |
| `server/` | Axum server, auth layer, rate limits, fallback responses |
| `routes/` | hand-written route modules (`chat_handlers.rs`, `product_handlers.rs`, `catalog.rs`, `org_handlers/`, `feature_routers/`, …) |
| `drive_monitors/` | Drive (MinIO `.gbai`) scanning, bot discovery, reload on ETag change |
| `git_bot_monitor/` | pulls git-owned bots every 15 s into the work dir |
| `state_builder.rs` | assembles `AppState` from config, pools and caches |
| `background.rs`, `auto_service.rs` | long-running loops |
| `event_dispatcher.rs`, `channel_delivery.rs` | outbound event and channel fan-out |
| `ui_plan/` | `__ui_plan__` handling for the web channel |
| `wizard_provision/` | first-run provisioning of stack components |
| `health.rs`, `shutdown.rs`, `migrations.rs` | health probes, graceful shutdown, Diesel migrations |

## `core/` — runtime state and engines

| Module | Responsibility |
|---|---|
| `state.rs` | `AppState` shared by every handler |
| `config.rs`, `config_reload.rs` | configuration read/reload |
| `secrets/` | Vault-backed secret resolution |
| `session/` | user session records and lifecycle |
| `bot/` | chat engine: `ws/` (handshake, frames, streaming), `pipeline/`, `api_catalog/`, `endpoint_inventory*.rs`, `kb_context/`, `sandbox/`, `runs/` |
| `directory/` | Zitadel-backed directory operations |
| `oauth/` | OAuth flows |
| `home_site.rs`, `peer_network*.rs` | per-tenant home sites and peer federation |
| `user_memory.rs`, `desktop_activity.rs`, `computer_use.rs` | memory hooks, desktop and computer-use surfaces |
| `ai_accounts*.rs` | per-tenant LLM account records |

## `basic/` — script host

`basic/` registers the BASIC keyword surface and runs `.gbdialog` scripts. The
keywords themselves live in `botserver/src/basic/keywords/` and in the
`botbasic_*` crates (`botbasic_core`, `botbasic_ai`, `botbasic_comms`,
`botbasic_data`, `botbasic_system`, `botbasic_compiler`, `botbasic_types`).
Execution is Rhai-based; `tables.bas` is parsed at compile time, not called.

## `apps/` and `security/`

- `apps/registry/` is the single source of the suite app catalog
  (`all_apps()`); `apps/commands/` holds per-app command surfaces used by
  `deep_link_params_for_app` and the UI planner.
- `security/` holds the mandatory guards: `command_guard` (`SafeCommand`),
  `error_sanitizer` (`log_and_sanitize`), `sql_guard`, rate limiting and CSRF.

## Layering

```
main.rs
 └── main_module (wiring: init → server → routes)
      └── core (state, bot engine, sessions, secrets)
           └── basic (script host) → botbasic_* crates
                └── feature crates (botcrm, botbilling, botdrive, …)
```

Dependencies point downward only. A feature crate never imports `botserver`
(it has no library target); it exposes `routes()`/handlers that `botserver`
mounts behind a Cargo feature.

## Error handling

Every service returns `anyhow::Result<T>` (or a crate-local error) and
propagates with `?`. `panic!`, `unwrap` and `expect` are forbidden outside
tests; user-facing HTTP errors go through `log_and_sanitize` so internal detail
never reaches the client.
