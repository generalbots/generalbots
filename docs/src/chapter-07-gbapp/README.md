# Chapter 07: Extending General Bots

Architecture and deployment reference for developers.

## Overview

BotServer is built in Rust with a modular architecture. Extend it by creating custom keywords, services, or entire applications.

## Architecture

```
┌─────────────────────────────────────────┐
│              Web Server (Axum)          │
├─────────────────────────────────────────┤
│         BASIC Runtime (Rhai)            │
├──────────┬──────────┬──────────┬────────┤
│   LLM    │ Storage  │  Vector  │ Cache  │
│ Service  │ (MinIO)  │ (Qdrant) │(Valkey)│
├──────────┴──────────┴──────────┴────────┤
│            PostgreSQL                   │
└─────────────────────────────────────────┘
```

## Deployment Options

| Method | Use Case | Guide |
|--------|----------|-------|
| **Local** | Development | [Installation](../chapter-01/installation.md) |
| **Docker** | Production | [Docker Deployment](./docker-deployment.md) |
| **LXC** | Isolated components | [Container Deployment](./containers.md) |

## Module Structure

| Module | Purpose |
|--------|---------|
| `web_server` | HTTP/WebSocket handling |
| `basic` | BASIC language runtime |
| `llm` | LLM provider integration |
| `drive` | Object storage |
| `shared` | Database models |

## Creating Custom Keywords

```rust
// In src/basic/keywords/my_keyword.rs
pub fn my_keyword(context: &mut EvalContext) -> Result<Dynamic, Box<EvalError>> {
    // Your keyword logic
    Ok(Dynamic::from("result"))
}
```

Register in `keywords/mod.rs` and rebuild.

## Chapter Contents

- [Architecture Overview](./architecture.md) - System design
- [Building from Source](./building.md) - Compilation guide
- [Container Deployment (LXC)](./containers.md) - Linux containers
- [Docker Deployment](./docker-deployment.md) - Docker setup
- [Scaling](./scaling.md) - Load balancing
- [Infrastructure](./infrastructure.md) - Hardware planning
- [Observability](./observability.md) - Monitoring
- [Custom Keywords](./custom-keywords.md) - Extending BASIC
- [Services](./services.md) - Service layer

## See Also

- [Installation](../chapter-01/installation.md) - Getting started
- [BASIC Reference](../chapter-06-gbdialog/README.md) - Scripting language