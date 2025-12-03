# Attendance Queue Module

Human-attendant queue management for hybrid bot/human support workflows.

## Overview

The attendance queue module manages handoffs from bot to human agents, tracking conversation queues, attendant availability, and real-time assignment.

## Configuration

Create `attendant.csv` in your bot's `.gbai` folder:

```csv
id,name,channel,preferences
att-001,John Smith,whatsapp,sales
att-002,Jane Doe,web,support
att-003,Bob Wilson,all,technical
```

## Queue Status

| Status | Description |
|--------|-------------|
| `waiting` | User waiting for attendant |
| `assigned` | Attendant assigned, not yet active |
| `active` | Conversation in progress |
| `resolved` | Conversation completed |
| `abandoned` | User left before assignment |

## Attendant Status

| Status | Description |
|--------|-------------|
| `online` | Available for new conversations |
| `busy` | Currently handling conversations |
| `away` | Temporarily unavailable |
| `offline` | Not working |

## REST API Endpoints

### GET /api/queue
List conversations in queue.

### POST /api/queue/assign
Assign conversation to attendant.

```json
{
    "session_id": "uuid",
    "attendant_id": "uuid"
}
```

### POST /api/queue/transfer
Transfer conversation between attendants.

```json
{
    "session_id": "uuid",
    "from_attendant_id": "uuid",
    "to_attendant_id": "uuid",
    "reason": "Specialist needed"
}
```

### GET /api/attendants
List all attendants with stats.

## BASIC Keywords

```basic
TRANSFER TO HUMAN "sales"
TRANSFER TO HUMAN "support", "high"
```

## See Also

- [Human Approval](../chapter-06-gbdialog/keyword-human-approval.md)