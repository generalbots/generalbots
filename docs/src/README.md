# General Bots Documentation

Welcome to the **General Bots** documentation. This guide explains how to install, configure, extend, and deploy conversational AI bots using General Bots' template-based package system and BASIC scripting language.

---

## About This Documentation

This documentation has been **recently updated** to accurately reflect the actual implementation of General Bots version 6.0.8. The following sections are now accurate:

✅ **Accurate Documentation:**
- Chapter 02: Package system (template-based `.gbai` structure)
- Chapter 06: Rust architecture (single-crate structure, module overview)
- Chapter 09: Core features
- Introduction: Architecture and capabilities

⚠️ **Partial Documentation:**
- Chapter 05: BASIC keywords (examples exist, full reference needs expansion)
- Chapter 08: Tool integration (concepts documented, implementation details needed)
- Chapter 11: Authentication (implemented but needs detail)

📝 **Needs Documentation:**
- UI module (`src/ui/`)
- UI tree module (`src/ui_tree/`)
- Riot compiler module (`src/riot_compiler/`)
- Prompt manager (`src/prompt_manager/`)
- API endpoints and UI server routes
- Drive (S3-compatible) integration details
- Video conferencing (LiveKit) integration

---

## What is General Bots?

General Bots is an open-source conversational AI platform written in Rust. It enables users to create intelligent chatbots using:

- **BASIC Scripting**: Simple `.bas` scripts for conversation flows
- **Template Packages**: Organize bots as `.gbai` directories with dialogs, knowledge bases, and configuration
- **Vector Search**: Semantic document retrieval with Qdrant
- **LLM Integration**: Local models, cloud APIs, and custom providers
- **Auto-Bootstrap**: Automated installation of PostgreSQL, cache, drive, and more
- **Multi-Bot Hosting**: Run multiple isolated bots on a single server

---

## Quick Start

1. **Installation**: Follow [Chapter 01: Run and Talk](chapter-01/README.md)
2. **Explore Templates**: Check `templates/announcements.gbai/` for examples
3. **Create a Bot**: Copy a template and modify it
4. **Learn BASIC**: Read [Chapter 05: BASIC Reference](chapter-05/README.md)
5. **Configure**: Edit `config.csv` in your `.gbot/` directory
6. **Deploy**: Restart General Bots to activate changes

---

## Table of Contents

### Part I - Getting Started
- [Chapter 01: Run and Talk](chapter-01/README.md) - Installation and first conversation

### Part II - Package System
- [Chapter 02: About Packages](chapter-02/README.md) - Overview of template-based packages
  - [.gbai Architecture](chapter-02/gbai.md) - Package structure and lifecycle
  - [.gbdialog Dialogs](chapter-02/gbdialog.md) - BASIC scripts
  - [.gbkb Knowledge Base](chapter-02/gbkb.md) - Document collections
  - [.gbot Configuration](chapter-02/gbot.md) - Bot parameters
  - [.gbtheme UI Theming](chapter-02/gbtheme.md) - Web interface customization
  - [.gbdrive File Storage](chapter-02/gbdrive.md) - Drive (S3-compatible) integration

### Part III - Knowledge Base
- [Chapter 03: gbkb Reference](chapter-03/README.md) - Semantic search and vector database

### Part IV - User Interface
- [Chapter 04: .gbui Interface Reference](chapter-04-gbui/README.md) - HTML templates and UI components

### Part V - Themes and Styling
- [Chapter 05: gbtheme CSS Reference](chapter-05-gbtheme/README.md) - CSS-based theme customization

### Part VI - BASIC Dialogs
- [Chapter 06: gbdialog Reference](chapter-06-gbdialog/README.md) - Complete BASIC scripting reference
  - Keywords: `TALK`, `HEAR`, `LLM`, `SET CONTEXT`, `USE KB`, and more

### Part VII - Extending General Bots
- [Chapter 07: gbapp Architecture Reference](chapter-07-gbapp/README.md) - Internal architecture
  - [Architecture Overview](chapter-07-gbapp/architecture.md) - Bootstrap process
  - [Building from Source](chapter-07-gbapp/building.md) - Compilation and features
  - [Module Structure](chapter-07-gbapp/crates.md) - Single-crate organization
  - [Service Layer](chapter-07-gbapp/services.md) - Module descriptions
  - [Creating Custom Keywords](chapter-07-gbapp/custom-keywords.md) - Extending BASIC
  - [Adding Dependencies](chapter-07-gbapp/dependencies.md) - Cargo.toml management

### Part VIII - Bot Configuration
- [Chapter 08: gbot Reference](chapter-08-config/README.md) - Configuration and parameters

### Part IX - Tools and Integration
- [Chapter 09: API and Tooling](chapter-09-api/README.md) - Function calling and tool integration

### Part X - Feature Deep Dive
- [Chapter 10: Feature Reference](chapter-10-features/README.md) - Complete feature list
  - [Core Features](chapter-10-features/core-features.md) - Platform capabilities

### Part XI - Community
- [Chapter 11: Contributing](chapter-11-community/README.md) - Development and contribution guidelines

### Part XII - Authentication and Security
- [Chapter 12: Authentication](chapter-12-auth/README.md) - Security features

### Appendices
- [Appendix I: Database Model](appendix-i/README.md) - Schema reference
- [Glossary](glossary.md) - Terms and definitions

---

## Architecture Overview

General Bots is a **monolithic Rust application** (single crate) with the following structure:

### Core Modules
- `auth` - Argon2 password hashing, session tokens
- `bot` - Bot lifecycle and management
- `session` - Persistent conversation state
- `basic` - BASIC interpreter (powered by Rhai)
- `llm` / `llm_models` - LLM provider integration
- `context` - Conversation memory management

### Infrastructure
- `bootstrap` - Auto-installation of components
- `package_manager` - Manages PostgreSQL, cache, drive, etc.
- `web_server` - Axum HTTP REST API
- `drive` - S3-compatible storage and vector DB
- `config` - Environment configuration

### Features
- `automation` - Cron scheduling and events
- `email` - IMAP/SMTP integration (optional)
- `meet` - LiveKit video conferencing
- `channels` - Multi-channel support
- `file` - Document processing (PDF, etc.)
- `drive_monitor` - File system watching

---

## Technology Stack

- **Language**: Rust 2021 edition
- **Web**: Axum + Tower + Tokio
- **Database**: Diesel ORM + PostgreSQL
- **Cache**: Valkey (Redis-compatible)
- **Storage**: AWS SDK S3 (drive component)
- **Vector DB**: Qdrant (optional)
- **Scripting**: Rhai engine
- **Security**: Argon2, AES-GCM
- **Desktop**: Tauri (optional)

---

## Project Information

- **Version**: 6.0.8
- **License**: AGPL-3.0
- **Repository**: https://github.com/GeneralBots/botserver
- **Community**: Open-source contributors from Pragmatismo.com.br

---

## Documentation Status

This documentation is a **living document** that evolves with the codebase. Contributions are welcome! If you find inaccuracies or gaps:

1. Check the source code in `src/` for ground truth
2. Submit documentation improvements via pull requests
3. Report issues on GitHub

See [TODO.txt](TODO.txt) for known documentation gaps.

---

## Next Steps

Start with [Introduction](introduction.md) for a comprehensive overview, or jump directly to [Chapter 01: Run and Talk](chapter-01/README.md) to install and run General Bots.