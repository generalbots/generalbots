# Workspaces API 🟡 BETA

> **Collaborative workspace management for organizing pages, members, and shared content.**

---

## Base URL

```
/api/workspaces
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Workspace Management

### List Workspaces

**`GET /api/workspaces/`**

Returns all workspaces the authenticated user has access to.

**Response:**

```json
[
  {
    "id": "ws-001",
    "name": "Engineering",
    "description": "Engineering team workspace",
    "owner_id": "user-admin-01",
    "memberCount": 12,
    "pageCount": 34,
    "created_at": "2026-01-10T08:00:00Z",
    "updated_at": "2026-06-03T17:45:00Z"
  }
]
```

---

### Create Workspace

**`POST /api/workspaces/`**

Creates a new workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Workspace name |
| `description` | string | No | Workspace description |
| `isPrivate` | boolean | No | Private workspace (default: false) |

**Request:**

```json
{
  "name": "Marketing",
  "description": "Marketing team collaboration space",
  "isPrivate": false
}
```

**Response:**

```json
{
  "id": "ws-002",
  "name": "Marketing",
  "description": "Marketing team collaboration space",
  "owner_id": "user-123",
  "isPrivate": false,
  "memberCount": 1,
  "pageCount": 0,
  "created_at": "2026-06-04T12:00:00Z"
}
```

---

### Get Workspace

**`GET /api/workspaces/:workspace_id`**

Returns detailed information about a specific workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |

**Request:**

```
GET /api/workspaces/ws-001
```

**Response:**

```json
{
  "id": "ws-001",
  "name": "Engineering",
  "description": "Engineering team workspace",
  "owner_id": "user-admin-01",
  "isPrivate": false,
  "memberCount": 12,
  "pageCount": 34,
  "createdAt": "2026-01-10T08:00:00Z",
  "updatedAt": "2026-06-03T17:45:00Z"
}
```

---

### Update Workspace

**`PUT /api/workspaces/:workspace_id`**

Updates workspace settings.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |
| `name` | string | No | New workspace name |
| `description` | string | No | New description |
| `isPrivate` | boolean | No | Privacy setting |

**Request:**

```json
{
  "name": "Engineering (Renamed)",
  "description": "Updated description"
}
```

**Response:**

```json
{
  "id": "ws-001",
  "name": "Engineering (Renamed)",
  "description": "Updated description",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Delete Workspace

**`DELETE /api/workspaces/:workspace_id`**

Permanently deletes a workspace and all its pages. Only the owner may perform this action.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |

**Request:**

```
DELETE /api/workspaces/ws-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "ws-002"
}
```

---

## Members

### Add Member

**`POST /api/workspaces/:workspace_id/members`**

Adds a user to a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |
| `user_id` | string | Yes | User to add |
| `role` | string | No | Role: `viewer`, `editor`, `admin` (default: `viewer`) |

**Request:**

```json
{
  "user_id": "user-456",
  "role": "editor"
}
```

**Response:**

```json
{
  "workspace_id": "ws-001",
  "user_id": "user-456",
  "role": "editor",
  "added_at": "2026-06-04T12:00:00Z"
}
```

---

### Remove Member

**`DELETE /api/workspaces/:workspace_id/members/:user_id`**

Removes a user from a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |
| `user_id` | string | Yes | User to remove (path param) |

**Request:**

```
DELETE /api/workspaces/ws-001/members/user-456
```

**Response:**

```json
{
  "removed": true,
  "workspace_id": "ws-001",
  "user_id": "user-456"
}
```

---

## Search

### Search Workspace

**`GET /api/workspaces/:workspace_id/search`**

Full-text search across all pages in a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |
| `q` | string | Yes | Search query (query param) |

**Request:**

```
GET /api/workspaces/ws-001/search?q=deployment+guide
```

**Response:**

```json
{
  "query": "deployment guide",
  "results": [
    {
      "pageId": "page-101",
      "title": "Production Deployment Guide",
      "excerpt": "Step-by-step guide for deploying to production...",
      "score": 0.95,
      "updatedAt": "2026-05-28T10:00:00Z"
    },
    {
      "pageId": "page-115",
      "title": "Staging Deployment Checklist",
      "excerpt": "Before deploying to staging, ensure...",
      "score": 0.82,
      "updatedAt": "2026-06-01T14:30:00Z"
    }
  ],
  "totalResults": 2
}
```

---

## Pages

### List Pages

**`GET /api/workspaces/:workspace_id/pages`**

Returns all pages in a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |

**Request:**

```
GET /api/workspaces/ws-001/pages
```

**Response:**

```json
[
  {
    "id": "page-101",
    "title": "Production Deployment Guide",
    "slug": "production-deployment-guide",
    "author_id": "user-admin-01",
    "createdAt": "2026-03-15T09:00:00Z",
    "updatedAt": "2026-05-28T10:00:00Z"
  },
  {
    "id": "page-102",
    "title": "Architecture Overview",
    "slug": "architecture-overview",
    "author_id": "user-789",
    "createdAt": "2026-02-01T14:00:00Z",
    "updatedAt": "2026-06-02T11:15:00Z"
  }
]
```

---

### Create Page

**`POST /api/workspaces/:workspace_id/pages`**

Creates a new page in a workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `workspace_id` | string | Yes | Workspace identifier (path param) |
| `title` | string | Yes | Page title |
| `content` | string | No | Page content (Markdown) |
| `parentId` | string | No | Parent page ID for nesting |

**Request:**

```json
{
  "title": "Runbook: Database Failover",
  "content": "# Database Failover\n\n## Prerequisites\n- Access to PostgreSQL primary...\n",
  "parentId": "page-101"
}
```

**Response:**

```json
{
  "id": "page-103",
  "title": "Runbook: Database Failover",
  "slug": "runbook-database-failover",
  "content": "# Database Failover\n\n## Prerequisites\n- Access to PostgreSQL primary...\n",
  "author_id": "user-123",
  "parentId": "page-101",
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

### Get Page

**`GET /api/workspaces/pages/:page_id`**

Returns full page content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page_id` | string | Yes | Page identifier (path param) |

**Request:**

```
GET /api/workspaces/pages/page-101
```

**Response:**

```json
{
  "id": "page-101",
  "title": "Production Deployment Guide",
  "slug": "production-deployment-guide",
  "content": "# Production Deployment Guide\n\n## Overview\nThis document covers...",
  "author_id": "user-admin-01",
  "createdAt": "2026-03-15T09:00:00Z",
  "updatedAt": "2026-05-28T10:00:00Z"
}
```

---

### Update Page

**`PUT /api/workspaces/pages/:page_id`**

Updates page content or metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page_id` | string | Yes | Page identifier (path param) |
| `title` | string | No | New title |
| `content` | string | No | New content (Markdown) |

**Request:**

```json
{
  "content": "# Production Deployment Guide\n\n## Updated Steps\n1. Pull latest image..."
}
```

**Response:**

```json
{
  "id": "page-101",
  "title": "Production Deployment Guide",
  "content": "# Production Deployment Guide\n\n## Updated Steps\n1. Pull latest image...",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

## Commands

### List Commands

**`GET /api/workspaces/commands`**

Returns available slash commands for workspace pages.

**Response:**

```json
[
  {
    "command": "/embed",
    "description": "Embed a bot or widget in a page",
    "usage": "/embed bot=<botname> width=400 height=600"
  },
  {
    "command": "/include",
    "description": "Include content from another page",
    "usage": "/include page=<page-slug>"
  },
  {
    "command": "/code",
    "description": "Execute BASIC code snippet inline",
    "usage": "/code lang=basic TALK \"Hello\""
  }
]
```

---

## Current Workspace & Page Editing

```http
GET  /api/ui/workspaces/commands            # slash command list
GET  /api/ui/workspaces/current/invite      # invite members modal
GET  /api/pages/current                     # current page fragment
PUT  /api/pages/current                     # { title } rename
GET  /api/ui/pages/current/blocks           # block editor content
```

## See Also

- [Groups API](../08-rest-api-tools/groups-api.md) — User group management
- [Users API](../08-rest-api-tools/users-api.md) — User account management
- [Files API](../08-rest-api-tools/files-api.md) — File storage and retrieval
