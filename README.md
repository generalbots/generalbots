
<p align="center"><img src="logo.svg" alt="General Bots" width="200"></p>

# General Bots

General Bots is an AI automation platform written in Rust. It gives you one workspace for building AI-powered bots, web interfaces, desktop applications and the integrations that connect them. The workspace is modular: each subproject can be developed and deployed on its own, while shared libraries and standards keep them consistent.

Documentation lives at **[docs.generalbots.org](https://docs.generalbots.org)**, and the **[BotBook](./botbook)** holds the local guides, API references and tutorials.

---

## Workspace Structure

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

## BotServer Component Architecture

### Infrastructure Components

BotServer installs, configures and runs its own infrastructure on first start, so none of these services need to be launched by hand. On boot it starts the stack, connects to Vault, loads each service's credentials from the `bot_configuration` table, and authenticates with them before serving traffic.

```
botserver starts
    ↓
launches PostgreSQL, MinIO, Valkey, Qdrant
    ↓
connects to Vault, loads service credentials
    ↓
authenticates against every service
    ↓
ready to handle requests
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

### Component Installation System

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

### Starting BotServer

Reach for `./restart.sh` in development. It stops lingering processes, builds botserver and botui in sequence so the builds cannot race, starts both with logging, and prints the process IDs and URLs.

```bash
./restart.sh
```

Let BotServer own the infrastructure: do not launch Vault, PostgreSQL, MinIO or the rest yourself, do not run `cargo run` on botserver directly, and do not hand-edit files under `botserver-stack/`.

For a release build:

```bash
cargo build --release -p botserver
RUST_LOG=info ./target/release/botserver --noconsole 2>&1 | tee botserver.log &
```

Under systemd or another supervisor, point `ExecStart` at the built binary:

```ini
ExecStart=/opt/gbo/bin/botserver --noconsole
```

### Component Communication

All components communicate through internal networks with mTLS:
- **Vault**: mTLS for secrets access
- **PostgreSQL**: TLS encrypted connections
- **MinIO**: TLS with client certificates
- **Zitadel**: mTLS for user authentication

Certificates auto-generated in: `botserver-stack/conf/system/certificates/`

### Component Status

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

## Component Dependency Graph

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

## Module Responsibility Matrix

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

### Starting Locally

```bash
./restart.sh
```

One command is enough. The script stops anything already running, builds botserver and botui in order, starts botserver (which brings up PostgreSQL, Vault, MinIO, Valkey and Qdrant and authenticates against them), then starts botui as the proxy in front of it.

- Web UI: http://localhost:3000
- API: http://localhost:9000

### Monitor & Debug

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

### Manual Startup (If needed)

Only reach for this if `restart.sh` fails.

```bash
cd botserver && cargo run -- --noconsole > ../botserver.log 2>&1 &
cd botui && BOTSERVER_URL="http://localhost:9000" cargo run > ../botui.log 2>&1 &
```

### Stop Servers

```bash
pkill -f botserver; pkill -f botui
```

### Common Issues

If Vault fails to initialise, clear the stale state and restart:
```bash
rm -rf botserver-stack/data/vault botserver-stack/conf/vault/init.json && ./restart.sh
```

If a port is already taken, find and free it:
```bash
lsof -ti:9000 | xargs kill -9
lsof -ti:3000 | xargs kill -9
```

### Where the Stack Lives

BotServer starts and manages every infrastructure service itself, and keeps them inside the repository rather than in system-wide installs. Do not install or point at a global PostgreSQL, Redis or Vault.

| What | Where |
|------|-------|
| Binaries | `botserver-stack/bin/` |
| Configuration | `botserver-stack/conf/` |
| Data | `botserver-stack/data/` |
| Logs | `botserver-stack/logs/` |
| Credentials | Vault, read by BotServer at startup |

Service errors surface in `botserver-stack/logs/<service>/`, which is the first place to look.

### Deploying UI Files

**Embedded UI (recommended for production)**

The `embed-ui` feature compiles the UI straight into the botui binary, so there are no separate files to deploy:

```bash
cargo build --release -p botui --features embed-ui
```

You end up with one self-contained binary, a faster start because nothing is read from disk, and a smaller attack surface.

**Filesystem (development)**

Development builds read the UI from `botui/ui/suite/` on disk, so edits are picked up on refresh.

**Manual deployment (legacy)**

Only if you must ship the UI files separately:

```bash
./botserver/deploy/deploy-ui.sh /opt/gbo
ls -la /opt/gbo/bin/ui/suite/index.html
```

See `botserver/deploy/README.md` for deployment scripts.

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

## AI Agent Guidelines

> **For LLM instructions, coding rules, security directives, testing workflows, and error handling patterns, see [AGENTS.md](./AGENTS.md).**

---

## Glossary

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



## UI Architecture (botui + botserver)

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

## Frontend Standards

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
botui/ui/suite/js/vendor/
├── htmx.min.js
├── htmx-ws.js
├── marked.min.js
└── gsap.min.js
```

---

## Project-Specific Guidelines

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
| `botserver/src/tasks/PROMPT.md` | AutoTask LLM executor |
| `botserver/crates/botautotask/src/designer_ai.rs` | App generation |

---

## Documentation

For complete documentation, guides, and API references:

- **[docs.generalbots.org](https://docs.generalbots.org)** - Full online documentation
- **[BotBook](./botbook)** - Local comprehensive guide with tutorials and examples
- **[General Bots Repository](https://github.com/GeneralBots/BotServer)** - Main project repository

---

## Technical Debt

### Critical Issues to Address

Counts drift, so each item links to a tracked issue rather than hardcoding a number here. Measure the current state with the command in the issue.

1. **Error Handling Debt**: `.unwrap()`/`.expect()` in production code - #1368
2. **Performance Debt**: excessive `clone()`/`to_string()` calls - #1369
3. **File Size Debt**: files exceeding the 450-line limit - #1370
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

## Repository Layout

This is one repository. Every subproject lives in it as an ordinary directory - there are no git submodules - so a root commit covers workspace files and subproject code together.

There are two remotes:

| Remote | Host | Purpose |
|--------|------|---------|
| `origin` | github.com/generalbots/generalbots | Public mirror |
| `alm` | alm.pragmatismo.com.br/GeneralBots/BotServer | Primary; pushing here triggers CI/CD |

```bash
git push origin main
git push alm main
```

Pushing to `alm` is not routine: it drives the pipeline that builds and deploys to production, so confirm before doing it.

---

## Development Workflow

1. Read this README for the workspace layout.
2. Read **[AGENTS.md](./AGENTS.md)** for coding rules and workflows.
3. Before creating any `.md` file, search `botbook/` for existing documentation.
4. Read the relevant `<project>/README.md` for project-specific rules.
5. Run diagnostics and fix every warning; never silence them with `#[allow()]`.

---

## License

General Bots is released under the MIT License. Each subproject carries its own LICENSE file:

| Project | License file |
|---------|--------------|
| Root | `LICENSE` |
| botserver | `botserver/LICENSE` |
| botui | `botui/LICENSE` |
| botapp | `botapp/LICENSE` |
| botlib | `botlib/LICENSE.txt` |
| botbook | `botbook/LICENSE` |
| botplugin | `botplugin/LICENSE` |
