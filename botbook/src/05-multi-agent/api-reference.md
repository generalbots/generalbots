# API Reference

Complete API reference for the AutoTask system. All endpoints require authentication.

---

## Base URL

All endpoints are relative to your bot's API base URL:

```
https://your-bot.example.com/api/autotask
```

---

## Intent Classification

### Classify Intent

Classify a natural language intent and optionally process it.

**POST** `/api/autotask/classify`

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `intent` | string | Yes | Natural language description |
| `auto_process` | boolean | No | If true, process immediately (default: true) |

#### Example Request

```json
{
  "intent": "Create an app for my clinic with patients and appointments",
  "auto_process": true
}
```

#### Response

```json
{
  "success": true,
  "intent_type": "APP_CREATE",
  "confidence": 0.92,
  "suggested_name": "clinic",
  "requires_clarification": false,
  "clarification_question": null,
  "result": {
    "success": true,
    "message": "Done:\npatients table created\nappointments table created\nApp available at /apps/clinic",
    "app_url": "/apps/clinic",
    "task_id": null,
    "schedule_id": null,
    "tool_triggers": [],
    "created_resources": [
      {"resource_type": "table", "name": "patients", "path": "tables.bas"},
      {"resource_type": "table", "name": "appointments", "path": "tables.bas"},
      {"resource_type": "page", "name": "Dashboard", "path": "index.html"}
    ],
    "next_steps": ["Open the app to start using it", "Use Designer to customize the app"]
  },
  "error": null
}
```

#### Intent Types

| Type | Description |
|------|-------------|
| `APP_CREATE` | Create full HTMX application |
| `TODO` | Save task to tasks table |
| `MONITOR` | Create ON CHANGE event handler |
| `ACTION` | Execute immediately |
| `SCHEDULE` | Create SET SCHEDULE automation |
| `GOAL` | Autonomous LLM loop with metrics |
| `TOOL` | Create voice/chat command |

---

## Intent Compilation

### Compile Intent

Compile an intent into an execution plan without executing.

**POST** `/api/autotask/compile`

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `intent` | string | Yes | Natural language description |
| `execution_mode` | string | No | `semi-automatic`, `supervised`, `fully-automatic`, `dry-run` |
| `priority` | string | No | `critical`, `high`, `medium`, `low`, `background` |

#### Example Request

```json
{
  "intent": "Create a CRM for tracking customers and orders",
  "execution_mode": "semi-automatic",
  "priority": "medium"
}
```

#### Response

```json
{
  "success": true,
  "plan_id": "550e8400-e29b-41d4-a716-446655440000",
  "plan_name": "CRM Application",
  "plan_description": "Customer relationship management system",
  "steps": [
    {
      "id": "step-1",
      "order": 1,
      "name": "Create customers table",
      "description": "Define customer data structure",
      "keywords": ["TABLE"],
      "priority": "HIGH",
      "risk_level": "LOW",
      "estimated_minutes": 1,
      "requires_approval": false
    }
  ],
  "alternatives": [],
  "confidence": 0.85,
  "risk_level": "LOW",
  "estimated_duration_minutes": 5,
  "estimated_cost": 0.02,
  "resource_estimate": {
    "compute_hours": 0.1,
    "storage_gb": 0.01,
    "api_calls": 5,
    "llm_tokens": 2000,
    "estimated_cost_usd": 0.02
  },
  "basic_program": "' Generated BASIC program\nTABLE customers...",
  "requires_approval": false,
  "mcp_servers": [],
  "external_apis": [],
  "risks": [],
  "error": null
}
```

---

## Plan Execution

### Execute Plan

Execute a compiled plan.

**POST** `/api/autotask/execute`

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `plan_id` | string | Yes | ID from compile response |
| `execution_mode` | string | No | Override execution mode |
| `priority` | string | No | Override priority |

#### Response

```json
{
  "success": true,
  "task_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "running",
  "error": null
}
```

### Simulate Plan

Simulate execution without making changes.

**POST** `/api/autotask/simulate/:plan_id`

#### Response

```json
{
  "success": true,
  "confidence": 0.95,
  "risk_score": 0.1,
  "risk_level": "LOW",
  "step_outcomes": [
    {
      "step_id": "step-1",
      "step_name": "Create customers table",
      "would_succeed": true,
      "success_probability": 0.98,
      "failure_modes": []
    }
  ],
  "impact": {
    "risk_score": 0.1,
    "risk_level": "LOW",
    "data_impact": {
      "records_created": 0,
      "records_modified": 0,
      "records_deleted": 0,
      "tables_affected": ["customers"],
      "reversible": true
    },
    "cost_impact": {
      "api_costs": 0.01,
      "compute_costs": 0.005,
      "storage_costs": 0.001,
      "total_estimated_cost": 0.016
    },
    "time_impact": {
      "estimated_duration_seconds": 30,
      "blocking": false
    },
    "security_impact": {
      "risk_level": "LOW",
      "credentials_accessed": [],
      "external_systems": [],
      "concerns": []
    }
  },
  "side_effects": [],
  "recommendations": [],
  "error": null
}
```

