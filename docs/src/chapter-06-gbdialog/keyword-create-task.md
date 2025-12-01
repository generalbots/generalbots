# CREATE TASK

Create and assign tasks within the task management system.

## Syntax

```basic
CREATE TASK title, description, assignee, due_date, priority
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | String | Task title/name |
| `description` | String | Detailed task description |
| `assignee` | String | Email or user ID of the assignee |
| `due_date` | String | Due date in format "YYYY-MM-DD" or relative like "tomorrow", "next week" |
| `priority` | String | Task priority: "low", "medium", "high", "urgent" |

## Description

The `CREATE TASK` keyword creates tasks in the task engine system with automatic assignment to users or groups, due date tracking and reminders, priority-based organization, integration with the calendar system, email notifications to assignees, and progress tracking capabilities.

## Examples

### Basic Task Creation

```basic
CREATE TASK "Review proposal", "Review and provide feedback on Q4 proposal", "john@example.com", "2024-01-15", "high"
```

### Task with Current User

```basic
user_email = GET "user.email"
CREATE TASK "Follow up", "Contact customer about renewal", user_email, "tomorrow", "medium"
```

### Bulk Task Creation

```basic
team = ["alice@example.com", "bob@example.com", "carol@example.com"]
FOR EACH member IN team
    CREATE TASK "Complete training", "Finish security awareness training", member, "next week", "medium"
NEXT
```

### Task from User Input

```basic
task_info = HEAR "What task should I create?"
CREATE TASK task_info, "User requested task", "support@example.com", "today", "high"
TALK "Task created and assigned to support team"
```

## Return Value

The keyword returns a task object containing the `task_id` as a unique task identifier, `status` indicating the task state (such as "created", "assigned", "in_progress", or "completed"), `created_at` with the creation timestamp, `url` providing a link to the task in the web interface, and `reminder_set` indicating whether a reminder was configured.

## Task Statuses

Tasks progress through a defined lifecycle. The `created` status indicates initial creation, followed by `assigned` when the task has been assigned to a user. Once work begins, the status changes to `in_progress`. If the task is waiting on a dependency, it enters the `blocked` state. When finished, it reaches `completed`, or alternatively `cancelled` if the task was terminated without completion.

## Integration Points

### Calendar Integration

Tasks automatically appear in the assignee's calendar when a due date is specified, calendar integration is enabled, and the user has calendar permissions.

### Email Notifications

The system sends notifications for task assignment, due date reminders, status changes, and when comments are added.

### Task Dependencies

Tasks can be linked together to create parent-child relationships:

```basic
parent_task = CREATE TASK "Project", "Main project", "pm@example.com", "next month", "high"
subtask = CREATE TASK "Research", "Initial research", "analyst@example.com", "next week", "medium"
LINK_TASKS parent_task.task_id, subtask.task_id
```

## Priority Levels

| Priority | Description | SLA |
|----------|-------------|-----|
| `urgent` | Immediate attention required | 4 hours |
| `high` | Important, time-sensitive | 1 day |
| `medium` | Standard priority | 3 days |
| `low` | Non-urgent | 1 week |

## Date Formats

The keyword supports multiple date formats. Absolute dates can be specified as "2024-01-15" or "01/15/2024". Relative dates include "today", "tomorrow", "next week", and "in 3 days". Natural language formats like "Monday", "next Friday", and "end of month" are also supported.

## Error Handling

The keyword validates that the assignee exists in the system, checks that the date is in the future, verifies the priority is valid, returns an error if task creation fails, and handles permission issues gracefully.

## Permissions

To create tasks, the user must have task creation permission, project member status, admin privileges, or delegation rights from the assignee.

## Best Practices

Use clear, action-oriented titles that describe what needs to be done. Include detailed descriptions with acceptance criteria so the assignee understands the requirements. Set realistic deadlines that can actually be achieved. Reserve high and urgent priorities for tasks that truly warrant them rather than marking everything as urgent. Verify the assignee can handle the task before assignment. Follow up periodically to check task status and provide assistance if needed.

## Advanced Usage

### Task Templates

```basic
template = GET_TASK_TEMPLATE("customer_onboarding")
CREATE TASK template.title, template.description, assigned_user, due_date, template.priority
```

### Conditional Creation

```basic
IF urgency = "high" AND department = "support" THEN
    CREATE TASK "Urgent Support", issue_description, "support-lead@example.com", "today", "urgent"
ELSE
    CREATE TASK "Support Request", issue_description, "support@example.com", "tomorrow", "medium"
END IF
```

### Task with Attachments

```basic
task = CREATE TASK "Review document", "Please review attached", reviewer, deadline, "high"
' Note: Use document sharing systems for attachments
```

## Related Keywords

The [BOOK](./keyword-book.md) keyword schedules meetings instead of tasks. Use [SET SCHEDULE](./keyword-set-schedule.md) to create recurring tasks. The [SEND MAIL](./keyword-send-mail.md) keyword sends task notifications, and [ADD MEMBER](./keyword-add-member.md) adds users to task groups.

## Database Tables

Tasks are stored across several database tables. The `tasks` table holds main task records. User assignments are tracked in `task_assignments`. Discussions happen in `task_comments`. Related files are referenced in `task_attachments`. The `task_history` table records status changes over time.

## Implementation

The CREATE TASK keyword is implemented in `src/basic/keywords/create_task.rs`. It integrates with the task engine module for task management, the calendar engine for scheduling, the email module for notifications, and the storage module for attachments.