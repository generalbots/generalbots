# Attendant Console API 🟡 BETA

> **Agent management, queue configuration, session control, canned responses, and real-time statistics**

---

## Base URL

```
/api/attendant
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Queue Management

### List Queues

**`GET /api/attendant/queues`**

Returns all configured queues.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `include_stats` | boolean | No | Include real-time statistics (default: false) |

**Response:**
```json
{
  "queues": [
    {
      "id": "queue_001",
      "name": "Vendas",
      "description": "Atendimento de vendas",
      "priority": 1,
      "max_wait_seconds": 300,
      "routing": "round_robin",
      "active_agents": 3,
      "waiting_sessions": 2,
      "created_at": "2026-06-01T10:00:00Z"
    }
  ]
}
```

---

### Create Queue

**`POST /api/attendant/queues`**

Creates a new support queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Queue name |
| `description` | string | No | Queue description |
| `priority` | integer | No | Priority level 1-10 (default: 5) |
| `max_wait_seconds` | integer | No | Max wait time before escalation |
| `routing` | string | No | Routing strategy: `round_robin`, `least_loaded`, `most_idle`, `skill_based` |
| `overflow_queue_id` | string | No | Queue to route overflow sessions |

**Response:**
```json
{
  "id": "queue_002",
  "name": "Suporte Técnico",
  "description": "Suporte técnico e manutenção",
  "priority": 2,
  "max_wait_seconds": 600,
  "routing": "skill_based",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Queue

**`GET /api/attendant/queues/:id`**

Returns details of a specific queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Queue identifier |

**Response:**
```json
{
  "id": "queue_001",
  "name": "Vendas",
  "description": "Atendimento de vendas",
  "priority": 1,
  "max_wait_seconds": 300,
  "routing": "round_robin",
  "agents": [
    {
      "id": "agent_001",
      "name": "Maria Santos",
      "status": "available",
      "current_load": 2
    }
  ],
  "waiting_sessions": [
    {
      "session_id": "sess_001",
      "wait_time_seconds": 45,
      "user_name": "João Silva"
    }
  ],
  "stats": {
    "total_sessions_today": 47,
    "average_wait_seconds": 120,
    "average_resolution_seconds": 480
  }
}
```

---

### Delete Queue

**`DELETE /api/attendant/queues/:id`**

Deletes a queue. Sessions in the queue are rerouted to the overflow queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Queue identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "queue_002",
  "rerouted_sessions": 3
}
```

---

### Add Agent to Queue

**`POST /api/attendant/queues/:id/agents`**

Adds an agent to a queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Queue identifier |
| `agent_id` | string | Yes | Agent to add |
| `priority` | integer | No | Agent priority within queue (1-10) |

**Response:**
```json
{
  "queue_id": "queue_001",
  "agent_id": "agent_003",
  "agent_name": "Carlos Lima",
  "added_at": "2026-06-04T10:00:00Z",
  "queue_position": 4
}
```

---

### Remove Agent from Queue

**`DELETE /api/attendant/queues/:queue_id/agents/:agent_id`**

Removes an agent from a queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `queue_id` | path | Yes | Queue identifier |
| `agent_id` | path | Yes | Agent identifier |

**Response:**
```json
{
  "queue_id": "queue_001",
  "agent_id": "agent_003",
  "removed": true,
  "rerouted_sessions": 1
}
```

---

## Session Management

### List Sessions

**`GET /api/attendant/sessions`**

Returns all sessions accessible to the authenticated agent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `active`, `waiting`, `resolved`, `all` |
| `queue_id` | string | No | Filter by queue |
| `assigned_to_me` | boolean | No | Only sessions assigned to caller |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "sessions": [
    {
      "id": "sess_001",
      "user": {
        "id": "user_001",
        "name": "João Silva",
        "email": "joao@example.com"
      },
      "queue": "Vendas",
      "status": "active",
      "assigned_to": "agent_001",
      "assigned_to_name": "Maria Santos",
      "created_at": "2026-06-04T09:55:00Z",
      "last_message_at": "2026-06-04T10:10:00Z",
      "message_count": 12,
      "tags": ["billing"],
      "priority": "high"
    }
  ],
  "total": 25,
  "page": 1
}
```

---

### Get Session Details

**`GET /api/attendant/sessions/:id`**

Returns full details of a session including messages.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `include_messages` | boolean | No | Include message history (default: true) |

