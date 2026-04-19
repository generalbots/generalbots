# Cargo Feature Map

This document provides a comprehensive reference for all Cargo.toml feature flags in General Bots, their dependencies, and how to use them for optimized builds.

## Overview

General Bots uses Rust's feature flag system to enable modular compilation. This allows you to:

- **Reduce binary size** by excluding unused features
- **Minimize dependencies** for faster compilation
- **Customize deployments** for specific use cases
- **Control resource usage** in constrained environments

## Feature Categories

```
┌─────────────────────────────────────────────────────────────┐
│                    BUNDLE FEATURES                          │
│  ┌─────────────┐ ┌─────────────┐ ┌───────────┐ ┌─────────┐ │
│  │communications│ │productivity │ │ documents │ │  full   │ │
│  └──────┬──────┘ └──────┬──────┘ └─────┬─────┘ └────┬────┘ │
└─────────┼───────────────┼──────────────┼────────────┼──────┘
          │               │              │            │
          ▼               ▼              ▼            ▼
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION FEATURES                      │
│  chat, mail, meet, calendar, tasks, drive, docs, paper...   │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│                    CORE TECHNOLOGIES                         │
│     cache, automation, llm, vectordb, monitoring...         │
└─────────────────────────────────────────────────────────────┘
```

## Default Features

The default feature set provides a minimal but functional installation:

```toml
default = ["chat", "drive", "tasks", "automation", "cache", "directory"]
```

| Feature | Purpose |
|---------|---------|
| `chat` | AI chat interface |
| `drive` | File storage and management |
| `tasks` | Task scheduling and management |
| `automation` | BASIC script engine (Rhai) |
| `cache` | Redis session caching |
| `directory` | User directory services |

## Communication Features

Features for messaging, email, and real-time communication.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `chat` | - | Core chat functionality |
| `people` | - | Contact management |
| `mail` | lettre, mailparse, imap, native-tls | Full email client (IMAP/SMTP) |
| `meet` | livekit | Video conferencing |
| `social` | - | Social network integrations |
| `whatsapp` | - | WhatsApp Business API |
| `telegram` | - | Telegram Bot API |
| `instagram` | - | Instagram messaging |
| `msteams` | - | Microsoft Teams integration |

### Bundle: `communications`

Enables all communication features:

```toml
communications = ["chat", "people", "mail", "meet", "social", 
                  "whatsapp", "telegram", "instagram", "msteams", "cache"]
```

## Productivity Features

Features for task management, scheduling, and project planning.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `calendar` | - | Event scheduling |
| `tasks` | cron, automation | Task management with scheduling |
| `project` | quick-xml | Project (MS Project style) |
| `goals` | - | Goal tracking and OKRs |
| `workspace` | - | Workspace management |
| `tickets` | - | Support ticket system |
| `billing` | - | Invoicing and payments |

### Bundle: `productivity`

```toml
productivity = ["calendar", "tasks", "project", "goals", "workspaces", "cache"]
```

## Document Features

Features for document processing and file management.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `drive` | aws-config, aws-sdk-s3, aws-smithy-async, pdf-extract | Cloud storage (S3-compatible) |
| `docs` | docx-rs, ooxmlsdk | Word document processing |
| `paper` | docs, pdf-extract | AI-assisted writing |
| `sheet` | calamine, spreadsheet-ods | Spreadsheet processing |
| `slides` | ooxmlsdk | Presentation processing |

### Bundle: `documents`

```toml
documents = ["paper", "docs", "sheet", "slides", "drive"]
```

## Media Features

Features for video, audio, and visual content.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `video` | - | Video processing |
| `player` | - | Media playback |
| `canvas` | - | Visual editor/whiteboard |

### Bundle: `media`

```toml
media = ["video", "player", "canvas"]
```

## Learning & Research Features

Features for AI-powered research and learning.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `learn` | - | Learning management |
| `research` | llm, vectordb | AI research assistant |
| `sources` | - | Knowledge source management |

### Bundle: `learning`

```toml
learning = ["learn", "research", "sources"]
```

## Analytics Features

Features for monitoring and business intelligence.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `analytics` | - | Usage analytics |
| `dashboards` | - | Custom BI dashboards |
| `monitoring` | sysinfo | System monitoring |

### Bundle: `analytics_suite`

