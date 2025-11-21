# Module Structure

BotServer is a single Rust crate (not a workspace) with multiple modules. The application is defined in `Cargo.toml` as the `botserver` crate, version 6.0.8.

## Main Entry Points

- **`src/main.rs`** - Application entry point, starts the Axum web server and initializes all components
- **`src/lib.rs`** - Public library interface, exports all major modules

## Core Modules

The following modules are exported in `src/lib.rs` and comprise the core functionality:

### User & Bot Management
- **`auth`** - User authentication, password hashing (Argon2), session token management
- **`bot`** - Bot lifecycle, configuration, and management
- **`session`** - User session handling and state management

### Conversation & Scripting
- **`basic`** - BASIC-like scripting language interpreter for `.gbdialog` files
- **`context`** - Conversation context and memory management
- **`channels`** - Multi-channel support (web, voice, messaging platforms)

### Knowledge & AI
- **`llm`** - LLM provider integration (OpenAI, local models)
- **`llm_models`** - Model-specific implementations and configurations
- **`nvidia`** - NVIDIA GPU acceleration support

### Infrastructure
- **`bootstrap`** - System initialization and auto-bootstrap process
- **`package_manager`** - Component installation and lifecycle management
- **`config`** - Application configuration and environment management
- **`shared`** - Shared utilities, database models, and common types
- **`web_server`** - Axum-based HTTP server and API endpoints

### Features & Integration
- **`automation`** - Scheduled tasks and event-driven triggers
- **`drive_monitor`** - File system monitoring and change detection
- **`email`** - Email integration (IMAP/SMTP) - conditional feature
- **`file`** - File handling and processing
- **`meet`** - Video meeting integration (LiveKit)

### Testing & Development
- **`tests`** - Test utilities and test suites

## Internal Modules

The following directories exist in `src/` but are either internal implementations or not fully integrated:

- **`api/`** - Contains `api/drive` subdirectory with drive-related API code
- **`drive/`** - MinIO/S3 integration and vector database (`vectordb.rs`)
- **`ui/`** - UI-related modules (`drive.rs`, `stream.rs`, `sync.rs`, `local-sync.rs`)
- **`ui_tree/`** - UI tree structure (used in main.rs but not exported in lib.rs)
- **`prompt_manager/`** - Prompt library storage (not a Rust module, contains `prompts.csv`)
- **`riot_compiler/`** - Riot.js component compiler (exists but unused)
- **`web_automation/`** - Empty directory (placeholder for future functionality)

## Dependency Management

All dependencies are managed through a single `Cargo.toml` at the project root. Key dependencies include:

- **Web Framework**: `axum`, `tower`, `tower-http`
- **Async Runtime**: `tokio`
- **Database**: `diesel` (PostgreSQL), `redis` (cache)
- **AI/ML**: `qdrant-client` (vector DB, optional feature)
- **Storage**: `aws-sdk-s3` (MinIO/S3 compatible)
- **Scripting**: `rhai` (BASIC-like language runtime)
- **Security**: `argon2` (password hashing), `aes-gcm` (encryption)
- **Desktop**: `tauri` (optional desktop feature)

## Feature Flags

The crate supports optional features:

```toml
[features]
default = ["desktop"]
vectordb = ["qdrant-client"]
email = ["imap"]
desktop = ["dep:tauri", "dep:tauri-plugin-dialog", "dep:tauri-plugin-opener"]
```

## Building

To build the project:

```bash
# Standard build
cargo build --release

# Build without desktop features
cargo build --release --no-default-features

# Build with vector database support
cargo build --release --features vectordb

# Build with all features
cargo build --release --all-features
```

## Module Organization Pattern

Most modules follow this structure:

```
src/module_name/
├── mod.rs              # Main module implementation
└── module_name.test.rs # Module-specific tests
```

Some modules have additional submodules or specialized files (e.g., `drive/vectordb.rs`, `ui/drive.rs`).