**Response:**
```json
{
  "id": "sess_001",
  "user": {
    "id": "user_001",
    "name": "João Silva",
    "email": "joao@example.com",
    "phone": "+5511999999999"
  },
  "queue": {
    "id": "queue_001",
    "name": "Vendas"
  },
  "status": "active",
  "assigned_to": {
    "id": "agent_001",
    "name": "Maria Santos"
  },
  "priority": "high",
  "tags": ["billing", "urgent"],
  "metadata": {
    "order_id": "ORD-12345"
  },
  "created_at": "2026-06-04T09:55:00Z",
  "messages": [
    {
      "id": "msg_001",
      "sender": "user",
      "content": "Preciso de ajuda com meu pedido",
      "timestamp": "2026-06-04T09:55:00Z"
    },
    {
      "id": "msg_002",
      "sender": "agent",
      "content": "Olá João! Vou verificar seu pedido agora.",
      "timestamp": "2026-06-04T09:56:00Z"
    }
  ],
  "transfer_history": [],
  "wrap_up_code": null
}
```

---

### Assign Session

**`PUT /api/attendant/sessions/:id/assign`**

Assigns a session to an agent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `agent_id` | string | Yes | Agent to assign |

**Response:**
```json
{
  "session_id": "sess_001",
  "assigned_to": "agent_002",
  "assigned_to_name": "Carlos Lima",
  "assigned_at": "2026-06-04T10:15:00Z",
  "previous_assignment": "agent_001"
}
```

---

### Transfer Session

**`PUT /api/attendant/sessions/:id/transfer`**

Transfers a session to another agent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `to_agent_id` | string | Yes | Target agent |
| `reason` | string | No | Transfer reason |
| `internal_note` | string | No | Note for the receiving agent |

**Response:**
```json
{
  "session_id": "sess_001",
  "from_agent": "agent_001",
  "to_agent": "agent_002",
  "transferred_at": "2026-06-04T10:20:00Z",
  "transfer_history": [
    {
      "from": "agent_001",
      "to": "agent_002",
      "reason": "Assunto técnico",
      "at": "2026-06-04T10:20:00Z"
    }
  ]
}
```

---

### End Session

**`PUT /api/attendant/sessions/:id/end`**

Marks a session as resolved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `resolution` | string | No | Resolution notes |
| `wrap_up_code` | string | No | Wrap-up code identifier |
| `satisfaction_rating` | integer | No | Customer rating 1-5 |

**Response:**
```json
{
  "session_id": "sess_001",
  "status": "resolved",
  "resolved_at": "2026-06-04T10:30:00Z",
  "duration_seconds": 2100,
  "resolution": "Pedido cancelado e reembolso processado",
  "satisfaction_rating": 5
}
```

---

### Rate Session

**`PUT /api/attendant/sessions/:id/rate`**

Records a customer satisfaction rating for a session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `rating` | integer | Yes | Rating 1-5 |
| `comment` | string | No | Optional feedback comment |

**Response:**
```json
{
  "session_id": "sess_001",
  "rating": 5,
  "comment": "Excelente atendimento!",
  "recorded_at": "2026-06-04T10:35:00Z"
}
```

---

### Send Message to Session

**`POST /api/attendant/sessions/:id/messages`**

Sends a message to a session from the agent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Session identifier |
| `content` | string | Yes | Message content |
| `type` | string | No | Message type: `text`, `file`, `image` (default: `text`) |
| `canned_id` | string | No | ID of a canned response used |

**Response:**
```json
{
  "message_id": "msg_010",
  "session_id": "sess_001",
  "sent_at": "2026-06-04T10:10:00Z",
  "delivered": true
}
```

---

## Agent Management

### List Agents

**`GET /api/attendant/agents`**

Returns all agents in the system.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `available`, `busy`, `offline`, `all` |
| `queue_id` | string | No | Filter by queue membership |

**Response:**
```json
{
  "agents": [
    {
      "id": "agent_001",
      "name": "Maria Santos",
      "email": "maria@example.com",
      "status": "available",
      "queues": ["Vendas", "Suporte"],
      "current_sessions": 2,
      "max_sessions": 5,
      "skills": ["billing", "technical", "vip"],
      "last_active_at": "2026-06-04T10:10:00Z",
      "stats": {
        "sessions_today": 12,
        "average_resolution_seconds": 300,
        "satisfaction_score": 4.8
      }
    }
  ]
}
```

---

### Update Agent Status

**`PUT /api/attendant/agents/:id/status`**

Updates an agent's availability status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Agent identifier |
| `status` | string | Yes | New status: `available`, `busy`, `offline`, `break` |
| `reason` | string | No | Status change reason (shown to supervisors) |

