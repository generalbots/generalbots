# CREATE_TASK

Create and assign tasks within the task management system.

## Syntax

```basic
CREATE_TASK title, description, assignee, due_date, priority
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

The `CREATE_TASK` keyword creates tasks in the task engine system with:

- Automatic assignment to users or groups
- Due date tracking and reminders
- Priority-based organization
- Integration with calendar system
- Email notifications to assignees
- Progress tracking capabilities

## Examples

### Basic Task Creation
```basic
CREATE_TASK "Review proposal", "Review and provide feedback on Q4 proposal", "john@example.com", "2024-01-15", "high"
```

### Task with Current User
```basic
user_email = GET_USER_EMAIL()
CREATE_TASK "Follow up", "Contact customer about renewal", user_email, "tomorrow", "medium"
```

### Bulk Task Creation
```basic
team = ["alice@example.com", "bob@example.com", "carol@example.com"]
FOR EACH member IN team
    CREATE_TASK "Complete training", "Finish security awareness training", member, "next week", "medium"
NEXT
```

### Task from User Input
```basic
task_info = HEAR "What task should I create?"
CREATE_TASK task_info, "User requested task", "support@example.com", "today", "high"
TALK "Task created and assigned to support team"
```

## Return Value

Returns a task object containing:
- `task_id`: Unique task identifier
- `status`: Task status ("created", "assigned", "in_progress", "completed")
- `created_at`: Creation timestamp
- `url`: Link to task in web interface
- `reminder_set`: Whether reminder was configured

## Task Statuses

Tasks progress through these statuses:
1. `created` - Initial creation
2. `assigned` - Assigned to user
3. `in_progress` - Work started
4. `blocked` - Waiting on dependency
5. `completed` - Task finished
6. `cancelled` - Task cancelled

## Integration Points

### Calendar Integration
Tasks automatically appear in assignee's calendar if:
- Due date is specified
- Calendar integration is enabled
- User has calendar permissions

### Email Notifications
Sends notifications for:
- Task assignment
- Due date reminders
- Status changes
- Comments added

### Task Dependencies
Can link tasks together:
```basic
parent_task = CREATE_TASK "Project", "Main project", "pm@example.com", "next month", "high"
subtask = CREATE_TASK "Research", "Initial research", "analyst@example.com", "next week", "medium"
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

Supports multiple date formats:
- Absolute: `"2024-01-15"`, `"01/15/2024"`
- Relative: `"today"`, `"tomorrow"`, `"next week"`, `"in 3 days"`
- Natural: `"Monday"`, `"next Friday"`, `"end of month"`

## Error Handling

- Validates assignee exists in system
- Checks date is in the future
- Verifies priority is valid
- Returns error if task creation fails
- Handles permission issues gracefully

## Permissions

User must have one of:
- Task creation permission
- Project member status
- Admin privileges
- Delegation rights from assignee

## Best Practices

1. **Clear Titles**: Use descriptive, action-oriented titles
2. **Detailed Descriptions**: Include acceptance criteria
3. **Realistic Dates**: Set achievable deadlines
4. **Appropriate Priority**: Don't mark everything as urgent
5. **Valid Assignees**: Verify user can handle the task
6. **Follow Up**: Check task status periodically

## Advanced Usage

### Task Templates
```basic
template = GET_TASK_TEMPLATE("customer_onboarding")
CREATE_TASK template.title, template.description, assigned_user, due_date, template.priority
```

### Conditional Creation
```basic
IF urgency = "high" AND department = "support" THEN
    CREATE_TASK "Urgent Support", issue_description, "support-lead@example.com", "today", "urgent"
ELSE
    CREATE_TASK "Support Request", issue_description, "support@example.com", "tomorrow", "medium"
END IF
```

### Task with Attachments
```basic
task = CREATE_TASK "Review document", "Please review attached", reviewer, deadline, "high"
ATTACH_FILE task.task_id, "proposal.pdf"
```

## Related Keywords

- [BOOK](./keyword-book.md) - Schedule meetings instead of tasks
- [SET_SCHEDULE](./keyword-set-schedule.md) - Create recurring tasks
- [SEND_MAIL](./keyword-send-mail.md) - Send task notifications
- [ADD_MEMBER](./keyword-add-member.md) - Add users to task groups

## Database Tables

Tasks are stored in:
- `tasks` - Main task records
- `task_assignments` - User assignments
- `task_comments` - Task discussions
- `task_attachments` - Related files
- `task_history` - Status changes

## Implementation

Located in `src/basic/keywords/create_task.rs`

Integrates with:
- Task engine module for task management
- Calendar engine for scheduling
- Email module for notifications
- Storage module for attachments