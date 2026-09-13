<p align="center"><img src="../logo.svg" alt="General Bots" width="200"></p>

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
<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

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

Each application is backed by a crate in [`crates/`](./crates) and a UI under `botui/ui/suite/`. The icons match the menu on [generalbots.org](https://generalbots.org); a copy lives in [`icons/`](./icons) so this README stands on its own.

| | Application | What it does |
|---|-------------|--------------|
| <img src="icons/server.svg" width="20"> | **Server**<br><sub>Core Platform</sub> | Web server, WebSocket messaging, REST API and request routing. Serves both the API and the web interface. |
| <img src="icons/auth.svg" width="20"> | **Auth / Identity**<br><sub>Core Platform</sub> | OAuth 2.0, OpenID Connect, JWT validation and session management, with role-based access control on every endpoint. SSO-ready. |
| <img src="icons/database.svg" width="20"> | **Shared / Database**<br><sub>Core Platform</sub> | PostgreSQL with managed schema definitions, shared models and common utilities used across every application. |
| <img src="icons/ai-engine.svg" width="20"> | **AI Engine**<br><sub>Core Platform</sub> | LLM provider orchestration across DeepSeek, Qwen, GLM, Kimi, MiniMax, Yi or any OpenAI-compatible API. Automatic failover, token counting, streaming and cost tracking. |
| <img src="icons/drive.svg" width="20"> | **Drive / Storage**<br><sub>Core Platform</sub> | S3-compatible object storage with upload, download, versioning and automatic indexing for search and RAG. Works with MinIO, AWS S3 and Wasabi. |
| <img src="icons/dashboards.svg" width="20"> | **Dashboards / Analytics**<br><sub>Core Platform</sub> | Real-time dashboards covering system health, business KPIs and custom visualisations. Exportable and embeddable. |
| <img src="icons/ai-search.svg" width="20"> | **AI Search**<br><sub>Capabilities</sub> | Retrieval-augmented generation over PDFs, Word and Excel with sub-second semantic retrieval and cited answers. |
| <img src="icons/bot-factory.svg" width="20"> | **Bot Factory**<br><sub>Capabilities</sub> | Rapid prototyping and multi-bot orchestration. Run many bots with distinct personalities from one dashboard. |
| <img src="icons/broadcast.svg" width="20"> | **Broadcast**<br><sub>Capabilities</sub> | Omnichannel outbound messaging with AI-driven personalisation across WhatsApp, Telegram and SMS. |
| <img src="icons/content-gen.svg" width="20"> | **Content Generation**<br><sub>Capabilities</sub> | Generate on-brand, SEO-optimised content in a consistent brand voice. |
| <img src="icons/apis-in-basic.svg" width="20"> | **APIs in BASIC**<br><sub>Capabilities</sub> | Build REST endpoints using simplified BASIC syntax, for legacy integration and rapid delivery. |
| <img src="icons/llm-tools.svg" width="20"> | **LLM Tools**<br><sub>Capabilities</sub> | Give the model real capabilities: web search, calculation and custom API calls. |
| <img src="icons/talk-to-data.svg" width="20"> | **Talk to Data**<br><sub>Capabilities</sub> | Query SQL databases, Excel files and CSVs in plain language. |
| <img src="icons/training.svg" width="20"> | **Training**<br><sub>Capabilities</sub> | Ingest institutional knowledge from Word, Excel and PDF documents with no coding. |
| <img src="icons/advanced-rag.svg" width="20"> | **Advanced RAG**<br><sub>Capabilities</sub> | Agentic RAG, Graph RAG, persistent memory and multi-modal retrieval over the knowledge base. |
| <img src="icons/calendar.svg" width="20"> | **Calendar**<br><sub>Business & Productivity</sub> | CalDAV integration with event creation, conflict detection and automated reminder workflows. Syncs with Google, Outlook and Apple Calendar. |
| <img src="icons/email.svg" width="20"> | **Email**<br><sub>Business & Productivity</sub> | IMAP/SMTP integration with automatic templating, attachment handling and trigger-based workflows across inbox, starred, sent and scheduled folders. |
| <img src="icons/meet.svg" width="20"> | **Meet / Video**<br><sub>Business & Productivity</sub> | Video conferencing with screen sharing, recording, transcription and auto-generated meeting notes. |
| <img src="icons/documents.svg" width="20"> | **Documents, Sheets & Slides**<br><sub>Business & Productivity</sub> | Office-compatible document processing. Read and generate Word, Excel and PowerPoint files from templates. |
| <img src="icons/channels.svg" width="20"> | **WhatsApp & Teams**<br><sub>Business & Productivity</sub> | WhatsApp Business API and the MS Teams bot framework, with template messages, proactive notifications and rich media. |
| <img src="icons/crm.svg" width="20"> | **People / CRM**<br><sub>Business & Productivity</sub> | Contact management with CRM-style relationship tracking, tagging, deal pipelines and automated follow-up reminders. |
| <img src="icons/knowledge-base.svg" width="20"> | **Knowledge Base**<br><sub>AI & Intelligence</sub> | Automatic document ingestion with chunking, embedding and semantic search. PDFs, Word files and web pages all become queryable. |
| <img src="icons/web-automation.svg" width="20"> | **Web Automation**<br><sub>AI & Intelligence</sub> | Headless browser automation. Scrape sites, fill forms, capture screenshots and trigger workflows when a page changes. |
| <img src="icons/search.svg" width="20"> | **Search**<br><sub>AI & Intelligence</sub> | Full-text and semantic search across everything indexed, combining keyword and vector matching with faceted filtering. |
| <img src="icons/security.svg" width="20"> | **Security**<br><sub>Operations</sub> | Encryption, threat detection, audit logging and compliance reporting, built to LGPD, GDPR and HIPAA expectations. |
| <img src="icons/monitoring.svg" width="20"> | **Monitoring**<br><sub>Operations</sub> | System health monitoring for CPU, memory, disk and network, with alert thresholds, escalation and uptime tracking. |
| <img src="icons/analytics.svg" width="20"> | **Analytics**<br><sub>Operations</sub> | Event tracking, funnel analysis and conversion metrics across every channel. |
| <img src="icons/rbac.svg" width="20"> | **RBAC & Multi-Tenancy**<br><sub>Enterprise</sub> | Fine-grained role-based access control and multi-tenant workspaces with complete data isolation between organisations. |
| <img src="icons/on-premise.svg" width="20"> | **On-Premise Deployment**<br><sub>Enterprise</sub> | Runs entirely inside your own infrastructure with no cloud dependency and no data leaving the network. Air-gapped deployments supported. |
| <img src="icons/compliance.svg" width="20"> | **Compliance (LGPD, GDPR)**<br><sub>Enterprise</sub> | Data subject requests, right to erasure, audit trails, retention policies and anonymisation tooling. |


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


## License

MIT License — see [LICENSE](./LICENSE).

<p align="center"><sub>Part of <a href="https://generalbots.org">General Bots</a> · built in Rust</sub></p>
