# General Bots Documentation (BotBook)

**Version:** 6.2.0  
**Purpose:** Comprehensive documentation for General Bots (mdBook format)

![General Bots Logo](https://github.com/GeneralBots/botserver/blob/main/logo.png?raw=true)

---

## Overview

BotBook is the official documentation repository for General Bots, built using [mdBook](https://rust-lang.github.io/mdBook/). It provides comprehensive guides, API references, tutorials, and architectural documentation for the entire General Bots platform - an enterprise-grade LLM orchestrator and AI automation platform.

For the latest live documentation, visit **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)**.

---

## 🏗️ Architecture

<img src="src/assets/platform-architecture.svg" alt="General Bots Platform Architecture" style="max-width: 100%; height: auto;">

---

## 📦 General Bots Repositories

| Repository | Description | Status |
|------------|-------------|--------|
| [**botserver**](https://github.com/GeneralBots/botserver) | Core API server - LLM orchestration, automation, integrations | ✅ Production |
| [**botui**](https://github.com/GeneralBots/botui) | Pure web UI - HTMX-based interface (suite & minimal) | ✅ Production |
| [**botapp**](https://github.com/GeneralBots/botapp) | Tauri desktop wrapper - native file access, system tray | ✅ Production |
| [**botlib**](https://github.com/GeneralBots/botlib) | Shared Rust library - common types, HTTP client, utilities | ✅ Production |
| [**bottemplates**](https://github.com/GeneralBots/bottemplates) | Templates - bots, apps, prompts, UI components | ✅ Production |
| [**botbook**](https://github.com/GeneralBots/botbook) | Documentation - mdBook format, multi-language | ✅ Production |

---

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **Git** - [Download from git-scm.com](https://git-scm.com/downloads)
- **mdBook** - `cargo install mdbook`

### Run the Server

```bash
# Clone and run
git clone https://github.com/GeneralBots/botserver
cd botserver
cargo run
```

On first run, botserver automatically:
- Installs required components (PostgreSQL, S3 storage, Cache, LLM)
- Sets up database with migrations
- Downloads AI models
- Starts HTTP server at `http://127.0.0.1:8080`

### Run the Desktop App

```bash
# Clone botui (pure web)
git clone https://github.com/GeneralBots/botui
cd botui
cargo run  # Starts web server at :3000

# In another terminal, clone and run botapp (Tauri desktop)
git clone https://github.com/GeneralBots/botapp
cd botapp
cargo tauri dev
```

### Build Documentation

```bash
# Clone botbook
git clone https://github.com/GeneralBots/botbook
cd botbook

# Build documentation
mdbook build

# Serve locally with hot reload
mdbook serve --open
```

---

## ✨ Key Features

### 🤖 Multi-Vendor LLM API
Unified interface for OpenAI, Groq, Claude, Anthropic, and local models.

### 🔧 MCP + LLM Tools Generation
Instant tool creation from code and functions - no complex configurations.

### 💾 Semantic Caching
Intelligent response caching achieving **70% cost reduction** on LLM calls.

### 🌐 Web Automation Engine
Browser automation combined with AI intelligence for complex workflows.

### 📊 Enterprise Data Connectors
Native integrations with CRM, ERP, databases, and external services.

### 🔄 Git-like Version Control
Full history with rollback capabilities for all configurations and data.

---

## 🎯 4 Essential Keywords

General Bots provides a minimal, focused system:

```basic
USE KB "knowledge-base"    ' Load knowledge base into vector database
CLEAR KB "knowledge-base"  ' Remove KB from session
USE TOOL "tool-name"       ' Make tool available to LLM
CLEAR TOOLS                ' Remove all tools from session
```

---

## 📁 Documentation Structure

```
botbook/
├── book.toml          # mdBook configuration
├── src/
│   ├── SUMMARY.md     # Table of contents
│   ├── README.md      # Introduction
│   ├── 01-introduction/   # Quick Start
│   ├── 02-templates/      # Package System
│   ├── 03-knowledge-base/ # Knowledge Base
│   ├── 04-gbui/          # UI Interface
│   ├── 06-gbdialog/       # BASIC Dialogs
│   ├── 08-config/         # Configuration
│   ├── 10-rest/           # REST API
│   ├── 12-auth/           # Authentication
│   └── assets/            # Images, diagrams
├── i18n/              # Translations
└── book/              # Generated output
```

---

## 📚 Documentation Writing Guidelines

### ✅ Keyword Naming Rules - MANDATORY

**Keywords NEVER use underscores. Always use spaces.**

| Write This | NOT This |
|------------|----------|
| `SEND MAIL` | `SEND_MAIL` |
| `GENERATE PDF` | `GENERATE_PDF` |
| `MERGE PDF` | `MERGE_PDF` |
| `DELETE` | `DELETE_HTTP` |
| `SET HEADER` | `SET_HEADER` |
| `FOR EACH` | `FOR_EACH` |

#### Correct Syntax Examples
```basic
SEND MAIL to, subject, body, attachments
GENERATE PDF template, data, output
MERGE PDF files, output
DELETE "url"
ON ERROR RESUME NEXT
SET BOT MEMORY key, value
KB STATISTICS
```

#### ❌ NEVER Use Underscores
```basic
SEND_MAIL          ' WRONG!
GENERATE_PDF       ' WRONG!
DELETE_HTTP        ' WRONG!
```

---

### 🎨 Official Icons - MANDATORY

**NEVER generate icons with LLM. Use official SVG icons from `botui/ui/suite/assets/icons/`**

#### Usage in Documentation
```markdown
<!-- Reference icons in docs -->
![Chat](../assets/icons/gb-chat.svg)

<!-- With HTML for sizing -->
<img src="../assets/icons/gb-analytics.svg" alt="Analytics" width="24">
```

#### Required Icons
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

### 🚫 NO ASCII Diagramrams - MANDATORY

**NEVER use ASCII art diagrams. ALL diagrams must be SVG.**

#### ❌ Prohibited ASCII Patterns
```
┌─────────┐    ╔═══════╗    +-------+
│  Box    │    ║ Box   ║    | Box   |
└─────────┘    ╚═══════╝    +-------+
```

#### ✅ What to Use Instead

| Instead of... | Use... |
|---------------|--------|
| ASCII box diagrams | SVG diagrams in `assets/` |
| ASCII flow charts | SVG with arrows and boxes |
| ASCII directory trees | Markdown tables |

---

### 🎨 SVG Diagram Guidelines

All SVGs must support light/dark modes:

```xml
<style>
  .title-text { fill: #1E1B4B; }
  .main-text { fill: #334155; }
  
  @media (prefers-color-scheme: dark) {
    .title-text { fill: #F1F5F9; }
    .main-text { fill: #E2E8F0; }
  }
</style>
```

---

### 💬 Conversation Examples

Use WhatsApp-style HTML format for bot interactions:

```html
<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Hello! How can I help?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to enroll</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>
```

---

### 📋 Source Code References

| Topic | Source Location |
|-------|-----------------|
| BASIC Keywords | `botserver/src/basic/keywords/` |
| Database Models | `botserver/src/shared/models.rs` |
| API Routes | `botserver/src/core/urls.rs` |
| Configuration | `botserver/src/core/config/` |
| Templates | `botserver/templates/` |

---

### 📖 Documentation Accuracy Rules

```
- All documentation MUST match actual source code
- Extract real keywords from botserver/src/basic/keywords/
- Use actual examples from botserver/templates/
- Version numbers must be 6.2.0
- No placeholder content - only verified features
```

---

## 🏛️ Architecture Details

### botserver (Core)
The main API server handling:
- LLM orchestration and prompt management
- Multi-channel communication (WhatsApp, Teams, Email, Web)
- File storage and drive management
- Task scheduling and automation
- Authentication and authorization

### botui (Web Interface)
Pure web UI with zero native dependencies:
- **Suite**: Full-featured multi-app interface
- **Minimal**: Lightweight single-page chat
- HTMX-powered for minimal JavaScript
- Works in any browser

### botapp (Desktop)
Tauri wrapper adding native capabilities:
- Local file system access
- System tray integration
- Native dialogs and notifications
- Desktop-specific features

### botlib (Shared Library)
Common Rust code shared across projects:
- HTTP client for botserver communication
- Shared types and models
- Branding and configuration utilities
- Error handling

---

## 🛡️ Security

- **AGPL-3.0 License** - True open source with contribution requirements
- **Self-hosted** - Your data stays on your infrastructure
- **Enterprise-grade** - 5+ years of stability
- **No vendor lock-in** - Open protocols and standards

Report security issues to: **security@pragmatismo.com.br**

---

## 🆚 Why General Bots?

| vs. Alternative | General Bots Advantage |
|-----------------|----------------------|
| **ChatGPT/Claude** | Automates entire business processes, not just chat |
| **n8n/Make** | Simpler approach with minimal programming |
| **Microsoft 365** | User control, not locked ecosystems |
| **Salesforce** | Open-source AI orchestration connecting all systems |

---

## 🔗 Links

- **Website:** [pragmatismo.com.br](https://pragmatismo.com.br)
- **Documentation:** [docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)
- **BotBook:** [Complete Documentation](https://github.com/GeneralBots/botbook)
- **Quick Start:** [Get Started in Minutes](https://github.com/GeneralBots/botserver/blob/main/docs/QUICK_START.md)
- **API Reference:** [REST API Documentation](https://github.com/GeneralBots/botserver/blob/main/docs/src/chapter-10-api/README.md)
- **Architecture:** [System Architecture Guide](https://github.com/GeneralBots/botserver/blob/main/docs/src/chapter-07-gbapp/README.md)
- **Stack Overflow:** Tag questions with `generalbots`
- **Video Tutorial:** [7 AI General Bots LLM Templates](https://www.youtube.com/watch?v=KJgvUPXi3Fw)

---

## 🤝 Contributing

We welcome contributions! See our [Contributing Guidelines](https://github.com/GeneralBots/botserver/blob/main/docs/src/chapter-13-community/README.md).

### Contributors

<a href="https://github.com/generalbots/botserver/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/botserver" />
</a>

---

## 🔑 Remember

- **Accuracy** - Must match botserver source code
- **Completeness** - No placeholder sections
- **Clarity** - Accessible to BASIC enthusiasts
- **Keywords** - NEVER use underscores - always spaces
- **NO ASCII art** - Use SVG diagrams only
- **Official icons** - Use icons from botui/ui/suite/assets/icons/
- **Version 6.2.0** - Always reference 6.2.0
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## 📄 License

General Bots is licensed under **AGPL-3.0**.

According to our dual licensing model, this program can be used either under the terms of the GNU Affero General Public License, version 3, or under a proprietary license.

Copyright (c) pragmatismo.com.br. All rights reserved.

---

> **Code Name:** [Guaribas](https://en.wikipedia.org/wiki/Guaribas) (a city in Brazil, state of Piauí)
>
> *"No one should have to do work that can be done by a machine."* - Roberto Mangabeira Unger