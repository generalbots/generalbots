# Tasks API

BotServer provides RESTful endpoints for creating, managing, and tracking tasks and workflows within bot conversations.

## Overview

The Tasks API enables:
- Task creation and assignment
- Workflow management
- Task tracking and status updates
- Deadline management
- Task prioritization
- Collaboration features

## Base URL

```
http://localhost:8080/api/v1/tasks
```

## Authentication

All Tasks API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Create Task

**POST** `/tasks`

Create a new task.

**Request Body:**
```json
{
  "title": "Review customer complaint",
  "description": "Investigate and respond to customer issue #1234",
  "assignee": "user456",
  "due_date": "2024-01-20T17:00:00Z",
  "priority": "high",
  "tags": ["support", "urgent"],
  "context": {
    "conversation_id": "conv_abc123",
    "bot_id": "support_bot"
  }
}
```

**Response:**
```json
{
  "task_id": "tsk_xyz789",
  "title": "Review customer complaint",
  "status": "pending",
  "created_at": "2024-01-15T10:00:00Z",
  "created_by": "user123"
}
```

### Get Task

**GET** `/tasks/{task_id}`

Retrieve task details.

**Response:**
```json
{
  "task_id": "tsk_xyz789",
  "title": "Review customer complaint",
  "description": "Investigate and respond to customer issue #1234",
  "status": "in_progress",
  "assignee": {
    "user_id": "user456",
    "name": "Jane Smith",
    "avatar_url": "https://example.com/avatar.jpg"
  },
  "priority": "high",
  "due_date": "2024-01-20T17:00:00Z",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T14:30:00Z",
  "progress": 60,
  "time_spent_minutes": 45,
  "comments_count": 3,
  "attachments_count": 2
}
```

### Update Task

**PATCH** `/tasks/{task_id}`

Update task properties.

**Request Body:**
```json
{
  "status": "in_progress",
  "progress": 60,
  "assignee": "user789"
}
```

**Response:**
```json
{
  "task_id": "tsk_xyz789",
  "updated": true,
  "updated_fields": ["status", "progress", "assignee"],
  "updated_at": "2024-01-15T14:30:00Z"
}
```

### List Tasks

**GET** `/tasks`

List tasks with filtering and pagination.

**Query Parameters:**
- `status` - Filter by status: `pending`, `in_progress`, `completed`, `cancelled`
- `assignee` - Filter by assignee user ID
- `priority` - Filter by priority: `low`, `medium`, `high`, `critical`
- `due_before` - Tasks due before date
- `due_after` - Tasks due after date
- `tags` - Comma-separated tags
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20)
- `sort` - Sort by: `created_at`, `due_date`, `priority`, `updated_at`
- `order` - Sort order: `asc`, `desc`

**Response:**
```json
{
  "tasks": [
    {
      "task_id": "tsk_xyz789",
      "title": "Review customer complaint",
      "status": "in_progress",
      "assignee": "user456",
      "priority": "high",
      "due_date": "2024-01-20T17:00:00Z",
      "progress": 60
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20
}
```

### Complete Task

**POST** `/tasks/{task_id}/complete`

Mark a task as completed.

**Request Body:**
```json
{
  "resolution": "Issue resolved - refund processed",
  "time_spent_minutes": 90,
  "outcomes": ["customer_satisfied", "refund_issued"]
}
```

**Response:**
```json
{
  "task_id": "tsk_xyz789",
  "status": "completed",
  "completed_at": "2024-01-15T16:00:00Z",
  "completed_by": "user456"
}
```

### Delete Task

**DELETE** `/tasks/{task_id}`

Delete a task.

**Response:**
```json
{
  "deleted": true,
  "task_id": "tsk_xyz789"
}
```

## Task Comments

### Add Comment

**POST** `/tasks/{task_id}/comments`

Add a comment to a task.

**Request Body:**
```json
{
  "text": "Contacted customer via email, waiting for response",
  "mentions": ["user123"],
  "attachments": ["file_abc123"]
}
```

**Response:**
```json
{
  "comment_id": "cmt_123",
  "task_id": "tsk_xyz789",
  "text": "Contacted customer via email, waiting for response",
  "author": "user456",
  "created_at": "2024-01-15T14:30:00Z"
}
```

### List Comments

**GET** `/tasks/{task_id}/comments`

Get task comments.

**Response:**
```json
{
  "comments": [
    {
      "comment_id": "cmt_123",
      "text": "Contacted customer via email",
      "author": {
        "user_id": "user456",
        "name": "Jane Smith"
      },
      "created_at": "2024-01-15T14:30:00Z"
    }
  ],
  "total": 3
}
```

## Task Attachments

### Upload Attachment

**POST** `/tasks/{task_id}/attachments`

Attach a file to a task.

**Request:**
- Method: `POST`
- Content-Type: `multipart/form-data`
- Form fields: `file` (binary)

