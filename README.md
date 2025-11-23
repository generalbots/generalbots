# General Bots - Enterprise-Grade LLM Orchestrator

![General Bot Logo](https://github.com/GeneralBots/BotServer/blob/main/logo.png?raw=true)

**A strongly-typed LLM conversational platform focused on convention over configuration and code-less approaches.**

## 🚀 Quick Links

- **[Complete Documentation](docs/INDEX.md)** - Full documentation index
- **[Quick Start Guide](docs/QUICK_START.md)** - Get started in minutes
- **[Current Status](docs/07-STATUS.md)** - Production readiness (v6.0.8)
- **[Changelog](CHANGELOG.md)** - Version history

## 📚 Documentation Structure

All documentation has been organized into the **[docs/](docs/)** directory:

### Core Documentation (Numbered Chapters)
- **[Chapter 0: Introduction & Getting Started](docs/00-README.md)**
- **[Chapter 1: Build & Development Status](docs/01-BUILD_STATUS.md)**
- **[Chapter 2: Code of Conduct](docs/02-CODE_OF_CONDUCT.md)**
- **[Chapter 3: Código de Conduta (PT-BR)](docs/03-CODE_OF_CONDUCT-pt-br.md)**
- **[Chapter 4: Contributing Guidelines](docs/04-CONTRIBUTING.md)**
- **[Chapter 5: Integration Status](docs/05-INTEGRATION_STATUS.md)**
- **[Chapter 6: Security Policy](docs/06-SECURITY.md)**
- **[Chapter 7: Production Status](docs/07-STATUS.md)**

### Technical Documentation
- **[KB & Tools System](docs/KB_AND_TOOLS.md)** - Core system architecture
- **[Security Features](docs/SECURITY_FEATURES.md)** - Security implementation
- **[Semantic Cache](docs/SEMANTIC_CACHE.md)** - LLM caching with 70% cost reduction
- **[SMB Deployment](docs/SMB_DEPLOYMENT_GUIDE.md)** - Small business deployment guide
- **[Universal Messaging](docs/BASIC_UNIVERSAL_MESSAGING.md)** - Multi-channel communication

### Book-Style Documentation
- **[Detailed Docs](docs/src/)** - Comprehensive book-format documentation

## 🎯 What is General Bots?

General Bots is a **self-hosted AI automation platform** that provides:

- ✅ **Multi-Vendor LLM API** - Unified interface for OpenAI, Groq, Claude, Anthropic
- ✅ **MCP + LLM Tools Generation** - Instant tool creation from code/functions
- ✅ **Semantic Caching** - Intelligent response caching (70% cost reduction)
- ✅ **Web Automation Engine** - Browser automation + AI intelligence
- ✅ **External Data APIs** - Integrated services via connectors
- ✅ **Enterprise Data Connectors** - CRM, ERP, database native integrations
- ✅ **Git-like Version Control** - Full history with rollback capabilities
- ✅ **Contract Analysis** - Legal document review and summary

## 🏆 Key Features

### 4 Essential Keywords
General Bots provides a minimal, focused system for managing Knowledge Bases and Tools:

```basic
USE KB "kb-name"        # Load knowledge base into vector database
CLEAR KB "kb-name"      # Remove KB from session
USE TOOL "tool-name"    # Make tool available to LLM
CLEAR TOOLS             # Remove all tools from session
```

### Strategic Advantages
- **vs ChatGPT/Claude**: Automates entire business processes, not just chat
- **vs n8n/Make**: Simpler approach with little programming needed
- **vs Microsoft 365**: User control, not locked systems
- **vs Salesforce**: Open-source AI orchestration connecting all systems

## 🚀 Quick Start

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
- Installs required components (PostgreSQL, MinIO, Redis, LLM)
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

## 📊 Current Status

**Version:** 6.0.8  
**Build Status:** ✅ SUCCESS  
**Production Ready:** YES  
**Compilation:** 0 errors  
**Warnings:** 82 (all Tauri desktop UI - intentional)

See **[docs/07-STATUS.md](docs/07-STATUS.md)** for detailed status.

## 🤝 Contributing

We welcome contributions! Please read:
- **[Contributing Guidelines](docs/04-CONTRIBUTING.md)**
- **[Code of Conduct](docs/02-CODE_OF_CONDUCT.md)**
- **[Build Status](docs/01-BUILD_STATUS.md)** for current development status

## 🔒 Security

Security issues should be reported to: **security@pragmatismo.com.br**

See **[docs/06-SECURITY.md](docs/06-SECURITY.md)** for our security policy.

## 📄 License

General Bot Copyright (c) pragmatismo.com.br. All rights reserved.  
Licensed under the **AGPL-3.0**.

According to our dual licensing model, this program can be used either under the terms of the GNU Affero General Public License, version 3, or under a proprietary license.

See [LICENSE](LICENSE) for details.

## 🌟 Key Facts

- ✅ LLM Orchestrator AGPL licensed (contribute back for custom-label SaaS)
- ✅ True community governance
- ✅ No single corporate control
- ✅ 5+ years of stability
- ✅ Never changed license
- ✅ Enterprise-grade
- ✅ Hosted locally or multicloud

## 📞 Support & Resources

- **Documentation:** [docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)
- **GitHub:** [github.com/GeneralBots/BotServer](https://github.com/GeneralBots/BotServer)
- **Stack Overflow:** Tag questions with `generalbots`
- **Video Tutorial:** [7 AI General Bots LLM Templates](https://www.youtube.com/watch?v=KJgvUPXi3Fw)

## 🎬 Demo

See conversational data analytics in action:

```basic
TALK "General Bots Labs presents FISCAL DATA SHOW BY BASIC"
result = GET "https://api.fiscaldata.treasury.gov/services/api/..."
data = SELECT YEAR(record_date) as Yr, SUM(...) AS Amount FROM data
img = CHART "bar", data
SEND FILE img
```

## 👥 Contributors

<a href="https://github.com/generalbots/botserver/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/botserver" />
</a>

---

**General Bots Code Name:** [Guaribas](https://en.wikipedia.org/wiki/Guaribas) (a city in Brazil, state of Piauí)

> "No one should have to do work that can be done by a machine." - Roberto Mangabeira Unger

<a href="https://stackoverflow.com/questions/ask?tags=generalbots">:speech_balloon: Ask a question</a> &nbsp;&nbsp;&nbsp;&nbsp; <a href="https://github.com/GeneralBots/BotBook">:book: Read the Docs</a>
