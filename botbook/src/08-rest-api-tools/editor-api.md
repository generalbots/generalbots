# Code Editor API 🟡 BETA

> **File browsing and editing API for in-browser code editor**

---

## Base URL

```
/api/editor
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Files

**`GET /api/editor/files`**

Lists files and directories in the workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | No | Directory path relative to workspace root (default: `/`) |
| `include_hidden` | boolean | No | Include hidden files (default: false) |

**Response:**
```json
{
  "success": true,
  "path": "/",
  "entries": [
    {
      "name": "botserver",
      "type": "directory",
      "size": 4096,
      "modified": "2026-01-15T10:30:00Z"
    },
    {
      "name": "Cargo.toml",
      "type": "file",
      "size": 1024,
      "modified": "2026-01-14T08:00:00Z"
    }
  ]
}
```

---

### Get File Contents

**`GET /api/editor/file/*path`**

Retrieves the contents of a file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `*path` | string | Yes | File path relative to workspace root (path param) |

**Response:**
```json
{
  "success": true,
  "path": "botserver/src/main.rs",
  "content": "use axum::routing::get;\nuse axum::Router;\n\n#[tokio::main]\nasync fn main() {\n    let app = Router::new().route(\"/health\", get(health));\n}",
  "size": 186,
  "modified": "2026-01-15T10:30:00Z",
  "language": "rust"
}
```

---

### Save File Contents

**`POST /api/editor/file/*path`**

Creates or overwrites a file with the provided content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `*path` | string | Yes | File path relative to workspace root (path param) |
| `content` | string | Yes | File content (JSON body) |

**Request Body:**
```json
{
  "content": "fn main() {\n    println!(\"Hello, world!\");\n}\n"
}
```

**Response:**
```json
{
  "success": true,
  "path": "examples/hello.rs",
  "size": 48,
  "message": "File saved"
}
```

---

## Error Responses

| Status | Description |
|--------|-------------|
| `400` | Invalid request (missing path or content) |
| `401` | Unauthorized (missing or invalid token) |
| `403` | Forbidden (path outside workspace or read-only) |
| `404` | File or directory not found |
| `413` | File too large (exceeds max size limit) |
| `500` | Internal server error |

---

## Supported Languages

The editor auto-detects language from file extension for syntax highlighting:

| Extension | Language |
|-----------|----------|
| `.rs` | Rust |
| `.py` | Python |
| `.js` | JavaScript |
| `.ts` | TypeScript |
| `.html` | HTML |
| `.css` | CSS |
| `.json` | JSON |
| `.yaml` / `.yml` | YAML |
| `.toml` | TOML |
| `.sql` | SQL |
| `.bas` | BASIC |
| `.md` | Markdown |

---

## Usage Example

```javascript
// List files in botserver/src
const res = await fetch('/api/editor/files?path=botserver/src', {
  headers: { 'Authorization': 'Bearer mytoken' }
});
const { entries } = await res.json();

// Read a file
const file = await fetch('/api/editor/file/botserver/src/main.rs', {
  headers: { 'Authorization': 'Bearer mytoken' }
});
const { content } = await file.json();

// Save a file
await fetch('/api/editor/file/examples/test.rs', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ content: 'fn main() {}' })
});
```

---

## Editor Session Endpoints

```http
GET  /api/editor/files              # workspace file list
GET  /api/editor/file/*path         # read file
POST /api/editor/file/*path         # save file (JSON { content })
POST /api/editor/save               # save form (path + content)
GET  /api/editor/save-as            # save-as dialog fragment
POST /api/editor/undo               # undo (form content)
POST /api/editor/redo               # redo (form content)
POST /api/editor/format             # pretty-print JSON (form content)
POST /api/editor/magic              # { code } -> { improved_code, explanation } (LLM)
POST /api/files/write               # { path, content (base64), bucket }
GET  /api/files/list                # explorer file grid (HTML)
GET  /api/files/download            # download file
```

Files are stored under `/tmp/gb-editor/{bucket}` on the server workspace.

## See Also

- [Files API](files-api.md) — Upload, download, and manage bot files
- [Storage API](storage-api.md) — MinIO/Drive storage operations
