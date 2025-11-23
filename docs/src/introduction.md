# Introduction to BotServer

BotServer is an open-source conversational AI platform that enables users to create, deploy, and manage intelligent chatbots using a simple BASIC-like scripting language. Built in Rust with a focus on performance and security, it provides a complete ecosystem for building production-ready bot applications.

## What is BotServer?

BotServer is a monolithic Rust application that combines multiple services into a unified platform for bot development:

- **BASIC Scripting**: Create conversation flows using simple, English-like `.bas` scripts
- **Template System**: Organize bots as `.gbai` packages with dialogs, knowledge bases, and configuration
- **Vector Search**: Semantic document retrieval using Qdrant vector database
- **LLM Integration**: Support for OpenAI, local models, and custom providers
- **Multi-Channel**: Web interface, WebSocket, voice, and extensible channel support
- **Auto-Bootstrap**: Automated installation and configuration of all dependencies
- **Enterprise Security**: Argon2 password hashing, AES-GCM encryption, secure session management

## Architecture

BotServer is implemented as a single Rust crate (version 6.0.8) with modular components:

### Core Modules

- **`auth`** - User authentication with Argon2 password hashing
- **`bot`** - Bot lifecycle and configuration management
- **`session`** - Persistent session state and conversation history
- **`basic`** - BASIC-like scripting language interpreter (powered by Rhai)
- **`llm`** - LLM provider integration (OpenAI, local models)
- **`context`** - Conversation context and memory management

### Infrastructure Modules

- **`bootstrap`** - Automated system initialization and component installation
- **`package_manager`** - Manages 20+ components (PostgreSQL, cache, drive, Qdrant, etc.)
- **`web_server`** - Axum-based HTTP API and WebSocket server
- **`drive`** - S3-compatible object storage and vector database integration
- **`config`** - Application configuration from `.env` and database

### Feature Modules

- **`automation`** - Cron scheduling and event-driven tasks
- **`email`** - IMAP/SMTP email integration (optional feature)
- **`meet`** - LiveKit video conferencing integration
- **`channels`** - Multi-channel abstraction layer
- **`file`** - Document processing (PDF extraction, parsing)
- **`drive_monitor`** - File system monitoring and automatic indexing

## Package System

Bots are organized as template-based packages in the `templates/` directory:

```
templates/
├── default.gbai/           # Minimal starter bot
│   └── default.gbot/
│       └── config.csv
└── announcements.gbai/     # Full-featured example
    ├── announcements.gbdialog/
    │   ├── start.bas
    │   ├── auth.bas
    │   └── *.bas
    ├── announcements.gbkb/
    │   ├── auxiliom/
    │   ├── news/
    │   └── toolbix/
    └── annoucements.gbot/
        └── config.csv
```

### Package Components

- **`.gbai`** - Root directory container for bot resources
- **`.gbdialog`** - BASIC scripts (`.bas` files) defining conversation logic
- **`.gbkb`** - Document collections for semantic search
- **`.gbot`** - Bot configuration in `config.csv` format
- **`.gbtheme`** - Optional UI customization (CSS/HTML)
- **`.gbdrive`** - Drive (S3-compatible) storage integration

## Key Features

### BASIC Scripting Language

Create conversations with simple, readable syntax:

```basic
let resume = GET_BOT_MEMORY("introduction");
SET_CONTEXT "general" AS resume;

TALK "Hello! I'm your assistant."
TALK "How can I help you today?"

let response = HEAR;
let answer = LLM("Answer the user's question: " + response);
TALK answer;
```

Custom keywords include:
- `TALK` / `HEAR` - Conversation I/O
- `LLM` - Call language models
- `GET_BOT_MEMORY` / `SET_BOT_MEMORY` - Persistent storage
- `SET_CONTEXT` / `USE_KB` - Knowledge base management
- `USE_TOOL` / `LIST_TOOLS` - Tool integration
- `SET_SCHEDULE` / `ON` - Automation and events
- `GET` / `FIND` / `SET` - Data operations
- `FOR EACH` / `EXIT FOR` - Control flow

### Knowledge Base & Semantic Search

- Store documents in `.gbkb/` collections
- Automatic indexing into Qdrant vector database
- Semantic search using embeddings
- Context retrieval for LLM augmentation
- Support for multiple document formats

### Auto-Bootstrap System

BotServer automatically installs and configures:

1. **PostgreSQL** - User accounts, sessions, bot configuration
2. **Cache (Valkey)** - Session cache and temporary data
3. **Drive** - S3-compatible object storage
4. **Qdrant** - Vector database for semantic search
5. **Local LLM** - Optional local model server
6. **Email Server** - Optional SMTP/IMAP
7. **LiveKit** - Optional video conferencing
8. And more...

