# Database API

> **Generic CRUD API for direct database table operations**

---

## Base URL

```
/api/db
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List / Create Records

**`GET /api/db/:table`**

Retrieves all records from the specified table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `limit` | integer | No | Max records to return (default: 100) |
| `offset` | integer | No | Pagination offset (default: 0) |
| `order_by` | string | No | Column to sort by |
| `order_dir` | string | No | Sort direction: `asc` or `desc` |

**Response:**
```json
{
  "success": true,
  "data": [
    { "id": "a1b2c3", "name": "Example", "created_at": "2026-01-15T10:30:00Z" }
  ],
  "count": 1
}
```

---

**`POST /api/db/:table`**

Inserts a new record into the specified table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `*` | any | Yes | JSON body with column-value pairs |

**Request Body:**
```json
{
  "name": "New Record",
  "value": 42
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "d4e5f6",
    "name": "New Record",
    "value": 42,
    "created_at": "2026-01-15T10:30:00Z"
  }
}
```

---

### Get / Update / Delete Single Record

**`GET /api/db/:table/:id`**

Retrieves a single record by its primary key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `id` | string | Yes | Record ID (path param) |

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "a1b2c3",
    "name": "Example",
    "value": 100,
    "created_at": "2026-01-15T10:30:00Z"
  }
}
```

---

**`PUT /api/db/:table/:id`**

Updates an existing record by its primary key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `id` | string | Yes | Record ID (path param) |
| `*` | any | Yes | JSON body with column-value pairs to update |

**Request Body:**
```json
{
  "name": "Updated Record",
  "value": 99
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "a1b2c3",
    "name": "Updated Record",
    "value": 99,
    "updated_at": "2026-01-15T11:00:00Z"
  }
}
```

---

**`DELETE /api/db/:table/:id`**

Deletes a record by its primary key.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `id` | string | Yes | Record ID (path param) |

**Response:**
```json
{
  "success": true,
  "message": "Record deleted"
}
```

---

### Record Count

**`GET /api/db/:table/count`**

Returns the total number of records in the specified table.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |

**Response:**
```json
{
  "success": true,
  "count": 142
}
```

---

### Search Records

**`POST /api/db/:table/search`**

Searches records using filtering criteria.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `table` | string | Yes | Table name (path param) |
| `filters` | object | No | Key-value pairs for equality filters |
| `like` | object | No | Key-value pairs for LIKE (partial match) filters |
| `limit` | integer | No | Max results (default: 100) |
| `offset` | integer | No | Pagination offset |

**Request Body:**
```json
{
  "filters": {
    "status": "active"
  },
  "like": {
    "name": "john"
  },
  "limit": 10,
  "offset": 0
}
```

**Response:**
```json
{
  "success": true,
  "data": [
    { "id": "a1b2c3", "name": "John Smith", "status": "active" }
  ],
  "count": 1
}
```

---

## See Also

- [Database Admin API](database-admin-api.md) — Advanced schema and query operations
- [Reports API](reports-api.md) — Generate reports from database data
