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

Every application is a self-contained module that can be deployed alone or as part of the whole platform. This repository ships **80 suite applications** and **113 backend crates**.

|  |  |  |  |
|:---:|:---:|:---:|:---:|
| <img src=".github/svg/app-chat.svg" width="24"><br>Chat | <img src=".github/svg/app-mail.svg" width="24"><br>Mail | <img src=".github/svg/app-calendar.svg" width="24"><br>Calendar | <img src=".github/svg/app-drive.svg" width="24"><br>Drive |
| <img src=".github/svg/app-crm.svg" width="24"><br>CRM | <img src=".github/svg/app-tasks.svg" width="24"><br>Tasks | <img src=".github/svg/app-meet.svg" width="24"><br>Meet | <img src=".github/svg/app-docs.svg" width="24"><br>Documents |
| <img src=".github/svg/app-sheet.svg" width="24"><br>Sheets | <img src=".github/svg/app-slides.svg" width="24"><br>Slides | <img src=".github/svg/app-search.svg" width="24"><br>Search | <img src=".github/svg/app-learn.svg" width="24"><br>Knowledge |
| <img src=".github/svg/app-automation.svg" width="24"><br>Automation | <img src=".github/svg/app-monitoring.svg" width="24"><br>Monitoring | <img src=".github/svg/app-security.svg" width="24"><br>Security | <img src=".github/svg/app-billing.svg" width="24"><br>Billing |

Each icon above links to a real application directory under `botui/ui/suite/`. The icons live in [`.github/svg/`](./.github/svg) and follow the same 24×24 stroke style as the menu on [generalbots.org](https://generalbots.org).

### Grouped by purpose

| Group | Applications |
|-------|--------------|
| **Conversation** | `chat`, `concierge`, `attendant`, `handoff`, `tickets`, `email`, `mail` |
| **CRM & Sales** | `crm`, `people`, `sales`, `campaigns`, `marketing`, `retail`, `products`, `pos` |
| **Office & Documents** | `paper`, `sheet`, `slides`, `docs`, `notepad`, `notes`, `templates` |
| **Productivity** | `calendar`, `tasks`, `goals`, `lists`, `project`, `clock`, `timer`, `timeclock` |
| **Knowledge & AI** | `learn`, `research`, `memory`, `search`, `vision`, `browser`, `vibe`, `designer`, `canvas` |
| **Operations** | `drive`, `database`, `monitoring`, `analytics`, `dashboards`, `maintenance`, `audit` |
| **Trust & Compliance** | `compliance`, `governance`, `kyc`, `fraud`, `biometry`, `settings`, `admin` |
| **Finance** | `billing`, `banking`, `tax`, `inventory` |
| **Communication** | `meet`, `video`, `minutes`, `social`, `integrations` |
| **Platform** | `workspace`, `plugins`, `terminal`, `tools`, `store`, `plan` |

Beyond the in-browser suite, the same platform drives **WhatsApp Business**, **MS Teams**, **Telegram** and **email** through the same agent layer.

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

### Contributors

General Bots has been built in the open since 2017. Thanks to everyone who has contributed.

<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

---

## License

MIT License — see [LICENSE](./LICENSE). Each subproject carries its own copy (`botserver/LICENSE`, `botui/LICENSE`, `botlib/LICENSE.txt`, …).

General Bots has been developed as open source since 2017.

<p align="center"><sub>Built in Rust · 536,000+ lines · <a href="https://generalbots.org">generalbots.org</a></sub></p>