```toml
analytics_suite = ["analytics", "dashboards", "monitoring"]
```

## Development Features

Features for bot development and automation.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `designer` | - | Visual bot builder |
| `editor` | - | Code editor |
| `automation` | rhai, cron | BASIC script engine |

### Bundle: `development`

```toml
development = ["designer", "editor", "automation"]
```

## Admin Features

Features for system administration.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `attendant` | - | Human handoff queue |
| `security` | - | Security settings |
| `settings` | - | Configuration UI |

### Bundle: `admin`

```toml
admin = ["attendant", "security", "settings"]
```

## Core Technologies

Low-level features that other features depend on.

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `llm` | - | LLM integration framework |
| `vectordb` | qdrant-client | Vector database for RAG |
| `nvidia` | - | NVIDIA GPU acceleration |
| `cache` | redis | Session caching |
| `compliance` | csv | Compliance/audit tools |
| `timeseries` | - | Time-series database |
| `weba` | - | Web accessibility features |
| `directory` | - | User directory services |
| `progress-bars` | indicatif | CLI progress indicators |
| `grpc` | tonic | gRPC server |
| `jemalloc` | tikv-jemallocator, tikv-jemalloc-ctl | Memory allocator |
| `console` | crossterm, ratatui, monitoring | Terminal UI mode |

## Build Profiles

### Minimal Build

Smallest possible binary with core functionality:

```bash
cargo build --release --no-default-features --features "chat,cache"
```

**Size**: ~15MB  
**Use case**: Embedded devices, edge computing

### Lightweight Build

Core office features without heavy dependencies:

```bash
cargo build --release --no-default-features \
  --features "chat,drive,tasks,cache,directory"
```

**Size**: ~25MB  
**Use case**: Small deployments, resource-constrained servers

### Standard Build (Default)

Balanced feature set for most deployments:

```bash
cargo build --release
```

**Size**: ~40MB  
**Use case**: General purpose, development

### Full Build

All features enabled:

```bash
cargo build --release --all-features
```

**Size**: ~80MB+  
**Use case**: Enterprise deployments, feature testing

## Dependency Matrix

```
Feature          | External Crates                      | Size Impact
-----------------+--------------------------------------+-------------
mail             | lettre, mailparse, imap, native-tls  | ~2MB
meet             | livekit                              | ~5MB
vectordb         | qdrant-client                        | ~3MB
drive            | aws-sdk-s3, aws-config               | ~4MB
docs             | docx-rs, ooxmlsdk                    | ~2MB
sheet            | calamine, spreadsheet-ods            | ~1MB
automation       | rhai, cron                           | ~2MB
cache            | redis                                | ~1MB
console          | crossterm, ratatui                   | ~1MB
monitoring       | sysinfo                              | ~500KB
```

## Runtime Configuration

Features can be further controlled at runtime via the `.product` file:

```ini
# .product file
name=My Custom Bot
apps=chat,drive,tasks
theme=dark
```

The effective app list is the **intersection** of:
1. Features compiled in Cargo.toml
2. Apps enabled in `.product` file

This means you can compile with many features but only expose a subset to users.

## API Endpoint

The `/api/product` endpoint returns the current feature matrix:

```json
{
  "name": "General Bots",
  "apps": ["chat", "drive", "tasks"],
  "compiled_features": ["chat", "drive", "tasks", "automation", "cache", "directory"],
  "theme": "sentient"
}
```

## Checking Compiled Features

In Rust code, use conditional compilation:

```rust
#[cfg(feature = "mail")]
fn handle_email() {
    // Only compiled when mail feature is enabled
}
```

At runtime, check the `COMPILED_FEATURES` constant:

```rust
use crate::core::features::COMPILED_FEATURES;

if COMPILED_FEATURES.contains(&"mail") {
    // Feature is available
}
```

## Best Practices

1. **Start minimal**: Begin with the default features and add as needed
2. **Use bundles**: Prefer bundle features over individual ones for consistency
3. **Test builds**: Verify functionality after changing features
4. **Document requirements**: List required features in deployment docs
5. **Monitor size**: Track binary size as features are added

## See Also

- [Building from Source](../07-gbapp/building.md)
- [Cargo Tools Reference](../07-gbapp/cargo-tools.md)
- [White Label Configuration](../12-ecosystem-reference/README.md)
