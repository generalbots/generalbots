# Attendance API 🟡 BETA

> **Queue management, attendant assignment, session handling, webhooks, and LLM-powered assistance**

---

## Base URL

```
/api/attendance
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Queue & Session Endpoints

### Get Queue

**`GET /api/attendance/queue`**

Returns all sessions currently waiting in the queue.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `waiting`, `active`, `resolved`, `all` (default: `waiting`) |
| `priority` | string | No | Filter: `high`, `medium`, `low` |
| `channel` | string | No | Filter: `chat`, `voice`, `email`, `webhook` |
| `limit` | integer | No | Max results (default: 50) |

**Response:**
```json
{
  "queue": [
    {
      "session_id": "sess_001",
      "user_name": "João Silva",
      "user_email": "joao@example.com",
      "channel": "chat",
      "priority": "high",
      "status": "waiting",
      "wait_time_seconds": 120,
      "queue_position": 1,
      "first_message": "Preciso de ajuda com meu pedido",
      "tags": ["billing", "urgent"]
    }
  ],
  "total_waiting": 8,
  "average_wait_seconds": 95
}
```

---

### Get Attendants

**`GET /api/attendance/attendants`**

Returns all available attendants and their current status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `available`, `busy`, `offline`, `all` |
| `skill` | string | No | Filter by skill tag |

**Response:**
```json
{
  "attendants": [
    {
      "id": "agent_001",
      "name": "Maria Santos",
      "status": "available",
      "current_sessions": 2,
      "max_sessions": 5,
      "skills": ["billing", "technical"],
      "average_resolution_time_seconds": 300,
      "satisfaction_score": 4.7
    }
  ],
  "total_available": 5,
  "total_busy": 3
}
```

---

### Assign Session

**`POST /api/attendance/assign`**

Manually assigns a waiting session to an available attendant.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session to assign |
| `attendant_id` | string | Yes | Attendant to receive the session |

**Response:**
```json
{
  "session_id": "sess_001",
  "attendant_id": "agent_001",
  "attendant_name": "Maria Santos",
  "assigned_at": "2026-06-04T10:05:00Z",
  "status": "active"
}
```

---

### Assign by Skill

**`POST /api/attendance/assign-by-skill`**

Assigns a session to the best available attendant based on required skills.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session to assign |
| `skills` | string[] | Yes | Required skills for the session |
| `priority` | string | No | `high`, `medium`, `low` (default: `medium`) |

**Response:**
```json
{
  "session_id": "sess_001",
  "attendant_id": "agent_001",
  "attendant_name": "Maria Santos",
  "matched_skills": ["billing"],
  "assigned_at": "2026-06-04T10:05:00Z"
}
```

---

### Transfer Session

**`POST /api/attendance/transfer`**

Transfers a session from one attendant to another.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session to transfer |
| `to_attendant_id` | string | Yes | Target attendant |
| `reason` | string | No | Transfer reason |

**Response:**
```json
{
  "session_id": "sess_001",
  "from_attendant_id": "agent_001",
  "to_attendant_id": "agent_002",
  "transferred_at": "2026-06-04T10:15:00Z",
  "transfer_history": [
    {
      "from": "agent_001",
      "to": "agent_002",
      "reason": "Specialized in billing",
      "at": "2026-06-04T10:15:00Z"
    }
  ]
}
```

---

### Resolve Session

**`POST /api/attendance/resolve`**

Marks a session as resolved with optional resolution notes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session to resolve |
| `resolution` | string | No | Resolution notes |
| `satisfaction_rating` | integer | No | Rating 1-5 |
| `tags` | string[] | No | Resolution tags |

**Response:**
```json
{
  "session_id": "sess_001",
  "status": "resolved",
  "resolved_at": "2026-06-04T10:30:00Z",
  "resolution": "Order refunded successfully",
  "satisfaction_rating": 5,
  "resolution_time_seconds": 900
}
```

---

### Session Insights

**`GET /api/attendance/insights/:session_id`**

Returns AI-generated insights and sentiment analysis for a session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | path | Yes | Session identifier |

**Response:**
```json
{
  "session_id": "sess_001",
  "sentiment": {
    "overall": "neutral",
    "score": 0.6,
    "trend": "improving",
    "detected_emotions": ["frustrated", "hopeful"]
  },
  "key_topics": ["billing", "refund", "order #1234"],
  "urgency_score": 0.7,
  "suggested_actions": [
    "Offer partial refund",
    "Explain billing policy",
    "Escalate to supervisor if not resolved"
  ],
  "conversation_quality": {
    "response_time_avg_seconds": 45,
    "resolution_attempted": true,
    "customer_engagement": "medium"
  }
}
```

---

### Kanban View

**`GET /api/attendance/kanban`**

Returns sessions organized in Kanban columns for visual queue management.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filter` | string | No | Comma-separated status filter |