---

## Task Management

### List Tasks

Get all tasks with optional filtering.

**GET** `/api/autotask/list`

#### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `filter` | string | `all`, `running`, `pending`, `completed`, `failed` |
| `status` | string | Specific status filter |
| `priority` | string | Priority filter |
| `limit` | integer | Max results (default: 50) |
| `offset` | integer | Pagination offset |

#### Response

```json
{
  "tasks": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "title": "Create CRM Application",
      "status": "running",
      "progress": 0.6,
      "current_step": 3,
      "total_steps": 5
    }
  ],
  "total": 1
}
```

### Get Task Stats

Get summary statistics.

**GET** `/api/autotask/stats`

#### Response

```json
{
  "total": 25,
  "running": 2,
  "pending": 5,
  "completed": 15,
  "failed": 3,
  "pending_approval": 1,
  "pending_decision": 0
}
```

### Execute Task

Start execution of a specific task.

**POST** `/api/autotask/:task_id/execute`

#### Response

```json
{
  "success": true,
  "task_id": "660e8400-e29b-41d4-a716-446655440001",
  "message": "Task execution started"
}
```

### Pause Task

Pause a running task.

**POST** `/api/autotask/:task_id/pause`

#### Response

```json
{
  "success": true,
  "message": "Task paused"
}
```

### Resume Task

Resume a paused task.

**POST** `/api/autotask/:task_id/resume`

#### Response

```json
{
  "success": true,
  "message": "Task resumed"
}
```

### Cancel Task

Cancel a task.

**POST** `/api/autotask/:task_id/cancel`

#### Response

```json
{
  "success": true,
  "message": "Task cancelled"
}
```

### Simulate Task

Run simulation on an existing task.

**POST** `/api/autotask/:task_id/simulate`

#### Response

Same as Plan Simulation response.

### Get Task Logs

Get execution logs for a task.

**GET** `/api/autotask/:task_id/logs`

#### Response

```json
{
  "task_id": "660e8400-e29b-41d4-a716-446655440001",
  "logs": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "level": "info",
      "message": "Task initialized"
    },
    {
      "timestamp": "2024-01-15T10:30:01Z",
      "level": "info",
      "message": "Step 1: Creating customers table"
    }
  ]
}
```

---

## Decisions

### Get Pending Decisions

Get decisions requiring user input.

**GET** `/api/autotask/:task_id/decisions`

#### Response

```json
{
  "decisions": [
    {
      "id": "dec-001",
      "title": "Choose database type",
      "description": "Select the database type for the application",
      "options": [
        {"id": "opt-1", "label": "PostgreSQL", "description": "Recommended for production"},
        {"id": "opt-2", "label": "SQLite", "description": "Simple, file-based"}
      ],
      "default_option": "opt-1",
      "timeout_seconds": 3600
    }
  ]
}
```

### Submit Decision

Submit a decision response.

**POST** `/api/autotask/:task_id/decide`

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `decision_id` | string | Yes | Decision ID |
| `option_id` | string | No | Selected option ID |
| `skip` | boolean | No | Skip and use default |

#### Example Request

```json
{
  "decision_id": "dec-001",
  "option_id": "opt-1"
}
```

#### Response

```json
{
  "success": true,
  "message": "Decision submitted"
}
```

---

## Approvals

### Get Pending Approvals

Get actions requiring approval.

**GET** `/api/autotask/:task_id/approvals`

#### Response

```json
{
  "approvals": [
    {
      "id": "apr-001",
      "title": "Bulk email send",
      "description": "This action will send 50 emails to customers",
      "risk_level": "MEDIUM",
      "impact_summary": "50 emails will be sent",
      "timeout_seconds": 3600
    }
  ]
}
```

### Submit Approval

Approve or reject an action.

**POST** `/api/autotask/:task_id/approve`

#### Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `approval_id` | string | Yes | Approval ID |
| `action` | string | Yes | `approve`, `reject`, `skip` |
| `comment` | string | No | Optional comment |

#### Example Request

```json
{
  "approval_id": "apr-001",
  "action": "approve",
  "comment": "Verified recipient list"
}
```

#### Response

```json
{
  "success": true,
  "message": "Action approved"
}
```

---

## Recommendations

### Apply Recommendation

Apply a recommendation from simulation results.

**POST** `/api/autotask/recommendations/:rec_id/apply`

#### Response

```json
{
  "success": true,
  "recommendation_id": "rec-001",
  "message": "Recommendation applied successfully"
}
```

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "success": false,
  "error": "Error description"
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not found |
| 500 | Internal server error |

---

## Next Steps

- [Workflow Guide](./workflow.md) — Understanding task execution
- [Examples](./examples.md) — Real-world use cases
- [Designer Guide](./designer.md) — Modifying apps through conversation