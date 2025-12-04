# General Bots Documentation

Welcome to the General Bots documentation. This guide covers everything you need to build, deploy, and manage AI-powered bots.

## Quick Navigation

| Section | Description |
|---------|-------------|
| [Getting Started](guides/getting-started.md) | Installation and first bot |
| [API Reference](api/README.md) | REST endpoints and WebSocket |
| [BASIC Language](reference/basic-language.md) | Dialog scripting reference |
| [Configuration](reference/configuration.md) | Environment and settings |

## Documentation Structure

```
docs/
├── api/                    # API documentation
│   ├── README.md           # API overview
│   ├── rest-endpoints.md   # HTTP endpoints
│   └── websocket.md        # Real-time communication
├── guides/                 # How-to guides
│   ├── getting-started.md  # Quick start
│   ├── deployment.md       # Production setup
│   └── templates.md        # Using templates
└── reference/              # Technical reference
    ├── basic-language.md   # BASIC keywords
    ├── configuration.md    # Config options
    └── architecture.md     # System design
```

## Core Concepts

### Knowledge Bases (KB)
Store documents, FAQs, and data that the AI can search and reference:
```basic
USE KB "company-docs"
```

### Tools
Functions the AI can call to perform actions:
```basic
USE TOOL "send-email"
USE TOOL "create-ticket"
```

### Dialogs
BASIC scripts that define conversation flows and automation:
```basic
TALK "Hello! How can I help?"
answer = HEAR
```

## Quick Links

- **[GitHub Repository](https://github.com/GeneralBots/BotServer)**
- **[Issue Tracker](https://github.com/GeneralBots/BotServer/issues)**
- **[Contributing Guide](../CONTRIBUTING.md)**

## Version

This documentation covers **General Bots v6.x**.