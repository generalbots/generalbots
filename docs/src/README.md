<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="400">
</div>

# General Bots Documentation

Welcome to the General Bots documentation. This guide explains how to install, configure, extend, and deploy conversational AI bots using General Bots' template-based package system and BASIC scripting language.


## About This Documentation

This documentation has been recently updated to accurately reflect the actual implementation of General Bots version 6.0.8.

The following chapters now contain accurate, verified documentation: Chapter 02 covering the package system with its template-based `.gbai` structure, Chapter 06 documenting the Rust architecture including the single-crate structure and module overview, Chapter 09 explaining core features, and the Introduction providing architecture and capabilities overview.

Several areas have partial documentation that continues to improve. Chapter 05 on BASIC keywords includes working examples though the full reference needs expansion. Chapter 08 on tool integration has concepts documented while implementation details are being added. Chapter 11 on authentication reflects the implemented functionality but needs additional detail.

Documentation work continues on several modules. The UI module in `src/ui/`, the UI tree module in `src/ui_tree/`, the Riot compiler module in `src/riot_compiler/`, and the prompt manager in `src/prompt_manager/` all need comprehensive documentation. API endpoints, UI server routes, Drive integration details for S3-compatible storage, and LiveKit video conferencing integration are also being documented.


## What is General Bots?

General Bots is an open-source conversational AI platform written in Rust. The platform enables users to create intelligent chatbots through several integrated capabilities.

BASIC Scripting provides simple `.bas` scripts for defining conversation flows without requiring traditional programming expertise. Template Packages organize bots as `.gbai` directories containing dialogs, knowledge bases, and configuration in a portable structure. Vector Search enables semantic document retrieval using Qdrant for intelligent information access. LLM Integration connects to local models, cloud APIs, and custom providers for natural language understanding. Auto-Bootstrap handles automated installation of PostgreSQL, cache, drive storage, and other dependencies. Multi-Bot Hosting allows running multiple isolated bots on a single server instance.


## Quick Start

Getting started with General Bots follows a straightforward path. Begin with installation by following Chapter 01 on Run and Talk. Then explore the templates directory, particularly `templates/announcements.gbai/`, to see working examples. Create your own bot by copying a template and modifying it to suit your needs. Learn the BASIC scripting language through Chapter 05's reference documentation. Configure your bot by editing the `config.csv` file in your `.gbot/` directory. Finally, deploy by restarting General Bots to activate your changes.


## Table of Contents

### Part I - Getting Started

Chapter 01 on Run and Talk covers installation and your first conversation with a bot.

### Part II - Package System

Chapter 02 on About Packages provides an overview of the template-based package system. This includes the `.gbai` Architecture explaining package structure and lifecycle, `.gbdialog` Dialogs for BASIC scripts, `.gbkb` Knowledge Base for document collections, `.gbot` Configuration for bot parameters, `.gbtheme` UI Theming for web interface customization, and `.gbdrive` File Storage for S3-compatible drive integration.

### Part III - Knowledge Base

Chapter 03 on gbkb Reference covers semantic search and vector database functionality.

### Part IV - User Interface

Chapter 04 on .gbui Interface Reference documents HTML templates and UI components.

### Part V - Themes and Styling

Chapter 05 on gbtheme CSS Reference explains CSS-based theme customization.

### Part VI - BASIC Dialogs

Chapter 06 on gbdialog Reference provides the complete BASIC scripting reference including keywords like TALK, HEAR, LLM, SET CONTEXT, USE KB, and many more.

### Part VII - Extending General Bots

Chapter 07 on gbapp Architecture Reference documents the internal architecture. This includes the Architecture Overview explaining the bootstrap process, Building from Source for compilation and features, Module Structure describing single-crate organization, Service Layer with module descriptions, Creating Custom Keywords for extending BASIC, and Adding Dependencies for Cargo.toml management.

### Part VIII - Bot Configuration

Chapter 08 on gbot Reference covers configuration and parameters.

### Part IX - Tools and Integration

Chapter 09 on API and Tooling explains function calling and tool integration.

### Part X - Feature Deep Dive

Chapter 10 on Feature Reference provides the complete feature list including Core Features documenting platform capabilities.

### Part XI - Community

Chapter 11 on Contributing provides development and contribution guidelines.

### Part XII - Authentication and Security

Chapter 12 on Authentication documents security features.

### Appendices

Appendix I on Database Model provides schema reference. The Glossary defines terms used throughout the documentation.


## Architecture Overview

General Bots is a monolithic Rust application organized as a single crate with clearly defined modules serving different purposes.

### Core Modules

The core modules handle fundamental bot functionality. The auth module provides Argon2 password hashing and session token management. The bot module manages bot lifecycle and coordination. The session module maintains persistent conversation state across interactions. The basic module implements the BASIC interpreter powered by the Rhai scripting engine. The llm and llm_models modules handle LLM provider integration for multiple backends. The context module manages conversation memory and context window optimization.

### Infrastructure Modules

Infrastructure modules provide the foundation for bot operations. The bootstrap module handles auto-installation of all required components. The package_manager module manages PostgreSQL, cache, drive storage, and other services. The web_server module implements the Axum HTTP REST API. The drive module integrates S3-compatible storage and vector database access. The config module handles environment configuration loading and validation.

### Feature Modules

Feature modules add specific capabilities to the platform. The automation module provides cron scheduling and event-driven processing. The email module offers optional IMAP and SMTP integration. The meet module enables LiveKit video conferencing. The channels module supports multi-channel deployment across different platforms. The file module handles document processing for PDF and other formats. The drive_monitor module watches file system changes for automatic updates.


## Technology Stack

General Bots is built on a modern Rust technology stack. The application uses Rust 2021 edition for safety and performance. Web handling uses Axum combined with Tower middleware and Tokio async runtime. Database access uses the Diesel ORM with PostgreSQL as the backing store. Caching uses a Redis-compatible cache component for session and data caching. Storage uses the AWS SDK for S3-compatible drive operations. Vector database functionality uses Qdrant for semantic search when enabled. Scripting uses the Rhai engine to power the BASIC interpreter. Security uses Argon2 for password hashing and AES-GCM for encryption. Optional desktop deployment uses Tauri for native applications.


## Project Information

The current version is 6.0.8 released under the AGPL-3.0 license. The source repository is available at https://github.com/GeneralBots/botserver. The project is maintained by open-source contributors from Pragmatismo.com.br and the broader community.


## Documentation Status

This documentation is a living document that evolves alongside the codebase. Contributions are welcome from anyone who wants to improve it.

If you find inaccuracies or gaps in the documentation, the best approach is to check the source code in `src/` for the ground truth implementation. Submit documentation improvements via pull requests on GitHub. Report issues through the GitHub issue tracker so they can be tracked and addressed.


## Next Steps

Start with the Introduction for a comprehensive overview of General Bots concepts and capabilities. Alternatively, jump directly to Chapter 01 on Run and Talk to install and run General Bots immediately.