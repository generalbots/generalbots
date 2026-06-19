# Slides API 🟡 BETA

> **API for creating, editing, and managing presentation slides with AI-powered content generation.**

---

## Base URL

```
/api/slides
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Presentations

**`GET /api/slides/list`**

Returns a list of all presentations accessible to the authenticated user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number for pagination (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `sort` | string | No | Sort field: `created_at`, `updated_at`, `name` |
| `order` | string | No | Sort order: `asc`, `desc` (default: `desc`) |

**Response:**
```json
{
  "presentations": [
    {
      "id": "uuid-string",
      "name": "Q4 Sales Report",
      "slide_count": 12,
      "created_at": "2026-01-15T10:30:00Z",
      "updated_at": "2026-01-20T14:45:00Z",
      "thumbnail_url": "/api/slides/uuid/thumbnail",
      "theme": "corporate-blue"
    }
  ],
  "total": 45,
  "page": 1,
  "limit": 20
}
```

---

### Search Presentations

**`GET /api/slides/search`**

Search presentations by name or content keywords.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query string |
| `limit` | integer | No | Max results to return (default: 10) |

**Response:**
```json
{
  "results": [
    {
      "id": "uuid-string",
      "name": "Q4 Sales Report",
      "match_score": 0.95,
      "snippet": "...quarterly revenue exceeded targets by 15%..."
    }
  ],
  "total": 5
}
```

---

### Load Presentation

**`GET /api/slides/load`**

Loads the full presentation data including all slides and their content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Presentation ID |

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Q4 Sales Report",
  "theme": "corporate-blue",
  "slides": [
    {
      "id": "slide-uuid",
      "index": 0,
      "type": "title",
      "content": {
        "title": "Q4 Sales Report",
        "subtitle": "October - December 2026",
        "background": "#1a365d"
      },
      "notes": "Opening slide"
    },
    {
      "id": "slide-uuid-2",
      "index": 1,
      "type": "content",
      "content": {
        "title": "Revenue Summary",
        "body": "Total revenue: $2.4M",
        "layout": "two-column"
      }
    }
  ],
  "metadata": {
    "author": "user-uuid",
    "version": 3
  }
}
```

---

### Save Presentation

**`POST /api/slides/save`**

Creates a new presentation or updates an existing one.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | No | Presentation ID (omit for new) |
| `name` | string | Yes | Presentation name |
| `slides` | array | Yes | Array of slide objects |
| `theme` | string | No | Theme identifier |

**Request Body:**
```json
{
  "name": "Q4 Sales Report",
  "theme": "corporate-blue",
  "slides": [
    {
      "type": "title",
      "content": {
        "title": "Q4 Sales Report",
        "subtitle": "October - December 2026"
      }
    }
  ]
}
```

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Q4 Sales Report",
  "saved_at": "2026-01-20T15:00:00Z"
}
```

---

### Delete Presentation

**`POST /api/slides/delete`**

Deletes a presentation by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Presentation ID to delete |

**Response:**
```json
{
  "deleted": true,
  "id": "uuid-string"
}
```

---

### New Presentation

**`GET /api/slides/new`**

Creates a new blank presentation with default settings.

**Response:**
```json
{
  "id": "new-uuid-string",
  "name": "Untitled Presentation",
  "theme": "default",
  "slides": [
    {
      "id": "slide-uuid",
      "index": 0,
      "type": "title",
      "content": {
        "title": "",
        "subtitle": ""
      }
    }
  ],
  "created_at": "2026-01-20T15:00:00Z"
}
```

---

### AI Generate Content

**`POST /api/slides/ai`**

Uses AI to generate slide content based on a prompt or topic.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompt` | string | Yes | Description of desired content |
| `presentation_id` | string | No | Existing presentation to add to |
| `slide_count` | integer | No | Number of slides to generate (default: 5) |
| `style` | string | No | Content style: `formal`, `casual`, `technical`, `creative` |

**Request Body:**
```json
{
  "prompt": "Create a presentation about quarterly sales performance with charts and key metrics",
  "slide_count": 8,
  "style": "formal"
}
```

