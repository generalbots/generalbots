# Architecture Reference

System architecture and design overview for General Bots.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clients                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │   Web    │  │  Mobile  │  │   API    │  │ WhatsApp │        │
│  │  (HTMX)  │  │   App    │  │ Clients  │  │ Telegram │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
└───────┼─────────────┼─────────────┼─────────────┼───────────────┘
        │             │             │             │
        └─────────────┴──────┬──────┴─────────────┘
                             │
                    ┌────────▼────────┐
                    │   HTTP Server   │
                    │     (Axum)      │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────▼───────┐  ┌─────────▼─────────┐  ┌──────▼──────┐
│  REST API     │  │    WebSocket      │  │   Static    │
│  Handlers     │  │    Handlers       │  │   Files     │
└───────┬───────┘  └─────────┬─────────┘  └─────────────┘
        │                    │
        └────────┬───────────┘
                 │
        ┌────────▼────────┐
        │   App State     │
        │  (Shared Arc)   │
        └────────┬────────┘
                 │
   ┌─────────────┼─────────────┬─────────────┐
   │             │             │             │
┌──▼──┐     ┌────▼────┐   ┌────▼────┐   ┌───▼───┐
│ DB  │     │  Cache  │   │ Storage │   │  LLM  │
│(PG) │     │ (Redis) │   │  (S3)   │   │(Multi)│
└─────┘     └─────────┘   └─────────┘   └───────┘
```

## Core Components

### HTTP Server (Axum)

The main entry point for all requests.

```rust
// Main router structure
Router::new()
    .nest("/api", api_routes())
    .nest("/ws", websocket_routes())
    .nest_service("/", static_files())
    .layer(middleware_stack())
```

**Responsibilities:**
- Request routing
- Middleware execution
- CORS handling
- Rate limiting
- Authentication

### App State

Shared application state passed to all handlers.

```rust
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub redis: RedisPool,
    pub s3_client: S3Client,
    pub llm_client: LlmClient,
    pub qdrant: QdrantClient,
    pub config: AppConfig,
}
```

**Contains:**
- Database connection pool
- Redis cache connection
- S3 storage client
- LLM provider clients
- Vector database client
- Application configuration

### Request Flow

```
Request → Router → Middleware → Handler → Response
                      │
                      ├── Auth Check
                      ├── Rate Limit
                      ├── Logging
                      └── Error Handling
```

## Module Structure

```
botserver/src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── core/                # Core infrastructure
│   ├── shared/          # Shared types and state
│   │   ├── state.rs     # AppState definition
│   │   ├── models.rs    # Common models
│   │   └── schema.rs    # Database schema
│   ├── urls.rs          # URL constants
│   ├── secrets/         # Vault integration
│   └── rate_limit.rs    # Rate limiting
├── basic/               # BASIC language
│   ├── compiler/        # Compiler/parser
│   ├── runtime/         # Execution engine
│   └── keywords/        # Keyword implementations
├── llm/                 # LLM integration
│   ├── mod.rs           # Provider abstraction
│   ├── openai.rs        # OpenAI client
│   ├── anthropic.rs     # Anthropic client
│   └── prompt_manager/  # Prompt management
├── multimodal/          # Media processing
│   ├── vision.rs        # Image analysis
│   ├── audio.rs         # Speech processing
│   └── document.rs      # Document parsing
├── security/            # Authentication
│   ├── auth.rs          # Auth middleware
│   ├── zitadel.rs       # Zitadel client
│   └── jwt.rs           # Token handling
├── analytics/           # Analytics module
├── calendar/            # Calendar/CalDAV
├── designer/            # Bot builder
├── drive/               # File storage
├── email/               # Email (IMAP/SMTP)
├── meet/                # Video conferencing
├── paper/               # Document editor
├── research/            # KB search
├── sources/             # Templates
└── tasks/               # Task management
```

## Data Flow

### Chat Message Flow

```
User Input
    │
    ▼
┌──────────────┐
│  WebSocket   │
│   Handler    │
└──────┬───────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│   Session    │────▶│   Message    │
│   Manager    │     │   History    │
└──────┬───────┘     └──────────────┘
       │
       ▼
