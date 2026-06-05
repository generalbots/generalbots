# Tickets API

> **Full-featured ticketing system with SLA tracking, canned responses, categories, and comments.**

---

## Base URL

```
/api/tickets
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Ticket Management

### List Tickets

**`GET /api/tickets/`**

Returns all tickets with optional filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `open`, `in_progress`, `resolved`, `closed` |
| `priority` | string | No | Filter: `low`, `medium`, `high`, `critical` |
| `assignee` | string | No | Filter by assignee user ID |
| `categoryId` | string | No | Filter by category |
| `search` | string | No | Search in title and description |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 50) |

**Request:**

```
GET /api/tickets/?status=open&priority=high&limit=10
```

**Response:**

```json
{
  "tickets": [
    {
      "id": "tkt-001",
      "title": "Login page returning 500 errors",
      "description": "Multiple users reporting 500 errors when attempting to log in via SSO.",
      "status": "open",
      "priority": "high",
      "categoryId": "cat-bug",
      "assigneeId": "user-support-01",
      "reporterId": "user-123",
      "createdAt": "2026-06-04T08:30:00Z",
      "updatedAt": "2026-06-04T09:15:00Z",
      "dueAt": "2026-06-05T17:00:00Z",
      "commentCount": 3
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 10,
    "total": 24,
    "totalPages": 3
  }
}
```

---

### Create Ticket

**`POST /api/tickets/`**

Creates a new support ticket.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Ticket title |
| `description` | string | Yes | Detailed description |
| `priority` | string | No | `low`, `medium`, `high`, `critical` (default: `medium`) |
| `categoryId` | string | No | Category identifier |
| `assigneeId` | string | No | Assignee user ID |
| `tags` | string[] | No | Tag labels |
| `dueAt` | string | No | Due date (ISO 8601) |

**Request:**

```json
{
  "title": "Dashboard charts not rendering",
  "description": "After the latest deploy, the analytics dashboard shows blank charts for all users.",
  "priority": "high",
  "categoryId": "cat-bug",
  "assigneeId": "user-dev-02",
  "tags": ["frontend", "analytics"],
  "dueAt": "2026-06-06T17:00:00Z"
}
```

**Response:**

```json
{
  "id": "tkt-002",
  "title": "Dashboard charts not rendering",
  "description": "After the latest deploy, the analytics dashboard shows blank charts for all users.",
  "status": "open",
  "priority": "high",
  "categoryId": "cat-bug",
  "assigneeId": "user-dev-02",
  "reporterId": "user-123",
  "tags": ["frontend", "analytics"],
  "createdAt": "2026-06-04T12:00:00Z",
  "dueAt": "2026-06-06T17:00:00Z"
}
```

---

### Get Ticket

**`GET /api/tickets/:id`**

Returns ticket details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |

**Request:**

```
GET /api/tickets/tkt-001
```

**Response:**

```json
{
  "id": "tkt-001",
  "title": "Login page returning 500 errors",
  "description": "Multiple users reporting 500 errors when attempting to log in via SSO.",
  "status": "open",
  "priority": "high",
  "categoryId": "cat-bug",
  "assigneeId": "user-support-01",
  "reporterId": "user-123",
  "tags": ["auth", "production"],
  "createdAt": "2026-06-04T08:30:00Z",
  "updatedAt": "2026-06-04T09:15:00Z",
  "dueAt": "2026-06-05T17:00:00Z",
  "resolvedAt": null,
  "closedAt": null,
  "commentCount": 3
}
```

---

### Update Ticket

**`PUT /api/tickets/:id`**

Updates ticket fields.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `title` | string | No | Ticket title |
| `description` | string | No | Description |
| `priority` | string | No | Priority level |
| `categoryId` | string | No | Category |
| `tags` | string[] | No | Tags |
| `dueAt` | string | No | Due date |

**Request:**

```json
{
  "priority": "critical",
  "tags": ["auth", "production", "sso"]
}
```

**Response:**

```json
{
  "id": "tkt-001",
  "priority": "critical",
  "tags": ["auth", "production", "sso"],
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Delete Ticket

**`DELETE /api/tickets/:id`**

Soft-deletes a ticket (sets status to `closed`).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |

**Request:**

```
DELETE /api/tickets/tkt-002
```

**Response:**

```json
{
  "deleted": true,
  "id": "tkt-002"
}
```

---

### Get Full Ticket

**`GET /api/tickets/:id/full`**

Returns ticket with all associated data: comments, history, and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |

**Request:**

```
GET /api/tickets/tkt-001/full
```

**Response:**

```json
{
  "id": "tkt-001",
  "title": "Login page returning 500 errors",
  "description": "Multiple users reporting 500 errors...",
  "status": "open",
  "priority": "high",
  "category": {
    "id": "cat-bug",
    "name": "Bug Report"
  },
  "assignee": {
    "id": "user-support-01",
    "name": "Maria Santos"
  },
  "reporter": {
    "id": "user-123",
    "name": "João Silva"
  },
  "tags": ["auth", "production"],
  "comments": [
    {
      "id": "cmt-001",
      "authorId": "user-support-01",
      "content": "Investigating the SSO endpoint logs.",
      "createdAt": "2026-06-04T09:00:00Z"
    }
  ],
  "history": [
    {
      "action": "created",
      "userId": "user-123",
      "at": "2026-06-04T08:30:00Z"
    },
    {
      "action": "assigned",
      "userId": "user-admin-01",
      "to": "user-support-01",
      "at": "2026-06-04T08:45:00Z"
    }
  ],
  "createdAt": "2026-06-04T08:30:00Z",
  "updatedAt": "2026-06-04T09:15:00Z"
}
```

---

## Status Transitions

### Assign Ticket

**`PUT /api/tickets/:id/assign`**

Assigns a ticket to a user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `assigneeId` | string | Yes | User ID to assign |

**Request:**

```json
{
  "assigneeId": "user-dev-03"
}
```

**Response:**

```json
{
  "id": "tkt-001",
  "assigneeId": "user-dev-03",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Update Status

**`PUT /api/tickets/:id/status`**

Changes the ticket status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `status` | string | Yes | New status: `open`, `in_progress`, `resolved`, `closed` |

**Request:**

```json
{
  "status": "in_progress"
}
```

**Response:**

```json
{
  "id": "tkt-001",
  "status": "in_progress",
  "updatedAt": "2026-06-04T12:00:00Z"
}
```

---

### Resolve Ticket

**`PUT /api/tickets/:id/resolve`**

Marks a ticket as resolved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `resolution` | string | No | Resolution notes |

**Request:**

```json
{
  "resolution": "Root cause: expired TLS certificate on SSO provider. Renewed and verified."
}
```

**Response:**

```json
{
  "id": "tkt-001",
  "status": "resolved",
  "resolvedAt": "2026-06-04T14:30:00Z",
  "resolution": "Root cause: expired TLS certificate on SSO provider. Renewed and verified."
}
```

---

### Close Ticket

**`PUT /api/tickets/:id/close`**

Closes a resolved ticket permanently.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |

**Request:**

```
PUT /api/tickets/tkt-001/close
```

**Response:**

```json
{
  "id": "tkt-001",
  "status": "closed",
  "closedAt": "2026-06-04T15:00:00Z"
}
```

---

### Reopen Ticket

**`PUT /api/tickets/:id/reopen`**

Reopens a closed or resolved ticket.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `reason` | string | No | Reason for reopening |

**Request:**

```json
{
  "reason": "Issue recurred for 3 additional users after initial fix."
}
```

**Response:**

```json
{
  "id": "tkt-001",
  "status": "open",
  "reopenCount": 1,
  "updatedAt": "2026-06-04T16:00:00Z"
}
```

---

## Comments

### List Comments

**`GET /api/tickets/:id/comments`**

Returns all comments on a ticket.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |

**Request:**

```
GET /api/tickets/tkt-001/comments
```

**Response:**

```json
[
  {
    "id": "cmt-001",
    "authorId": "user-support-01",
    "authorName": "Maria Santos",
    "content": "Investigating the SSO endpoint logs. Found TLS certificate warning.",
    "isInternal": false,
    "createdAt": "2026-06-04T09:00:00Z"
  },
  {
    "id": "cmt-002",
    "authorId": "user-support-01",
    "authorName": "Maria Santos",
    "content": "Internal note: certificate expires in 2 days, need renewal.",
    "isInternal": true,
    "createdAt": "2026-06-04T09:10:00Z"
  }
]
```

---

### Add Comment

**`POST /api/tickets/:id/comments`**

Adds a comment to a ticket.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Ticket identifier (path param) |
| `content` | string | Yes | Comment text |
| `isInternal` | boolean | No | Internal note (default: false) |

**Request:**

```json
{
  "content": "Certificate has been renewed. SSO login restored. Monitoring for recurrence.",
  "isInternal": false
}
```

**Response:**

```json
{
  "id": "cmt-003",
  "authorId": "user-support-01",
  "authorName": "Maria Santos",
  "content": "Certificate has been renewed. SSO login restored. Monitoring for recurrence.",
  "isInternal": false,
  "createdAt": "2026-06-04T14:30:00Z"
}
```

---

## Canned Responses

### List Canned Responses

**`GET /api/tickets/canned`**

Returns saved response templates.

**Response:**

```json
[
  {
    "id": "canned-001",
    "name": "Acknowledged",
    "content": "Thank you for reporting this issue. We are investigating and will update you within 24 hours.",
    "categoryId": "cat-bug"
  },
  {
    "id": "canned-002",
    "name": "Resolved - Fixed in Production",
    "content": "This issue has been resolved in the latest deployment. Please verify and let us know if the problem persists.",
    "categoryId": "cat-bug"
  }
]
```

---

### Create Canned Response

**`POST /api/tickets/canned`**

Creates a new canned response template.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Template name |
| `content` | string | Yes | Response content |
| `categoryId` | string | No | Category association |

**Request:**

```json
{
  "name": "Out of Office",
  "content": "The assigned team member is currently unavailable. Your ticket will be reviewed upon their return.",
  "categoryId": "cat-general"
}
```

**Response:**

```json
{
  "id": "canned-003",
  "name": "Out of Office",
  "content": "The assigned team member is currently unavailable. Your ticket will be reviewed upon their return.",
  "categoryId": "cat-general",
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## Categories

### List Categories

**`GET /api/tickets/categories`**

Returns all ticket categories.

**Response:**

```json
[
  {
    "id": "cat-bug",
    "name": "Bug Report",
    "color": "#e74c3c",
    "ticketCount": 45
  },
  {
    "id": "cat-feature",
    "name": "Feature Request",
    "color": "#3498db",
    "ticketCount": 22
  },
  {
    "id": "cat-general",
    "name": "General Inquiry",
    "color": "#95a5a6",
    "ticketCount": 12
  }
]
```

---

### Create Category

**`POST /api/tickets/categories`**

Creates a new ticket category.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Category name |
| `color` | string | No | Hex color code |
| `description` | string | No | Category description |

**Request:**

```json
{
  "name": "Security",
  "color": "#e67e22",
  "description": "Security-related issues and vulnerability reports"
}
```

**Response:**

```json
{
  "id": "cat-security",
  "name": "Security",
  "color": "#e67e22",
  "description": "Security-related issues and vulnerability reports",
  "ticketCount": 0,
  "createdAt": "2026-06-04T12:00:00Z"
}
```

---

## SLA & Tags

### Get SLA Configuration

**`GET /api/tickets/sla`**

Returns SLA rules and current compliance status.

**Response:**

```json
{
  "rules": [
    {
      "id": "sla-001",
      "name": "Critical Response",
      "priority": "critical",
      "responseTimeHours": 1,
      "resolutionTimeHours": 4,
      "currentlyBreached": 0
    },
    {
      "id": "sla-002",
      "name": "High Priority",
      "priority": "high",
      "responseTimeHours": 4,
      "resolutionTimeHours": 24,
      "currentlyBreached": 1
    },
    {
      "id": "sla-003",
      "name": "Medium Priority",
      "priority": "medium",
      "responseTimeHours": 8,
      "resolutionTimeHours": 72,
      "currentlyBreached": 0
    },
    {
      "id": "sla-004",
      "name": "Low Priority",
      "priority": "low",
      "responseTimeHours": 24,
      "resolutionTimeHours": 168,
      "currentlyBreached": 0
    }
  ],
  "overallCompliance": 96.5
}
```

---

### List Tags

**`GET /api/tickets/tags`**

Returns all tags used across tickets.

**Response:**

```json
[
  { "name": "auth", "ticketCount": 12 },
  { "name": "frontend", "ticketCount": 18 },
  { "name": "production", "ticketCount": 31 },
  { "name": "sso", "ticketCount": 5 },
  { "name": "analytics", "ticketCount": 7 }
]
```

---

## Statistics

### Get Ticket Stats

**`GET /api/tickets/stats`**

Returns aggregate ticket statistics.

**Response:**

```json
{
  "total": 128,
  "byStatus": {
    "open": 24,
    "in_progress": 12,
    "resolved": 45,
    "closed": 47
  },
  "byPriority": {
    "critical": 2,
    "high": 8,
    "medium": 30,
    "low": 14
  },
  "avgResolutionTimeHours": 18.4,
  "avgResponseTimeHours": 2.1,
  "slaCompliance": 96.5,
  "lastUpdated": "2026-06-04T12:00:00Z"
}
```

---

### Get Overdue Tickets

**`GET /api/tickets/overdue`**

Returns tickets that have exceeded their SLA or due date.

**Response:**

```json
[
  {
    "id": "tkt-045",
    "title": "Payment gateway timeout",
    "priority": "critical",
    "status": "in_progress",
    "assigneeId": "user-dev-01",
    "dueAt": "2026-06-03T17:00:00Z",
    "overdueByHours": 19,
    "slaBreached": true
  }
]
```

---

## See Also

- [Tasks API](../08-rest-api-tools/tasks-api.md) — Task management linked to tickets
- [Notifications API](../08-rest-api-tools/notifications-api.md) — Ticket notification delivery
- [Analytics API](../08-rest-api-tools/analytics-api.md) — Reporting and dashboards