**Response:**
```json
{
  "presentation_id": "uuid-string",
  "slides": [
    {
      "type": "title",
      "content": {
        "title": "Q4 Sales Performance",
        "subtitle": "Data-Driven Insights"
      }
    },
    {
      "type": "content",
      "content": {
        "title": "Key Metrics",
        "body": "Revenue growth: 23%\nNew customers: 1,247\nRetention rate: 94%"
      }
    }
  ],
  "generation_time_ms": 3450
}
```

---

### Get Slide by ID

**`GET /api/slides/:id`**

Returns a single slide from a presentation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Slide ID |

**Response:**
```json
{
  "id": "slide-uuid",
  "index": 2,
  "type": "chart",
  "content": {
    "title": "Revenue by Region",
    "chart_type": "bar",
    "data": {
      "labels": ["North", "South", "East", "West"],
      "values": [450000, 380000, 520000, 410000]
    }
  },
  "notes": "Discuss regional performance"
}
```

---

### Add Slide

**`POST /api/slides/slide/add`**

Adds a new slide to an existing presentation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Parent presentation ID |
| `slide` | object | Yes | Slide definition |
| `position` | integer | No | Insert position (default: end) |

**Request Body:**
```json
{
  "presentation_id": "uuid-string",
  "slide": {
    "type": "content",
    "content": {
      "title": "Market Analysis",
      "body": "Key findings from Q4 market research...",
      "layout": "full-width"
    }
  },
  "position": 3
}
```

**Response:**
```json
{
  "slide_id": "new-slide-uuid",
  "index": 3,
  "total_slides": 13
}
```

---

### Set Theme

**`POST /api/slides/theme`**

Applies a theme to the entire presentation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Presentation ID |
| `theme` | string | Yes | Theme identifier |

**Request Body:**
```json
{
  "presentation_id": "uuid-string",
  "theme": "corporate-blue"
}
```

**Response:**
```json
{
  "updated": true,
  "theme": "corporate-blue",
  "slides_affected": 12
}
```

---

### Update Cursor Position

**`POST /api/slides/cursor`**

Updates the collaborative editing cursor position for real-time collaboration.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Presentation ID |
| `slide_index` | integer | Yes | Current slide index |
| `user_name` | string | Yes | Display name of user |

**Request Body:**
```json
{
  "presentation_id": "uuid-string",
  "slide_index": 5,
  "user_name": "Ana"
}
```

**Response:**
```json
{
  "updated": true
}
```

---

### Get Cursors

**`GET /api/slides/cursors`**

Returns all active collaborative cursors for a presentation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Presentation ID |

**Response:**
```json
{
  "cursors": [
    {
      "user_name": "Ana",
      "slide_index": 5,
      "updated_at": "2026-01-20T15:05:00Z"
    },
    {
      "user_name": "Carlos",
      "slide_index": 2,
      "updated_at": "2026-01-20T15:04:50Z"
    }
  ]
}
```

---

### Upload Media

**`POST /api/slides/media`**

Uploads an image or media file to a slide.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Presentation ID |
| `slide_id` | string | Yes | Target slide ID |
| `file` | binary | Yes | Media file (image, video) |
| `position` | string | No | Placement: `background`, `inline`, `float` |

**Response:**
```json
{
  "media_id": "media-uuid",
  "url": "/api/slides/media/media-uuid",
  "type": "image/png",
  "size_bytes": 245760
}
```

---

### List Media

**`GET /api/slides/media/list`**

Returns all media assets for a presentation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `presentation_id` | string | Yes | Presentation ID |

**Response:**
```json
{
  "media": [
    {
      "id": "media-uuid",
      "filename": "chart-q4.png",
      "type": "image/png",
      "size_bytes": 245760,
      "url": "/api/slides/media/media-uuid",
      "used_in_slides": [2, 5, 8]
    }
  ],
  "total_size_bytes": 1048576
}
```

---

## Slide Types

| Type | Description |
|------|-------------|
| `title` | Title slide with heading and subtitle |
| `content` | Standard content slide with text |
| `chart` | Data visualization slide |
| `image` | Full-image or image-focused slide |
| `two-column` | Side-by-side content layout |
| `comparison` | Before/after or comparison layout |
| `quote` | Quote or testimonial slide |
| `blank` | Empty slide for custom content |

---

## See Also

- [Docs API](docs-api.md) - Word processor documents
- [Paper API](paper-api.md) - Notes and simple documents
- [Research API](research-api.md) - Research collections and web search
