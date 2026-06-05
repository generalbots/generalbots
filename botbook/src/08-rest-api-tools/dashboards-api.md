# Dashboards API

> **Endpoints for creating, managing, and querying custom dashboards with configurable widgets and data sources.**

---

## Base URL

```
/api/dashboards
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List / Create Dashboards

**`GET /api/dashboards`**

Retrieves all dashboards accessible to the authenticated user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `search` | string | No | Filter by dashboard name |

**Response:**

```json
{
  "dashboards": [
    {
      "id": "dash_abc123",
      "name": "Sales Overview",
      "description": "Real-time sales metrics",
      "created_by": "user_001",
      "created_at": "2025-06-01T10:00:00Z",
      "updated_at": "2025-06-03T14:30:00Z",
      "widget_count": 6,
      "is_public": true
    }
  ],
  "total": 12,
  "page": 1,
  "limit": 20
}
```

---

**`POST /api/dashboards`**

Creates a new dashboard.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Dashboard name |
| `description` | string | No | Dashboard description |
| `is_public` | boolean | No | Visibility (default: false) |
| `template_id` | string | No | Base template ID |

**Request Body:**

```json
{
  "name": "Marketing Analytics",
  "description": "Campaign performance dashboard",
  "is_public": true,
  "template_id": "tmpl_001"
}
```

**Response:**

```json
{
  "id": "dash_xyz789",
  "name": "Marketing Analytics",
  "description": "Campaign performance dashboard",
  "created_by": "user_001",
  "created_at": "2025-06-04T09:00:00Z",
  "is_public": true
}
```

---

### Get Dashboard Templates

**`GET /api/dashboards/templates`**

Returns available dashboard templates.

**Response:**

```json
{
  "templates": [
    {
      "id": "tmpl_001",
      "name": "Sales Overview",
      "description": "Pre-configured sales metrics dashboard",
      "category": "sales",
      "widget_count": 8,
      "preview_url": "/templates/sales-overview/preview.png"
    },
    {
      "id": "tmpl_002",
      "name": "System Health",
      "description": "Infrastructure monitoring dashboard",
      "category": "operations",
      "widget_count": 5,
      "preview_url": "/templates/system-health/preview.png"
    }
  ]
}
```

---

### Get / Update / Delete Dashboard

**`GET /api/dashboards/:id`**

Retrieves a single dashboard with its widgets.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |

**Response:**

```json
{
  "id": "dash_abc123",
  "name": "Sales Overview",
  "description": "Real-time sales metrics",
  "created_by": "user_001",
  "created_at": "2025-06-01T10:00:00Z",
  "updated_at": "2025-06-03T14:30:00Z",
  "is_public": true,
  "widgets": [
    {
      "widget_id": "wid_001",
      "type": "line_chart",
      "title": "Revenue Trend",
      "position": { "x": 0, "y": 0, "w": 6, "h": 4 },
      "data_source_id": "src_001",
      "config": { "time_range": "30d", "metric": "revenue" }
    }
  ]
}
```

---

**`PUT /api/dashboards/:id`**

Updates an existing dashboard's metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |
| `name` | string | No | Updated name |
| `description` | string | No | Updated description |
| `is_public` | boolean | No | Updated visibility |

**Request Body:**

```json
{
  "name": "Sales Overview v2",
  "description": "Updated sales metrics with Q2 data",
  "is_public": false
}
```

**Response:**

```json
{
  "id": "dash_abc123",
  "name": "Sales Overview v2",
  "updated_at": "2025-06-04T11:00:00Z"
}
```

---

**`DELETE /api/dashboards/:id`**

Deletes a dashboard and all associated widgets.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |

**Response:**

```json
{
  "success": true,
  "message": "Dashboard deleted"
}
```

---

### Widget Management

**`POST /api/dashboards/:id/widgets`**

Adds a new widget to a dashboard.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |
| `type` | string | Yes | Widget type: `line_chart`, `bar_chart`, `pie_chart`, `stat_card`, `table`, `gauge`, `heatmap` |
| `title` | string | Yes | Widget title |
| `data_source_id` | string | Yes | Data source to query |
| `position` | object | No | Grid position `{x, y, w, h}` |
| `config` | object | No | Widget-specific configuration |

**Request Body:**

```json
{
  "type": "stat_card",
  "title": "Total Revenue",
  "data_source_id": "src_001",
  "position": { "x": 0, "y": 0, "w": 3, "h": 2 },
  "config": {
    "metric": "revenue",
    "aggregation": "sum",
    "time_range": "30d",
    "format": "currency",
    "currency": "USD"
  }
}
```

**Response:**

```json
{
  "widget_id": "wid_002",
  "type": "stat_card",
  "title": "Total Revenue",
  "dashboard_id": "dash_abc123",
  "created_at": "2025-06-04T11:00:00Z"
}
```

---

**`PUT /api/dashboards/:id/widgets/:widget_id`**

Updates an existing widget's configuration.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |
| `widget_id` | string | Yes | Widget ID |
| `title` | string | No | Updated title |
| `position` | object | No | Updated grid position |
| `config` | object | No | Updated widget config |

**Request Body:**

```json
{
  "title": "Total Revenue (USD)",
  "config": {
    "metric": "revenue",
    "aggregation": "sum",
    "time_range": "90d",
    "format": "currency",
    "currency": "USD"
  }
}
```

**Response:**

```json
{
  "widget_id": "wid_002",
  "title": "Total Revenue (USD)",
  "updated_at": "2025-06-04T12:00:00Z"
}
```

---

**`DELETE /api/dashboards/:id/widgets/:widget_id`**

Removes a widget from a dashboard.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |
| `widget_id` | string | Yes | Widget ID |

**Response:**

```json
{
  "success": true,
  "message": "Widget removed"
}
```

---

### Widget Data

**`GET /api/dashboards/:id/widgets/:widget_id/data`**

Fetches the data payload for a specific widget.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Dashboard ID |
| `widget_id` | string | Yes | Widget ID |
| `time_range` | string | No | Override time range: `1h`, `24h`, `7d`, `30d`, `90d` |
| `refresh` | boolean | No | Force cache bypass (default: false) |

**Response (line_chart):**

```json
{
  "widget_id": "wid_001",
  "type": "line_chart",
  "data": {
    "labels": ["2025-06-01", "2025-06-02", "2025-06-03", "2025-06-04"],
    "series": [
      {
        "name": "Revenue",
        "values": [12500, 14200, 13800, 16100]
      }
    ]
  },
  "generated_at": "2025-06-04T12:00:00Z"
}
```

**Response (stat_card):**

```json
{
  "widget_id": "wid_002",
  "type": "stat_card",
  "data": {
    "value": 56780.50,
    "label": "Total Revenue",
    "format": "currency",
    "currency": "USD",
    "change_percent": 12.5,
    "change_direction": "up"
  },
  "generated_at": "2025-06-04T12:00:00Z"
}
```

---

### Data Sources

**`GET /api/dashboards/sources`**

Lists all configured data sources.

**Response:**

```json
{
  "sources": [
    {
      "id": "src_001",
      "name": "Sales Database",
      "type": "postgresql",
      "host": "tables.local",
      "database": "sales",
      "status": "connected",
      "last_sync": "2025-06-04T11:55:00Z"
    },
    {
      "id": "src_002",
      "name": "Analytics API",
      "type": "rest_api",
      "endpoint": "https://analytics.internal/api",
      "status": "connected",
      "last_sync": "2025-06-04T12:00:00Z"
    }
  ]
}
```

---

**`POST /api/dashboards/sources`**

Registers a new data source.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Data source name |
| `type` | string | Yes | Type: `postgresql`, `rest_api`, `csv`, `influxdb` |
| `config` | object | Yes | Connection details (varies by type) |

**Request Body:**

```json
{
  "name": "Inventory DB",
  "type": "postgresql",
  "config": {
    "host": "tables.local",
    "port": 5432,
    "database": "inventory",
    "user": "reader",
    "password": "s3cur3_p4ss",
    "ssl": true
  }
}
```

**Response:**

```json
{
  "id": "src_003",
  "name": "Inventory DB",
  "type": "postgresql",
  "status": "connected",
  "created_at": "2025-06-04T12:00:00Z"
}
```

---

**`POST /api/dashboards/sources/:id/test`**

Tests connectivity to a data source.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Data source ID |

**Response:**

```json
{
  "source_id": "src_003",
  "status": "ok",
  "latency_ms": 12,
  "message": "Connection successful"
}
```

---

**`DELETE /api/dashboards/sources/:id`**

Removes a data source. Dashboards using this source will show errors.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Data source ID |

**Response:**

```json
{
  "success": true,
  "message": "Data source removed"
}
```

---

### Ad-hoc Query

**`POST /api/dashboards/query`**

Executes an ad-hoc query against a registered data source.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source_id` | string | Yes | Data source ID |
| `query` | string | Yes | SQL or API query |
| `params` | object | No | Query parameters |
| `limit` | integer | No | Max rows (default: 100) |