┌──────────────┐
│  KB Search   │◀────── Vector DB (Qdrant)
│  (Context)   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ Tool Check   │◀────── Registered Tools
│              │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  LLM Call    │◀────── Semantic Cache (Redis)
│              │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Response    │
│  Streaming   │
└──────┬───────┘
       │
       ▼
User Response
```

### Tool Execution Flow

```
LLM Response (tool_call)
    │
    ▼
┌──────────────┐
│ Tool Router  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│BASIC Runtime │
└──────┬───────┘
       │
       ├──▶ Database Operations
       ├──▶ HTTP Requests
       ├──▶ File Operations
       └──▶ Email/Notifications
       │
       ▼
┌──────────────┐
│ Tool Result  │
└──────┬───────┘
       │
       ▼
LLM (continue or respond)
```

## Database Schema

### Core Tables

```sql
-- User sessions
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY,
    user_id VARCHAR(255),
    bot_id VARCHAR(100),
    status VARCHAR(20),
    metadata JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- Message history
CREATE TABLE message_history (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES user_sessions(id),
    role VARCHAR(20),
    content TEXT,
    tokens_used INTEGER,
    created_at TIMESTAMPTZ
);

-- Bot configurations
CREATE TABLE bot_configs (
    id UUID PRIMARY KEY,
    bot_id VARCHAR(100) UNIQUE,
    config JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- Knowledge base documents
CREATE TABLE kb_documents (
    id UUID PRIMARY KEY,
    collection VARCHAR(100),
    content TEXT,
    embedding_id VARCHAR(100),
    metadata JSONB,
    created_at TIMESTAMPTZ
);
```

### Indexes

```sql
CREATE INDEX idx_sessions_user ON user_sessions(user_id);
CREATE INDEX idx_sessions_bot ON user_sessions(bot_id);
CREATE INDEX idx_messages_session ON message_history(session_id);
CREATE INDEX idx_kb_collection ON kb_documents(collection);
```

## Caching Strategy

### Cache Layers

```
┌─────────────────────────────────────────┐
│            Semantic Cache               │
│  (LLM responses by query similarity)    │
└───────────────────┬─────────────────────┘
                    │
┌───────────────────▼─────────────────────┐
│            Session Cache                │
│    (Active sessions, user context)      │
└───────────────────┬─────────────────────┘
                    │
┌───────────────────▼─────────────────────┐
│            Data Cache                   │
│   (KB results, config, templates)       │
└─────────────────────────────────────────┘
```

### Cache Keys

| Pattern | TTL | Description |
|---------|-----|-------------|
| `session:{id}` | 30m | Active session data |
| `semantic:{hash}` | 24h | LLM response cache |
| `kb:{collection}:{hash}` | 1h | KB search results |
| `config:{bot_id}` | 5m | Bot configuration |
| `user:{id}` | 15m | User preferences |

### Semantic Cache

```rust
// Query similarity check
let cache_key = compute_embedding_hash(query);
if let Some(cached) = redis.get(&cache_key).await? {
    if similarity(query, cached.query) > 0.95 {
        return cached.response;
    }
}
```

## LLM Integration

### Provider Abstraction

```rust
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) 
        -> Result<CompletionResponse>;
    
    async fn complete_stream(&self, request: CompletionRequest) 
        -> Result<impl Stream<Item = StreamChunk>>;
    
    async fn embed(&self, text: &str) 
        -> Result<Vec<f32>>;
}
```

### Supported Providers

| Provider | Models | Features |
|----------|--------|----------|
| OpenAI | GPT-5, GPT-4o, o3 | Streaming, Functions, Vision |
| Anthropic | Claude Sonnet 4.5, Claude Opus 4.5 | Streaming, Long context |
| Groq | Llama 3.3, Mixtral | Fast inference |
| Ollama | Any local | Self-hosted |

### Request Flow

```rust
// 1. Build messages with context
let messages = build_messages(history, kb_context, system_prompt);

// 2. Add tools if registered
let tools = get_registered_tools(session);

// 3. Check semantic cache
if let Some(cached) = semantic_cache.get(&messages).await? {
    return Ok(cached);
}

// 4. Call LLM
let response = llm.complete(CompletionRequest {
    messages,
    tools,
    temperature: config.temperature,
    max_tokens: config.max_tokens,
}).await?;

