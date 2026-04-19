# Module Structure

botserver is a single Rust crate (not a workspace) with multiple modules. The application is defined in `Cargo.toml` as the `botserver` crate, version 6.0.8.

## Main Entry Points

The primary entry point is `src/main.rs`, which starts the Axum web server and initializes all components. The public library interface in `src/lib.rs` exports all major modules for external use.

## Core Modules

The following modules are exported in `src/lib.rs` and comprise the core functionality:

### User & Bot Management

The `auth` module handles user authentication, password hashing using Argon2, and session token management. The `bot` module manages bot lifecycle, configuration, and runtime operations. The `session` module provides user session handling and state management across conversations.

### Conversation & Scripting

The `basic` module implements the BASIC-like scripting language interpreter for `.gbdialog` files. The `context` module manages conversation context and memory throughout user interactions. The `channels` module provides multi-channel support for web, voice, and various messaging platforms.

### Knowledge & AI

The `llm` module provides LLM provider integration for OpenAI and local models. The `llm_models` module contains model-specific implementations and configurations. The `nvidia` module offers NVIDIA GPU acceleration support for local inference.

### Infrastructure

The `bootstrap` module handles system initialization and the auto-bootstrap process. The `package_manager` module manages component installation and lifecycle. The `config` module provides application configuration and environment management. The `shared` module contains shared utilities, database models, and common types used throughout the codebase. The `web_server` module implements the Axum-based HTTP server and API endpoints.

### Features & Integration

The `automation` module provides scheduled tasks and event-driven triggers. The `drive_monitor` module handles file system monitoring and change detection. The `email` module provides email integration via IMAP and SMTP as a conditional feature. The `file` module handles file processing and operations. The `meet` module integrates video meeting functionality through LiveKit.

### Testing & Development

The `tests` module contains test utilities and test suites for validating functionality across the codebase.

## Internal Modules

Several directories exist in `src/` that are either internal implementations or not fully integrated into the public API.

The `api/` directory contains the `api/drive` subdirectory with drive-related API code. The `drive/` directory provides drive (S3-compatible) integration and vector database functionality through `vectordb.rs`. The `ui/` directory contains UI-related modules including `drive.rs`, `stream.rs`, `sync.rs`, and `local-sync.rs`. The `ui_tree/` directory provides UI tree structure functionality used in main.rs but not exported in lib.rs. The `prompt_manager/` directory stores the prompt library and is not a Rust module but contains `prompts.csv`. The `riot_compiler/` directory contains a Riot.js component compiler that exists but is currently unused. The `web_automation/` directory is an empty placeholder for future functionality.

## Dependency Management

All dependencies are managed through a single `Cargo.toml` at the project root.

The web framework layer uses `axum`, `tower`, and `tower-http` for HTTP handling. The async runtime is `tokio` for concurrent operations. Database access uses `diesel` for PostgreSQL and `redis` for cache component connectivity. AI and ML functionality relies on `qdrant-client` for vector database operations as an optional feature. Storage operations use `aws-sdk-s3` for drive and S3-compatible storage backends. Scripting uses `rhai` as the BASIC-like language runtime. Security features include `argon2` for password hashing and `aes-gcm` for encryption. Desktop support uses `tauri` as an optional feature.

## Feature Flags

The crate supports optional features for customizing builds:

```toml
[features]
default = ["desktop"]
vectordb = ["qdrant-client"]
email = ["imap"]
desktop = ["dep:tauri", "dep:tauri-plugin-dialog", "dep:tauri-plugin-opener"]
```

## Building

To build the project with different configurations:

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

Most modules follow a consistent structure with a `mod.rs` file containing the main module implementation and a `module_name.test.rs` file for module-specific tests. Some modules have additional submodules or specialized files such as `drive/vectordb.rs` and `ui/drive.rs` for feature-specific functionality.