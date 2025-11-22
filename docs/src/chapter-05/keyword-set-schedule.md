# SET_SCHEDULE

Schedule recurring tasks or events to run at specified intervals.

## Syntax

```basic
SET_SCHEDULE schedule_name, cron_expression, task
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `schedule_name` | String | Unique identifier for the schedule |
| `cron_expression` | String | Cron expression or time interval |
| `task` | String | Task to execute (dialog name or command) |

## Description

The `SET_SCHEDULE` keyword creates recurring scheduled tasks that execute at specified times or intervals. It supports both cron expressions for complex scheduling and simple interval syntax for common patterns.

Schedules persist across bot restarts and run independently of user sessions, making them ideal for automated tasks, reminders, reports, and maintenance operations.

## Examples

### Daily Reminder
```basic
SET_SCHEDULE "daily_standup", "0 9 * * MON-FRI", "send_standup_reminder.bas"
TALK "Daily standup reminder scheduled for 9 AM on weekdays"
```

### Hourly Task
```basic
SET_SCHEDULE "check_inventory", "every 1 hour", "inventory_check.bas"
TALK "Inventory will be checked every hour"
```

### Weekly Report
```basic
SET_SCHEDULE "weekly_report", "0 10 * * MON", "generate_weekly_report.bas"
TALK "Weekly reports will be generated every Monday at 10 AM"
```

### Monthly Billing
```basic
SET_SCHEDULE "monthly_billing", "0 0 1 * *", "process_billing.bas"
TALK "Billing will process on the 1st of each month"
```

## Schedule Formats

### Cron Expression Format
Standard cron format with five fields:
```
* * * * *
│ │ │ │ │
│ │ │ │ └─── Day of week (0-7, MON-SUN)
│ │ │ └───── Month (1-12)
│ │ └─────── Day of month (1-31)
│ └───────── Hour (0-23)
└─────────── Minute (0-59)
```

Examples:
- `"0 9 * * *"` - Every day at 9:00 AM
- `"*/15 * * * *"` - Every 15 minutes
- `"0 0 * * SUN"` - Every Sunday at midnight
- `"0 8,12,16 * * *"` - At 8 AM, noon, and 4 PM daily

### Simple Interval Format
Human-readable intervals:
- `"every 30 minutes"`
- `"every 2 hours"`
- `"every day"`
- `"every week"`
- `"every month"`

### Relative Time Format
- `"in 5 minutes"` - One-time schedule
- `"tomorrow at 3pm"`
- `"next Monday"`
- `"end of month"`

## Task Types

### Dialog Execution
```basic
SET_SCHEDULE "greeting", "0 8 * * *", "morning_greeting.bas"
```

### Function Call
```basic
SET_SCHEDULE "backup", "0 2 * * *", "BACKUP_DATABASE()"
```

### Email Task
```basic
SET_SCHEDULE "reminder", "0 9 * * MON", "SEND_MAIL admin@example.com, 'Weekly Tasks', 'Please review weekly tasks'"
```

### Multi-Step Task
```basic
task = "FETCH_DATA(); PROCESS_DATA(); SEND_REPORT()"
SET_SCHEDULE "data_pipeline", "*/30 * * * *", task
```

## Schedule Management

### List Active Schedules
```basic
schedules = GET_SCHEDULES()
FOR EACH schedule IN schedules
    TALK schedule.name + ": " + schedule.expression + " - " + schedule.task
NEXT
```

### Cancel Schedule
```basic
CANCEL_SCHEDULE "daily_reminder"
TALK "Daily reminder has been cancelled"
```

### Update Schedule
```basic
' Cancel old and create new
CANCEL_SCHEDULE "report"
SET_SCHEDULE "report", "0 10 * * *", "new_report.bas"
```

### Check Schedule Status
```basic
status = GET_SCHEDULE_STATUS("backup")
TALK "Last run: " + status.last_run
TALK "Next run: " + status.next_run
```

## Advanced Features

### Conditional Scheduling
```basic
IF is_production THEN
    SET_SCHEDULE "prod_backup", "0 1 * * *", "backup_prod.bas"
