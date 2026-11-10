# Module Structure 🟡 BETA

`botserver` is **not** a single crate. The repository is a Cargo workspace with
**123 members**, version **7.0.0** (`[workspace.package] version` in the root
`Cargo.toml`). Two vendored office-document crates (`3rdparty/docx-rs`,
`3rdparty/umya-spreadsheet`) are explicitly excluded from the workspace.

Everything on this page is derived from the manifests and the source tree:

```bash
python3 - <<'PY'
import re
t = open("Cargo.toml", encoding="utf-8").read()
i = t.index("\nmembers = [") + 1
j = t.index("[", i)
depth = 0
for k in range(j, len(t)):
    if t[k] == "[": depth += 1
    elif t[k] == "]":
        depth -= 1
        if depth == 0:
            print(len(re.findall(r'"([^"]+)"', t[j:k])))
            break
PY
```

## Workspace layout

| Group | Members | Location |
|---|---:|---|
| Application crates | 6 | `botserver`, `botui`, `botlib`, `botschema`, `bottest`, `botapp` |
| Feature crates | 117 | `botserver/crates/*` |
| Vendored, excluded | 2 | `3rdparty/docx-rs`, `3rdparty/umya-spreadsheet` |

`[workspace] default-members` is `botserver`, `botui`, `botlib`, `botschema` —
a bare `cargo build` at the root builds those four, not the 117 feature crates.

| Member | Kind | Responsibility |
|---|---|---|
| `botserver` | binary crate | HTTP/WS server, bootstrap, route mounting, BASIC host |
| `botui` | binary crate | static front ends for suite (3000), cloud (4000), login (5000) |
| `botlib` | library | shared models, `MessageType`, i18n, database helpers |
| `botschema` | library | the single home of every Diesel `table!` macro |
| `bottest` | test crate | cross-crate integration suites |
| `botapp` | binary (Tauri 2) | desktop shell that supervises the botserver sidecar |

## botserver has no library target

`botserver/Cargo.toml` declares no `[lib]` and no `[[bin]]`; the binary is the
default `src/main.rs`. There is **no `src/lib.rs`**, so nothing is "exported for
external use". Consequences:

- `cargo test -p botserver` fails with *"no library targets found"*; unit tests
  build with `cargo test -p botserver --bin botserver`.
- Other crates cannot `use botserver::...`; they depend on `botcore`,
  `botbasic_*`, `botsecurity*` and the other libraries instead.

`src/main.rs` declares three unconditional modules and re-exports the bootstrap
surface:

```rust
pub mod main_module;
pub mod session_pool;
pub mod soon_features;
pub use main_module::{BootstrapProgress, health_check, health_check_simple,
                      receive_client_errors};
```

Every other top-level module is feature-gated (`#[cfg(feature = "…")]`), e.g.
`analytics`, `billing`, `management` (`saas`), `vibe`, `basic` (`scripting`).

## Request flow

```
src/main.rs
  └── main_module::init            bootstrap: config → Vault → DB pool → cache
        └── main_module::server    Axum router, middleware, auth + rate limits
              ├── main_module/routes/*        hand-written route modules
              ├── botserver/crates/*/routes*  feature routers mounted per crate
              └── core::bot/ws               WebSocket chat pipeline
                    └── basic/               BASIC interpreter (.gbdialog → .ast)
                          └── crates/botbasic_*  keyword families
```

- **`main_module/`** — process-level wiring: `init.rs`, `server/`, `routes/`,
  `drive_monitors/`, `git_bot_monitor/`, `ui_plan/`, `state_builder.rs`,
  `background.rs`, `shutdown.rs`.
- **`core/`** — runtime state and the chat engine: `state.rs`, `config.rs`,
  `secrets/`, `session/`, `bot/` (with `ws/`, `api_catalog/`,
  `endpoint_inventory*.rs`, `pipeline/`), `directory/`, `oauth/`.
- **`basic/`** — the BASIC host: parser wiring and `keywords/` registration.
- **`apps/`** — the suite app registry (`registry/`) and per-app command
  surfaces (`commands/`).
- **`security/`** — `SafeCommand`, `log_and_sanitize`, SQL guards, rate limits.
- Domain directories — `crm/`, `billing/`, `people/`, `drive/`, `learn/`,
  `vibe/`, `meet/`, `whatsapp/`, … each a thin host over a feature crate.

