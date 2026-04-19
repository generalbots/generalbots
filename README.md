# General Bots Workspace

## ⚠️ CRITICAL SECURITY WARNING

**NEVER CREATE FILES WITH SECRETS IN THE REPOSITORY ROOT**

Secret files MUST be placed in `/tmp/` only:
- ✅ `/tmp/vault-token-gb` - Vault root token
- ✅ `/tmp/vault-unseal-key-gb` - Vault unseal key
- ❌ `vault-unseal-keys` - FORBIDDEN (tracked by git)
- ❌ `start-and-unseal.sh` - FORBIDDEN (contains secrets)

**Files added to .gitignore:** `vault-unseal-keys`, `start-and-unseal.sh`, `vault-token-*`

**Why `/tmp/`?**
- Cleared on reboot (ephemeral)
- Not tracked by git
- Standard Unix security practice
- Prevents accidental commits

---


**Version:** 6.3.0  
**Type:** Rust Workspace (Monorepo with Independent Subproject Repos)

---

## Overview

General Bots is a comprehensive automation platform built with Rust, providing a unified workspace for building AI-powered bots, web interfaces, desktop applications, and integration tools. The workspace follows a modular architecture with independent subprojects that can be developed and deployed separately while sharing common libraries and standards.

For comprehensive documentation, see **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** or the **[BotBook](./botbook)** for detailed guides, API references, and tutorials.

---

## 📁 Workspace Structure

| Crate | Purpose | Port | Tech Stack |
|-------|---------|------|------------|
| **botserver** | Main API server, business logic | 9000 | Axum, Diesel, Rhai BASIC |
| **botui** | Web UI server (dev) + proxy | 3000 | Axum, HTML/HTMX/CSS |
| **botapp** | Desktop app wrapper | - | Tauri 2 |
| **botlib** | Shared library | - | Core types, errors |
| **botbook** | Documentation | - | mdBook |
| **bottest** | Integration tests | - | tokio-test |
| **botdevice** | IoT/Device support | - | Rust |
| **botmodels** | Data models visualization | - | - |
| **botplugin** | Browser extension | - | JS |

### Key Paths
- **Binary:** `target/debug/botserver`
- **Run from:** `botserver/` directory
- **Env file:** `botserver/.env`
- **Stack:** `botserver-stack/`
- **UI Files:** `botui/ui/suite/`
- **Local Bot Data:** `/opt/gbo/data/` (place `.gbai` packages here)

### Local Bot Data Directory

Place local bot packages in `/opt/gbo/data/` for automatic loading and monitoring:

**Directory Structure:**
```
/opt/gbo/data/
└── mybot.gbai/
    ├── mybot.gbdialog/
    │   ├── start.bas
    │   └── main.bas
    └── mybot.gbot/
        └── config.csv
```

**Features:**
- **Auto-loading:** Bots automatically mounted on server startup
- **Auto-compilation:** `.bas` files compiled to `.ast` on change
- **Auto-creation:** New bots automatically added to database
- **Hot-reload:** Changes trigger immediate recompilation
- **Monitored by:** LocalFileMonitor and ConfigWatcher services

**Usage:**
1. Create bot directory structure in `/opt/gbo/data/`
2. Add `.bas` files to `<bot_name>.gbai/<bot_name>.gbdialog/`
3. Server automatically detects and loads the bot
4. Optional: Add `config.csv` for bot configuration

---

## 🏗️ BotServer Component Architecture

### 🔧 Infrastructure Components (Auto-Managed)

BotServer automatically installs, configures, and manages all infrastructure components on first run. **DO NOT manually start these services** - BotServer handles everything.

**Automatic Service Lifecycle:**
1. **Start**: When botserver starts, it automatically launches all infrastructure components (PostgreSQL, Vault, MinIO, Valkey, Qdrant, etc.)
2. **Credentials**: BotServer retrieves all service credentials (passwords, tokens, API keys) from Vault
3. **Connection**: BotServer uses these credentials to establish secure connections to each service
4. **Query**: All database queries, cache operations, and storage requests are authenticated using Vault-managed credentials

