# Service Layer

BotServer's service layer is organized into functional modules that handle specific aspects of the platform. Each module encapsulates related functionality and provides a clear API for interaction with other parts of the system. This chapter describes each service module and its responsibilities within the overall architecture.


## Core Service Modules

### Authentication and Security

The `auth` module provides secure user authentication and session management throughout the platform. Password hashing uses the Argon2 algorithm for secure password storage that resists both CPU and GPU-based attacks. Session token generation creates and validates unique tokens for maintaining authenticated state. User verification authenticates users against the database using stored credentials. Bot authentication manages bot-level authentication for API access, allowing bots to make authenticated requests to external services.

The module's key responsibilities include hashing passwords with Argon2 before storage, generating cryptographically secure session tokens, validating user credentials during login, and managing the complete session lifecycle from creation through expiration.

### Bot Management

The `bot` module handles bot lifecycle and configuration throughout the system. Bot creation initializes new bot instances with their required components. Configuration management loads and applies bot settings from config.csv files. Bot state tracking monitors bot status and health for operational awareness. Multi-tenant support isolates bots by tenant to prevent data leakage between organizations.

This module creates and deletes bot instances, loads bot configuration from the database, manages bot lifecycle including start, stop, and restart operations, and associates bots with users and sessions for proper isolation.

### Session Management

The `session` module maintains user conversation state across interactions. Session storage persists conversation context to both cache and database. State management tracks user progress through dialogs and remembers variable values. Session cleanup removes expired sessions to free resources. Multi-user support isolates sessions by user to ensure privacy.

The module creates new sessions when users connect, stores and retrieves session variables, maintains conversation history for context, and cleans up abandoned sessions after timeout periods.


## Conversation and Scripting Services

### BASIC Interpreter

The `basic` module implements the BASIC-like scripting language for `.gbdialog` files. Script parsing reads BASIC dialog scripts and converts them to executable form. The execution engine powered by the Rhai scripting engine runs the parsed scripts. Keyword implementation provides custom keywords like TALK, HEAR, and LLM for bot functionality. Variable management handles script variables and maintains execution context across statements.

This module loads and parses `.gbdialog` scripts from bot packages, executes BASIC commands in sequence, provides custom keywords that extend the language for bot functionality, and manages script execution context including variables and flow control state.

### Context Management

The `context` module manages conversation context and memory for LLM interactions. Conversation history storage maintains the message history for each session. Context retrieval loads relevant context for LLM calls based on the current query. Memory management limits context size to fit within model token limits. Context compaction summarizes old conversations to preserve meaning while reducing tokens.

The module appends messages to conversation history as they occur, retrieves appropriate context for LLM queries, implements context window management to stay within limits, and provides context to knowledge base queries for improved relevance.

### Channel Abstraction

The `channels` module provides a unified interface for multiple communication channels. The web interface enables browser-based chat through the default UI. WebSocket support provides real-time bidirectional communication for responsive interactions. Voice integration handles audio input and output for voice-enabled bots. Platform adapters provide an extensible channel system for adding new platforms.

This module abstracts channel-specific implementations behind a common interface, routes messages to appropriate handlers based on channel type, formats responses appropriately for specific channels, and handles channel-specific features like typing indicators and read receipts.


## AI and Knowledge Services

### LLM Integration

The `llm` module integrates with large language models for natural language understanding and generation. Provider abstraction supports multiple LLM providers through a common interface. API communication handles API calls to LLM services including authentication and rate limiting. Streaming responses support token streaming for real-time response display. Error handling provides graceful degradation when API calls fail.

The module sends prompts to LLM providers using appropriate formats, parses and streams responses back to callers, handles API authentication and key management, and manages rate limiting with automatic retries when necessary.

### LLM Models

The `llm_models` module contains model-specific implementations for different providers. Model configurations define parameters and capabilities for different models. Prompt templates handle model-specific prompt formatting requirements. Token counting estimates token usage before making API calls. Model selection chooses the appropriate model for each task based on requirements.

This module defines model capabilities and limits for each supported model, formats prompts according to each model's expectations, calculates token costs for usage tracking, and selects optimal models for specific query types.

### NVIDIA Integration

The `nvidia` module provides GPU acceleration support for local model inference. GPU detection identifies available NVIDIA GPUs in the system. Acceleration enables GPU-accelerated inference for local models. Resource management allocates GPU resources among concurrent requests.


## Infrastructure Services

### Bootstrap

The `bootstrap` module handles system initialization and first-time setup. Component installation downloads and installs required components including PostgreSQL, cache, and drive storage. Database setup creates schemas and applies migrations to prepare the database. Credential generation creates secure passwords for all services. Environment configuration writes `.env` files with generated settings. Template upload deploys bot templates to storage for immediate use.

The module detects installation mode to determine whether it is running locally or in containers, installs and starts all system components in the correct order, initializes the database with migrations and seed data, configures drive storage with appropriate buckets, and creates default bots from included templates.

### Package Manager

The `package_manager` module manages component installation and lifecycle. The component registry tracks available components and their versions. Installation downloads and installs components from configured sources. Lifecycle management starts, stops, and restarts components as needed. Dependency resolution ensures components start in the correct order based on their dependencies.