The bootstrap process:
- Generates secure credentials automatically
- Applies database migrations
- Creates bots from templates
- Uploads resources to storage
- Configures all components

### Multi-Bot Hosting

A single BotServer instance can host multiple bots:

- Each bot is isolated with separate configuration
- Bots share infrastructure (database, cache, storage)
- Independent update cycles per bot
- Optional multi-tenancy support

### Security Features

- **Password Hashing**: Argon2 with secure parameters
- **Encryption**: AES-GCM for sensitive data at rest
- **Session Management**: Cryptographically random tokens
- **API Authentication**: Token-based access control
- **Credential Generation**: Automatic secure password creation
- **Database Security**: Parameterized queries via Diesel ORM

### LLM Integration

Flexible AI provider support:

- **OpenAI**: GPT-3.5, GPT-4, and newer models
- **Local Models**: Self-hosted LLM servers (llama.cpp compatible)
- **Streaming**: Token-by-token response streaming
- **Context Management**: Automatic context window handling
- **Model Selection**: Choose models based on task complexity

### Storage Architecture

- **PostgreSQL**: Structured data (users, bots, sessions, messages)
- **Cache**: Session cache and rate limiting
- **Drive (S3)**: Documents, templates, and assets
- **Qdrant**: Vector embeddings for semantic search
- **File System**: Optional local caching

## How It Works

1. **Bootstrap**: System scans `templates/` and creates bots from `.gbai` packages
2. **User Connection**: User connects via web interface or API
3. **Session Creation**: System creates authenticated session with unique token
4. **Dialog Execution**: Bot loads and executes `.gbdialog` scripts
5. **Knowledge Retrieval**: Queries vector database for relevant context
6. **LLM Integration**: Sends context + user message to language model
7. **Response**: Bot responds through appropriate channel
8. **State Persistence**: Session and conversation history saved to database

## Technology Stack

- **Language**: Rust (2021 edition)
- **Web Framework**: Axum + Tower
- **Async Runtime**: Tokio
- **Database**: Diesel ORM with PostgreSQL
- **Cache**: Valkey client (Redis-compatible, tokio-comp)
- **Storage**: AWS SDK S3 (drive compatible)
- **Vector DB**: Qdrant client (optional feature)
- **Scripting**: Rhai engine for BASIC interpreter
- **Security**: Argon2, AES-GCM, HMAC-SHA256
- **Desktop**: Tauri (optional feature)
- **Video**: LiveKit SDK

## Installation Modes

BotServer supports multiple deployment modes:

- **Local**: Install components directly on the host system
- **Container**: Use LXC containers for isolation

The `package_manager` handles component lifecycle in all modes.

## Use Cases

- **Customer Support**: Automated help desk with knowledge base
- **Internal Tools**: Employee assistance with company documentation
- **Product Catalogs**: Conversational product search and recommendations
- **Announcements**: Broadcast system with intelligent Q&A
- **Meeting Bots**: AI assistant for video conferences
- **Email Automation**: Intelligent email response and routing
- **Document Management**: Semantic search across document collections

## Getting Started

1. **Install BotServer** following Chapter 01 instructions
2. **Run Bootstrap** to install dependencies automatically
3. **Explore Templates** in `templates/announcements.gbai/`
4. **Create Your Bot** by adding a new `.gbai` package
5. **Write Dialogs** using BASIC scripts in `.gbdialog/`
6. **Add Knowledge** by placing documents in `.gbkb/`
7. **Configure** via `config.csv` in `.gbot/`
8. **Restart** to activate your bot

## Documentation Structure

This book is organized into the following parts:

- **Part I**: Getting started and basic usage
- **Part II**: Package system (`.gbai`, `.gbdialog`, `.gbkb`, `.gbot`, `.gbtheme`, `.gbdrive`)
- **Part III**: Knowledge base and vector search
- **Part IV**: UI theming and customization
- **Part V**: BASIC scripting language reference
- **Part VI**: Rust architecture and extending BotServer
- **Part VII**: Bot configuration parameters
- **Part VIII**: Tool integration and external APIs
- **Part IX**: Complete feature matrix
- **Part X**: Contributing and development
- **Part XI**: Authentication and security
- **Appendices**: Database schema and reference material

## Project Information

- **Version**: 6.0.8
- **License**: AGPL-3.0
- **Repository**: https://github.com/GeneralBots/BotServer
- **Language**: Rust (monolithic crate)
- **Community**: Open-source contributors from Pragmatismo.com.br and beyond

## Next Steps

Continue to [Chapter 01: Run and Talk](./chapter-01/README.md) to install BotServer and start your first conversation.