**Credential Flow:**
```
botserver starts
    ↓
Launch PostgreSQL, MinIO, Valkey, Qdrant
    ↓
Connect to Vault
    ↓
Retrieve service credentials (from database)
    ↓
Authenticate with each service using retrieved credentials
    ↓
Ready to handle requests
```

| Component | Purpose | Port | Binary Location | Credentials From |
|-----------|---------|------|-----------------|------------------|
| **Vault** | Secrets management | 8200 | `botserver-stack/bin/vault/vault` | Auto-unsealed |
| **PostgreSQL** | Primary database | 5432 | `botserver-stack/bin/tables/bin/postgres` | Vault → database |
| **MinIO** | Object storage (S3-compatible) | 9000/9001 | `botserver-stack/bin/drive/minio` | Vault → database |
| **Zitadel** | Identity/Authentication | 8300 | `botserver-stack/bin/directory/zitadel` | Vault → database |
| **Qdrant** | Vector database (embeddings) | 6333 | `botserver-stack/bin/vector_db/qdrant` | Vault → database |
| **Valkey** | Cache/Queue (Redis-compatible) | 6379 | `botserver-stack/bin/cache/valkey-server` | Vault → database |
| **Llama.cpp** | Local LLM server | 8081 | `botserver-stack/bin/llm/build/bin/llama-server` | Vault → database |

### 📦 Component Installation System

Components are defined in `botserver/3rdparty.toml` and managed by the `PackageManager` (`botserver/src/core/package_manager/`):

```toml
[components.cache]
name = "Valkey Cache (Redis-compatible)"
url = "https://github.com/valkey-io/valkey/archive/refs/tags/8.0.2.tar.gz"
filename = "valkey-8.0.2.tar.gz"

[components.llm]
name = "Llama.cpp Server"
url = "https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-x64.zip"
filename = "llama-b7345-bin-ubuntu-x64.zip"
```

**Installation Flow:**
1. **Download:** Components downloaded to `botserver-installers/` (cached)
2. **Extract/Build:** Binaries placed in `botserver-stack/bin/<component>/`
3. **Configure:** Config files generated in `botserver-stack/conf/<component>/`
4. **Start:** Components started with proper TLS certificates
5. **Monitor:** Components monitored and auto-restarted if needed

**Bootstrap Process:**
- First run: Full bootstrap (downloads, installs, configures all components)
- Subsequent runs: Only starts existing components (uses cached binaries)
- Config stored in: `botserver-stack/conf/system/bootstrap.json`

### 🚀 PROPER STARTUP PROCEDURES

**❌ FORBIDDEN:**
- NEVER manually start infrastructure components (Vault, PostgreSQL, MinIO, etc.)
- NEVER run `cargo run` or `cargo build` for botserver directly without ./restart.sh
- NEVER modify botserver-stack/ files manually (use botserver API)

**✅ REQUIRED:**

**Option 1: Development (Recommended)**
```bash
./restart.sh
```
This script:
1. Kills existing processes cleanly
2. Builds botserver and botui sequentially (no race conditions)
3. Starts botserver in background with logging to `botserver.log`
4. Starts botui in background with logging to `botui.log`
5. Shows process IDs and access URLs

**Option 2: Production/Release**
```bash
# Build release binary first
cargo build --release -p botserver

# Start with release binary
RUST_LOG=info ./target/release/botserver --noconsole 2>&1 | tee botserver.log &
```

**Option 3: Using Exec (Systemd/Supervisord)**
```bash
# In systemd service or similar
ExecStart=/home/rodriguez/src/gb/target/release/botserver --noconsole
```

### 🔒 Component Communication

All components communicate through internal networks with mTLS:
- **Vault**: mTLS for secrets access
- **PostgreSQL**: TLS encrypted connections
- **MinIO**: TLS with client certificates
- **Zitadel**: mTLS for user authentication

Certificates auto-generated in: `botserver-stack/conf/system/certificates/`

### 📊 Component Status

