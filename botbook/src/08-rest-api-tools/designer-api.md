# Designer API 🟡 BETA

> **UI designer for visual bot dialog creation — load, edit, validate, and export `.gbdialog` files through a visual interface.**

---

## Base URL

```
/api/ui/designer
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Files

**`GET /api/ui/designer/files`**

Lists all dialog files available in the designer.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | No | Filter by bot name (default: all bots) |
| `path` | string | No | Subdirectory path (default: root) |

**Response:**

```json
{
  "files": [
    {
      "name": "start.bas",
      "path": "start.bas",
      "size": 512,
      "modified_at": "2025-06-04T10:00:00Z",
      "bot": "default"
    },
    {
      "name": "check_inventory.bas",
      "path": "tools/check_inventory.bas",
      "size": 1024,
      "modified_at": "2025-06-03T14:00:00Z",
      "bot": "default"
    },
    {
      "name": "tables.bas",
      "path": "tables.bas",
      "size": 256,
      "modified_at": "2025-06-01T08:00:00Z",
      "bot": "default"
    }
  ],
  "directories": [
    "tools/",
    "handlers/",
    "schedulers/"
  ]
}
```

---

### Load File

**`GET /api/ui/designer/load`**

Loads a file's content for editing.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | Yes | Bot name |
| `path` | string | Yes | Relative file path |
| `version` | string | No | Specific version (default: latest) |

**Response:**

```json
{
  "bot": "default",
  "path": "start.bas",
  "content": "ADD SUGGESTION \"Check inventory\"\nADD SUGGESTION \"Create report\"\nTALK \"Hello! How can I help?\"",
  "modified_at": "2025-06-04T10:00:00Z",
  "size": 128,
  "line_count": 3,
  "syntax_valid": true
}
```

---

### Save File

**`POST /api/ui/designer/save`**

Saves content to a file in the bot's dialog directory.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | Yes | Bot name |
| `path` | string | Yes | Relative file path |
| `content` | string | Yes | File content |
| `validate` | boolean | No | Run validation before saving (default: true) |

**Request Body:**

```json
{
  "bot": "default",
  "path": "tools/check_inventory.bas",
  "content": "items = GET FROM inventory WHERE quantity < 10\nIF COUNT(items) = 0 THEN\n    TALK \"All items well stocked!\"\nELSE\n    response = \"Low stock items:\\n\"\n    FOR EACH item IN items\n        response = response + \"- \" + item.name + \": \" + item.quantity + \"\\n\"\n    NEXT\n    TALK response\nEND IF",
  "validate": true
}
```

**Response:**

```json
{
  "success": true,
  "bot": "default",
  "path": "tools/check_inventory.bas",
  "size": 312,
  "line_count": 10,
  "syntax_valid": true,
  "saved_at": "2025-06-04T11:00:00Z"
}
```

---

### Validate File

**`POST /api/ui/designer/validate`**

Validates BASIC script syntax without saving.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | Yes | Bot name |
| `content` | string | Yes | BASIC script content |
| `filename` | string | No | Filename for context (helps with error messages) |

**Request Body:**

```json
{
  "bot": "default",
  "content": "TALK \"Hello\"\nINVALID_KEYWORD \"test\"",
  "filename": "test.bas"
}
```

**Response (errors):**

```json
{
  "valid": false,
  "errors": [
    {
      "line": 2,
      "column": 1,
      "message": "Unknown keyword: INVALID_KEYWORD",
      "severity": "error"
    }
  ],
  "warnings": []
}
```

**Response (success):**

```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "line": 5,
      "column": 1,
      "message": "Unused variable 'temp'",
      "severity": "warning"
    }
  ]
}
```

---

### Export File

**`GET /api/ui/designer/export`**

Exports a compiled `.ast` file for a BASIC script.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | Yes | Bot name |
| `path` | string | Yes | Source `.bas` file path |
| `format` | string | No | `ast` (default), `json`, `txt` |

**Response:**

Returns the file as a download:

```http
HTTP/1.1 200 OK
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="check_inventory.ast"
```

The response body contains the compiled bytecode. For `json` format, returns:

```json
{
  "source": "check_inventory.bas",
  "format": "ast",
  "compiled_at": "2025-06-04T11:00:00Z",
  "size_bytes": 1024,
  "checksum": "sha256:abc123..."
}
```

---

### Dialog Management

**`GET /api/ui/designer/dialogs`**

Lists all available dialog templates.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `bot` | string | No | Filter by bot |
| `category` | string | No | Filter by category |

**Response:**

```json
{
  "dialogs": [
    {
      "id": "dlg_001",
      "name": "Welcome Dialog",
      "category": "onboarding",
      "description": "Initial greeting with suggestion buttons",
      "bot": "default",
      "file_count": 1,
      "created_at": "2025-06-01T08:00:00Z"
    },
    {
      "id": "dlg_002",
      "name": "Inventory Checker",
      "category": "tools",
      "description": "Checks inventory levels and reports low stock",
      "bot": "default",
      "file_count": 2,
      "created_at": "2025-06-02T10:00:00Z"
    }
  ]
}
```

---

**`POST /api/ui/designer/dialogs`**

Creates a new dialog from a template.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Dialog name |
| `bot` | string | Yes | Target bot |
| `category` | string | No | Category |
| `description` | string | No | Description |
| `template` | string | No | Base template to clone from |
| `files` | array | No | Initial files to include |

**Request Body:**

```json
{
  "name": "Customer Lookup",
  "bot": "default",
  "category": "tools",
  "description": "Search customers by name or email",
  "files": [
    {
      "path": "tools/customer_lookup.bas",
      "content": "HEAR \"Enter customer name or email\" AS query\nresults = FIND query IN customers\nIF COUNT(results) = 0 THEN\n    TALK \"No customers found.\"\nELSE\n    FOR EACH c IN results\n        TALK c.name + \" - \" + c.email\n    NEXT\nEND IF"
    }
  ]
}
```

**Response:**

```json
{
  "id": "dlg_003",
  "name": "Customer Lookup",
  "bot": "default",
  "category": "tools",
  "file_count": 1,
  "created_at": "2025-06-04T12:00:00Z"
}
```

---

**`GET /api/ui/designer/dialogs/:id`**

Retrieves a dialog with all its files.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dialog ID |

**Response:**

```json
{
  "id": "dlg_002",
  "name": "Inventory Checker",
  "category": "tools",
  "description": "Checks inventory levels and reports low stock",
  "bot": "default",
  "files": [
    {
      "path": "tools/check_inventory.bas",
      "size": 312,
      "line_count": 10,
      "syntax_valid": true,
      "modified_at": "2025-06-03T14:00:00Z"
    },
    {
      "path": "tools/restock.bas",
      "size": 256,
      "line_count": 8,
      "syntax_valid": true,
      "modified_at": "2025-06-03T14:30:00Z"
    }
  ],
  "created_at": "2025-06-02T10:00:00Z"
}
```

---

## Designer File Types

| Extension | Description | Editable |
|-----------|-------------|----------|
| `.bas` | BASIC source script | Yes |
| `.ast` | Compiled BASIC bytecode | No (export only) |
| `.html` | HTMX dialog markup | Yes |
| `.css` | Dialog styles | Yes |
| `.js` | Client-side scripts | Yes |
| `.json` | Configuration files | Yes |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request (validation errors) |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | File not found |
| 409 | Conflict (file modified externally) |
| 500 | Internal Server Error |

---

## See Also

- [File Operations](./files-api.md) — raw file management
- [Compilation](./compilation.md) — BASIC compilation pipeline
- [Tool Definition](./tool-definition.md) — how tools are registered
- [Storage API](./storage-api.md) — Drive (MinIO) integration
