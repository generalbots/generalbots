# General Bots - Enterprise-Grade LLM Orchestrator

<p align="center"><img src="../logo.svg" alt="General Bots" width="200"></p>

**Version:** 6.2.0  
**Purpose:** Main API server for General Bots (Axum + Diesel + Rhai BASIC)

---

## Overview

General Bots is a self-hosted AI automation platform built around convention over configuration, so most bots are assembled from scripts and data rather than custom code. This crate is the core API server: it handles LLM orchestration, business logic, database access, and multi-channel messaging.

Full documentation lives at **[docs.generalbots.org](https://docs.generalbots.org)**; the **[BotBook](../botbook)** has the longer guides and API references.

---

## Quick Start

### Prerequisites

- **Rust** (1.75+) - [Install from rustup.rs](https://rustup.rs/)
- **Git** - [Download from git-scm.com](https://git-scm.com/downloads)
- **Mold** - `sudo apt-get install mold`

### Installation

```bash
git clone https://github.com/GeneralBots/botserver
cd botserver
cargo install sccache
sudo apt-get install mold  # or build from source
cargo run
```

On first run, botserver automatically:
- Installs required components (PostgreSQL, S3 storage, Redis cache, LLM)
- Sets up database with migrations
- Downloads AI models
- Starts HTTP server at `http://localhost:9000`

### Command-Line Options

```bash
cargo run                    # Default: console UI + web server
cargo run -- --noconsole     # Background service mode
cargo run -- --desktop       # Desktop application (Tauri)
cargo run -- --tenant <name> # Specify tenant
cargo run -- --container     # LXC container mode
```

---

## Key Features

### Multi-Vendor LLM API
Unified interface for OpenAI, Groq, Claude, Anthropic, and local models.

### MCP + LLM Tools Generation
Instant tool creation from code and functions - no complex configurations.

### Semantic Caching
Intelligent response caching achieving **70% cost reduction** on LLM calls.

### Web Automation Engine
Browser automation combined with AI intelligence for complex workflows.

### Enterprise Data Connectors
Native integrations with CRM, ERP, databases, and external services.

### Git-like Version Control
Full history with rollback capabilities for all configurations and data.

---

## 4 Essential Keywords

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

---

## Project Structure

```
src/
├── main.rs         # Entry point
├── main_module/    # Bootstrap, config, HTTP routes, drive monitors
├── core/           # Bot pipeline, LLM orchestration, shared types
├── basic/          # Rhai BASIC interpreter
│   └── keywords/   # BASIC keyword implementations
├── security/       # Security modules
│   ├── command_guard.rs     # Safe command execution
│   ├── error_sanitizer.rs  # Error message sanitization
│   └── sql_guard.rs        # SQL injection prevention
├── apps/           # Suite app registry and LLM automation surface
└── <feature>/      # Thin shims that re-export a feature crate
                    # e.g. src/drive/mod.rs -> pub use botdrive::*;

crates/             # Feature crates (bottasks, botlearn, botdrive, botcloud, ...)
migrations/         # Database migrations
botserver-stack/    # Stack deployment files
```

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

### Absolute Prohibitions

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
❌ NEVER use CDN links - all assets must be local
❌ NEVER build SQL queries with format! - use parameterized queries
❌ NEVER pass user input to Command::new() without validation
❌ NEVER log passwords, tokens, API keys, or PII
```

---

## Security Requirements

### Error Handling

A panic aborts the process and drops every active session, so every error path must propagate via `Result` or be handled locally.

**Target**: 0 `unwrap()`/`expect()` in production code (tests excluded). Tracked in #1368.

```rust
// ❌ WRONG - aborts the process
let value = something.unwrap();
let value = something.expect("msg");

// ✅ CORRECT - Required replacements
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
let value = something.unwrap_or_default();
let value = something.unwrap_or_else(|e| { 
    log::error!("Operation failed: {e}"); 
    default_value 
});
```

### Performance Issues

**Target**: Minimize allocations, use references where possible. Tracked in #1369.

```rust
// ❌ WRONG - Excessive allocations
let name = user.name.clone();
let msg = format!("Hello {}", name.to_string());

// ✅ CORRECT - Minimize allocations  
let name = &user.name;
let msg = format!("Hello {name}");

// ✅ CORRECT - Use Cow for conditional ownership
use std::borrow::Cow;
fn process_name(name: Cow<str>) -> String {
    match name {
        Cow::Borrowed(s) => s.to_uppercase(),
        Cow::Owned(s) => s.to_uppercase(),
    }
}
```

### SQL Injection Prevention

```rust
// ❌ WRONG
let query = format!("SELECT * FROM {}", table_name);

// ✅ CORRECT - whitelist validation
const ALLOWED_TABLES: &[&str] = &["users", "sessions"];
if !ALLOWED_TABLES.contains(&table_name) {
    return Err(Error::InvalidTable);
}
```

### Command Injection Prevention

```rust
// ❌ WRONG
Command::new("tool").arg(user_input).output()?;

// ✅ CORRECT - Use SafeCommand
use crate::security::command_guard::SafeCommand;
SafeCommand::new("allowed_command")?
    .arg("safe_arg")?
    .execute()
```

### Error Responses - Use ErrorSanitizer

```rust
// ❌ WRONG
Json(json!({ "error": e.to_string() }))
format!("Database error: {}", e)

// ✅ CORRECT
use crate::security::error_sanitizer::log_and_sanitize;
let sanitized = log_and_sanitize(&e, "context", None);
(StatusCode::INTERNAL_SERVER_ERROR, sanitized)
```

---

## Mandatory Code Patterns

### Format Strings - Inline Variables

```rust
// ❌ WRONG
format!("Hello {}", name)

// ✅ CORRECT
format!("Hello {name}")
```

### Self Usage in Impl Blocks

```rust
// ❌ WRONG
impl MyStruct {
    fn new() -> MyStruct { MyStruct { } }
}

// ✅ CORRECT
impl MyStruct {
    fn new() -> Self { Self { } }
}
```

### Derive Eq with PartialEq

```rust
// ❌ WRONG
#[derive(PartialEq)]
struct MyStruct { }

// ✅ CORRECT
#[derive(PartialEq, Eq)]
struct MyStruct { }
```

### Option Handling

```rust
// ✅ CORRECT
opt.unwrap_or(default)
opt.unwrap_or_else(|| compute_default())
opt.map_or(default, |x| transform(x))
```

### Chrono DateTime

```rust
// ❌ WRONG
date.with_hour(9).unwrap().with_minute(0).unwrap()

// ✅ CORRECT
date.with_hour(9).and_then(|d| d.with_minute(0)).unwrap_or(date)
```

---

## File Size Limits - MANDATORY

### Maximum 450 Lines Per File

When a file grows beyond this limit:

1. **Identify logical groups** - Find related functions
2. **Create subdirectory module** - e.g., `handlers/`
3. **Split by responsibility:**
   - `types.rs` - Structs, enums, type definitions
   - `handlers.rs` - HTTP handlers and routes
   - `operations.rs` - Core business logic
   - `utils.rs` - Helper functions
   - `mod.rs` - Re-exports and configuration
4. **Keep files focused** - Single responsibility
5. **Update mod.rs** - Re-export all public items

**NEVER let a single file exceed 450 lines - split proactively at 350 lines**

### Files Requiring Refactoring

There is no fixed list to work through - oversized files move as the code does. Measure the current set when you need it:

```bash
find src crates -name '*.rs' -exec wc -l {} + \
  | awk '$1 > 450 && $2 != "total"' | sort -rn
```

Standing rule: any file you touch should come out under 450 lines, or smaller than it was. Tracked in #1370.

---

## Database Standards

- **TABLES AND INDEXES ONLY** (no stored procedures, nothing, no views, no triggers, no functions)
- **JSON columns:** use TEXT with `_json` suffix
- **ORM:** Use diesel - no sqlx
- **Migrations:** Located in `botserver/migrations/`

---

## Frontend Rules

- **Use HTMX** - minimize JavaScript
- **NO external CDN** - all assets local
- **Server-side rendering** with Askama templates

---

## Key Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| axum | 0.7.5 | Web framework |
| diesel | 2.1 | PostgreSQL ORM |
| tokio | 1.41 | Async runtime |
| rhai | git | BASIC scripting |
| reqwest | 0.12 | HTTP client |
| serde | 1.0 | Serialization |
| askama | 0.12 | HTML Templates |

---

## CI/CD Workflow

When configuring CI/CD pipelines (e.g., Forgejo Actions):

- **Minimal Checkout**: Clone only the root `gb` and the `botlib` submodule. Do NOT recursively clone everything.
- **BotServer Context**: Replace the empty `botserver` directory with the current set of files being tested.

**Example Step:**
```yaml
- name: Setup Workspace
  run: |
    # 1. Clone only the root workspace configuration
    git clone --depth 1 <your-git-repo-url> workspace
    
    # 2. Setup only the necessary dependencies (botlib)
    cd workspace
    git submodule update --init --depth 1 botlib
    cd ..

    # 3. Inject current BotServer code
    rm -rf workspace/botserver
    mv botserver workspace/botserver
```

---

## Documentation

### Documentation Structure

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

### Additional Resources

- **[docs.generalbots.org](https://docs.generalbots.org)** - Full online documentation
- **[BotBook](../botbook)** - Local comprehensive guide with tutorials and examples
- **[API Reference](docs/api/README.md)** - REST and WebSocket endpoints
- **[BASIC Language](docs/reference/basic-language.md)** - Dialog scripting reference

---

## Related Projects

| Project | Description |
|---------|-------------|
| [botui](https://github.com/GeneralBots/botui) | Pure web UI (HTMX-based) |
| [botapp](https://github.com/GeneralBots/botapp) | Tauri desktop wrapper |
| [botlib](https://github.com/GeneralBots/botlib) | Shared Rust library |
| [botbook](https://github.com/GeneralBots/botbook) | Documentation |
| [bottemplates](https://github.com/GeneralBots/bottemplates) | Templates and examples |

---

## Security

- **MIT License** - Permissive open source
- **Self-hosted** - Your data stays on your infrastructure
- **Enterprise-grade** - 5+ years of stability
- **No vendor lock-in** - Open protocols and standards

Report security issues to: **security@generalbots.org**

---

## Contributing

We welcome contributions! Please read our contributing guidelines before submitting PRs.

### Contributors

<a href="https://github.com/generalbots/botserver/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=generalbots/botserver" />
</a>

---

## Remember

- **ZERO WARNINGS** - Fix every clippy warning
- **ZERO COMMENTS** - No comments, no doc comments
- **NO ALLOW IN CODE** - Configure exceptions in Cargo.toml only
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? or combinators (#1368)
- **MINIMIZE CLONES** - Avoid excessive allocations (#1369)
- **PARAMETERIZED SQL** - Never format! for queries
- **VALIDATE COMMANDS** - Never pass raw user input
- **INLINE FORMAT ARGS** - `format!("{name}")` not `format!("{}", name)`
- **USE SELF** - In impl blocks, use Self not type name
- **FILE SIZE LIMIT** - Max 450 lines per file, refactor at 350 lines
- **Version 6.2.0** - Do not change without approval
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## License

General Bot Copyright (c) generalbots.org. All rights reserved.  
Licensed under the **MIT License**.

This program is free software, released under the MIT License. You may use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, subject to the conditions in the LICENSE file. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

---

## Links

- **Website:** [generalbots.org](https://generalbots.org)
- **Documentation:** [docs.generalbots.org](https://docs.generalbots.org)
- **GitHub:** [github.com/GeneralBots/botserver](https://github.com/GeneralBots/botserver)
- **Stack Overflow:** Tag questions with `generalbots`
- **Video Tutorial:** [7 AI General Bots LLM Templates](https://www.youtube.com/watch?v=KJgvUPXi3Fw)

---

**General Bots Code Name:** [Guaribas](https://en.wikipedia.org/wiki/Guaribas)

> "No one should have to do work that can be done by a machine." - Roberto Mangabeira Unger
