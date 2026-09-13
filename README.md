<p align="center"><img src="logo.svg" alt="General Bots" width="200"></p>

# General Bots

**Multi-agent AI platform for autonomous agents, agentic AI orchestration and LLM workflow automation.**

Self-hosted, open source, and built in Rust. General Bots gives you 80 sovereign applications — Chat, CRM, Mail, Drive, Calendar, Meetings, Documents and Advanced RAG — that run as one platform on your own infrastructure. Your cloud, your data, your rules.

[Website](https://generalbots.org) · [Documentation](https://docs.generalbots.org) · [Features](https://generalbots.org/features/) · [Blog](https://generalbots.org/blog/) · [BotBook](./botbook)

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-6.3.1-informational.svg)](https://github.com/generalbots/generalbots/releases)
[![Contributors](https://img.shields.io/github/contributors/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/graphs/contributors)
[![Stars](https://img.shields.io/github/stars/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/stargazers)
[![Forks](https://img.shields.io/github/forks/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/network/members)
[![Issues](https://img.shields.io/github/issues/generalbots/generalbots)](https://github.com/generalbots/generalbots/issues)
[![Last commit](https://img.shields.io/github/last-commit/generalbots/generalbots)](https://github.com/generalbots/generalbots/commits/main)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/generalbots/generalbots/pulls)
[![Self-hosted](https://img.shields.io/badge/self--hosted-yes-success.svg)](#getting-started)
[![Runs offline](https://img.shields.io/badge/runs-100%25%20offline-success.svg)](#why-general-bots)
[![Multi-agent AI](https://img.shields.io/badge/multi--agent%20AI-7c3aed.svg)](#why-general-bots)
[![Data sovereign](https://img.shields.io/badge/data-sovereign-7c3aed.svg)](#why-general-bots)
[![Open source since 2017](https://img.shields.io/badge/open%20source-since%202017-blue.svg)](#license)
<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

---

## Why General Bots

Most "AI agent" tooling stops at a chat window. General Bots is the platform underneath it: an orchestration layer that connects frontier or local LLMs to real business systems, real documents and real workflows.

- **Multi-agent, not multi-chat.** Orchestrate several bots with distinct personalities, tools and knowledge bases from one dashboard.
- **Agentic AI workflows.** Autonomous task execution that plans, calls tools and reports back — in chat or over WhatsApp.
- **Sovereign by default.** Runs entirely offline or air-gapped. No data leaves your network unless you route it out.
- **Model-agnostic.** DeepSeek, Qwen, GLM, Kimi, MiniMax, Yi, Step, Doubao, OpenAI, Anthropic — or any OpenAI-compatible endpoint, including a local llama.cpp server.
- **Advanced RAG.** Agentic RAG, Graph RAG, persistent memory and multi-modal retrieval over PDFs, Word, Excel and web pages.
- **Low-code by design.** Bot behaviour is defined in BASIC dialogs and documents, so domain experts can contribute without a computer science degree.

You write this, and the bot is live:

```basic
' start.bas
USE KB "product-manual"
USE TOOL "create_ticket"

ADD_SUGGESTION "Check my order"
ADD_SUGGESTION "Open a support ticket"

TALK "Hi! I can check orders or open a ticket for you."
```

---

## Applications

Every application is a self-contained module that can be deployed alone or as part of the whole platform. This repository ships **80 suite applications** and **113 backend crates**; the catalogue below is the platform map, and every entry has its own icon in [`.github/svg/`](./.github/svg).

| | Application | What it does |
|---|-------------|--------------|
| <img src=".github/svg/server.svg" width="20"> | **Server**<br><sub>Core Platform</sub> | Web server, WebSocket messaging, REST API and request routing. Serves both the API and the web interface. |
| <img src=".github/svg/auth.svg" width="20"> | **Auth / Identity**<br><sub>Core Platform</sub> | OAuth 2.0, OpenID Connect, JWT validation and session management, with role-based access control on every endpoint. SSO-ready. |
| <img src=".github/svg/database.svg" width="20"> | **Shared / Database**<br><sub>Core Platform</sub> | PostgreSQL with managed schema definitions, shared models and common utilities used across every application. |
| <img src=".github/svg/ai-engine.svg" width="20"> | **AI Engine**<br><sub>Core Platform</sub> | LLM provider orchestration across DeepSeek, Qwen, GLM, Kimi, MiniMax, Yi or any OpenAI-compatible API. Automatic failover, token counting, streaming and cost tracking. |
| <img src=".github/svg/drive.svg" width="20"> | **Drive / Storage**<br><sub>Core Platform</sub> | S3-compatible object storage with upload, download, versioning and automatic indexing for search and RAG. Works with MinIO, AWS S3 and Wasabi. |
| <img src=".github/svg/dashboards.svg" width="20"> | **Dashboards / Analytics**<br><sub>Core Platform</sub> | Real-time dashboards covering system health, business KPIs and custom visualisations. Exportable and embeddable. |
| <img src=".github/svg/ai-search.svg" width="20"> | **AI Search**<br><sub>Capabilities</sub> | Retrieval-augmented generation over PDFs, Word and Excel with sub-second semantic retrieval and cited answers. |
| <img src=".github/svg/bot-factory.svg" width="20"> | **Bot Factory**<br><sub>Capabilities</sub> | Rapid prototyping and multi-bot orchestration. Run many bots with distinct personalities from one dashboard. |
| <img src=".github/svg/broadcast.svg" width="20"> | **Broadcast**<br><sub>Capabilities</sub> | Omnichannel outbound messaging with AI-driven personalisation across WhatsApp, Telegram and SMS. |
| <img src=".github/svg/content-gen.svg" width="20"> | **Content Generation**<br><sub>Capabilities</sub> | Generate on-brand, SEO-optimised content in a consistent brand voice. |
| <img src=".github/svg/apis-in-basic.svg" width="20"> | **APIs in BASIC**<br><sub>Capabilities</sub> | Build REST endpoints using simplified BASIC syntax, for legacy integration and rapid delivery. |
| <img src=".github/svg/llm-tools.svg" width="20"> | **LLM Tools**<br><sub>Capabilities</sub> | Give the model real capabilities: web search, calculation and custom API calls. |
| <img src=".github/svg/talk-to-data.svg" width="20"> | **Talk to Data**<br><sub>Capabilities</sub> | Query SQL databases, Excel files and CSVs in plain language. |
| <img src=".github/svg/training.svg" width="20"> | **Training**<br><sub>Capabilities</sub> | Ingest institutional knowledge from Word, Excel and PDF documents with no coding. |
| <img src=".github/svg/advanced-rag.svg" width="20"> | **Advanced RAG**<br><sub>Capabilities</sub> | Agentic RAG, Graph RAG, persistent memory and multi-modal retrieval over the knowledge base. |
| <img src=".github/svg/calendar.svg" width="20"> | **Calendar**<br><sub>Business & Productivity</sub> | CalDAV integration with event creation, conflict detection and automated reminder workflows. Syncs with Google, Outlook and Apple Calendar. |
| <img src=".github/svg/email.svg" width="20"> | **Email**<br><sub>Business & Productivity</sub> | IMAP/SMTP integration with automatic templating, attachment handling and trigger-based workflows across inbox, starred, sent and scheduled folders. |
| <img src=".github/svg/meet.svg" width="20"> | **Meet / Video**<br><sub>Business & Productivity</sub> | Video conferencing with screen sharing, recording, transcription and auto-generated meeting notes. |
| <img src=".github/svg/documents.svg" width="20"> | **Documents, Sheets & Slides**<br><sub>Business & Productivity</sub> | Office-compatible document processing. Read and generate Word, Excel and PowerPoint files from templates. |
| <img src=".github/svg/channels.svg" width="20"> | **WhatsApp & Teams**<br><sub>Business & Productivity</sub> | WhatsApp Business API and the MS Teams bot framework, with template messages, proactive notifications and rich media. |
| <img src=".github/svg/crm.svg" width="20"> | **People / CRM**<br><sub>Business & Productivity</sub> | Contact management with CRM-style relationship tracking, tagging, deal pipelines and automated follow-up reminders. |
| <img src=".github/svg/knowledge-base.svg" width="20"> | **Knowledge Base**<br><sub>AI & Intelligence</sub> | Automatic document ingestion with chunking, embedding and semantic search. PDFs, Word files and web pages all become queryable. |
| <img src=".github/svg/web-automation.svg" width="20"> | **Web Automation**<br><sub>AI & Intelligence</sub> | Headless browser automation. Scrape sites, fill forms, capture screenshots and trigger workflows when a page changes. |
| <img src=".github/svg/search.svg" width="20"> | **Search**<br><sub>AI & Intelligence</sub> | Full-text and semantic search across everything indexed, combining keyword and vector matching with faceted filtering. |
| <img src=".github/svg/security.svg" width="20"> | **Security**<br><sub>Operations</sub> | Encryption, threat detection, audit logging and compliance reporting, built to LGPD, GDPR and HIPAA expectations. |
| <img src=".github/svg/monitoring.svg" width="20"> | **Monitoring**<br><sub>Operations</sub> | System health monitoring for CPU, memory, disk and network, with alert thresholds, escalation and uptime tracking. |
| <img src=".github/svg/analytics.svg" width="20"> | **Analytics**<br><sub>Operations</sub> | Event tracking, funnel analysis and conversion metrics across every channel. |
| <img src=".github/svg/rbac.svg" width="20"> | **RBAC & Multi-Tenancy**<br><sub>Enterprise</sub> | Fine-grained role-based access control and multi-tenant workspaces with complete data isolation between organisations. |
| <img src=".github/svg/on-premise.svg" width="20"> | **On-Premise Deployment**<br><sub>Enterprise</sub> | Runs entirely inside your own infrastructure with no cloud dependency and no data leaving the network. Air-gapped deployments supported. |
| <img src=".github/svg/compliance.svg" width="20"> | **Compliance (LGPD, GDPR)**<br><sub>Enterprise</sub> | Data subject requests, right to erasure, audit trails, retention policies and anonymisation tooling. |

Beyond the in-browser suite, the same agent layer drives **WhatsApp Business**, **MS Teams**, **Telegram** and **email**.


---

## Architecture

Two Rust services, one workspace.

```
Browser / WhatsApp / Teams / Telegram
        │
        ▼
┌───────────────────────┐        ┌──────────────────────────────┐
│  botui  :3000 /:4000  │ ─────▶ │  botserver  :8080            │
│  suite · cloud · login│  proxy │  API · WebSocket · agents    │
└───────────────────────┘        └──────────────┬───────────────┘
                                                │
                     ┌──────────────────────────┼──────────────────────────┐
                     ▼                          ▼                          ▼
              PostgreSQL + Vault          MinIO (S3)              Qdrant + llama.cpp
              state & secrets             drive & files           vectors & local LLM
```

**botserver** is the platform core: LLM orchestration, the BASIC (Rhai) scripting engine, the drive compiler, the suite app registry and the API catalog that lets the LLM act on your data. Business logic lives in `botserver/crates/` — one crate per domain (`botcrm`, `botcalendar`, `botdrive`, `botlearn`, `botcloud`, …).

**botui** serves the front end on three ports from a single binary: the desktop suite on `3000`, the cloud/SaaS pages on `4000`, and login/signup on `5000`.

A message flows through the system as: WebSocket → session → `start.bas` (once per session) → knowledge-base injection → LLM or direct tool execution → streamed response back over the socket.

### Ports

| Port | Service | What it serves |
|------|---------|----------------|
| 3000 | botui (suite) | Desktop suite and the 80 applications |
| 4000 | botui (cloud) | Store, plans, dashboard, organizations |
| 5000 | botui (login) | Login and signup — the only auth surface |
| 8080 | botserver | REST API and WebSocket |

---

## Getting Started

### Requirements

- **Rust** — a recent stable toolchain from [rustup.rs](https://rustup.rs) (edition 2021)
- **Git**
- **mold** (optional, faster linking) — `sudo apt-get install mold`

### Run it

```bash
git clone https://github.com/generalbots/generalbots
cd generalbots
./restart.sh
```

`restart.sh` stops anything already running, builds botserver and botui in order, and starts both. BotServer then provisions its own stack — PostgreSQL, Vault, MinIO, Valkey, Zitadel, Qdrant and a local llama.cpp server — and reads every credential from Vault. No global database or Redis install is needed or wanted.

Once it is up:

- **Suite** → http://localhost:3000
- **Cloud** → http://localhost:4000
- **Login** → http://localhost:5000/login
- **API** → http://localhost:8080/health

First boot downloads the stack and models, so give it a few minutes. Logs go to `botserver.log` and `botui.log`:

```bash
tail -f botserver.log botui.log
```

### Run just the API

```bash
cd botserver
cargo run -- --noconsole
```

---

## Building Bots

Bots are directories, not code. A bot lives in Drive as `{bot}.gbai/` and is picked up automatically:

```
mybot.gbai/
├── mybot.gbdialog/     # BASIC scripts: start.bas, tables.bas, {tool}.bas
├── mybot.gbkb/         # documents to index for retrieval
├── mybot.gbot/         # bot configuration
└── mybot.gbdrive/      # generated files and reports
```

Four keywords cover most of it:

```basic
USE KB "manual"          ' index documents and retrieve from them
USE TOOL "create_ticket" ' expose a tool to the LLM
TALK "How can I help?"   ' reply to the user
CALL "other_script"      ' run another dialog
```

See the **[BotBook](./botbook)** for the full keyword reference and worked examples.

---

## Repository Layout

| Directory | What it is |
|-----------|------------|
| `botserver/` | Platform core — API, agents, BASIC engine, 113 crates |
| `botui/` | Web front end — suite, cloud and login servers |
| `botapp/` | Tauri desktop wrapper |
| `botlib/` | Shared types and utilities |
| `botbook/` | Documentation source (mdBook) |
| `bottemplates/` | Ready-made bot templates |
| `bottest/` | Cross-crate integration tests |
| `botdevice/` | Device and IoT integrations |
| `botplugin/` | Browser extension |
| `botmodels/` | Data-model tooling |

This is a single repository — there are no git submodules. Push to `origin` for the public mirror; pushing to `alm` triggers the CI/CD pipeline that builds and deploys, so confirm first.

---

## Contributing

Read **[AGENTS.md](./AGENTS.md)** before opening a pull request. It carries the coding rules, security directives and testing workflow the project holds to: no `unwrap()`/`expect()` in production paths, no `#[allow()]` suppressions, no CDN assets, and files kept under 450 lines.

Before adding a `.md` file, search `botbook/` for existing documentation.


---

## License

MIT License — see [LICENSE](./LICENSE). Each subproject carries its own copy (`botserver/LICENSE`, `botui/LICENSE`, `botlib/LICENSE.txt`, …).

General Bots has been developed as open source since 2017.

<p align="center"><sub>Built in Rust · 536,000+ lines · <a href="https://generalbots.org">generalbots.org</a></sub></p>
