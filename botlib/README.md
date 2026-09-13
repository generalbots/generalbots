# BotLib - General Bots Shared Library

<p align="center"><img src="../logo.svg" alt="General Bots" width="200"></p>

**Version:** 6.3.1  
**Purpose:** Shared library for General Bots workspace

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.txt)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-6.3.1-informational.svg)](https://github.com/generalbots/generalbots/releases)
[![Kind](https://img.shields.io/badge/kind-shared%20library-6f42c1.svg)](./src)
[![Contributors](https://img.shields.io/github/contributors/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/graphs/contributors)
[![Issues](https://img.shields.io/github/issues/generalbots/generalbots)](https://github.com/generalbots/generalbots/issues)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/generalbots/generalbots/pulls)
[![Part of General Bots](https://img.shields.io/badge/part%20of-General%20Bots-7c3aed.svg)](https://generalbots.org)
<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

---

## Overview

BotLib is the foundational shared library for the General Bots workspace, providing common types, error handling, HTTP client functionality, and utilities used across all projects. It serves as the core dependency for botserver, botui, botapp, and other workspace members, ensuring consistency and reducing code duplication.

For comprehensive documentation, see **[docs.generalbots.org](https://docs.generalbots.org)** or the **[BotBook](../botbook)** for detailed guides and API references.

---

## Module Structure

```
src/
├── lib.rs           # Public exports, feature gates
├── error.rs         # Error types (thiserror)
├── models.rs        # Shared data models
├── message_types.rs # Message type definitions
├── http_client.rs   # HTTP client wrapper (feature-gated)
├── branding.rs      # Version, branding constants
└── version.rs       # Version information
```

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

### Absolute Prohibitions

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use _ prefix for unused variables - DELETE or USE them
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
```

---

## Key Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| anyhow | 1.0 | Error handling |
| thiserror | 2.0 | Error derive |
| chrono | 0.4 | Date/time |
| serde | 1.0 | Serialization |
| uuid | 1.11 | UUIDs |
| diesel | 2.1 | Database ORM |
| reqwest | 0.12 | HTTP client |

---

## Features

### Feature Gates

BotLib uses Cargo features to enable optional functionality:

```toml
[features]
default = []
http-client = ["reqwest"]  # Enable HTTP client
# Add more features as needed
```

### Using Features

```toml
# In dependent crate's Cargo.toml
[dependencies.botlib]
workspace = true
features = ["http-client"]  # Enable HTTP client
```

---

## Mandatory Code Patterns

### Error Handling

```rust
// ❌ WRONG
let value = something.unwrap();

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
```

### Self Usage

```rust
impl MyStruct {
    fn new() -> Self { Self { } }  // ✅ Not MyStruct
}
```

### Format Strings

```rust
format!("Hello {name}")  // ✅ Not format!("{}", name)
```

### Display vs ToString

```rust
// ❌ WRONG
impl ToString for MyType { }

// ✅ CORRECT
impl std::fmt::Display for MyType { }
```

### Derive Eq with PartialEq

```rust
#[derive(PartialEq, Eq)]  // ✅ Always both
struct MyStruct { }
```

---

## Platform applications

BotLib is the shared foundation every application below is built on: common types, error handling and utilities. Icons match the menu on [generalbots.org](https://generalbots.org).

| | Application | What it does |
|---|-------------|--------------|
| <img src="../.github/svg/server.svg" width="20"> | **Server**<br><sub>Core Platform</sub> | Web server, WebSocket messaging, REST API and request routing. Serves both the API and the web interface. |
| <img src="../.github/svg/auth.svg" width="20"> | **Auth / Identity**<br><sub>Core Platform</sub> | OAuth 2.0, OpenID Connect, JWT validation and session management, with role-based access control on every endpoint. SSO-ready. |
| <img src="../.github/svg/database.svg" width="20"> | **Shared / Database**<br><sub>Core Platform</sub> | PostgreSQL with managed schema definitions, shared models and common utilities used across every application. |
| <img src="../.github/svg/ai-engine.svg" width="20"> | **AI Engine**<br><sub>Core Platform</sub> | LLM provider orchestration across DeepSeek, Qwen, GLM, Kimi, MiniMax, Yi or any OpenAI-compatible API. Automatic failover, token counting, streaming and cost tracking. |
| <img src="../.github/svg/drive.svg" width="20"> | **Drive / Storage**<br><sub>Core Platform</sub> | S3-compatible object storage with upload, download, versioning and automatic indexing for search and RAG. Works with MinIO, AWS S3 and Wasabi. |
| <img src="../.github/svg/dashboards.svg" width="20"> | **Dashboards / Analytics**<br><sub>Core Platform</sub> | Real-time dashboards covering system health, business KPIs and custom visualisations. Exportable and embeddable. |
| <img src="../.github/svg/ai-search.svg" width="20"> | **AI Search**<br><sub>Capabilities</sub> | Retrieval-augmented generation over PDFs, Word and Excel with sub-second semantic retrieval and cited answers. |
| <img src="../.github/svg/bot-factory.svg" width="20"> | **Bot Factory**<br><sub>Capabilities</sub> | Rapid prototyping and multi-bot orchestration. Run many bots with distinct personalities from one dashboard. |
| <img src="../.github/svg/broadcast.svg" width="20"> | **Broadcast**<br><sub>Capabilities</sub> | Omnichannel outbound messaging with AI-driven personalisation across WhatsApp, Telegram and SMS. |
| <img src="../.github/svg/content-gen.svg" width="20"> | **Content Generation**<br><sub>Capabilities</sub> | Generate on-brand, SEO-optimised content in a consistent brand voice. |
| <img src="../.github/svg/apis-in-basic.svg" width="20"> | **APIs in BASIC**<br><sub>Capabilities</sub> | Build REST endpoints using simplified BASIC syntax, for legacy integration and rapid delivery. |
| <img src="../.github/svg/llm-tools.svg" width="20"> | **LLM Tools**<br><sub>Capabilities</sub> | Give the model real capabilities: web search, calculation and custom API calls. |
| <img src="../.github/svg/talk-to-data.svg" width="20"> | **Talk to Data**<br><sub>Capabilities</sub> | Query SQL databases, Excel files and CSVs in plain language. |
| <img src="../.github/svg/training.svg" width="20"> | **Training**<br><sub>Capabilities</sub> | Ingest institutional knowledge from Word, Excel and PDF documents with no coding. |
| <img src="../.github/svg/advanced-rag.svg" width="20"> | **Advanced RAG**<br><sub>Capabilities</sub> | Agentic RAG, Graph RAG, persistent memory and multi-modal retrieval over the knowledge base. |
| <img src="../.github/svg/calendar.svg" width="20"> | **Calendar**<br><sub>Business & Productivity</sub> | CalDAV integration with event creation, conflict detection and automated reminder workflows. Syncs with Google, Outlook and Apple Calendar. |
| <img src="../.github/svg/email.svg" width="20"> | **Email**<br><sub>Business & Productivity</sub> | IMAP/SMTP integration with automatic templating, attachment handling and trigger-based workflows across inbox, starred, sent and scheduled folders. |
| <img src="../.github/svg/meet.svg" width="20"> | **Meet / Video**<br><sub>Business & Productivity</sub> | Video conferencing with screen sharing, recording, transcription and auto-generated meeting notes. |
| <img src="../.github/svg/documents.svg" width="20"> | **Documents, Sheets & Slides**<br><sub>Business & Productivity</sub> | Office-compatible document processing. Read and generate Word, Excel and PowerPoint files from templates. |
| <img src="../.github/svg/channels.svg" width="20"> | **WhatsApp & Teams**<br><sub>Business & Productivity</sub> | WhatsApp Business API and the MS Teams bot framework, with template messages, proactive notifications and rich media. |
| <img src="../.github/svg/crm.svg" width="20"> | **People / CRM**<br><sub>Business & Productivity</sub> | Contact management with CRM-style relationship tracking, tagging, deal pipelines and automated follow-up reminders. |
| <img src="../.github/svg/knowledge-base.svg" width="20"> | **Knowledge Base**<br><sub>AI & Intelligence</sub> | Automatic document ingestion with chunking, embedding and semantic search. PDFs, Word files and web pages all become queryable. |
| <img src="../.github/svg/web-automation.svg" width="20"> | **Web Automation**<br><sub>AI & Intelligence</sub> | Headless browser automation. Scrape sites, fill forms, capture screenshots and trigger workflows when a page changes. |
| <img src="../.github/svg/search.svg" width="20"> | **Search**<br><sub>AI & Intelligence</sub> | Full-text and semantic search across everything indexed, combining keyword and vector matching with faceted filtering. |
| <img src="../.github/svg/security.svg" width="20"> | **Security**<br><sub>Operations</sub> | Encryption, threat detection, audit logging and compliance reporting, built to LGPD, GDPR and HIPAA expectations. |
| <img src="../.github/svg/monitoring.svg" width="20"> | **Monitoring**<br><sub>Operations</sub> | System health monitoring for CPU, memory, disk and network, with alert thresholds, escalation and uptime tracking. |
| <img src="../.github/svg/analytics.svg" width="20"> | **Analytics**<br><sub>Operations</sub> | Event tracking, funnel analysis and conversion metrics across every channel. |
| <img src="../.github/svg/rbac.svg" width="20"> | **RBAC & Multi-Tenancy**<br><sub>Enterprise</sub> | Fine-grained role-based access control and multi-tenant workspaces with complete data isolation between organisations. |
| <img src="../.github/svg/on-premise.svg" width="20"> | **On-Premise Deployment**<br><sub>Enterprise</sub> | Runs entirely inside your own infrastructure with no cloud dependency and no data leaving the network. Air-gapped deployments supported. |
| <img src="../.github/svg/compliance.svg" width="20"> | **Compliance (LGPD, GDPR)**<br><sub>Enterprise</sub> | Data subject requests, right to erasure, audit trails, retention policies and anonymisation tooling. |

---

## Documentation

For complete documentation, guides, and API references:

- **[docs.generalbots.org](https://docs.generalbots.org)** - Full online documentation
- **[BotBook](../botbook)** - Local comprehensive guide with tutorials and examples
- **[General Bots Repository](https://github.com/generalbots/generalbots)** - Main project repository

---

## Related Projects

- **[botserver](https://github.com/GeneralBots/botserver)** - Main API server
- **[botui](https://github.com/GeneralBots/botui)** - Web UI interface
- **[botapp](https://github.com/GeneralBots/botapp)** - Desktop application
- **[botbook](https://github.com/GeneralBots/botbook)** - Documentation

---

## Remember

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **INLINE FORMAT ARGS** - `format!("{name}")` not `format!("{}", name)`
- **USE SELF** - In impl blocks, use Self not the type name
- **DERIVE EQ** - Always derive Eq with PartialEq
- **DISPLAY NOT TOSTRING** - Implement Display, not ToString
- **Version 6.2.0** - Do not change without approval
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## License

MIT - See [LICENSE](LICENSE) for details.