Check component status anytime:
```bash
# Check if all components are running
ps aux | grep -E "vault|postgres|minio|zitadel|qdrant|valkey" | grep -v grep

# View component logs
tail -f botserver-stack/logs/vault/vault.log
tail -f botserver-stack/logs/tables/postgres.log
tail -f botserver-stack/logs/drive/minio.log

# Test component connectivity
cd botserver-stack/bin/vault && ./vault status
cd botserver-stack/bin/cache && ./valkey-cli ping
```

---

## 🏗️ Component Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client Layer                            │
├─────────────────────────────────────────────────────────────────┤
│  botui (Web UI)    │  botapp (Desktop)   │  botplugin (Ext)   │
│  HTMX + Axum       │  Tauri 2 Wrapper    │  Browser Extension  │
└─────────┬───────────────────┬──────────────────┬─────────────────┘
          │                   │                  │
          └───────────────────┼──────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   botlib          │
                    │  (Shared Types)   │
                    └─────────┬─────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
    ┌─────▼─────┐      ┌─────▼─────┐      ┌─────▼─────┐
    │ botserver │      │ bottest   │      │ botdevice  │
    │ API Core  │      │ Tests     │      │ IoT/Device │
    └───────────┘      └───────────┘      └───────────┘
```

### Dependency Rules

| Crate | Depends On | Why |
|-------|-----------|-----|
| **botserver** | botlib | Shared types, error handling, models |
| **botui** | botlib | Common data structures, API client |
| **botapp** | botlib | Shared types, desktop-specific utilities |
| **bottest** | botserver, botlib | Integration testing with real components |
| **botdevice** | botlib | Device types, communication protocols |
| **botplugin** | - | Standalone browser extension (JS) |

**Key Principle:** `botlib` contains ONLY shared types and utilities. No business logic. All business logic lives in botserver or specialized crates.

## 📦 Module Responsibility Matrix

### botserver/src/ Module Structure

| Module | Responsibility | Key Types | Dependencies |
|--------|---------------|-----------|--------------|
| **core/bot/** | WebSocket handling, bot orchestration | BotOrchestrator, UserMessage | basic, shared |
| **core/session/** | Session management, conversation history | SessionManager, UserSession | shared, database |
| **basic/** | Rhai BASIC scripting engine | ScriptService, Engine | rhai, keywords |
| **basic/keywords/** | BASIC keyword implementations (TALK, HEAR, etc.) | Keyword functions | basic, state |
| **llm/** | Multi-vendor LLM API integration | LLMClient, ModelConfig | reqwest, shared |
| **drive/** | S3 file storage and monitoring | DriveMonitor, compile_tool | s3, basic |
| **security/** | Security guards (command, SQL, error) | SafeCommand, ErrorSanitizer | state |
| **shared/** | Database models, schema definitions | Bot, Session, Message | diesel |
| **tasks/** | AutoTask execution system | TaskRunner, TaskScheduler | core/basic |
| **auto_task/** | LLM-powered app generation | AppGenerator, template engine | llm, tasks |
| **learn/** | Knowledge base management | KBManager, vector storage | database, drive |
| **attendance/** | LLM-assisted customer service | AttendantManager, queue | core/bot |

### Data Flow Patterns

```
1. User Request Flow:
   Client → WebSocket → botserver/src/core/bot/mod.rs
                          ↓
                    BotOrchestrator::stream_response()
                          ↓
              ┌───────────┴───────────┐
              │                       │
         LLM API Call            Script Execution
         (llm/mod.rs)            (basic/mod.rs)
              │                       │
              └───────────┬───────────┘
                          ↓
                    Response → WebSocket → Client

2. File Sync Flow:
   S3 Drive → drive_monitor/src/drive_monitor/mod.rs
                          ↓
                    Download .bas files
                          ↓
              compile_file() → Generate .ast
                          ↓
              Store in ./work/{bot_name}.gbai/

3. Script Execution Flow:
   .bas file → ScriptService::compile()
                    ↓
              preprocess_basic_script()
                    ↓
              engine.compile() → AST
                    ↓
              ScriptService::run() → Execute
                    ↓
              TALK commands → WebSocket messages
