# Core Features

Technical overview of botserver capabilities. For the complete feature matrix, see [Feature Reference](./README.md).

## Multi-Channel Communication

| Channel | Protocol | Keywords |
|---------|----------|----------|
| Web Chat | WebSocket | `TALK`, `HEAR` |
| WhatsApp | Cloud API | `SEND`, `SEND TEMPLATE` |
| Email | SMTP/IMAP | `SEND MAIL` |
| Teams | Graph API | `SEND` |
| Voice | WebRTC | `PLAY`, `RECORD` |

All channels share the same conversation logic through a unified abstraction.

## Authentication & Sessions

- **Password Hashing**: Argon2 with secure defaults
- **Session Tokens**: Cryptographically secure generation
- **Session Persistence**: Survives restarts (database-backed)
- **User Isolation**: Each user has isolated session state

## BASIC Scripting

Built on the Rhai engine with custom keywords:

```basic
TALK "Hello!"                    ' Output
HEAR name AS NAME                ' Input with validation
result = LLM "Summarize: " + text  ' AI integration
USE KB "docs"                    ' Knowledge base
```

Scripts stored as `.gbdialog` files in bot packages.

## LLM Integration

| Provider | Models | Features |
|----------|--------|----------|
| OpenAI | GPT-5, o3 | Streaming, function calling |
| Anthropic | Claude Sonnet 4.5, Opus 4.5 | Analysis, coding, guidelines |
| Local | GGUF models | GPU acceleration, offline |

Features: prompt templates, context injection, token management, cost optimization.

## Knowledge Base

- **Vector Database**: Qdrant for semantic search
- **Document Processing**: PDF, DOCX, HTML, TXT extraction
- **Auto-Indexing**: Documents indexed on upload
- **Context Retrieval**: Automatic injection into LLM prompts

## Storage

### Object Storage (S3-Compatible)

- Bucket management
- Secure credential-based access
- Template and asset storage

### File Monitoring

- Real-time change detection
- Automatic processing triggers
- Event-driven workflows

## Database

PostgreSQL with Diesel ORM:

- Connection pooling (R2D2)
- Automatic migrations
- ACID transactions

Key tables: `users`, `bots`, `sessions`, `messages`, `conversations`

## Automation

```basic
SET SCHEDULE "0 9 * * *"  ' Daily at 9 AM
SEND MAIL "team@company.com", "Daily Report", report
```

- Cron scheduling
- Event triggers
- Background jobs

## Security

| Feature | Implementation |
|---------|----------------|
| Password Storage | Argon2 |
| Data at Rest | AES-GCM |
| Sessions | Cryptographic tokens |
| API Access | Token-based auth |
| Transport | TLS via proxy |

## Optional Components

| Component | Port | Purpose |
|-----------|------|---------|
| Email Server | 25/993 | SMTP/IMAP |
| Video Server | 7880 | LiveKit meetings |
| Vector DB | 6333 | Qdrant search |
| Time-Series | 8086 | InfluxDB metrics |

## Extensibility

- **Custom Keywords**: Add BASIC keywords in Rust
- **Tool Integration**: Call external APIs from scripts
- **Custom Channels**: Implement new communication channels
- **LLM Providers**: Add new AI providers

## See Also

- [Feature Reference](./README.md) - Complete feature matrix
- [AI and LLM](./ai-llm.md) - AI integration details
- [Automation](./automation.md) - Scheduling and triggers