**Response:**
```json
{
  "columns": {
    "waiting": [
      {
        "session_id": "sess_002",
        "user_name": "Ana Costa",
        "priority": "medium",
        "wait_time_seconds": 60,
        "channel": "chat"
      }
    ],
    "active": [
      {
        "session_id": "sess_001",
        "user_name": "João Silva",
        "attendant_name": "Maria Santos",
        "duration_seconds": 300,
        "channel": "chat"
      }
    ],
    "resolved": [
      {
        "session_id": "sess_000",
        "user_name": "Carlos Lima",
        "resolved_at": "2026-06-04T09:50:00Z",
        "satisfaction_rating": 5,
        "channel": "voice"
      }
    ]
  },
  "summary": {
    "waiting": 3,
    "active": 5,
    "resolved": 12
  }
}
```

---

### Respond to Session

**`POST /api/attendance/respond`**

Sends a message to a session from the attendance system.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Target session |
| `message` | string | Yes | Message content |
| `attendant_id` | string | No | Attendant sending (auto-detected if omitted) |

**Response:**
```json
{
  "session_id": "sess_001",
  "message_id": "msg_005",
  "sent_at": "2026-06-04T10:10:00Z",
  "delivered": true
}
```

---

### WebSocket Connection

**`GET /api/attendance/ws`**

Establishes a WebSocket connection for real-time queue updates.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `token` | query | Yes | Authentication token |

**WebSocket Messages (Server → Client):**
```json
{
  "event": "session_queued",
  "data": {
    "session_id": "sess_002",
    "user_name": "Ana Costa",
    "priority": "medium",
    "queue_position": 2
  }
}
```

```json
{
  "event": "session_assigned",
  "data": {
    "session_id": "sess_001",
    "attendant_id": "agent_001",
    "attendant_name": "Maria Santos"
  }
}
```

```json
{
  "event": "queue_stats",
  "data": {
    "waiting": 3,
    "active": 5,
    "available_agents": 2
  }
}
```

---

## Webhooks

### List Webhooks

**`GET /api/attendance/webhooks`**

Returns all configured webhooks for the attendance system.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `active` | boolean | No | Filter by active status |

**Response:**
```json
{
  "webhooks": [
    {
      "id": "wh_001",
      "name": "Slack Notification",
      "url": "https://hooks.slack.com/services/T00/B00/xxx",
      "events": ["session_queued", "session_resolved"],
      "active": true,
      "created_at": "2026-06-01T10:00:00Z"
    }
  ]
}
```

---

### Create Webhook

**`POST /api/attendance/webhooks`**

Creates a new webhook subscription.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Webhook name |
| `url` | string | Yes | Target URL |
| `events` | string[] | Yes | Events to subscribe: `session_queued`, `session_assigned`, `session_resolved`, `session_transferred` |
| `headers` | object | No | Custom headers |
| `active` | boolean | No | Enable immediately (default: true) |

**Response:**
```json
{
  "id": "wh_002",
  "name": "CRM Sync",
  "url": "https://crm.example.com/webhook",
  "events": ["session_resolved"],
  "headers": { "X-API-Key": "secret_key" },
  "active": true,
  "secret": "webhook_secret_abc"
}
```

---

### Get Webhook

**`GET /api/attendance/webhooks/:id`**

Returns details of a specific webhook.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Webhook identifier |

**Response:**
```json
{
  "id": "wh_001",
  "name": "Slack Notification",
  "url": "https://hooks.slack.com/services/T00/B00/xxx",
  "events": ["session_queued", "session_resolved"],
  "active": true,
  "last_triggered_at": "2026-06-04T10:30:00Z",
  "total_deliveries": 342,
  "failed_deliveries": 2
}
```

---

### Update Webhook

**`PUT /api/attendance/webhooks/:id`**

Updates an existing webhook configuration.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Webhook identifier |
| `name` | string | No | Updated name |
| `url` | string | No | Updated URL |
| `events` | string[] | No | Updated events |
| `active` | boolean | No | Toggle active status |

**Response:**
```json
{
  "id": "wh_001",
  "name": "Slack Notification",
  "updated_at": "2026-06-04T10:45:00Z"
}
```

---

### Delete Webhook

**`DELETE /api/attendance/webhooks/:id`**