```

### Common Architectural Patterns

| Pattern | Where Used | Purpose |
|---------|-----------|---------|
| **State via Arc<AppState>** | All handlers | Shared async state (DB, cache, config) |
| **Extension(state) extractor** | Axum handlers | Inject Arc<AppState> into route handlers |
| **tokio::spawn_blocking** | CPU-intensive tasks | Offload blocking work from async runtime |
| **WebSocket with split()** | Real-time comms | Separate sender/receiver for WS streams |
| **ErrorSanitizer for responses** | All HTTP errors | Prevent leaking sensitive info in errors |
| **SafeCommand for execution** | Command running | Whitelist-based command validation |
| **Rhai for scripting** | BASIC interpreter | Embeddable scripting language |
| **Diesel ORM** | Database access | Type-safe SQL queries |
| **Redis for cache** | Session data | Fast key-value storage |
| **S3 for storage** | File system | Scalable object storage |

---

## Quick Start

### 🚀 Simple Startup (ALWAYS USE restart.sh)

```bash
./restart.sh
```

**⚠️ CRITICAL: ALWAYS use restart.sh - NEVER start servers individually!**

The script handles BOTH servers properly:
1. Stop existing processes cleanly
2. Build botserver and botui sequentially (no race conditions)
3. Start botserver in background → **automatically starts all infrastructure services (PostgreSQL, Vault, MinIO, Valkey, Qdrant)**
4. BotServer retrieves credentials from Vault and authenticates with all services
5. Start botui in background → proxy to botserver
6. Show process IDs and monitoring commands

**Infrastructure services are fully automated - no manual configuration required!**

**Monitor startup:**
```bash
tail -f botserver.log botui.log
```

**Access:**
- Web UI: http://localhost:3000
- API: http://localhost:9000

### 📊 Monitor & Debug

```bash
tail -f botserver.log botui.log
```

**Quick status check:**
```bash
ps aux | grep -E "botserver|botui" | grep -v grep
```

**Quick error scan:**
```bash
grep -E " E |W |CLIENT:" botserver.log | tail -20
```

### 🔧 Manual Startup (If needed)

**⚠️ WARNING: Only use if restart.sh fails. Always prefer restart.sh!**

```bash
cd botserver && cargo run -- --noconsole > ../botserver.log 2>&1 &
cd botui && BOTSERVER_URL="http://localhost:9000" cargo run > ../botui.log 2>&1 &
```

### 🛑 Stop Servers

```bash
pkill -f botserver; pkill -f botui
```

### ⚠️ Common Issues

**Vault init error?** Delete stale state:
```bash
rm -rf botserver-stack/data/vault botserver-stack/conf/vault/init.json && ./restart.sh
```

**Port in use?** Find and kill:
```bash
lsof -ti:9000 | xargs kill -9
lsof -ti:3000 | xargs kill -9
```

**⚠️ IMPORTANT: Stack Services Management**
All infrastructure services (PostgreSQL, Vault, Redis, Qdrant, MinIO, etc.) are **automatically started by botserver** and managed through `botserver-stack/` directory, NOT global system installations. The system uses:

- **Local binaries:** `botserver-stack/bin/` (PostgreSQL, Vault, Redis, etc.)
- **Configurations:** `botserver-stack/conf/`
- **Data storage:** `botserver-stack/data/`
- **Service logs:** `botserver-stack/logs/` (check here for troubleshooting)
- **Credentials:** Stored in Vault, retrieved by botserver at startup

**Do NOT install or reference global PostgreSQL, Redis, or other services.** When botserver starts, it automatically:
1. Launches all required stack services
2. Connects to Vault
3. Retrieves credentials from the `bot_configuration` database table
4. Authenticates with each service using retrieved credentials
5. Begins handling requests with authenticated connections

If you encounter service errors, check the individual service logs in `./botserver-stack/logs/[service]/` directories.

### UI File Deployment - Production Options

**Option 1: Embedded UI (Recommended for Production)**

The `embed-ui` feature compiles UI files directly into the botui binary, eliminating the need for separate file deployment:

```bash
# Build with embedded UI files
cargo build --release -p botui --features embed-ui