// 5. Cache response
semantic_cache.set(&messages, &response).await?;
```

## BASIC Runtime

### Compilation Pipeline

```
Source Code (.bas)
       │
       ▼
┌──────────────┐
│    Lexer     │──▶ Tokens
└──────────────┘
       │
       ▼
┌──────────────┐
│    Parser    │──▶ AST
└──────────────┘
       │
       ▼
┌──────────────┐
│   Analyzer   │──▶ Validated AST
└──────────────┘
       │
       ▼
┌──────────────┐
│   Runtime    │──▶ Execution
└──────────────┘
```

### Execution Context

```rust
pub struct RuntimeContext {
    pub session: Session,
    pub variables: HashMap<String, Value>,
    pub tools: Vec<Tool>,
    pub kb: Vec<KnowledgeBase>,
    pub state: Arc<AppState>,
}
```

## Security Architecture

### Authentication Flow

```
Client Request
       │
       ▼
┌──────────────┐
│   Extract    │
│    Token     │
└──────┬───────┘
       │
       ▼
┌──────────────┐     ┌──────────────┐
│   Validate   │────▶│   Zitadel    │
│     JWT      │     │   (OIDC)     │
└──────┬───────┘     └──────────────┘
       │
       ▼
┌──────────────┐
│  Check       │
│  Permissions │
└──────┬───────┘
       │
       ▼
Handler Execution
```

### Security Layers

1. **Transport**: TLS 1.3 (rustls)
2. **Authentication**: JWT/OAuth 2.0 (Zitadel)
3. **Authorization**: Role-based access control
4. **Rate Limiting**: Per-IP token bucket
5. **Input Validation**: Type-safe parameters
6. **Output Sanitization**: HTML escaping

## Deployment Architecture

### Single Instance

```
┌─────────────────────────────────────┐
│           Single Server             │
│  ┌─────────────────────────────┐   │
│  │        botserver            │   │
│  └─────────────────────────────┘   │
│  ┌────────┐ ┌────────┐ ┌───────┐   │
│  │PostgreSQL│ │ Redis │ │ MinIO │   │
│  └────────┘ └────────┘ └───────┘   │
└─────────────────────────────────────┘
```

### Clustered

```
┌─────────────────────────────────────────────────────┐
│                   Load Balancer                      │
└───────────────────────┬─────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
   ┌────▼────┐    ┌─────▼────┐    ┌────▼────┐
   │botserver│    │botserver │    │botserver│
   │   #1    │    │    #2    │    │   #3    │
   └────┬────┘    └─────┬────┘    └────┬────┘
        │               │               │
        └───────────────┼───────────────┘
                        │
   ┌────────────────────┼────────────────────┐
   │                    │                    │
┌──▼───┐          ┌─────▼────┐         ┌────▼────┐
│  PG  │          │  Redis   │         │   S3    │
│Cluster│         │  Cluster │         │ Cluster │
└──────┘          └──────────┘         └─────────┘
```

## Performance Characteristics

| Operation | Latency | Throughput |
|-----------|---------|------------|
| REST API | < 10ms | 10,000 req/s |
| WebSocket message | < 5ms | 50,000 msg/s |
| KB search | < 50ms | 1,000 req/s |
| LLM call (cached) | < 20ms | 5,000 req/s |
| LLM call (uncached) | 500-3000ms | 50 req/s |

## Monitoring Points

| Metric | Description |
|--------|-------------|
| `http_requests_total` | Total HTTP requests |
| `http_request_duration` | Request latency |
| `ws_connections` | Active WebSocket connections |
| `llm_requests_total` | LLM API calls |
| `llm_cache_hits` | Semantic cache hit rate |
| `db_pool_size` | Active DB connections |
| `memory_usage` | Process memory |

## Extension Points

### Adding a New Module

1. Create module in `src/`
2. Define routes in `mod.rs`
3. Register in `lib.rs`
4. Add to router in `main.rs`

### Adding a New LLM Provider

1. Implement `LlmProvider` trait
2. Add provider enum variant
3. Register in provider factory

### Adding a New BASIC Keyword

1. Add keyword to lexer
2. Implement AST node
3. Add runtime handler
4. Update documentation