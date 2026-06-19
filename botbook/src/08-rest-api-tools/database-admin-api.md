# Database Admin API 🟡 BETA

> **Advanced database administration: schema inspection, data manipulation, and direct queries**

---

## Base URL

```
/api/database
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Admin privileges required for write operations.

---

## Endpoints

### Get Schema

**`GET /api/database/schema`**

Returns the full database schema including all tables, columns, types, and constraints.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | No | Filter to a specific table name |

**Response:**
```json
{
  "success": true,
  "schema": [
    {
      "table_name": "users",
      "columns": [
        {
          "name": "id",
          "type": "uuid",
          "nullable": false,
          "is_primary": true,
          "default": "gen_random_uuid()"
        },
        {
          "name": "email",
          "type": "varchar(255)",
          "nullable": false,
          "is_primary": false,
          "default": null
        },
        {
          "name": "created_at",
          "type": "timestamptz",
          "nullable": false,
          "is_primary": false,
          "default": "now()"
        }
      ],
      "indexes": ["users_email_idx"],
      "row_count": 142
    }
  ]
}
```

---

### Get Table Data

**`GET /api/database/table/:name/data`**

Retrieves paginated data from a specific table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `limit` | integer | No | Max rows (default: 50, max: 1000) |
| `offset` | integer | No | Pagination offset (default: 0) |
| `sort` | string | No | Column to sort by |
| `order` | string | No | `asc` or `desc` (default: `asc`) |
| `filter` | string | No | SQL WHERE clause fragment (e.g., `status = 'active'`) |

**Response:**
```json
{
  "success": true,
  "table": "users",
  "data": [
    {
      "id": "a1b2c3",
      "email": "user@example.com",
      "created_at": "2026-01-15T10:30:00Z"
    }
  ],
  "total": 142,
  "limit": 50,
  "offset": 0
}
```

---

### Execute Query

**`POST /api/database/query`**

Executes a raw SQL query. SELECT queries are read-only; write queries require admin privileges.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | SQL query to execute (JSON body) |
| `params` | array | No | Query parameters for prepared statements |

**Request Body:**
```json
{
  "query": "SELECT * FROM users WHERE created_at > $1 AND status = $2 LIMIT 10",
  "params": ["2026-01-01", "active"]
}
```

**Response:**
```json
{
  "success": true,
  "rows": [
    {
      "id": "a1b2c3",
      "email": "user@example.com",
      "status": "active",
      "created_at": "2026-01-15T10:30:00Z"
    }
  ],
  "row_count": 1,
  "execution_time_ms": 12
}
```

---

### Insert / Update Row

**`POST /api/database/table/:name/row`**

Inserts a new row into the specified table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `*` | any | Yes | JSON body with column-value pairs |

**Request Body:**
```json
{
  "email": "new@example.com",
  "name": "New User",
  "status": "active"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "d4e5f6",
    "email": "new@example.com",
    "name": "New User",
    "status": "active",
    "created_at": "2026-01-15T11:00:00Z"
  }
}
```

---

**`PUT /api/database/table/:name/row`**

Updates an existing row identified by its primary key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `id` | string | Yes | Row primary key (JSON body) |
| `*` | any | Yes | JSON body with column-value pairs to update |

**Request Body:**
```json
{
  "id": "d4e5f6",
  "email": "updated@example.com",
  "status": "inactive"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "d4e5f6",
    "email": "updated@example.com",
    "status": "inactive",
    "updated_at": "2026-01-15T12:00:00Z"
  }
}
```

---

### Delete Row

**`DELETE /api/database/table/:name/row/:id`**

Deletes a single row by its primary key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `id` | string | Yes | Row primary key (path param) |

**Response:**
```json
{
  "success": true,
  "message": "Row deleted from users"
}
```

---

### Batch Delete Rows

**`POST /api/database/table/:name/rows/batch-delete`**

Deletes multiple rows by their primary keys in a single operation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `ids` | array | Yes | Array of primary key values (JSON body) |

**Request Body:**
```json
{
  "ids": ["a1b2c3", "d4e5f6", "g7h8i9"]
}
```

**Response:**
```json
{
  "success": true,
  "deleted_count": 3,
  "message": "3 rows deleted from users"
}
```

---

### Create Table

**`POST /api/database/table`**

Creates a new database table with the specified columns.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (JSON body) |
| `columns` | array | Yes | Array of column definitions (JSON body) |

**Column Definition:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Column name |
| `type` | string | Yes | SQL data type (e.g., `uuid`, `varchar(255)`, `integer`, `timestamptz`) |
| `nullable` | boolean | No | Allow NULL values (default: true) |
| `default` | string | No | Default value expression |
| `primary` | boolean | No | Is primary key (default: false) |

**Request Body:**
```json
{
  "name": "tickets",
  "columns": [
    { "name": "id", "type": "uuid", "primary": true, "default": "gen_random_uuid()" },
    { "name": "title", "type": "varchar(255)", "nullable": false },
    { "name": "status", "type": "varchar(50)", "nullable": false, "default": "'open'" },
    { "name": "created_at", "type": "timestamptz", "nullable": false, "default": "now()" }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "table": "tickets",
  "message": "Table created successfully"
}
```

---

### Add Column to Table

**`POST /api/database/table/:name/column`**

Adds a new column to an existing table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Table name (path param) |
| `column_name` | string | Yes | Column name (JSON body) |
| `column_type` | string | Yes | SQL data type (JSON body) |
| `nullable` | boolean | No | Allow NULL values (default: true) |
| `default` | string | No | Default value expression |

**Request Body:**
```json
{
  "column_name": "priority",
  "column_type": "integer",
  "nullable": true,
  "default": "0"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Column 'priority' added to table 'tickets'"
}
```

---

## Error Responses

| Status | Description |
|--------|-------------|
| `400` | Invalid request (malformed SQL, missing parameters) |
| `401` | Unauthorized (missing or invalid token) |
| `403` | Forbidden (insufficient privileges) |
| `404` | Table not found |
| `409` | Conflict (table/column already exists) |
| `422` | Unprocessable Entity (invalid column type) |
| `500` | Internal server error |

---

## Usage Example

```javascript
// Get schema for all tables
const schema = await fetch('/api/database/schema', {
  headers: { 'Authorization': 'Bearer mytoken' }
});

// Query specific data
const users = await fetch('/api/database/query', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    query: 'SELECT id, email FROM users WHERE status = $1',
    params: ['active']
  })
});

// Create a new table
await fetch('/api/database/table', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer mytoken',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    name: 'audit_log',
    columns: [
      { name: 'id', type: 'uuid', primary: true },
      { name: 'action', type: 'varchar(100)', nullable: false },
      { name: 'performed_at', type: 'timestamptz', default: 'now()' }
    ]
  })
});
```

---

## See Also

- [Database API](db-api.md) — Simple CRUD operations
- [Reports API](reports-api.md) — Generate reports from query results
- [Security API](security-api.md) — SQL guard and input sanitization
