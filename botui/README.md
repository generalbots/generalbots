# BotUI - General Bots Web Interface

<p align="center"><img src="../logo.svg" alt="General Bots" width="200"></p>

**Version:** 6.3.1  
**Purpose:** Web UI server for General Bots (Axum + HTMX + CSS)

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-6.3.1-informational.svg)](https://github.com/generalbots/generalbots/releases)
[![UI](https://img.shields.io/badge/UI-HTMX-3366cc.svg)](https://htmx.org/)
[![Contributors](https://img.shields.io/github/contributors/generalbots/generalbots?style=flat)](https://github.com/generalbots/generalbots/graphs/contributors)
[![Issues](https://img.shields.io/github/issues/generalbots/generalbots)](https://github.com/generalbots/generalbots/issues)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/generalbots/generalbots/pulls)
[![Part of General Bots](https://img.shields.io/badge/part%20of-General%20Bots-7c3aed.svg)](https://generalbots.org)
<a href="https://github.com/generalbots/generalbots/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/generalbots" alt="Contributors to General Bots" />
</a>

---

## Overview

BotUI is a modern web interface for General Bots, built with Rust, Axum, and HTMX. It provides a clean, responsive interface for interacting with the General Bots platform, featuring real-time updates via WebSocket connections and a minimalist JavaScript approach powered by HTMX.

The interface supports multiple features including chat, file management, tasks, calendar, analytics, and more - all served through a fast, efficient Rust backend with a focus on server-rendered HTML and minimal client-side JavaScript.

For comprehensive documentation, see **[docs.generalbots.org](https://docs.generalbots.org)** or the **[BotBook](./botbook)** for detailed guides and API references.

---

## Quick Start

```bash
# Development mode - starts Axum server on port 9000
cargo run

# Desktop mode (Tauri) - starts native window
cargo tauri dev
```

### Environment Variables

- `BOTUI_PORT` - Server port (default: 9000)

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use _ prefix for unused variables - DELETE or USE them
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
❌ NEVER use CDN links - all assets must be local
```

---

## ARCHITECTURE

### Dual Modes

| Mode | Command | Description |
|------|---------|-------------|
| Web | `cargo run` | Axum server on port 9000 |
| Desktop | `cargo tauri dev` | Tauri native window |

### Code Organization

```
src/
├── main.rs           # Entry point - mode detection
├── lib.rs            # Feature-gated module exports
├── http_client.rs    # HTTP wrapper for botserver
├── ui_server/
│   └── mod.rs        # Axum router + UI serving
├── desktop/
│   ├── mod.rs        # Desktop module organization
│   ├── drive.rs      # File operations via Tauri
│   └── tray.rs       # System tray
└── shared/
    └── state.rs      # Shared application state

ui/
├── suite/            # Main UI (HTML/CSS/JS)
│   ├── js/vendor/    # Local JS libraries
│   └── css/          # Stylesheets
└── minimal/          # Minimal chat UI
```

---

## HTMX-FIRST FRONTEND

### Core Principle
- **Use HTMX** to minimize JavaScript
- **Server returns HTML fragments**, not JSON
- **Delegate ALL logic** to Rust server

### HTMX Usage

| Use Case | Solution |
|----------|----------|
| Data fetching | `hx-get`, `hx-post` |
| Form submission | `hx-post`, `hx-put` |
| Real-time updates | `hx-ext="ws"` |
| Content swapping | `hx-target`, `hx-swap` |
| Polling | `hx-trigger="every 5s"` |
| Loading states | `hx-indicator` |

### When JS is Required

| Use Case | Why JS Required |
|----------|-----------------|
| Modal show/hide | DOM manipulation |
| Toast notifications | Dynamic element creation |
| Clipboard operations | `navigator.clipboard` API |
| Keyboard shortcuts | `keydown` event handling |
| Complex animations | GSAP or custom |

---

## LOCAL ASSETS ONLY - NO CDN

```
ui/suite/js/vendor/
├── htmx.min.js
├── htmx-ws.js
├── marked.min.js
├── gsap.min.js
└── livekit-client.umd.min.js
```

```html
<!-- ✅ CORRECT -->
<script src="js/vendor/htmx.min.js"></script>

<!-- ❌ WRONG -->
<script src="https://unpkg.com/htmx.org@1.9.10"></script>
```

---

## OFFICIAL ICONS - MANDATORY

**NEVER generate icons with LLM. Use official SVG icons:**

```
ui/suite/assets/icons/
├── gb-logo.svg        # Main GB logo
├── gb-bot.svg         # Bot/assistant
├── gb-analytics.svg   # Analytics
├── gb-calendar.svg    # Calendar
├── gb-chat.svg        # Chat
├── gb-drive.svg       # File storage
├── gb-mail.svg        # Email
├── gb-meet.svg        # Video meetings
├── gb-tasks.svg       # Task management
└── ...
```

All icons use `stroke="currentColor"` for CSS theming.

---

## SECURITY ARCHITECTURE

### Centralized Auth Engine

All authentication is handled by `security-bootstrap.js` which MUST be loaded immediately after HTMX:

```html
<head>
    <!-- 1. HTMX first -->
    <script src="js/vendor/htmx.min.js"></script>
    <script src="js/vendor/htmx-ws.js"></script>
    
    <!-- 2. Security bootstrap immediately after -->
    <script src="js/security-bootstrap.js"></script>
    
    <!-- 3. Other scripts -->
    <script src="js/api-client.js"></script>
</head>
```

### DO NOT Duplicate Auth Logic

```javascript
// ❌ WRONG - Don't add auth headers manually
fetch("/api/data", {
    headers: { "Authorization": "Bearer " + token }
});

// ✅ CORRECT - Let security-bootstrap.js handle it
fetch("/api/data");
```

---

## DESIGN SYSTEM

### Layout Standards

```css
.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}

.main-content {
    display: grid;
    grid-template-columns: 320px 1fr;
    flex: 1;
    overflow: hidden;
}

.list-panel {
    overflow-y: scroll;
    scrollbar-width: auto;
}

.detail-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
```

### Theme Variables Required

```css
[data-theme="your-theme"] {
    --bg: #0a0a0a;
    --surface: #161616;
    --surface-hover: #1e1e1e;
    --border: #2a2a2a;
    --text: #ffffff;
    --text-secondary: #888888;
    --primary: #c5f82a;
    --success: #22c55e;
    --warning: #f59e0b;
    --error: #ef4444;
}
```

---

## CODE PATTERNS

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

### Derive Eq with PartialEq

```rust
#[derive(PartialEq, Eq)]  // ✅ Always both
struct MyStruct { }
```

---

## KEY DEPENDENCIES

| Library | Version | Purpose |
|---------|---------|---------|
| axum | 0.7.5 | Web framework |
| reqwest | 0.12 | HTTP client |
| tokio | 1.41 | Async runtime |
| askama | 0.12 | HTML Templates |

---

## Applications

BotUI serves these applications straight from `botui/ui/suite/`. The icons match the menu on [generalbots.org](https://generalbots.org).

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
- **[BotBook](./botbook)** - Local comprehensive guide
- **[General Bots Repository](https://github.com/generalbots/generalbots)** - Main project repository

---

## REMEMBER

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **HTMX first** - Minimize JS, delegate to server
- **Local assets** - No CDN, all vendor files local
- **No business logic** - All logic in botserver
- **HTML responses** - Server returns fragments, not JSON
- **Version 6.2.0** - do not change without approval