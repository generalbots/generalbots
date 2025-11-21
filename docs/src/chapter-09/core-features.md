# Core Features

BotServer provides a comprehensive set of features for building and deploying conversational AI applications. This page outlines the core capabilities implemented across the various modules.

## Multi-Channel Communication

BotServer supports multiple communication channels through the `channels` module:

- **Web Interface**: Browser-based chat interface served by the Axum web server
- **WebSocket Support**: Real-time bidirectional communication for instant messaging
- **Voice Integration**: Audio input/output capabilities for voice interactions
- **Channel Abstraction**: Unified API that abstracts channel-specific implementations

All channels share the same conversation logic, ensuring consistent behavior regardless of how users interact with the bot.

## Authentication & Session Management

### User Authentication

The `auth` module provides enterprise-grade security:

- **Argon2 Password Hashing**: Industry-standard password hashing algorithm
- **Session Tokens**: Cryptographically secure session token generation
- **User Verification**: Database-backed user authentication
- **Bot Authentication**: API-level authentication for bot access

### Session Persistence

The `session` module maintains conversation state:

- **Persistent Sessions**: Sessions survive server restarts (stored in database)
- **Multi-User Isolation**: Each user has isolated session state
- **Session Variables**: Store custom data within user sessions
- **Conversation History**: Full message history per session
- **Automatic Cleanup**: Remove expired or abandoned sessions

## BASIC Scripting Language

The `basic` module implements a BASIC-like scripting language for creating dialog flows:

- **Simple Syntax**: English-like commands that are easy to learn
- **Custom Keywords**: Specialized commands like `TALK`, `HEAR`, `LLM`, `ADD_KB`
- **Rhai-Powered**: Built on the Rhai scripting engine for Rust
- **Variable Management**: Store and manipulate data within scripts
- **Control Flow**: Conditions, loops, and branching logic
- **Tool Integration**: Call external APIs and services from scripts

Scripts are stored as `.gbdialog` files and can be deployed as part of bot packages.

## LLM Integration

### Multiple Provider Support

The `llm` and `llm_models` modules provide flexible AI integration:

- **OpenAI**: GPT-3.5, GPT-4, and newer models
- **Local Models**: Support for self-hosted LLM servers
- **Model Selection**: Choose appropriate models based on task complexity
- **Streaming Responses**: Token-by-token streaming for responsive UX
- **Error Handling**: Graceful fallback on API failures

### Prompt Management

- **Template System**: Reusable prompt templates
- **Context Injection**: Automatically include relevant context
- **Token Counting**: Estimate and manage token usage
- **Cost Optimization**: Select cost-effective models when appropriate

## Knowledge Base & Semantic Search

### Vector Database Integration

The `drive` module includes vector database support via Qdrant:

- **Semantic Search**: Find relevant information using embeddings
- **Document Indexing**: Automatically index uploaded documents
- **Context Retrieval**: Fetch relevant context for LLM queries
- **Collection Management**: Organize knowledge into collections

### Document Processing

The `file` module handles various document types:

- **PDF Extraction**: Extract text from PDF documents
- **Document Parsing**: Support for multiple document formats
- **Automatic Indexing**: Index documents on upload
- **Metadata Storage**: Store document metadata for retrieval

## Object Storage

### MinIO/S3 Integration

The `drive` module provides cloud-native storage:

- **S3-Compatible API**: Use AWS SDK with MinIO or AWS S3
- **Bucket Management**: Create and manage storage buckets
- **Object Operations**: Upload, download, list, delete files
- **Secure Access**: Credential-based authentication
- **Template Storage**: Store bot templates and assets

### File Monitoring

The `drive_monitor` module watches for file changes:

- **Real-Time Detection**: Detect file creation, modification, deletion
- **Automatic Processing**: Trigger indexing on file changes
- **Event Handling**: React to file system events

## Database Management

### PostgreSQL Backend

The `shared` module defines the database schema using Diesel:

- **Connection Pooling**: R2D2-based connection pool for performance
- **Migrations**: Automatic schema migrations on bootstrap
- **ORM**: Type-safe database queries with Diesel
- **Transactions**: ACID-compliant transaction support

### Schema

Key database tables include:

- `users` - User accounts and credentials
- `bots` - Bot configurations and metadata
- `sessions` - Active user sessions
- `messages` - Conversation history
- `bot_configuration` - Component configuration per bot
- `conversations` - Conversation metadata

## Caching

Redis/Valkey integration via the `cache` component:

- **Session Caching**: Fast session state retrieval
- **Query Caching**: Cache expensive database queries
- **Rate Limiting**: Implement rate limits with Redis
- **Distributed State**: Share state across multiple instances

## Automation & Scheduling

The `automation` module enables scheduled and event-driven tasks:

- **Cron Scheduling**: Execute tasks on a schedule
- **Event Triggers**: React to system events (file changes, messages, etc.)
- **Background Jobs**: Run long-running tasks asynchronously
- **Job Management**: Track, pause, and cancel scheduled jobs

Common automation use cases:
- Send scheduled notifications
- Generate periodic reports
- Clean up old data
- Monitor system health

## Email Integration

The `email` module provides email capabilities (optional feature):

- **IMAP Support**: Read emails from inbox
- **SMTP Support**: Send emails via Lettre library
- **Email Parsing**: Extract text and attachments with mailparse
- **Template Rendering**: Generate HTML emails
- **TLS/SSL**: Secure email connections

## Video Conferencing

The `meet` module integrates with LiveKit:

- **Room Creation**: Create video meeting rooms
- **Token Generation**: Generate secure access tokens
- **Participant Management**: Track who's in meetings
- **Recording**: Record meeting sessions

## Bootstrap & Installation

The `bootstrap` module provides automated setup:

- **Component Installation**: Install PostgreSQL, Redis, MinIO, etc.
- **Credential Generation**: Generate secure passwords automatically
- **Database Initialization**: Apply migrations and create schema
- **Environment Configuration**: Write `.env` files with settings
- **Template Upload**: Upload bot templates to storage
- **Multi-Mode Support**: Install locally or in containers

### Package Manager

The `package_manager` handles component lifecycle:

- **Component Registry**: 20+ pre-configured components
- **Dependency Resolution**: Start components in correct order
- **Health Checks**: Monitor component status
- **Start/Stop/Restart**: Manage component lifecycle

Available components include:
- `tables` (PostgreSQL)
- `cache` (Redis/Valkey)
- `drive` (MinIO)
- `llm` (Local LLM server)
- `vector_db` (Qdrant)
- `email`, `proxy`, `directory`, `dns`, `meeting`, and more

## Web Server & API

The `web_server` module provides the HTTP interface using Axum:

- **RESTful API**: Standard HTTP endpoints for bot interaction
- **WebSocket Server**: Real-time bidirectional communication
- **Static File Serving**: Serve web UI assets
- **CORS Support**: Cross-origin resource sharing
- **Middleware**: Logging, authentication, error handling
- **Multipart Upload**: Handle file uploads

## Desktop Application

When built with the `desktop` feature, BotServer includes Tauri integration:

- **Native Application**: Run as desktop app on Windows, macOS, Linux
- **System Integration**: Native file dialogs and OS integration
- **Local Sync**: Synchronize files between desktop and cloud
- **Offline Support**: Work without constant internet connection

## GPU Acceleration

The `nvidia` module enables GPU acceleration:

- **NVIDIA GPU Detection**: Identify available GPUs
- **Accelerated Inference**: Faster LLM inference on GPU
- **Resource Management**: Allocate GPU memory efficiently

## Security Features

Security is implemented across multiple modules:

- **Password Hashing**: Argon2 with secure defaults
- **Encryption**: AES-GCM for sensitive data at rest
- **Session Tokens**: Cryptographically random tokens
- **API Authentication**: Token-based API access
- **HTTPS/TLS**: Secure communication (via proxy component)
- **Secure Credentials**: Automatic generation of strong passwords
- **Isolated Sessions**: User data isolation

## Testing & Quality

The `tests` module provides testing infrastructure:

- **Integration Tests**: End-to-end testing
- **Unit Tests**: Per-module test files (`.test.rs`)
- **Mock Services**: Mock external dependencies with mockito
- **Test Fixtures**: Reusable test data and setup
- **CI/CD Ready**: Automated testing in CI pipelines

## Logging & Monitoring

Built-in observability features:

- **Structured Logging**: Using the `log` and `tracing` crates
- **Error Context**: Detailed error messages with anyhow
- **Performance Metrics**: Track request timing and throughput
- **Health Endpoints**: Monitor component health

## Extensibility

BotServer is designed to be extended:

- **Custom Keywords**: Add new BASIC keywords in Rust
- **Plugin Architecture**: Modular design allows adding features
- **Tool Integration**: Call external APIs from BASIC scripts
- **Custom Channels**: Implement new communication channels
- **Provider Plugins**: Add new LLM providers