**Response:**
```json
{
  "attachment_id": "att_789",
  "task_id": "tsk_xyz789",
  "filename": "screenshot.png",
  "size_bytes": 102400,
  "mime_type": "image/png",
  "uploaded_at": "2024-01-15T14:45:00Z"
}
```

## Task Templates

### Create Template

**POST** `/templates`

Create a reusable task template.

**Request Body:**
```json
{
  "name": "Customer Complaint",
  "description_template": "Investigate issue: {{issue_id}}",
  "default_priority": "high",
  "default_tags": ["support"],
  "checklist": [
    "Review conversation history",
    "Contact customer",
    "Provide resolution",
    "Follow up"
  ]
}
```

### Create Task from Template

**POST** `/tasks/from-template`

Create a task from a template.

**Request Body:**
```json
{
  "template_id": "tpl_123",
  "variables": {
    "issue_id": "#1234"
  },
  "assignee": "user456",
  "due_date": "2024-01-20T17:00:00Z"
}
```

## Workflows

### Create Workflow

**POST** `/workflows`

Create a multi-step workflow.

**Request Body:**
```json
{
  "name": "Customer Onboarding",
  "steps": [
    {
      "name": "Account Setup",
      "assignee": "user456",
      "duration_hours": 2
    },
    {
      "name": "Training",
      "assignee": "user789",
      "duration_hours": 4,
      "depends_on": ["Account Setup"]
    }
  ]
}
```

### Get Workflow Status

**GET** `/workflows/{workflow_id}/status`

Get workflow progress.

**Response:**
```json
{
  "workflow_id": "wf_123",
  "name": "Customer Onboarding",
  "status": "in_progress",
  "progress": 50,
  "completed_steps": 1,
  "total_steps": 2,
  "current_step": "Training",
  "estimated_completion": "2024-01-16T12:00:00Z"
}
```

## Task Automation

### Create Automation Rule

**POST** `/automations`

Create rules for automatic task creation.

**Request Body:**
```json
{
  "name": "High Priority Support",
  "trigger": {
    "type": "conversation_tag",
    "value": "urgent"
  },
  "action": {
    "type": "create_task",
    "template": "tpl_urgent",
    "auto_assign": true,
    "priority": "critical"
  }
}
```

## Notifications

### Task Notifications

Configure notifications for task events:

```json
{
  "events": [
    "task_assigned",
    "task_completed",
    "task_overdue",
    "comment_added"
  ],
  "channels": ["email", "in_app"],
  "recipients": ["assignee", "watchers"]
}
```

## Analytics

### Task Analytics

**GET** `/tasks/analytics`

Get task performance metrics.

**Response:**
```json
{
  "summary": {
    "total_tasks": 234,
    "completed": 189,
    "in_progress": 35,
    "overdue": 10,
    "completion_rate": 0.81,
    "average_completion_time_hours": 4.5
  },
  "by_priority": {
    "critical": {"total": 10, "completed": 8},
    "high": {"total": 45, "completed": 40},
    "medium": {"total": 120, "completed": 100},
    "low": {"total": 59, "completed": 41}
  },
  "by_assignee": [
    {
      "user_id": "user456",
      "name": "Jane Smith",
      "tasks_completed": 45,
      "average_time_hours": 3.2
    }
  ]
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "invalid_due_date",
  "message": "Due date must be in the future"
}
```

### 404 Not Found
```json
{
  "error": "task_not_found",
  "message": "Task tsk_xyz789 not found"
}
```

### 403 Forbidden
```json
{
  "error": "permission_denied",
  "message": "You don't have permission to modify this task"
}
```

## Best Practices

1. **Clear Titles**: Use descriptive, action-oriented task titles
2. **Set Priorities**: Always set appropriate priority levels
3. **Add Context**: Include conversation or bot context
4. **Use Templates**: Create templates for recurring task types
5. **Track Progress**: Update progress regularly
6. **Set Realistic Deadlines**: Allow adequate time for completion
7. **Use Tags**: Categorize tasks with consistent tags

## Integration with BASIC

Tasks can be created from BASIC scripts:

```basic
' Create task from conversation
task_id = CREATE TASK "Follow up with customer", "user456"
SET TASK PRIORITY task_id, "high"
SET TASK DUE DATE task_id, NOW() + 24 * 3600

' Check task status
status = GET TASK STATUS task_id
IF status = "completed" THEN
    TALK "Task has been completed"
END IF
```

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Create Task | 100/hour | Per user |
| Update Task | 200/hour | Per user |
| List Tasks | 60/minute | Per user |
| Add Comment | 50/hour | Per user |

## Related APIs

- [Notifications API](./notifications-api.md) - Task notifications
- [Analytics API](./analytics-api.md) - Task analytics
- [User API](./user-security.md) - User management