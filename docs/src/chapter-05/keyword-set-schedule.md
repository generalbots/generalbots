# SET SCHEDULE

Schedule a script or task to run at specified times using cron expressions.

## Syntax

```basic
SET SCHEDULE cron_expression
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `cron_expression` | String | Standard cron expression defining when to run |

## Description

The `SET SCHEDULE` keyword schedules the current script to run automatically at specified intervals. It uses standard cron syntax for maximum flexibility in scheduling.

## Cron Expression Format

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

## Examples

### Every Hour
```basic
SET SCHEDULE "0 * * * *"
' Runs at the start of every hour
```

### Daily at Specific Time
```basic
SET SCHEDULE "0 9 * * *"
' Runs every day at 9:00 AM
```

### Every 5 Minutes
```basic
SET SCHEDULE "*/5 * * * *"
' Runs every 5 minutes
```

### Weekdays Only
```basic
SET SCHEDULE "0 8 * * 1-5"
' Runs at 8 AM Monday through Friday
```

### Multiple Times Daily
```basic
SET SCHEDULE "0 9,12,17 * * *"
' Runs at 9 AM, 12 PM, and 5 PM
```

### Monthly Reports
```basic
SET SCHEDULE "0 6 1 * *"
' Runs at 6 AM on the first day of each month
```

## Common Patterns

| Pattern | Cron Expression | Description |
|---------|----------------|-------------|
| Every minute | `* * * * *` | Runs every minute |
| Every hour | `0 * * * *` | Start of every hour |
| Every 30 minutes | `*/30 * * * *` | Every 30 minutes |
| Daily at midnight | `0 0 * * *` | Every day at 12:00 AM |
| Weekly on Monday | `0 0 * * 1` | Every Monday at midnight |
| Last day of month | `0 0 28-31 * *` | End of month (approximate) |
| Business hours | `0 9-17 * * 1-5` | Every hour 9 AM-5 PM weekdays |

## Practical Use Cases

### Daily Summary Generation
```basic
SET SCHEDULE "0 6 * * *"

' Fetch and summarize daily data
data = GET "reports/daily.json"
summary = LLM "Summarize key metrics: " + data
SET BOT MEMORY "daily_summary", summary
```

### Hourly Data Refresh
```basic
SET SCHEDULE "0 * * * *"

' Update cached data every hour
fresh_data = FETCH_EXTERNAL_API()
SET BOT MEMORY "cached_data", fresh_data
```

### Weekly Newsletter
```basic
SET SCHEDULE "0 10 * * 1"

' Send weekly newsletter every Monday at 10 AM
subscribers = GET_SUBSCRIBERS()
FOR EACH email IN subscribers
    SEND MAIL email, "Weekly Update", newsletter_content
NEXT
```

### Periodic Cleanup
```basic
SET SCHEDULE "0 2 * * *"

' Clean old data daily at 2 AM
DELETE_OLD_LOGS(30)  ' Keep 30 days
VACUUM_DATABASE()
```

## Schedule Management

### View Active Schedules
```basic
schedules = GET_SCHEDULES()
FOR EACH schedule IN schedules
    TALK "Task: " + schedule.name
    TALK "Next run: " + schedule.next_run
NEXT
```

### Cancel Schedule
```basic
' Schedules are automatically canceled when script ends
' Or use:
CANCEL SCHEDULE "task_id"
```

## Best Practices

1. **Start Time Consideration**: Avoid scheduling all tasks at the same time
   ```basic
   ' Bad: Everything at midnight
   SET SCHEDULE "0 0 * * *"
   
   ' Good: Stagger tasks
   SET SCHEDULE "0 2 * * *"  ' Cleanup at 2 AM
   SET SCHEDULE "0 3 * * *"  ' Backup at 3 AM
   ```

2. **Resource Management**: Consider system load
   ```basic
   ' Heavy processing during off-hours
   SET SCHEDULE "0 2-4 * * *"
   ```

3. **Error Handling**: Include error recovery
   ```basic
   SET SCHEDULE "0 * * * *"
   
   TRY
       PROCESS_DATA()
   CATCH
       LOG "Schedule failed: " + ERROR_MESSAGE
       SEND MAIL "admin@example.com", "Schedule Error", ERROR_DETAILS
   END TRY
   ```

4. **Idempotency**: Make scheduled tasks safe to re-run
   ```basic
   ' Check if already processed
   last_run = GET BOT MEMORY "last_process_time"
   IF TIME_DIFF(NOW(), last_run) > 3600 THEN
       PROCESS()
       SET BOT MEMORY "last_process_time", NOW()
   END IF
   ```

## Limitations

- Maximum 100 scheduled tasks per bot
- Minimum interval: 1 minute
- Scripts timeout after 5 minutes by default
- Schedules persist until explicitly canceled or bot restarts
- Time zone is server's local time unless specified

## Time Zone Support

```basic
' Specify time zone (if supported)
SET SCHEDULE "0 9 * * *" TIMEZONE "America/New_York"
```

## Monitoring

Scheduled tasks are logged for monitoring:
- Execution start/end times
- Success/failure status
- Error messages
- Performance metrics

## Related Keywords

- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Store schedule state
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Persist data between runs
- [LLM](./keyword-llm.md) - Process data in scheduled tasks
- [SEND MAIL](./keyword-send-mail.md) - Send scheduled reports
- [GET](./keyword-get.md) - Fetch data for processing

## Implementation

Located in `src/basic/keywords/set_schedule.rs`

The implementation:
- Uses cron parser for expression validation
- Integrates with system scheduler
- Persists schedules in database
- Handles concurrent execution
- Provides retry logic for failures