## Feature crates

`botserver/Cargo.toml` defines the bundles; each one pulls a feature crate:

| Bundle | Expands to |
|---|---|
| `minimal` | `chat` |
| `minimal-chat` | `chat`, `automation`, `cache` (no security) |
| `lightweight` | `chat`, `tasks`, `people` |
| `full` | `chat`, `people`, `mail`, `tasks`, `calendar`, `drive`, `docs`, `llm`, `cache`, `compliance`, `tax` |
| `chat-drive` | `chat`, `drive`, `agent-vm` and the office workbench |
| `desktop-local` | `chat`, `drive`, `security`, `directory`, `llm`, `vectordb`, `settings`, `no-security`, `kb-extraction`, `botcore/desktop-local` |
| `default` | `all-apps` + `botcore/default` |

Bundles `core`, `standard` and `enterprise` do **not** exist — see
`03-knowledge-ai/editions.md`.

## Crate map (117 members)

Description column is the crate's own `description` in its `Cargo.toml`; `—`
means the manifest carries none (do not assume a responsibility from the name).

| Crate | Responsibility (from `Cargo.toml`) |
|---|---|
| `botagent` | — |
| `botanalytics` | General Bots analytics, insights, OKR goals tracking, and reporting |
| `botapi` | — |
| `botattendance` | General Bots attendance module - queue, SLA, webhooks, LLM assist, keyword services |
| `botattendant` | General Bots contact center attendant queue, session, and agent management |
| `botautomation` | NL scheduled adaptive agents, planner-executor-verifier engine and progress dashboard API (#1170, #1171). |
| `botautotask` | — |
| `botbanking` | Banking reconciliation and delivery-platform integration. |
| `botbasic_ai` | BASIC AI/intelligence keywords - LLM, AI tools, MCP, sandbox, orchestration, web scraping |
| `botbasic_comms` | BASIC communication keywords - messaging, email, SMS, social, chat, webhooks, calendar |
| `botbasic_compiler` | BASIC compiler - syntax transforms, goto logic, save conversion, tool parsing |
| `botbasic_core` | BASIC language core keywords - arrays, math, validation, datetime, strings, control flow, hearing, procedures, errors |
| `botbasic_data` | BASIC data keywords - database operations, tables, memory, KB, CRM, products |
| `botbasic_system` | BASIC system keywords - bot management, file ops, scheduling, security, app server, face API |
| `botbasic_types` | Shared types and runtime trait for BASIC keyword crates |
| `botbilling` | General Bots billing, invoicing, quotas, and subscription management |
| `botbiometry` | Biometric identity verification, KYC, and digital signature — Zitadel integration. |
| `botbrazil` | Brazilian tax compliance crate (NFe, NFSe, CTe, MDFe, SPED, EFD, ICMS modules) |
| `botbrowser` | General Bots browser automation |
| `botbrowserpolicy` | Hardened agentic browsing control plane: domain policy, budgets, browsing memory and task orchestration API (#1175 backend). |
| `botcalendar` | — |
| `botcanvas` | — |
| `botchannelbindings` | General Bots channel bindings - per-bot default phone, WhatsApp and Telegram identities plus call logs |
| `botchannels-core` | General Bots channels core: OAuth and media upload |
| `botchannels` | General Bots social media channel integrations |
| `botclock` | Electronic time clock (Registro de Ponto Eletrônico) with Brazilian CLT rules in botbrazil. |
| `botcloud` | General Bots SaaS - public checkout & subscriber dashboard using botbilling |
| `botcompliance` | — |
| `botconnectors` | Enterprise knowledge connectors - permissioned indexing, redaction and search backend over external chat, mail and drive sources. |
| `botconsent` | General Bots per-app agent consent system - grants, pending prompts and audit for application action classes |
| `botcontacts` | — |
| `botcore` | — |
| `botcorebot` | — |
| `botcoredirectory` | — |
| `botcoreoauth` | — |
| `botcorepkg` | — |
| `botcoresecrets` | — |
| `botcoresession` | — |
| `botdashboards` | — |
| `botdatabase` | — |
| `botdeployment` | General Bots deployment infrastructure for VibeCode platform |
| `botdesigner` | — |
| `botdesktop` | Desktop session proxy — WebSocket-to-TCP relay for remote desktop connections |
| `botdocs` | General Bots document processing, collaboration, and conversion |
| `botdomains` | Domain registrar adapters and DNS provisioning for General Bots |
| `botdrive` | — |
| `boteditor` | — |
| `botemail` | — |
| `boterp` | — |
| `boteval` | LLM response evaluation framework (datasets, contracts, runner, CI gate) |
| `botevents` | General Bots run event contract: envelope, sequencer, validation and redaction |
| `botfraud` | — |
| `botgit` | — |
| `botgl` | — |
| `bothandoff` | — |
| `bothr` | — |
| `botinstagram` | — |
| `botintegrations` | — |
| `botinventory` | — |
| `botitsm` | — |
| `botkyc` | — |
| `botlearn` | Learning Management System (LMS) module for General Bots |
| `botlegal` | — |
| `botllm` | General Bots LLM provider implementations and core types |
| `botlog` | Redacting log facade for sensitive identifiers |
| `botm365` | Microsoft 365 integration: SharePoint connector, OAuth2 with Zitadel, Graph client. |
| `botmaintenance` | General Bots system maintenance and cleanup |
| `botmarketing` | General Bots marketing campaigns, email, WhatsApp, IP routing, warmup, and advisor |
| `botmarketplace` | General Bots Skills Marketplace - public catalog, publish/install into bot Drive buckets, starter skills |
| `botmeet` | — |
| `botmemory` | General Bots durable per-user and per-branch memory store - extraction hook, recall injection and import/export (issue #1178) |
| `botminutes` | — |
| `botmodelsbridge` | — |
| `botmonitoring` | General Bots monitoring, metrics collection, alerting, and distributed tracing |
| `botmsteams` | — |
| `botmultimodal` | General Bots multimodal AI client (image, video, audio, speech) |
| `botnvidia` | General Bots NVIDIA GPU monitoring module |
| `botpaper` | — |
| `botpeople` | — |
| `botplan` | General Bots project planner (Gantt + Kanban) with real-time collaboration |
| `botplayer` | General Bots media player |
| `botpos` | — |
| `botproducts` | General Bots products, services, inventory, and pricing management |
| `botproject` | General Bots project management |
| `botproviders` | — |
| `botpublic` | P2P payloads and public profile for General Bots — #1866 #1867 (in progress) |
| `botqdrant` | General Bots Qdrant vector database client |
| `botresearch` | Research module for General Bots - web search, knowledge base exploration, and deep research |
| `botretail` | Retail management (Bling-style): products, stock, variations, price lists, POS, NFCe via botbrazil. |
| `botsales` | — |
| `botsampledata` | Idempotent demo data seeding for a production-ready General Bots instance. |
| `botsearch` | General Bots full-text search service |
| `botsecurity-auth` | General Bots auth: authentication, authorization, session management |
| `botsecurity-core` | General Bots security core: guards, sanitizers, validators |
| `botsecurity-crypto` | General Bots crypto: TLS, CA, encryption, certificates |
| `botsecurity-protection` | General Bots protection: firewall, IDS, hardening automation |
| `botsecurity` | General Bots security infrastructure (facade crate) |
| `botsettings` | — |
| `botsheet-core` | General Bots sheet core: types, formulas, state |
| `botsheet` | — |
| `botslides` | — |
| `botsocial` | — |
| `botsources` | — |
| `bottasks` | — |
| `bottax` | — |
| `bottelegram` | — |
| `bottemplates` | — |
| `bottickets` | — |
| `bottimeclock` | — |
| `bottimeseries` | General Bots time-series metrics service (InfluxDB-compatible) |
| `botuifragments` | HTMX UI fragment routes for bot apps. |
| `botvibe` | — |
| `botvideo` | — |
| `botvision` | — |
| `botweba` | General Bots web application builder |
| `botwhatsapp` | — |
| `botworkspaces` | — |

## Build and test commands that work

```bash
cargo build -p botserver                    # the server binary
cargo build -p botserver --no-default-features --features minimal
cargo test  -p botcore                      # any library crate
cargo test  -p botserver --bin botserver    # binary-crate unit tests
cargo test  -p bottest                      # cross-crate integration
cargo check -p <crate>                      # syntax/type gate
```

Never claim a workspace crate is a module of the `botserver` binary: a crate is
reached through `dep:<crate>` in a feature and mounted by a host module.
