# System API 🟡 BETA

> **System information, version management, and environment setup**

---

## Base URL

```
/api/system
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### Get Versions

**`GET /api/system/versions`**

Returns version information for all system components.

**Response:**
```json
{
  "success": true,
  "versions": {
    "botserver": {
      "version": "0.9.0",
      "commit": "a1b2c3d4",
      "build_date": "2026-01-15T10:00:00Z",
      "rust_version": "1.75.0"
    },
    "botui": {
      "version": "0.9.0",
      "commit": "a1b2c3d4",
      "build_date": "2026-01-15T10:00:00Z"
    },
    "database": {
      "engine": "PostgreSQL",
      "version": "16.1"
    },
    "cache": {
      "engine": "Valkey",
      "version": "7.2.4"
    },
    "storage": {
      "engine": "MinIO",
      "version": "2024-01-18T22:51:28Z"
    },
    "vectordb": {
      "engine": "Qdrant",
      "version": "1.7.4"
    }
  }
}
```

---

### Check for Updates

**`POST /api/system/check-updates`**

Checks for available updates to system components by querying the configured ALM/Forgejo instance.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `components` | array | No | Specific components to check (default: all) |

**Request Body:**
```json
{
  "components": ["botserver", "botui"]
}
```

**Response:**
```json
{
  "success": true,
  "updates_available": true,
  "current": {
    "botserver": "a1b2c3d4",
    "botui": "a1b2c3d4"
  },
  "latest": {
    "botserver": "d4e5f6a7",
    "botui": "d4e5f6a7"
  },
  "details": [
    {
      "component": "botserver",
      "current_commit": "a1b2c3d4",
      "latest_commit": "d4e5f6a7",
      "ahead": 3,
      "message": "feat: add new endpoints"
    }
  ]
}
```

---

### Get Setup Status

**`GET /api/setup/status`**

Returns the current bootstrap/setup status of the system. Useful for checking if the initial configuration has been completed.

**Response:**
```json
{
  "success": true,
  "setup": {
    "completed": true,
    "initialized_at": "2026-01-15T10:00:00Z",
    "services": {
      "database": "ready",
      "cache": "ready",
      "storage": "ready",
      "vectordb": "ready",
      "directory": "ready",
      "llm": "ready"
    },
    "admin_user": {
      "created": true,
      "email": "admin@example.com"
    }
  }
}
```

---

### Configure System

**`POST /api/setup/configure`**

Applies system configuration. Used during initial setup or to update environment settings.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `llm_url` | string | No | LLM API endpoint URL |
| `llm_key` | string | No | LLM API key |
| `llm_model` | string | No | Default LLM model name |
| `llm_provider` | string | No | LLM provider type (openai, anthropic, nvidia) |
| `drive_host` | string | No | MinIO/Drive host address |
| `drive_access_key` | string | No | MinIO access key |
| `drive_secret` | string | No | MinIO secret key |
| `encryption_key` | string | No | System encryption key |

**Request Body:**
```json
{
  "llm_url": "https://integrate.api.nvidia.com/v1/chat/completions",
  "llm_model": "openai/gpt-oss-120b",
  "llm_provider": "nvidia"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Configuration updated successfully",
  "restart_required": false
}
```

---

## Error Responses

| Status | Description |
|--------|-------------|
| `400` | Invalid request (malformed parameters) |
| `401` | Unauthorized (missing or invalid token) |
| `403` | Forbidden (admin privileges required) |
| `404` | Setup endpoint not found |
| `409` | Conflict (system already configured) |
| `500` | Internal server error |

---

## Usage Example

```javascript
// Check system versions
const versions = await fetch('/api/system/versions', {
  headers: { 'Authorization': 'Bearer mytoken' }
});
const { versions: sysVersions } = await versions.json();
console.log(`BotServer: ${sysVersions.botserver.version}`);

// Check for updates
const updates = await fetch('/api/system/check-updates', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ components: ['botserver'] })
});
const { updates_available } = await updates.json();

// Get setup status
const status = await fetch('/api/setup/status', {
  headers: { 'Authorization': 'Bearer mytoken' }
});
const { setup } = await status.json();
if (!setup.completed) {
  console.log('System not yet configured');
}

// Apply configuration
await fetch('/api/setup/configure', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    llm_model: 'openai/gpt-oss-120b',
    llm_provider: 'nvidia'
  })
});
```

---

## See Also

- [Git API](git-api.md) — Version control and deployment
- [Monitoring API](monitoring-api.md) — System health and metrics
- [Security API](security-api.md) — Security configuration
