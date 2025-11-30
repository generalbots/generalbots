# General Bots - Enterprise-Grade LLM Orchestrator

![General Bot Logo](https://github.com/GeneralBots/BotServer/blob/main/logo.png?raw=true)

**A strongly-typed LLM conversational platform focused on convention over configuration and code-less approaches.**

## Quick Links

- **[Complete Documentation](docs/src/SUMMARY.md)** - Full documentation index
- **[Quick Start Guide](docs/QUICK_START.md)** - Get started in minutes
- **[Changelog](CHANGELOG.md)** - Version history

## Documentation Structure

All documentation has been organized into the **[docs/](docs/)** directory:

### Core Documentation
- **[Introduction & Getting Started](docs/src/chapter-01/README.md)**
- **[Package System](docs/src/chapter-02/README.md)**
- **[Knowledge Base Reference](docs/src/chapter-03/README.md)**
- **[User Interface](docs/src/chapter-04-gbui/README.md)**
- **[BASIC Dialogs](docs/src/chapter-06-gbdialog/README.md)**
- **[Architecture Reference](docs/src/chapter-07-gbapp/README.md)**
- **[Configuration](docs/src/chapter-08-config/README.md)**
- **[REST API Reference](docs/src/chapter-10-api/README.md)**
- **[Security & Authentication](docs/src/chapter-12-auth/README.md)**

### Technical References
- **[KB & Tools System](docs/src/chapter-03/kb-and-tools.md)** - Core system architecture
- **[Semantic Cache](docs/src/chapter-03/caching.md)** - LLM caching with 70% cost reduction
- **[Universal Messaging](docs/src/chapter-06-gbdialog/universal-messaging.md)** - Multi-channel communication
- **[External Services](docs/src/appendix-external-services/README.md)** - Service integrations

## What is General Bots?

General Bots is a **self-hosted AI automation platform** that provides:

- **Multi-Vendor LLM API** - Unified interface for OpenAI, Groq, Claude, Anthropic
- **MCP + LLM Tools Generation** - Instant tool creation from code/functions
- **Semantic Caching** - Intelligent response caching (70% cost reduction)
- **Web Automation Engine** - Browser automation + AI intelligence
- **External Data APIs** - Integrated services via connectors
- **Enterprise Data Connectors** - CRM, ERP, database native integrations
- **Git-like Version Control** - Full history with rollback capabilities
- **Contract Analysis** - Legal document review and summary

## Command-Line Options

```bash
# Run with default settings (console UI enabled)
cargo run

# Run without console UI
cargo run -- --noconsole

# Run in desktop mode (Tauri)
cargo run -- --desktop

# Run without any UI
cargo run -- --noui

# Specify tenant
cargo run -- --tenant <tenant_name>

# LXC container mode
cargo run -- --container
```

### Default Behavior
- **Console UI is enabled by default** - Shows real-time system status, logs, and file browser
- **Minimal UI is served by default** at `http://localhost:8080` - Lightweight, fast-loading interface
- Full suite UI available at `http://localhost:8080/suite` - Complete multi-application interface
- Use `--noconsole` to disable the terminal UI and run as a background service
- The HTTP server always runs on port 8080 unless in desktop mode


## Key Features

### 4 Essential Keywords
General Bots provides a minimal, focused system for managing Knowledge Bases and Tools:

```basic
USE KB "kb-name"        ' Load knowledge base into vector database
CLEAR KB "kb-name"      ' Remove KB from session
USE TOOL "tool-name"    ' Make tool available to LLM
CLEAR TOOLS             ' Remove all tools from session
```

### Strategic Advantages
- **vs ChatGPT/Claude**: Automates entire business processes, not just chat
- **vs n8n/Make**: Simpler approach with little programming needed
- **vs Microsoft 365**: User control, not locked systems
- **vs Salesforce**: Open-source AI orchestration connecting all systems

## Quick Start

### Prerequisites
- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **Git** (latest stable) - [Download from git-scm.com](https://git-scm.com/downloads)

### Installation

```bash
# Clone the repository
git clone https://github.com/GeneralBots/BotServer
cd BotServer

# Run the server (auto-installs dependencies)
cargo run
```

On first run, BotServer automatically:
- Installs required components (PostgreSQL, S3-compatible storage, Cache, LLM)
- Sets up database with migrations
- Downloads AI models
- Uploads template bots
- Starts HTTP server at `http://127.0.0.1:8080`

### Management Commands
```bash
botserver start              # Start all components
botserver stop               # Stop all components
botserver restart            # Restart all components
botserver list               # List available components
botserver status <component> # Check component status
```

## Current Status

**Version:** 6.0.8  
**Build Status:** SUCCESS  
**Production Ready:** YES  
**Compilation:** 0 errors  

## Deployment

General Bots supports deployment via **LXC containers** for isolated, lightweight virtualization:

```bash
# Deploy with LXC container isolation
cargo run -- --container
```

See [Container Deployment](docs/src/chapter-07-gbapp/containers.md) for detailed LXC setup instructions.

## Environment Variables

General Bots uses minimal environment configuration. Only Directory service variables are required:

| Variable | Purpose |
|----------|---------|
| `DIRECTORY_URL` | Zitadel instance URL |
| `DIRECTORY_CLIENT_ID` | OAuth client ID |
| `DIRECTORY_CLIENT_SECRET` | OAuth client secret |

All service credentials (database, storage, cache) are managed automatically by the Directory service. Application configuration is done through `config.csv` files in each bot's `.gbot` folder.

See [Environment Variables](docs/src/appendix-env-vars/README.md) for details.

## Contributing

We welcome contributions! Please read:
- **[Contributing Guidelines](docs/src/chapter-13-community/README.md)**
- **[Development Setup](docs/src/chapter-13-community/setup.md)**
- **[Testing Guide](docs/src/chapter-13-community/testing.md)**

## Security

Security issues should be reported to: **security@pragmatismo.com.br**

See **[Security Policy](docs/src/chapter-12-auth/security-policy.md)** for our security guidelines.

## License

General Bot Copyright (c) pragmatismo.com.br. All rights reserved.  
Licensed under the **AGPL-3.0**.

According to our dual licensing model, this program can be used either under the terms of the GNU Affero General Public License, version 3, or under a proprietary license.

See [LICENSE](LICENSE) for details.

## Key Facts

- LLM Orchestrator AGPL licensed (contribute back for custom-label SaaS)
- True community governance
- No single corporate control
- 5+ years of stability
- Never changed license
- Enterprise-grade
- Hosted locally or multicloud

## Support & Resources

- **Documentation:** [docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)
- **GitHub:** [github.com/GeneralBots/BotServer](https://github.com/GeneralBots/BotServer)
- **Stack Overflow:** Tag questions with `generalbots`
- **Video Tutorial:** [7 AI General Bots LLM Templates](https://www.youtube.com/watch?v=KJgvUPXi3Fw)

## Demo

See conversational data analytics in action:

```basic
TALK "General Bots Labs presents FISCAL DATA SHOW BY BASIC"
result = GET "https://api.fiscaldata.treasury.gov/services/api/..."
data = SELECT YEAR(record_date) as Yr, SUM(...) AS Amount FROM data
img = CHART "bar", data
SEND FILE img
```

## Contributors

<a href="https://github.com/generalbots/botserver/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/botserver" />
</a>

---

**General Bots Code Name:** [Guaribas](https://en.wikipedia.org/wiki/Guaribas) (a city in Brazil, state of Piauí)

> "No one should have to do work that can be done by a machine." - Roberto Mangabeira Unger

<a href="https://stackoverflow.com/questions/ask?tags=generalbots">Ask a question</a> | <a href="https://github.com/GeneralBots/BotBook">Read the Docs</a>