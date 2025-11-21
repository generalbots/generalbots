# Service Layer

BotServer's service layer is organized into functional modules that handle specific aspects of the platform. Each module encapsulates related functionality and provides a clear API for interaction with other parts of the system.

## Core Service Modules

### Authentication & Security (`auth`)

The `auth` module provides secure user authentication and session management:

- **Password Hashing**: Uses Argon2 for secure password storage
- **Session Tokens**: Generates and validates unique session tokens
- **User Verification**: Authenticates users against the database
- **Bot Authentication**: Manages bot-level authentication for API access

Key responsibilities:
- Hash passwords with Argon2 before storage
- Generate cryptographically secure session tokens
- Validate user credentials
- Manage session lifecycle

### Bot Management (`bot`)

The `bot` module handles bot lifecycle and configuration:

- **Bot Creation**: Initialize new bot instances
- **Configuration Management**: Load and apply bot settings
- **Bot State**: Track bot status and health
- **Multi-Tenant Support**: Isolate bots by tenant

Key responsibilities:
- Create and delete bot instances
- Load bot configuration from database
- Manage bot lifecycle (start, stop, restart)
- Associate bots with users and sessions

### Session Management (`session`)

The `session` module maintains user conversation state:

- **Session Storage**: Persist conversation context
- **State Management**: Track user progress through dialogs
- **Session Cleanup**: Remove expired sessions
- **Multi-User Support**: Isolate sessions by user

Key responsibilities:
- Create new sessions on user connection
- Store and retrieve session variables
- Maintain conversation history
- Clean up abandoned sessions

## Conversation & Scripting Services

### BASIC Interpreter (`basic`)

The `basic` module implements the BASIC-like scripting language for `.gbdialog` files:

- **Script Parsing**: Parse BASIC dialog scripts
- **Execution Engine**: Powered by the Rhai scripting engine
- **Keyword Implementation**: Custom keywords like `TALK`, `HEAR`, `LLM`
- **Variable Management**: Handle script variables and context

Key responsibilities:
- Load and parse `.gbdialog` scripts
- Execute BASIC commands
- Provide custom keywords for bot functionality
- Manage script execution context

### Context Management (`context`)

The `context` module manages conversation context and memory:

- **Conversation History**: Store message history
- **Context Retrieval**: Load relevant context for LLM calls
- **Memory Management**: Limit context size to fit token limits
- **Context Compaction**: Summarize old conversations

Key responsibilities:
- Append messages to conversation history
- Retrieve context for LLM queries
- Implement context window management
- Provide context to knowledge base queries

### Channel Abstraction (`channels`)

The `channels` module provides a unified interface for multiple communication channels:

- **Web Interface**: Browser-based chat
- **WebSocket Support**: Real-time bidirectional communication
- **Voice Integration**: Audio input/output
- **Platform Adapters**: Extensible channel system

Key responsibilities:
- Abstract channel-specific implementations
- Route messages to appropriate handlers
- Format responses for specific channels
- Handle channel-specific features (typing indicators, etc.)

## AI & Knowledge Services

### LLM Integration (`llm`)

The `llm` module integrates with large language models:

- **Provider Abstraction**: Support multiple LLM providers
- **API Communication**: Handle API calls to LLM services
- **Streaming Responses**: Support token streaming
- **Error Handling**: Graceful degradation on API failures

Key responsibilities:
- Send prompts to LLM providers
- Parse and stream responses
- Handle API authentication
- Manage rate limiting and retries

### LLM Models (`llm_models`)

The `llm_models` module contains model-specific implementations:

- **Model Configurations**: Parameters for different models
- **Prompt Templates**: Model-specific prompt formatting
- **Token Counting**: Estimate token usage
- **Model Selection**: Choose appropriate model for task

Key responsibilities:
- Define model capabilities and limits
- Format prompts for specific models
- Calculate token costs
- Select optimal model for queries

### NVIDIA Integration (`nvidia`)

The `nvidia` module provides GPU acceleration support:

- **GPU Detection**: Identify available NVIDIA GPUs
- **Acceleration**: Enable GPU-accelerated inference
- **Resource Management**: Allocate GPU resources

## Infrastructure Services

### Bootstrap (`bootstrap`)

The `bootstrap` module handles system initialization:

- **Component Installation**: Install required components (PostgreSQL, Redis, MinIO)
- **Database Setup**: Create schemas and apply migrations
- **Credential Generation**: Generate secure passwords for services
- **Environment Configuration**: Write `.env` files
- **Template Upload**: Upload bot templates to storage

Key responsibilities:
- Detect installation mode (local vs container)
- Install and start system components
- Initialize database with migrations
- Configure MinIO/S3 storage
- Create default bots from templates

### Package Manager (`package_manager`)

The `package_manager` module manages component installation:

- **Component Registry**: Track available components
- **Installation**: Download and install components
- **Lifecycle Management**: Start, stop, restart components
- **Dependency Resolution**: Ensure components start in correct order

