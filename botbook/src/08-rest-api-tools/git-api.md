# Git API 🟡 BETA

> **Git version control operations: status, diff, commit, push, and branch management**

---

## Base URL

```
/api/git
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### Get Status

**`GET /api/git/status`**

Returns the current git status of the repository including staged, modified, and untracked files.

**Response:**
```json
{
  "success": true,
  "status": {
    "branch": "main",
    "ahead": 0,
    "behind": 2,
    "clean": false,
    "staged": [
      {
        "path": "botserver/src/main.rs",
        "status": "modified",
        "staging": "staged"
      }
    ],
    "modified": [
      {
        "path": "botserver/Cargo.toml",
        "status": "modified",
        "staging": "unstaged"
      }
    ],
    "untracked": [
      {
        "path": "botserver/src/new_module.rs",
        "status": "new"
      }
    ],
    "conflicts": []
  }
}
```

**File Status Values:**

| Status | Description |
|--------|-------------|
| `modified` | File has been changed |
| `new` | File is untracked |
| `deleted` | File has been deleted |
| `renamed` | File has been renamed |
| `copied` | File has been copied |
| `conflicted` | Merge conflict present |

---

### Get Diff

**`GET /api/git/diff/:file`**

Returns the diff for a specific file (unstaged changes) or between commits.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file` | string | Yes | File path (path param) |
| `staged` | boolean | No | Show staged changes instead of unstaged (default: false) |
| `from` | string | No | Commit hash or ref for comparison (default: HEAD) |
| `to` | string | No | Target commit hash or ref (default: working tree) |

**Response:**
```json
{
  "success": true,
  "file": "botserver/src/main.rs",
  "diff": "--- a/botserver/src/main.rs\n+++ b/botserver/src/main.rs\n@@ -10,6 +10,8 @@\n use axum::Router;\n \n+use crate::api::routes;\n+\n #[tokio::main]\n async fn main() {\n-    let app = Router::new().route(\"/health\", get(health));\n+    let app = routes::create_router();\n }",
  "additions": 3,
  "deletions": 1,
  "binary": false
}
```

---

### Commit Changes

**`POST /api/git/commit`**

Stages and commits changes to the repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `message` | string | Yes | Commit message (JSON body) |
| `files` | array | No | Specific files to stage (JSON body). If omitted, stages all tracked modifications |
| `all` | boolean | No | Stage all modified and deleted files (default: false) |
| `amend` | boolean | No | Amend the last commit (default: false) |

**Request Body:**
```json
{
  "message": "feat: Add new database admin endpoints\n\n- Added schema inspection endpoint\n- Added row CRUD operations\n- Added batch delete support",
  "files": ["botserver/src/api/database.rs"],
  "all": false
}
```

**Response:**
```json
{
  "success": true,
  "commit": {
    "hash": "a1b2c3d4e5f6",
    "message": "feat: Add new database admin endpoints\n\n- Added schema inspection endpoint\n- Added row CRUD operations\n- Added batch delete support",
    "author": "Developer <dev@example.com>",
    "date": "2026-01-15T10:30:00Z",
    "files_changed": 1,
    "insertions": 85,
    "deletions": 12
  }
}
```

---

### Push Changes

**`POST /api/git/push`**

Pushes committed changes to the remote repository.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `remote` | string | No | Remote name (default: `origin`) |
| `branch` | string | No | Branch name (default: current branch) |
| `force` | boolean | No | Force push (default: false). **Warning: use with caution** |

**Request Body:**
```json
{
  "remote": "origin",
  "branch": "main",
  "force": false
}
```

**Response:**
```json
{
  "success": true,
  "push": {
    "remote": "origin",
    "branch": "main",
    "commits_pushed": 1,
    "from": "a1b2c3d",
    "to": "d4e5f6a"
  }
}
```

---

### List Branches

**`GET /api/git/branches`**

Lists all local and remote branches.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `remote` | boolean | No | Include remote branches (default: true) |

**Response:**
```json
{
  "success": true,
  "current": "main",
  "branches": [
    {
      "name": "main",
      "current": true,
      "remote": false,
      "upstream": "origin/main",
      "ahead": 0,
      "behind": 0
    },
    {
      "name": "feature/new-api",
      "current": false,
      "remote": false,
      "upstream": "origin/feature/new-api",
      "ahead": 3,
      "behind": 1
    },
    {
      "name": "origin/feature/new-api",
      "current": false,
      "remote": true,
      "upstream": null,
      "ahead": 0,
      "behind": 0
    }
  ]
}
```

---

### Create Branch

**`POST /api/git/branch/:name`**

Creates and optionally switches to a new branch.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | New branch name (path param) |
| `from` | string | No | Starting point — branch name or commit hash (default: current branch HEAD) |
| `checkout` | boolean | No | Switch to the new branch after creation (default: true) |

**Request Body:**
```json
{
  "from": "main",
  "checkout": true
}
```

**Response:**
```json
{
  "success": true,
  "branch": {
    "name": "feature/database-admin",
    "created_from": "main",
    "checkout": true
  },
  "message": "Branch 'feature/database-admin' created and checked out"
}
```

---

### Get Log

**`GET /api/git/log`**

Returns recent commit history.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `count` | integer | No | Number of commits to return (default: 20) |
| `branch` | string | No | Branch to show log for (default: current branch) |
| `since` | string | No | Only commits after this date (ISO 8601) |
| `author` | string | No | Filter by author name or email |

**Response:**
```json
{
  "success": true,
  "branch": "main",
  "commits": [
    {
      "hash": "a1b2c3d4e5f6",
      "short_hash": "a1b2c3d",
      "message": "feat: Add new database admin endpoints",
      "author": "Developer <dev@example.com>",
      "date": "2026-01-15T10:30:00Z",
      "files_changed": 3,
      "insertions": 120,
      "deletions": 15
    },
    {
      "hash": "b2c3d4e5f6a1",
      "short_hash": "b2c3d4e",
      "message": "fix: Correct WebSocket reconnection handling",
      "author": "Developer <dev@example.com>",
      "date": "2026-01-14T16:00:00Z",
      "files_changed": 1,
      "insertions": 8,
      "deletions": 3
    }
  ]
}
```

---

## Error Responses

| Status | Description |
|--------|-------------|
| `400` | Invalid request (missing message, invalid branch name) |
| `401` | Unauthorized (missing or invalid token) |
| `403` | Forbidden (force push not allowed, or insufficient privileges) |
| `404` | File or branch not found |
| `409` | Conflict (branch already exists, merge in progress) |
| `422` | Unprocessable Entity (nothing to commit, working tree clean) |
| `500` | Internal server error (git operation failed) |

---

## Usage Example

```javascript
// Check status
const status = await fetch('/api/git/status', {
  headers: { 'Authorization': 'Bearer mytoken' }
});
const { status: gitStatus } = await status.json();

// Get diff for a modified file
const diff = await fetch('/api/git/diff/botserver/src/main.rs', {
  headers: { 'Authorization': 'Bearer mytoken' }
});

// Create a branch and commit
await fetch('/api/git/branch/feature/my-change', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ from: 'main', checkout: true })
});

await fetch('/api/git/commit', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    message: 'feat: Add my new change',
    all: true
  })
});

// Push to remote
await fetch('/api/git/push', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ remote: 'origin', branch: 'feature/my-change' })
});
```

---

## See Also

- [Editor API](editor-api.md) — Edit files before committing
- [System API](system-api.md) — System version and deployment status
- [CI/CD Pipeline](../08-rest-api-tools/system-api.md) — Automated build and deploy
