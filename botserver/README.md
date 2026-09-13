<p align="center"><img src="icons/botserver-banner.svg" alt="botserver — the General Bots platform core" width="720"></p>

# botserver

**The platform core of General Bots — multi-agent AI orchestration, agentic workflows and the BASIC (Rhai) runtime that drives 80 applications.**

`botserver` is where autonomous agents actually run. It owns the LLM orchestration layer, the BASIC scripting engine, the Drive compiler, the suite app registry, and the API catalog that lets a model act on real data instead of just talking about it.

[Website](https://generalbots.org) · [Documentation](https://docs.generalbots.org) · [BotBook](../botbook) · [Features](https://generalbots.org/features/)

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-6.3.1-informational.svg)](https://github.com/generalbots/generalbots/releases)
[![Crates](https://img.shields.io/badge/crates-113-blueviolet.svg)](./crates)
[![API](https://img.shields.io/badge/API-localhost%3A8080-informational.svg)](#getting-started)
[![Contributors](https://img.shields.io/github/contributors/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/graphs/contributors)
[![Issues](https://img.shields.io/github/issues/generalbots/generalbots)](https://github.com/generalbots/generalbots/issues)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/generalbots/generalbots/pulls)
[![Runs offline](https://img.shields.io/badge/runs-100%25%20offline-success.svg)](#what-it-does)
[![Multi-agent AI](https://img.shields.io/badge/multi--agent%20AI-7c3aed.svg)](#what-it-does)

**Version:** 6.3.1 · **Rust:** edition 2021 (stable) · **API:** `http://localhost:8080`

---

## What it does

- **Multi-agent orchestration.** Run several bots with distinct personalities, tools and knowledge bases, each with its own session, memory and Drive.
- **Agentic AI workflows.** Autonomous tasks that plan, call tools, and report results — triggered from chat, WhatsApp, Teams, Telegram or a schedule.
- **LLM orchestration across providers.** DeepSeek, Qwen, GLM, Kimi, MiniMax, Yi, Step, Doubao, OpenAI, Anthropic, or any OpenAI-compatible endpoint — including a local llama.cpp server. Prompt versioning, failover and cost routing are handled for you.
- **Advanced RAG.** Agentic RAG, Graph RAG, persistent memory and multi-modal retrieval over PDFs, Word, Excel and web pages, backed by Qdrant.
- **Low-code bot definition.** Bots are directories of BASIC scripts and documents, hot-reloaded from Drive. No recompilation, no redeployment.
- **Enterprise controls.** RBAC, multi-tenant workspace isolation, audit logging, CSRF and rate limiting, and a sanitised error surface.

---

## Applications this server drives

Each application is backed by a crate in [`crates/`](./crates) and a UI under `botui/ui/suite/`. The icons match the menu on [generalbots.org](https://generalbots.org) and are shared with the root README from [`../.github/svg/`](../.github/svg).

|  |  |  |  |
|:---:|:---:|:---:|:---:|
| <img src="../.github/svg/app-chat.svg" width="24"><br>Chat | <img src="../.github/svg/app-mail.svg" width="24"><br>Mail | <img src="../.github/svg/app-calendar.svg" width="24"><br>Calendar | <img src="../.github/svg/app-drive.svg" width="24"><br>Drive |
| <img src="../.github/svg/app-crm.svg" width="24"><br>CRM | <img src="../.github/svg/app-tasks.svg" width="24"><br>Tasks | <img src="../.github/svg/app-meet.svg" width="24"><br>Meet | <img src="../.github/svg/app-docs.svg" width="24"><br>Documents |
| <img src="../.github/svg/app-sheet.svg" width="24"><br>Sheets | <img src="../.github/svg/app-slides.svg" width="24"><br>Slides | <img src="../.github/svg/app-search.svg" width="24"><br>Search | <img src="../.github/svg/app-learn.svg" width="24"><br>Knowledge |
| <img src="../.github/svg/app-automation.svg" width="24"><br>Automation | <img src="../.github/svg/app-monitoring.svg" width="24"><br>Monitoring | <img src="../.github/svg/app-security.svg" width="24"><br>Security | <img src="../.github/svg/app-billing.svg" width="24"><br>Billing |
| <img src="../.github/svg/app-database.svg" width="24"><br>Database | <img src="../.github/svg/app-workspace.svg" width="24"><br>Workspace | | |

---

## Architecture

```
WebSocket / REST / WhatsApp / Teams / Telegram
                    │
                    ▼
        main_module/ws/handler.rs        ← session, rate limits
                    │
                    ▼
        start.bas  (once per session)    ← suggestions, bot memory, context
                    │
        ┌───────────┴────────────┐
        ▼                        ▼
  message_type = 6          everything else
  TOOL_EXEC                 USE KB → RAG → LLM
  runs .ast directly        streamed response
        │                        │
        └───────────┬────────────┘
                    ▼
              response → client
```

Message types drive the routing, so tools can bypass the model entirely:

| ID | Name | LLM used |
|----|------|----------|
| 1 | `USER` | Yes — knowledge-base injection, then generation |
| 2 | `BOT_RESPONSE` | No |
| 4 | `SUGGESTION` | Quick-reply buttons |
| 6 | `TOOL_EXEC` | No — the `.ast` runs directly via Rhai |

### Layout

```
src/
├── main.rs          entry point
├── main_module/     bootstrap, HTTP routes, WebSocket, drive monitors
├── core/            bot pipeline, LLM orchestration, shared types
├── basic/           Rhai BASIC interpreter and keyword implementations
├── security/        SafeCommand, ErrorSanitizer, sql_guard
├── apps/            suite app registry and the LLM automation surface
└── <feature>/       thin shims re-exporting the matching crate
                     (e.g. src/drive/mod.rs -> pub use botdrive::*;)

crates/             113 domain crates: botcrm, botcalendar, botdrive,
                    botlearn, botcloud, botvibe, botsecurity-auth, ...
migrations/         Diesel migrations
```

Business logic belongs in `crates/`. The `src/<feature>/` directories exist only to re-export, which is why most of them are one line.

---

## Getting Started

The supported path is the repository root script — it builds botserver and botui together and lets BotServer provision its own stack (PostgreSQL, Vault, MinIO, Valkey, Zitadel, Qdrant, llama.cpp):

```bash
./restart.sh        # from the repository root
```

To run just the server:

```bash
cd botserver
cargo run -- --noconsole
```

Command-line options:

```bash
cargo run                     # console UI + web server
cargo run -- --noconsole      # background service
cargo run -- --desktop        # desktop application (Tauri)
cargo run -- --tenant <name>  # select a tenant
cargo run -- --container      # LXC container mode
```

Health check: `curl http://localhost:8080/health`

### Configuration

Only `VAULT_*` variables belong in `.env` — everything else (database, Drive, cache, directory, LLM and per-bot settings) is read from Vault at boot. Per-bot LLM configuration lives at `secret/gbo/{org_id}/{branch_id}/{bot_id}` and falls back to `secret/gbo/llm`.

Never hardcode a credential; never commit one.

---

## Engineering Standards

This service handles many concurrent sessions, so a panic is never acceptable — it aborts the process and drops every open connection.

- No `unwrap()` or `expect()` outside tests. Propagate with `?` or handle locally with `log::error!` and a safe default.
- No `panic!()`, `todo!()`, `unimplemented!()`, or `#[allow(...)]`. Fix the code.
- Commands go through `SafeCommand`; SQL identifiers through `sql_guard`; HTTP errors through `log_and_sanitize`.
- Use `rustfmt` defaults and keep `cargo clippy --workspace` clean.
- Files stay under 450 lines — split at 350.
- No CDN assets. Everything is served locally.

The full rule set, security directives and testing workflow live in **[AGENTS.md](../AGENTS.md)**.

---

## Documentation

- **[docs.generalbots.org](https://docs.generalbots.org)** — published documentation
- **[BotBook](../botbook)** — guides, tutorials and the BASIC keyword reference
- **[generalbots.org/features](https://generalbots.org/features/)** — the application catalogue

## Contributors

Thanks to everyone who has contributed to General Bots since 2017.

<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

## License

MIT License — see [LICENSE](./LICENSE).

<p align="center"><sub>Part of <a href="https://generalbots.org">General Bots</a> · built in Rust</sub></p>