Deletes a webhook subscription.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Webhook identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "wh_001"
}
```

---

### Test Webhook

**`POST /api/attendance/webhooks/:id/test`**

Sends a test payload to the webhook URL.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Webhook identifier |

**Response:**
```json
{
  "webhook_id": "wh_001",
  "test_sent": true,
  "response_status": 200,
  "response_body": "ok",
  "delivered_at": "2026-06-04T10:50:00Z"
}
```

---

## LLM Assist API

### Get Tips

**`POST /api/attendance/llm-assist/tips`**

Returns AI-generated response tips for the current conversation context.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session identifier |
| `context` | string | No | Additional context about the conversation |

**Response:**
```json
{
  "session_id": "sess_001",
  "tips": [
    {
      "type": "suggestion",
      "text": "Mentione que o reembolso será processado em até 3 dias úteis",
      "confidence": 0.85,
      "category": "billing"
    },
    {
      "type": "warning",
      "text": "Cliente demonstra frustração. Considere oferecer desconto de 10%",
      "confidence": 0.72,
      "category": "escalation"
    }
  ],
  "generated_at": "2026-06-04T10:05:00Z"
}
```

---

### Polish Response

**`POST /api/attendance/llm-assist/polish`**

Refines a draft response for better clarity and professionalism.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Draft response to polish |
| `tone` | string | No | `formal`, `friendly`, `empathetic` (default: `formal`) |
| `language` | string | No | Target language (default: `pt-BR`) |

**Response:**
```json
{
  "original": "Seu pedido foi cancelado. O dinheiro volta em 3 dias.",
  "polished": "Prezado(a) cliente, informamos que seu pedido foi cancelado com sucesso. O valor será estornado em sua conta em até 3 dias úteis.",
  "tone": "formal",
  "changes": [
    {
      "type": "greeting",
      "before": "(none)",
      "after": "Prezado(a) cliente"
    },
    {
      "type": "formality",
      "before": "O dinheiro volta",
      "after": "O valor será estornado"
    }
  ]
}
```

---

### Suggest Replies

**`POST /api/attendance/llm-assist/replies`**

Generates multiple suggested replies for the attendant to choose from.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session identifier |
| `count` | integer | No | Number of suggestions (default: 3) |
| `context` | string | No | Additional context |

**Response:**
```json
{
  "session_id": "sess_001",
  "replies": [
    {
      "text": "Entendo sua frustração, João. Vou verificar o status do seu pedido agora mesmo.",
      "tone": "empathetic",
      "confidence": 0.91
    },
    {
      "text": "Olá! Peço desculpas pelo inconveniente. Permita-me verificar os detalhes do seu pedido.",
      "tone": "formal",
      "confidence": 0.87
    },
    {
      "text": "Oi João! Vou dar uma olhada no seu pedido. Um momento, por favor.",
      "tone": "friendly",
      "confidence": 0.82
    }
  ],
  "generated_at": "2026-06-04T10:05:00Z"
}
```

---

### Session Summary

**`GET /api/attendance/llm-assist/summary/:session_id`**

Returns an AI-generated summary of the entire session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | path | Yes | Session identifier |

**Response:**
```json
{
  "session_id": "sess_001",
  "summary": "Cliente João Silva entrou em contato sobre cancelamento do pedido #12345. Motivo: produto com defeito. Atendente Maria Santos processou reembolso de R$ 149,90. Cliente satisfeito com resolução.",
  "key_points": [
    "Pedido #12345 cancelado por defeito",
    "Reembolso de R$ 149,90 processado",
    "Cliente satisfeito com resolução"
  ],
  "resolution_type": "refund",
  "duration_seconds": 600,
  "satisfaction_rating": 5,
  "follow_up_required": false
}
```

---

### Sentiment Analysis

**`POST /api/attendance/llm-assist/sentiment`**

Returns real-time sentiment analysis for a conversation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id` | string | Yes | Session identifier |
| `messages` | string[] | No | Specific messages to analyze |

**Response:**
```json
{
  "session_id": "sess_001",
  "overall_sentiment": "neutral",
  "score": 0.6,
  "trend": "improving",
  "timeline": [
    {
      "message_index": 0,
      "sentiment": "negative",
      "score": 0.2
    },
    {
      "message_index": 1,
      "sentiment": "neutral",
      "score": 0.5
    },
    {
      "message_index": 2,
      "sentiment": "positive",
      "score": 0.8
    }
  ],
  "emotions_detected": ["frustration", "relief", "satisfaction"],
  "escalation_risk": "low"
}
```

---

## See Also

- [Attendant API](attendant-api.md) — attendant console and agent management
- [Conversations API](conversations-api.md) — chat and message management
- [Meet API](meet-api.md) — video and voice channel integration
- [Analytics API](analytics-api.md) — reporting and dashboards