**Request Body:**

```json
{
  "source_id": "src_001",
  "query": "SELECT date_trunc('day', created_at) AS day, SUM(amount) AS revenue FROM orders WHERE created_at > NOW() - INTERVAL '30 days' GROUP BY day ORDER BY day",
  "limit": 50
}
```

**Response:**

```json
{
  "source_id": "src_001",
  "columns": ["day", "revenue"],
  "rows": [
    ["2025-06-01T00:00:00Z", 12500.00],
    ["2025-06-02T00:00:00Z", 14200.50],
    ["2025-06-03T00:00:00Z", 13800.75]
  ],
  "row_count": 3,
  "execution_time_ms": 45
}
```

---

## Widget Types

| Type | Description | Config Options |
|------|-------------|----------------|
| `line_chart` | Time series line chart | `metric`, `time_range`, `series` |
| `bar_chart` | Vertical bar chart | `metric`, `aggregation`, `group_by` |
| `pie_chart` | Circular proportion chart | `metric`, `group_by`, `max_slices` |
| `stat_card` | Single value KPI card | `metric`, `aggregation`, `format` |
| `table` | Tabular data grid | `columns`, `sort`, `pagination` |
| `gauge` | Progress/ratio gauge | `metric`, `min`, `max`, `thresholds` |
| `heatmap` | Color-intensity matrix | `metric`, `x_axis`, `y_axis` |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (deletion) |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden (insufficient permissions) |
| 404 | Dashboard / widget / source not found |
| 500 | Internal Server Error |
| 502 | Data source connection failed |

---

## See Also

- [Monitoring API](./monitoring-api.md) — system health and metrics
- [Analytics API](./analytics-api.md) — deep analytics endpoints
- [Reports API](./reports-api.md) — static report generation