# The binary now contains all UI files - no additional deployment needed!
# The botui binary is self-contained and production-ready
```

**Benefits of embed-ui:**
- ✅ Single binary deployment (no separate UI files)
- ✅ Faster startup (no filesystem access)
- ✅ Smaller attack surface
- ✅ Simpler deployment process

**Option 2: Filesystem Deployment (Development Only)**

For development, UI files are served from the filesystem:

```bash
# UI files must exist at botui/ui/suite/
# This is automatically available in development builds
```

**Option 3: Manual File Deployment (Legacy)**

If you need to deploy UI files separately (not recommended):

```bash
# Deploy UI files to production location
./botserver/deploy/deploy-ui.sh /opt/gbo

# Verify deployment
ls -la /opt/gbo/bin/ui/suite/index.html
```

See `botserver/deploy/README.md` for deployment scripts.

### Start Both Servers (Automated)
```bash
# Use restart script (RECOMMENDED)
./restart.sh
```

### Start Both Servers (Manual)
```bash
# Terminal 1: botserver
cd botserver && cargo run -- --noconsole

# Terminal 2: botui  
cd botui && BOTSERVER_URL="http://localhost:9000" cargo run
```

### Build Commands
```bash
# Check single crate
cargo check -p botserver

# Build workspace
cargo build