ELSE
    SET_SCHEDULE "dev_backup", "0 3 * * SUN", "backup_dev.bas"
END IF
```

### Dynamic Scheduling
```basic
hour = USER_PREFERENCE("reminder_hour")
cron = "0 " + hour + " * * *"
SET_SCHEDULE "user_reminder", cron, "send_reminder.bas"
```

### Schedule with Parameters
```basic
' Pass parameters to scheduled task
task_with_params = "PROCESS_REPORT('daily', 'sales', true)"
SET_SCHEDULE "daily_sales", "0 6 * * *", task_with_params
```

### Error Handling in Schedules
```basic
task = "TRY; RUN_TASK(); CATCH; LOG_ERROR(); END TRY"
SET_SCHEDULE "safe_task", "*/10 * * * *", task
```

## Timezone Handling

Schedules use the bot's configured timezone:
```basic
' Set timezone in config.csv
' timezone = "America/New_York"

SET_SCHEDULE "ny_task", "0 9 * * *", "task.bas"
' Runs at 9 AM New York time
```

## Return Value

Returns schedule object:
- `id`: Unique schedule identifier
- `name`: Schedule name
- `created`: Creation timestamp
- `next_run`: Next execution time
- `active`: Schedule status

## Persistence

- Schedules stored in database
- Survive bot restarts
- Automatic recovery after crashes
- Execution history tracked

## Best Practices

1. **Use descriptive names**: Make schedule purpose clear
2. **Test cron expressions**: Verify timing is correct
3. **Handle failures**: Include error handling in tasks
4. **Avoid overlaps**: Ensure tasks complete before next run
5. **Monitor execution**: Check logs for failures
6. **Clean up old schedules**: Remove unused schedules
7. **Document schedules**: Keep list of active schedules

## Common Use Cases

### Daily Reports
```basic
SET_SCHEDULE "daily_summary", "0 18 * * *", "generate_daily_summary.bas"
```

### Data Synchronization
```basic
SET_SCHEDULE "sync_data", "*/15 * * * *", "sync_with_external_api.bas"
```

### Maintenance Tasks
```basic
SET_SCHEDULE "cleanup", "0 3 * * *", "cleanup_old_files.bas"
SET_SCHEDULE "optimize", "0 4 * * SUN", "optimize_database.bas"
```

### User Reminders
```basic
SET_SCHEDULE "medication", "0 8,14,20 * * *", "send_medication_reminder.bas"
```

### Business Processes
```basic
SET_SCHEDULE "payroll", "0 10 L * *", "process_payroll.bas"  ' Last day of month
SET_SCHEDULE "invoicing", "0 9 1 * *", "generate_invoices.bas"  ' First of month
```

## Limitations

- Maximum 100 active schedules per bot
- Minimum interval: 1 minute
- Tasks timeout after 5 minutes
- No sub-second precision
- Schedules run in bot context (not user)

## Troubleshooting

### Schedule Not Running
- Verify cron expression syntax
- Check bot is active
- Review task for errors
- Check timezone settings

### Task Failures
- Check task dialog exists
- Verify permissions
- Review error logs
- Test task manually first

### Performance Issues
- Limit concurrent schedules
- Optimize task execution
- Use appropriate intervals
- Monitor resource usage

## Related Keywords

- [CREATE_TASK](./keyword-create-task.md) - Create one-time tasks
- [WAIT](./keyword-wait.md) - Delay execution
- [ON](./keyword-on.md) - Event-based triggers
- [SEND_MAIL](./keyword-send-mail.md) - Often used in scheduled tasks

## Implementation

Located in `src/basic/keywords/set_schedule.rs`

Uses cron parser for expression validation and tokio scheduler for execution management.