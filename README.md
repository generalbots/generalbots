# General Bots - Enterprise-Grade LLM Orchestrator

![General Bot Logo](https://github.com/GeneralBots/botserver/blob/main/logo.png?raw=true)

**A strongly-typed LLM conversational platform focused on convention over configuration and code-less approaches.**

## Quick Links

- **[Getting Started](docs/guides/getting-started.md)** - Installation and first bot
- **[API Reference](docs/api/README.md)** - REST and WebSocket endpoints
- **[BASIC Language](docs/reference/basic-language.md)** - Dialog scripting reference

## What is General Bots?

General Bots is a **self-hosted AI automation platform** that provides:

- **Multi-Vendor LLM API** - Unified interface for OpenAI, Groq, Claude, Anthropic
- **MCP + LLM Tools Generation** - Instant tool creation from code/functions
- **Semantic Caching** - Intelligent response caching (70% cost reduction)
- **Web Automation Engine** - Browser automation + AI intelligence
- **Enterprise Data Connectors** - CRM, ERP, database native integrations
- **Git-like Version Control** - Full history with rollback capabilities

## Quick Start

### Prerequisites

- **Rust** (1.75+) - [Install from rustup.rs](https://rustup.rs/)
- **Git** - [Download from git-scm.com](https://git-scm.com/downloads)

### Installation

```bash
git clone https://github.com/GeneralBots/botserver
cd botserver
cargo run
```

On first run, botserver automatically sets up PostgreSQL, S3 storage, Redis cache, and downloads AI models.

The server will be available at `http://localhost:8080`.

## Documentation

```
docs/
├── api/                        # API documentation
│   ├── README.md               # API overview
│   ├── rest-endpoints.md       # HTTP endpoints
│   └── websocket.md            # Real-time communication
├── guides/                     # How-to guides
│   ├── getting-started.md      # Quick start
│   ├── deployment.md           # Production setup
│   └── templates.md            # Using templates
└── reference/                  # Technical reference
    ├── basic-language.md       # BASIC keywords
    ├── configuration.md        # Config options
    └── architecture.md         # System design
```

## Key Features

### 4 Essential Keywords

```basic
USE KB "kb-name"        ' Load knowledge base into vector database
CLEAR KB "kb-name"      ' Remove KB from session
USE TOOL "tool-name"    ' Make tool available to LLM
CLEAR TOOLS             ' Remove all tools from session
```

### Example Bot

```basic
' customer-support.bas
USE KB "support-docs"
USE TOOL "create-ticket"
USE TOOL "check-order"

SET CONTEXT "support" AS "You are a helpful customer support agent."

TALK "Welcome! How can I help you today?"
```

## Command-Line Options

```bash
cargo run                    # Default: console UI + web server
cargo run -- --noconsole     # Background service mode
cargo run -- --desktop       # Desktop application (Tauri)
cargo run -- --tenant <name> # Specify tenant
cargo run -- --container     # LXC container mode
```

## Environment Variables

Only directory service variables are required:

| Variable | Purpose |
|----------|---------|
| `DIRECTORY_URL` | Zitadel instance URL |
| `DIRECTORY_CLIENT_ID` | OAuth client ID |
| `DIRECTORY_CLIENT_SECRET` | OAuth client secret |

All service credentials are managed automatically. See [Configuration](docs/reference/configuration.md) for details.

## Current Status

**Version:** 6.0.8  
**Build Status:** SUCCESS  
**Production Ready:** YES

## Deployment

See [Deployment Guide](docs/guides/deployment.md) for:

- Single server setup
- Docker Compose
- LXC containers
- Kubernetes
- Reverse proxy configuration

## Contributing

We welcome contributions! Please read our contributing guidelines before submitting PRs.

## Security

Security issues should be reported to: **security@pragmatismo.com.br**

## License

General Bot Copyright (c) pragmatismo.com.br. All rights reserved.  
Licensed under the **AGPL-3.0**.

According to our dual licensing model, this program can be used either under the terms of the GNU Affero General Public License, version 3, or under a proprietary license.

## Support

- **GitHub Issues:** [github.com/GeneralBots/botserver/issues](https://github.com/GeneralBots/botserver/issues)
- **Stack Overflow:** Tag questions with `generalbots`
- **Video Tutorial:** [7 AI General Bots LLM Templates](https://www.youtube.com/watch?v=KJgvUPXi3Fw)

## Contributors

<a href="https://github.com/generalbots/botserver/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/botserver" />
</a>

---

**General Bots Code Name:** [Guaribas](https://en.wikipedia.org/wiki/Guaribas)

> "No one should have to do work that can be done by a machine." - Roberto Mangabeira Unger