# Run tests
cargo test -p bottest
```

---

## 🤖 AI Agent Guidelines

> **For LLM instructions, coding rules, security directives, testing workflows, and error handling patterns, see [AGENTS.md](./AGENTS.md).**

---

## 📖 Glossary

| Term | Definition | Usage |
|------|-----------|-------|
| **Bot** | AI agent with configuration, scripts, and knowledge bases | Primary entity in system |
| **Session** | Single conversation instance between user and bot | Stored in `sessions` table |
| **Dialog** | Collection of BASIC scripts (.bas files) for bot logic | Stored in `{bot_name}.gbdialog/` |
| **Tool** | Reusable function callable by LLM | Defined in .bas files, compiled to .ast |
| **Knowledge Base (KB)** | Vector database of documents for semantic search | Managed in `learn/` module |
| **Scheduler** | Time-triggered task execution | Cron-like scheduling in BASIC scripts |
| **Drive** | S3-compatible storage for files | Abstracted in `drive/` module |
| **Rhai** | Embedded scripting language for BASIC dialect | Rhai engine in `basic/` module |
| **WebSocket Adapter** | Component that sends messages to connected clients | `web_adapter` in state |
| **AutoTask** | LLM-generated task automation system | In `auto_task/` and `tasks/` modules |
| **Orchestrator** | Coordinates LLM, tools, KBs, and user input | `BotOrchestrator` in `core/bot/` |

---



## 🖥️ UI Architecture (botui + botserver)

### Two Servers During Development

| Server | Port | Purpose |
|--------|------|---------|
| **botui** | 3000 | Serves UI files + proxies API to botserver |
| **botserver** | 9000 | Backend API + embedded UI fallback |

### How It Works

```
Browser → localhost:3000 → botui (serves HTML/CSS/JS)
                        → /api/* proxied to botserver:9000
                        → /suite/* served from botui/ui/suite/
```

### Adding New Suite Apps

1. Create folder: `botui/ui/suite/<appname>/`
2. Add to `SUITE_DIRS` in `botui/src/ui_server/mod.rs`
3. Rebuild botui: `cargo build -p botui`
4. Add menu entry in `botui/ui/suite/index.html`

### Hot Reload

- **UI files (HTML/CSS/JS)**: Edit & refresh browser (no restart)
- **botui Rust code**: Rebuild + restart botui
- **botserver Rust code**: Rebuild + restart botserver

### Production (Single Binary)

When `botui/ui/suite/` folder not found, botserver uses **embedded UI** compiled into binary via `rust-embed`.

---

## 🎨 Frontend Standards

### HTMX-First Approach
- Use HTMX to minimize JavaScript
- Server returns HTML fragments, not JSON
- Use `hx-get`, `hx-post`, `hx-target`, `hx-swap`
- WebSocket via htmx-ws extension

### Local Assets Only - NO CDN
```html
<!-- ✅ CORRECT -->
<script src="js/vendor/htmx.min.js"></script>

<!-- ❌ WRONG -->
<script src="https://unpkg.com/htmx.org@1.9.10"></script>
```

### Vendor Libraries Location
```
ui/suite/js/vendor/
├── htmx.min.js
├── htmx-ws.js
├── marked.min.js
└── gsap.min.js
```

---

## 📋 Project-Specific Guidelines

Each crate has its own README.md with specific guidelines:

| Crate | README.md Location | Focus |
|-------|-------------------|-------|
| botserver | `botserver/README.md` | API, security, Rhai BASIC |
| botui | `botui/README.md` | UI, HTMX, CSS design system |
| botapp | `botapp/README.md` | Tauri, desktop features |
| botlib | `botlib/README.md` | Shared types, errors |
| botbook | `botbook/README.md` | Documentation, mdBook |
| bottest | `bottest/README.md` | Test infrastructure |

### Special Prompts
| File | Purpose |
|------|---------|
| `botserver/src/tasks/README.md` | AutoTask LLM executor |
| `botserver/src/auto_task/APP_GENERATOR_PROMPT.md` | App generation |

---

## 📚 Documentation

For complete documentation, guides, and API references:

- **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** - Full online documentation
- **[BotBook](./botbook)** - Local comprehensive guide with tutorials and examples
- **[General Bots Repository](https://github.com/GeneralBots/BotServer)** - Main project repository

---

## 🔧 Immediate Technical Debt

### Critical Issues to Address

1. **Error Handling Debt**: 955 instances of `unwrap()`/`expect()` in production code
2. **Performance Debt**: 12,973 excessive `clone()`/`to_string()` calls
3. **File Size Debt**: 7 files exceed 450 lines (largest: 3220 lines)
4. **Test Coverage**: Missing integration tests for critical paths
5. **Documentation**: Missing inline documentation for complex algorithms

### Weekly Maintenance Tasks

```bash
# Check for duplicate dependencies
cargo tree --duplicates

# Remove unused dependencies  
cargo machete

# Check binary size
cargo build --release && ls -lh target/release/botserver

# Performance profiling
cargo bench

# Security audit
cargo audit
```

---

## Git Structure

**Note:** Each subproject has its own git repository. This root repository only tracks workspace-level files:

- `Cargo.toml` - Workspace configuration
- `README.md` - This file
- `.gitignore` - Ignore patterns
- `ADDITIONAL-SUGGESTIONS.md` - Enhancement ideas
- `TODO-*.md` - Task tracking files

Subprojects (botapp, botserver, botui, etc.) are **independent repositories referenced as git submodules**.

### ⚠️ CRITICAL: Submodule Push Workflow

When making changes to any submodule (botserver, botui, botlib, etc.):

1. **Commit and push changes within the submodule directory:**
   ```bash
   cd botserver
   git add .
   git commit -m "Your changes"
   git push pragmatismo main
   git push github main
   ```

2. **Update the global gb repository submodule reference:**
   ```bash
   cd ..  # Back to gb root
   git add botserver
   git commit -m "Update botserver submodule to latest commit"
   git push pragmatismo main
   git push github main
   ```

**Failure to push the global gb repository will cause submodule changes to not trigger CI/CD pipelines.**

Both repositories must be pushed for changes to take effect in production.

---

## Development Workflow

1. Read this README.md (workspace structure)
2. Read **[AGENTS.md](./AGENTS.md)** (coding rules & workflows)
3. **BEFORE creating any .md file, search botbook/ for existing documentation**
4. Read `<project>/README.md` (project-specific rules)
5. Use diagnostics tool to check warnings
6. Fix all warnings with full file rewrites
7. Verify with diagnostics after each file
8. Never suppress warnings with `#[allow()]`

---



## License

See individual project repositories for license information.