Managed components include tables for PostgreSQL database, cache for Valkey caching, drive for S3-compatible object storage, llm for local LLM server, email for email server integration, proxy for reverse proxy functionality, directory for LDAP directory services, alm for application lifecycle management, dns for DNS server operations, meeting for LiveKit video conferencing, and vector_db for Qdrant vector database functionality.

### Configuration

The `config` module loads and validates application configuration. Environment variables load from `.env` files and system environment. Validation ensures all required configuration is present before startup. Defaults provide sensible values for optional settings. Type safety parses configuration into strongly-typed structs for compile-time checking.

The module loads DATABASE_URL, DRIVE_SERVER, API keys, and other settings, validates configuration completeness at startup, provides configuration access to other modules through a shared struct, and handles configuration errors with helpful messages.

### Shared Utilities

The `shared` module contains common functionality used across the system. Database models define the Diesel schema and models for all tables. Connection pooling manages R2D2 connection pools for efficient database access. Utilities provide common helper functions for repeated tasks. Types define shared type definitions used throughout the codebase.

This module defines the database schema with Diesel macros, provides database connection helpers for consistent access patterns, implements common utility functions for string manipulation and data transformation, and shares types across modules to ensure consistency.

### Web Server

The `web_server` module implements the HTTP API using Axum. API routes define RESTful endpoints for bot interaction and management. The WebSocket handler manages real-time communication channels. Static files serve web UI assets for the browser interface. CORS configuration enables cross-origin resource sharing for embedded deployments. Middleware handles logging, authentication, and error handling for all requests.

This module defines API routes and their handlers, processes HTTP requests and generates responses, manages WebSocket connections for real-time chat, and serves static web interface files for the UI.


## Feature Services

### Automation

The `automation` module provides scheduled and event-driven task execution. Cron scheduling runs tasks on defined schedules using standard cron syntax. Event triggers react to system events by executing associated handlers. Background jobs execute long-running tasks without blocking the main thread. Job management tracks running jobs and allows cancellation when needed.

### Drive Monitor

The `drive_monitor` module watches for file system changes in bot packages. File watching detects file creation, modification, and deletion events. Event processing handles file change events by triggering appropriate actions. Automatic indexing adds new documents to the knowledge base when they appear in monitored directories.

### Email Integration

The `email` module handles email communication as an optional feature. IMAP support reads emails from configured inbox folders. SMTP support sends emails via the Lettre library. Email parsing extracts text content and attachments from received messages. Template rendering generates HTML emails from templates with variable substitution.

### File Handling

The `file` module processes various file types for knowledge base ingestion. PDF extraction pulls text from PDF documents using pdf-extract. Document parsing handles various document formats including Word and plain text. File upload processes multipart file uploads from users. Storage integration saves processed files to drive storage for persistence.

### Meeting Integration

The `meet` module integrates with LiveKit for video conferencing capabilities. Room creation establishes meeting rooms with appropriate settings. Token generation creates access tokens for meeting participants. Participant management tracks who is in each meeting. Recording captures meeting sessions for later review.


## Storage Services

### Drive

The `drive` module provides S3-compatible object storage integration. Drive integration uses the AWS SDK S3 client for compatibility with various providers. Bucket management creates and manages storage buckets for different bots. Object operations handle upload, download, and delete operations for files. Vector database integration connects to Qdrant for semantic search functionality.

### UI Components

The `ui` module contains UI-related functionality for the web interface. Drive UI provides a file browser interface for managing documents. Stream handling implements server-sent events for real-time updates. Sync logic manages synchronization between local and remote files. Local sync enables desktop app file synchronization for offline access.


## Testing

The `tests` module provides test utilities and integration tests for the platform. Test fixtures provide common test data and setup procedures. Integration tests validate end-to-end functionality across modules. Mock services substitute for external dependencies during testing. Test helpers provide utilities for writing consistent, readable tests.


## Service Interaction Patterns

### Layered Architecture

Services are organized into layers with clear dependencies. The infrastructure layer contains bootstrap, package_manager, config, shared, and web_server modules that provide foundational capabilities. The data layer contains drive, file, and session modules that handle persistence. The domain layer contains bot, auth, context, and basic modules that implement core business logic. The AI layer contains llm, llm_models, and nvidia modules for machine learning integration. The feature layer contains automation, email, meet, and drive_monitor modules that add optional capabilities. The presentation layer contains channels and ui modules that handle user interaction.

### Dependency Injection

Services use Rust's module system and trait-based design for dependency injection. Database connections are shared via connection pools managed by R2D2. Configuration is passed through the AppConfig struct which is initialized at startup and shared immutably. Services access their dependencies through function parameters rather than global state.

### Error Handling

All services use `anyhow::Result<T>` for error handling, allowing errors to propagate up the call stack with context. Each layer adds relevant context to errors before propagating them. Critical services log errors using the `log` crate with appropriate severity levels. User-facing errors are translated to helpful messages without exposing internal details.

### Async Operations

Most services are async and use Tokio as the runtime. This design allows concurrent handling of multiple user sessions without blocking. External API calls run concurrently to minimize latency. Background tasks use Tokio's task spawning for parallel execution. The async design enables efficient resource utilization even under high load.