**Response:**
```json
{
  "agent_id": "agent_001",
  "status": "break",
  "reason": "Intervalo",
  "changed_at": "2026-06-04T10:30:00Z",
  "estimated_return": "2026-06-04T10:45:00Z"
}
```

---

## Canned Responses

### List Canned Responses

**`GET /api/attendant/canned`**

Returns all available canned responses.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `category` | string | No | Filter by category |
| `search` | string | No | Search in title and content |

**Response:**
```json
{
  "canned_responses": [
    {
      "id": "canned_001",
      "title": "Saudação inicial",
      "category": "greetings",
      "content": "Olá! Bem-vindo ao suporte. Como posso ajudá-lo hoje?",
      "shortcut": "/oi",
      "language": "pt-BR",
      "usage_count": 234
    },
    {
      "id": "canned_002",
      "title": "Processamento de reembolso",
      "category": "billing",
      "content": "Seu reembolso foi processado e será creditado em sua conta em até 3 dias úteis.",
      "shortcut": "/reembolso",
      "language": "pt-BR",
      "usage_count": 156
    }
  ]
}
```

---

### Create Canned Response

**`POST /api/attendant/canned`**

Creates a new canned response template.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `title` | string | Yes | Template title |
| `content` | string | Yes | Template content |
| `category` | string | No | Category name |
| `shortcut` | string | No | Keyboard shortcut (e.g., `/greet`) |
| `language` | string | No | Language code (default: `pt-BR`) |

**Response:**
```json
{
  "id": "canned_003",
  "title": "Aguarde",
  "content": "Por favor, aguarde um momento enquanto verifico a informação.",
  "category": "common",
  "shortcut": "/aguarde",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

## Reference Data

### List Tags

**`GET /api/attendant/tags`**

Returns all available session tags.

**Response:**
```json
{
  "tags": [
    { "id": "tag_001", "name": "urgent", "color": "#EF4444", "usage_count": 89 },
    { "id": "tag_002", "name": "billing", "color": "#F59E0B", "usage_count": 234 },
    { "id": "tag_003", "name": "technical", "color": "#3B82F6", "usage_count": 156 },
    { "id": "tag_004", "name": "vip", "color": "#8B5CF6", "usage_count": 45 }
  ]
}
```

---

### List Wrap-Up Codes

**`GET /api/attendant/wrap-up-codes`**

Returns all available wrap-up codes for session resolution.

**Response:**
```json
{
  "wrap_up_codes": [
    { "id": "wuc_001", "code": "REFUND", "description": "Reembolso processado", "category": "billing" },
    { "id": "wuc_002", "code": "RESOLVED", "description": "Problema resolvido", "category": "general" },
    { "id": "wuc_003", "code": "ESCALATED", "description": "Escalar para supervisor", "category": "escalation" },
    { "id": "wuc_004", "code": "NO_RESPONSE", "description": "Cliente não respondeu", "category": "general" }
  ]
}
```

---

## Statistics

### Get Statistics

**`GET /api/attendant/stats`**

Returns aggregated attendant console statistics.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | `today`, `week`, `month` (default: `today`) |
| `agent_id` | string | No | Filter by specific agent |
| `queue_id` | string | No | Filter by queue |

**Response:**
```json
{
  "period": "today",
  "totals": {
    "sessions_handled": 142,
    "sessions_waiting": 3,
    "sessions_active": 12,
    "sessions_resolved": 127
  },
  "performance": {
    "average_wait_seconds": 95,
    "average_resolution_seconds": 480,
    "first_response_time_seconds": 30,
    "satisfaction_score": 4.6,
    "resolution_rate": 0.89
  },
  "by_queue": [
    {
      "queue_id": "queue_001",
      "queue_name": "Vendas",
      "sessions": 67,
      "avg_wait": 80
    },
    {
      "queue_id": "queue_002",
      "queue_name": "Suporte",
      "sessions": 75,
      "avg_wait": 110
    }
  ],
  "by_agent": [
    {
      "agent_id": "agent_001",
      "agent_name": "Maria Santos",
      "sessions": 28,
      "avg_resolution": 350,
      "satisfaction": 4.8
    }
  ]
}
```

---

## See Also

- [Attendance API](attendance-api.md) — queue operations and LLM assist
- [Conversations API](conversations-api.md) — chat session details
- [Analytics API](analytics-api.md) — reporting and dashboards
- [Users API](users-api.md) — user and permission management
