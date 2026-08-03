# Sources API 🟡 BETA

> **Unified interface for managing knowledge bases, MCP integrations, and browsing available resources.**

---

## Base URL

```
/api/ui/sources
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Knowledge Base (KB) Endpoints

### List Knowledge Bases

**`GET /api/ui/sources/kb/`**

Returns all knowledge base configurations for the authenticated bot.

**Response:**

```json
[
  {
    "id": "kb-001",
    "name": "Product Manual",
    "description": "Complete product documentation",
    "documentCount": 42,
    "chunkCount": 310,
    "created_at": "2026-01-15T10:30:00Z",
    "updated_at": "2026-05-20T14:22:00Z"
  }
]
```

---

### Create Knowledge Base

**`POST /api/ui/sources/kb/`**

Creates a new knowledge base bucket for the bot.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Human-readable KB name |
| `description` | string | No | Description of the KB purpose |
| `chunkSize` | integer | No | Token chunk size (default: 512) |
| `chunkOverlap` | integer | No | Overlap between chunks (default: 50) |

**Request:**

```json
{
  "name": "Support FAQ",
  "description": "Frequently asked questions from support tickets",
  "chunkSize": 512,
  "chunkOverlap": 50
}
```

**Response:**

```json
{
  "id": "kb-faq-001",
  "name": "Support FAQ",
  "description": "Frequently asked questions from support tickets",
  "documentCount": 0,
  "chunkCount": 0,
  "created_at": "2026-06-04T12:00:00Z"
}
```

---

### Query Knowledge Base

**`POST /api/ui/sources/kb/query`**

Searches a KB with semantic similarity and returns relevant chunks for LLM context injection.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `kbId` | string | Yes | Knowledge base identifier |
| `query` | string | Yes | Natural language query |
| `topK` | integer | No | Max results to return (default: 5) |
| `threshold` | number | No | Min similarity score 0.0–1.0 (default: 0.7) |

**Request:**

```json
{
  "kbId": "kb-001",
  "query": "How do I reset the admin password?",
  "topK": 3,
  "threshold": 0.75
}
```

**Response:**

```json
{
  "results": [
    {
      "chunkId": "chunk-101",
      "documentId": "doc-12",
      "content": "To reset the admin password, navigate to Settings > Security > Password Reset...",
      "score": 0.94,
      "metadata": {
        "source": "admin-guide.pdf",
        "page": 14
      }
    }
  ],
  "queryTimeMs": 42
}
```

---

### Reindex Knowledge Base

**`POST /api/ui/sources/kb/reindex`**

Triggers a full reindex of all documents in a KB. Useful after bulk uploads or schema changes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `kbId` | string | Yes | Knowledge base identifier |

**Request:**

```json
{
  "kbId": "kb-001"
}
```

**Response:**

```json
{
  "status": "reindex_started",
  "kbId": "kb-001",
  "estimatedChunks": 310
}
```

---

### Get Knowledge Base Stats

**`GET /api/ui/sources/kb/stats`**

Returns aggregate statistics for a knowledge base.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `kbId` | string | Yes | Knowledge base identifier (query param) |

**Request:**

```
GET /api/ui/sources/kb/stats?kbId=kb-001
```

**Response:**

```json
{
  "kbId": "kb-001",
  "name": "Product Manual",
  "documentCount": 42,
  "chunkCount": 310,
  "totalTokens": 158720,
  "avgChunkTokens": 512,
  "lastIndexed": "2026-05-20T14:22:00Z",
  "vectorStoreSizeBytes": 26214400
}
```

---

### Delete Knowledge Base

**`DELETE /api/ui/sources/kb/:id`**

Permanently removes a knowledge base and all its documents and vectors.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | KB identifier (path param) |

**Request:**

```
DELETE /api/ui/sources/kb/kb-001
```

**Response:**

```json
{
  "deleted": true,
  "id": "kb-001"
}
```

---

## MCP (Model Context Protocol) Endpoints

### Scan MCP Servers

**`POST /api/ui/sources/mcp/scan`**

Scans configured MCP servers and discovers available tools.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `urls` | string[] | No | Specific server URLs to scan (empty = all configured) |

**Request:**

```json
{
  "urls": ["http://localhost:8082", "http://10.0.0.5:9000"]
}
```

**Response:**

```json
{
  "scanned": 2,
  "discoveredTools": 18,
  "servers": [
    {
      "url": "http://localhost:8082",
      "name": "local-tools",
      "toolCount": 12,
      "status": "healthy"
    },
    {
      "url": "http://10.0.0.5:9000",
      "name": "data-service",
      "toolCount": 6,
      "status": "healthy"
    }
  ]
}
```

---

### List MCP Examples

**`GET /api/ui/sources/mcp/examples`**

Returns example MCP server configurations and usage patterns.

**Response:**

```json
[
  {
    "name": "Filesystem Server",
    "url": "http://localhost:8082",
    "description": "Local filesystem access for reading/writing files",
    "tools": ["read_file", "write_file", "list_directory"]
  },
  {
    "name": "Database Server",
    "url": "http://localhost:8083",
    "description": "PostgreSQL query execution and schema inspection",
    "tools": ["query", "list_tables", "describe_table"]
  }
]
```

---

### List All MCP Tools

**`GET /api/ui/sources/mcp/tools`**

Returns a flat list of all tools discovered across all MCP servers.

**Response:**

```json
[
  {
    "server": "local-tools",
    "name": "read_file",
    "description": "Read contents of a file",
    "inputSchema": {
      "type": "object",
      "properties": {
        "path": { "type": "string", "description": "File path" }
      },
      "required": ["path"]
    }
  },
  {
    "server": "data-service",
    "name": "query",
    "description": "Execute a SQL query",
    "inputSchema": {
      "type": "object",
      "properties": {
        "sql": { "type": "string", "description": "SQL statement" }
      },
      "required": ["sql"]
    }
  }
]
```

---

### Test MCP Server

**`POST /api/ui/sources/mcp/:name/test`**

Tests connectivity and tool availability for a specific MCP server.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | MCP server name (path param) |

**Request:**

```
POST /api/ui/sources/mcp/local-tools/test
```

**Response:**

```json
{
  "server": "local-tools",
  "status": "healthy",
  "latencyMs": 12,
  "toolCount": 12,
  "version": "1.0.3"
}
```

---

### Get MCP Server Tools

**`GET /api/ui/sources/mcp/:name/tools`**

Returns tools for a specific MCP server.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | MCP server name (path param) |

**Request:**

```
GET /api/ui/sources/mcp/local-tools/tools
```

**Response:**

```json
{
  "server": "local-tools",
  "tools": [
    {
      "name": "read_file",
      "description": "Read contents of a file",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": { "type": "string" }
        },
        "required": ["path"]
      }
    },
    {
      "name": "write_file",
      "description": "Write content to a file",
      "inputSchema": {
        "type": "object",
        "properties": {
          "path": { "type": "string" },
          "content": { "type": "string" }
        },
        "required": ["path", "content"]
      }
    }
  ]
}
```

---

### Enable/Disable MCP Server

**`POST /api/ui/sources/mcp/:name/enable`**

Enables or disables an MCP server for bot tool usage.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | MCP server name (path param) |
| `enabled` | boolean | Yes | Whether to enable or disable |

**Request:**

```json
{
  "enabled": true
}
```

**Response:**

```json
{
  "server": "local-tools",
  "enabled": true,
  "activeToolCount": 12
}
```

---

## Browse Endpoints

### List Repositories

**`GET /api/ui/sources/repositories`**

Returns available code repositories.

**Response:**

```json
[
  {
    "id": "repo-1",
    "name": "botserver",
    "url": "https://github.com/org/botserver",
    "branch": "main",
    "lastSync": "2026-06-04T08:00:00Z"
  }
]
```

---

### List Apps

**`GET /api/ui/sources/apps`**

Returns available HTMX applications.

**Response:**

```json
[
  {
    "id": "app-dashboard",
    "name": "Dashboard",
    "path": "/apps/dashboard",
    "description": "Main analytics dashboard"
  }
]
```

---

### List Prompts

**`GET /api/ui/sources/prompts`**

Returns saved prompt templates.

**Response:**

```json
[
  {
    "id": "prompt-01",
    "name": "Summarizer",
    "template": "Summarize the following text in 3 bullet points:\n\n{{text}}",
    "variables": ["text"]
  }
]
```

---

### List Templates

**`GET /api/ui/sources/templates`**

Returns available email and document templates.

**Response:**

```json
[
  {
    "id": "tpl-welcome",
    "name": "Welcome Email",
    "type": "email",
    "subject": "Welcome to {{company}}"
  }
]
```

---

### List News

**`GET /api/ui/sources/news`**

Returns recent news items or changelog entries.

**Response:**

```json
[
  {
    "id": "news-001",
    "title": "MCP Server Support Added",
    "date": "2026-06-01",
    "summary": "New integration with Model Context Protocol servers for extended tool access."
  }
]
```

---

### List MCP Servers

**`GET /api/ui/sources/mcp-servers`**

Returns all configured MCP servers with status.

**Response:**

```json
[
  {
    "name": "local-tools",
    "url": "http://localhost:8082",
    "enabled": true,
    "status": "healthy",
    "toolCount": 12
  }
]
```

---

### List LLM Tools

**`GET /api/ui/sources/llm-tools`**

Returns built-in LLM tools available to the bot.

**Response:**

```json
[
  {
    "id": "tool-web-search",
    "name": "Web Search",
    "description": "Search the web for information",
    "enabled": true
  },
  {
    "id": "tool-code-exec",
    "name": "Code Execution",
    "description": "Execute Python/JS code in sandbox",
    "enabled": false
  }
]
```

---

### List Models

**`GET /api/ui/sources/models`**

Returns available LLM models.

**Response:**

```json
[
  {
    "id": "gpt-4o",
    "provider": "openai",
    "contextWindow": 128000,
    "maxOutput": 16384
  },
  {
    "id": "claude-sonnet-4-20250514",
    "provider": "anthropic",
    "contextWindow": 200000,
    "maxOutput": 8192
  }
]
```

---

### Search Sources

**`GET /api/ui/sources/search`**

Performs a unified search across all sources.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query (query param) |
| `type` | string | No | Filter by type: `kb`, `mcp`, `app`, `prompt` |

**Request:**

```
GET /api/ui/sources/search?q=password+reset&type=kb
```

**Response:**

```json
{
  "query": "password reset",
  "results": [
    {
      "type": "kb",
      "sourceId": "kb-001",
      "sourceName": "Product Manual",
      "match": "To reset the admin password, navigate to Settings...",
      "score": 0.92
    }
  ],
  "totalResults": 1
}
```

---

### List Mentions

**`GET /api/ui/sources/mentions`**

Returns available mention triggers for bot commands.

**Response:**

```json
[
  {
    "trigger": "@search",
    "description": "Search knowledge bases",
    "handler": "kb_search"
  },
  {
    "trigger": "@tool",
    "description": "Execute an MCP tool",
    "handler": "mcp_execute"
  }
]
```

---

## API Keys

### List API Keys

**`GET /api/ui/sources/api-keys`**

Returns all API keys for the bot.

**Response:**

```json
[
  {
    "id": "key-001",
    "name": "Production Key",
    "prefix": "gbo_****_a3f2",
    "scopes": ["kb:read", "mcp:execute"],
    "createdAt": "2026-03-10T09:00:00Z",
    "expiresAt": "2027-03-10T09:00:00Z"
  }
]
```

---

### Create API Key

**`POST /api/ui/sources/api-keys`**

Creates a new API key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Human-readable key name |
| `scopes` | string[] | Yes | Permission scopes |
| `expiresInDays` | integer | No | TTL in days (default: 365) |

**Request:**

```json
{
  "name": "CI Pipeline Key",
  "scopes": ["kb:read", "kb:write"],
  "expiresInDays": 90
}
```

**Response:**

```json
{
  "id": "key-002",
  "name": "CI Pipeline Key",
  "key": "gbo_live_k8x2m9p4q1r7...",
  "prefix": "gbo_****_q1r7",
  "scopes": ["kb:read", "kb:write"],
  "createdAt": "2026-06-04T12:00:00Z",
  "expiresAt": "2026-09-02T12:00:00Z"
}
```

> **Note:** The full key is only shown once at creation time.

---

### Delete API Key

**`DELETE /api/ui/sources/api-keys/:id`**

Permanently revokes an API key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | API key identifier (path param) |

**Request:**

```
DELETE /api/ui/sources/api-keys/key-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "key-002"
}
```

---

## Skills & Prompts

```http
POST /api/ui/skills/install              # { name, bot_id } install a skill
POST /api/ui/sources/prompts/save        # form { prompt_id, collection, prompt }
POST /api/ui/sources/mcp/add-from-catalog # form { server_id } add catalog MCP server
```

## See Also

- [MCP Format](../08-rest-api-tools/mcp-format.md) — Tool definition schema
- [Tool Definition](../08-rest-api-tools/tool-definition.md) — How tools are defined in BASIC
- [KB Integration](../08-rest-api-tools/get-integration.md) — Knowledge base injection into LLM context