Components managed:
- `tables` - PostgreSQL database
- `cache` - Redis/Valkey cache
- `drive` - MinIO object storage
- `llm` - Local LLM server
- `email` - Email server
- `proxy` - Reverse proxy
- `directory` - LDAP directory
- `alm` - Application lifecycle management
- `dns` - DNS server
- `meeting` - Video conferencing (LiveKit)
- `vector_db` - Qdrant vector database
- And more...

### Configuration (`config`)

The `config` module loads and validates application configuration:

- **Environment Variables**: Load from `.env` files
- **Validation**: Ensure required config is present
- **Defaults**: Provide sensible default values
- **Type Safety**: Parse config into strongly-typed structs

Key responsibilities:
- Load `DATABASE_URL`, `DRIVE_SERVER`, API keys
- Validate configuration completeness
- Provide config access to other modules
- Handle configuration errors gracefully

### Shared Utilities (`shared`)

The `shared` module contains common functionality:

- **Database Models**: Diesel schema and models
- **Connection Pooling**: R2D2 connection pool management
- **Utilities**: Common helper functions
- **Types**: Shared type definitions

Key responsibilities:
- Define database schema with Diesel
- Provide database connection helpers
- Implement common utility functions
- Share types across modules

### Web Server (`web_server`)

The `web_server` module implements the HTTP API using Axum:

- **API Routes**: RESTful endpoints for bot interaction
- **WebSocket Handler**: Real-time communication
- **Static Files**: Serve web UI assets
- **CORS**: Cross-origin resource sharing
- **Middleware**: Logging, authentication, error handling

Key responsibilities:
- Define API routes and handlers
- Handle HTTP requests and responses
- Manage WebSocket connections
- Serve static web interface files

## Feature Services

### Automation (`automation`)

The `automation` module provides scheduled and event-driven tasks:

- **Cron Scheduling**: Run tasks on schedule
- **Event Triggers**: React to system events
- **Background Jobs**: Execute long-running tasks
- **Job Management**: Track and cancel jobs

### Drive Monitor (`drive_monitor`)

The `drive_monitor` module watches for file system changes:

- **File Watching**: Detect file creation, modification, deletion
- **Event Processing**: Handle file change events
- **Automatic Indexing**: Index new documents in knowledge base

### Email Integration (`email`)

The `email` module handles email communication (optional feature):

- **IMAP Support**: Read emails from inbox
- **SMTP Support**: Send emails via Lettre
- **Email Parsing**: Extract text and attachments
- **Template Rendering**: Generate HTML emails

### File Handling (`file`)

The `file` module processes various file types:

- **PDF Extraction**: Extract text from PDFs
- **Document Parsing**: Parse various document formats
- **File Upload**: Handle multipart file uploads
- **Storage Integration**: Save files to MinIO

### Meeting Integration (`meet`)

The `meet` module integrates with LiveKit for video conferencing:

- **Room Creation**: Create meeting rooms
- **Token Generation**: Generate access tokens
- **Participant Management**: Track meeting participants
- **Recording**: Record meeting sessions

## Storage Services

### Drive (`drive`)

The `drive` module provides S3-compatible object storage:

- **MinIO Integration**: AWS SDK S3 client
- **Bucket Management**: Create and manage buckets
- **Object Operations**: Upload, download, delete objects
- **Vector Database**: Qdrant integration for semantic search

### UI Components (`ui`)

The `ui` module contains UI-related functionality:

- **Drive UI**: File browser interface
- **Stream Handling**: Server-sent events for real-time updates
- **Sync Logic**: Synchronization between local and remote files
- **Local Sync**: Desktop app file synchronization

## Testing (`tests`)

The `tests` module provides test utilities and integration tests:

- **Test Fixtures**: Common test data and setup
- **Integration Tests**: End-to-end testing
- **Mock Services**: Mock external dependencies
- **Test Helpers**: Utilities for writing tests

## Service Interaction Patterns

### Layered Architecture

Services are organized in layers:

1. **Infrastructure Layer**: `bootstrap`, `package_manager`, `config`, `shared`, `web_server`
2. **Data Layer**: `drive`, `file`, `session`
3. **Domain Layer**: `bot`, `auth`, `context`, `basic`
4. **AI Layer**: `llm`, `llm_models`, `nvidia`
5. **Feature Layer**: `automation`, `email`, `meet`, `drive_monitor`
6. **Presentation Layer**: `channels`, `ui`

### Dependency Injection

Services use Rust's module system and trait-based design for dependency injection. Database connections are shared via connection pools, and configuration is passed through the `AppConfig` struct.

### Error Handling

All services use `anyhow::Result<T>` for error handling, allowing errors to propagate up the call stack with context. Critical services log errors using the `log` crate.

### Async/Await

Most services are async and use Tokio as the runtime. This allows for concurrent handling of multiple user sessions and external API calls without blocking.