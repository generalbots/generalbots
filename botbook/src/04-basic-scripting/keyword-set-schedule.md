# SET SCHEDULE

Schedule a script or task to run at specified times using natural language or cron expressions.

## Syntax

```basic
SET SCHEDULE expression
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `expression` | String | Natural language schedule or cron expression |

## Description

The `SET SCHEDULE` keyword schedules the current script to run automatically at specified intervals. It supports **natural language expressions** that are automatically converted to cron format, making scheduling intuitive and readable.

## Natural Language Patterns

### Time Intervals

```basic
SET SCHEDULE "every minute"
SET SCHEDULE "every 5 minutes"
SET SCHEDULE "every 15 minutes"
SET SCHEDULE "every 30 minutes"
SET SCHEDULE "every hour"
SET SCHEDULE "every 2 hours"
SET SCHEDULE "every 6 hours"
SET SCHEDULE "every day"
SET SCHEDULE "every week"
SET SCHEDULE "every month"
SET SCHEDULE "every year"
```

### Aliases

```basic
SET SCHEDULE "hourly"      ' Same as "every hour"
SET SCHEDULE "daily"       ' Same as "every day"
SET SCHEDULE "weekly"      ' Same as "every week"
SET SCHEDULE "monthly"     ' Same as "every month"
SET SCHEDULE "yearly"      ' Same as "every year"
```

### Specific Times

```basic
SET SCHEDULE "at 9am"
SET SCHEDULE "at 9:30am"
SET SCHEDULE "at 2pm"
SET SCHEDULE "at 14:00"
SET SCHEDULE "at midnight"
SET SCHEDULE "at noon"
```

### Day-Specific

```basic
SET SCHEDULE "every monday"
SET SCHEDULE "every friday"
SET SCHEDULE "every sunday"
SET SCHEDULE "every monday at 9am"
SET SCHEDULE "every friday at 5pm"
```

### Weekdays & Weekends

```basic
SET SCHEDULE "weekdays"              ' Monday-Friday at midnight
SET SCHEDULE "every weekday"         ' Same as above
SET SCHEDULE "weekdays at 8am"       ' Monday-Friday at 8 AM
SET SCHEDULE "weekends"              ' Saturday & Sunday at midnight
SET SCHEDULE "weekends at 10am"      ' Saturday & Sunday at 10 AM
```

### Combined Patterns

```basic
SET SCHEDULE "every day at 9am"
SET SCHEDULE "every day at 6:30pm"
SET SCHEDULE "every hour from 9 to 17"
```

### Business Hours

```basic
SET SCHEDULE "business hours"                           ' Every hour 9-17, Mon-Fri
SET SCHEDULE "every hour during business hours"         ' Same as above
SET SCHEDULE "every 30 minutes during business hours"   ' Every 30 min, 9-17, Mon-Fri
SET SCHEDULE "every 15 minutes during business hours"
```

### Raw Cron (Advanced)

You can still use standard cron expressions for maximum flexibility:

```basic
SET SCHEDULE "0 * * * *"       ' Every hour at minute 0
SET SCHEDULE "*/5 * * * *"     ' Every 5 minutes
SET SCHEDULE "0 9-17 * * 1-5"  ' Hourly 9AM-5PM on weekdays
SET SCHEDULE "0 0 1 * *"       ' First day of each month
```

## Cron Expression Format (Reference)

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

## Quick Reference Table

| Natural Language | Cron Equivalent | Description |
|-----------------|-----------------|-------------|
| `every minute` | `* * * * *` | Runs every minute |
| `every 5 minutes` | `*/5 * * * *` | Every 5 minutes |
| `every hour` | `0 * * * *` | Start of every hour |
| `hourly` | `0 * * * *` | Same as every hour |
| `every day` | `0 0 * * *` | Daily at midnight |
| `daily` | `0 0 * * *` | Same as every day |
| `at 9am` | `0 9 * * *` | Daily at 9 AM |
| `at 9:30am` | `30 9 * * *` | Daily at 9:30 AM |
| `at noon` | `0 12 * * *` | Daily at noon |
| `at midnight` | `0 0 * * *` | Daily at midnight |
| `every monday` | `0 0 * * 1` | Monday at midnight |
| `every monday at 9am` | `0 9 * * 1` | Monday at 9 AM |
| `weekdays` | `0 0 * * 1-5` | Mon-Fri at midnight |
| `weekdays at 8am` | `0 8 * * 1-5` | Mon-Fri at 8 AM |
| `weekends` | `0 0 * * 0,6` | Sat-Sun at midnight |
| `every week` | `0 0 * * 0` | Sunday at midnight |
| `weekly` | `0 0 * * 0` | Same as every week |
| `every month` | `0 0 1 * *` | 1st of month |
| `monthly` | `0 0 1 * *` | Same as every month |
| `business hours` | `0 9-17 * * 1-5` | Hourly 9-5 weekdays |
| `every hour from 9 to 17` | `0 9-17 * * *` | Hourly 9 AM - 5 PM |

## Examples

### Daily Report at 9 AM

```basic
SET SCHEDULE "every day at 9am"

data = GET "reports/daily.json"
summary = LLM "Summarize key metrics: " + data
SEND MAIL "team@company.com", "Daily Report", summary
```

### Hourly Data Sync

```basic
SET SCHEDULE "every hour"

fresh_data = GET "https://api.example.com/data"
SET BOT MEMORY "cached_data", fresh_data
PRINT "Data refreshed at " + NOW()
```

### Every 15 Minutes Monitoring

```basic
SET SCHEDULE "every 15 minutes"

status = GET "https://api.example.com/health"
IF status.healthy = false THEN
    SEND MAIL "ops@company.com", "Alert: Service Down", status.message
END IF
```

### Weekly Newsletter (Monday 10 AM)

```basic
SET SCHEDULE "every monday at 10am"

subscribers = FIND "subscribers", "active=true"
content = LLM "Generate weekly newsletter with latest updates"

FOR EACH email IN subscribers
    SEND MAIL email.address, "Weekly Update", content
NEXT
```

### Business Hours Support Check

```basic
SET SCHEDULE "every 30 minutes during business hours"

tickets = FIND "support_tickets", "status=open AND priority=high"
IF LEN(tickets) > 5 THEN
    TALK TO "support-manager", "High priority ticket queue: " + LEN(tickets) + " tickets waiting"
END IF
```

### Weekend Backup

```basic
SET SCHEDULE "weekends at 3am"

PRINT "Starting weekend backup..."
result = POST "https://backup.service/run", { "type": "full" }
SET BOT MEMORY "last_backup", NOW()
SEND MAIL "admin@company.com", "Backup Complete", result
```

### End of Month Report

```basic
SET SCHEDULE "monthly"

' Runs on 1st of each month at midnight
month_data = AGGREGATE "sales", "SUM(amount)", "month=" + MONTH(DATEADD("month", -1, NOW()))
report = LLM "Generate monthly sales report for: " + month_data
SEND MAIL "finance@company.com", "Monthly Sales Report", report
```

## Best Practices

1. **Use Natural Language**: Prefer readable expressions like `"every day at 9am"` over cron syntax

2. **Stagger Tasks**: Avoid scheduling all tasks at the same time
   ```basic
   ' Good: Different times
   SET SCHEDULE "every day at 2am"   ' Cleanup
   SET SCHEDULE "every day at 3am"   ' Backup
   SET SCHEDULE "every day at 4am"   ' Reports
   ```

3. **Consider Time Zones**: Schedule times are in server's local time

4. **Error Handling**: Always include error recovery
   ```basic
   SET SCHEDULE "every hour"
   
   TRY
       PROCESS_DATA()
   CATCH
       PRINT "Schedule failed: " + ERROR_MESSAGE
       SEND MAIL "admin@example.com", "Schedule Error", ERROR_DETAILS
   END TRY
   ```

5. **Idempotency**: Make scheduled tasks safe to re-run
   ```basic
   last_run = GET BOT MEMORY "last_process_time"
   IF DATEDIFF("minute", last_run, NOW()) > 55 THEN
       PROCESS()
       SET BOT MEMORY "last_process_time", NOW()
   END IF
   ```

## Cancel Schedule

Schedules are automatically canceled when `SET SCHEDULE` is removed from the `.bas` file. Simply delete or comment out the line:

```basic
' SET SCHEDULE "every hour"   ' Commented out = disabled
```

## Limitations

- Maximum 100 scheduled tasks per bot
- Minimum interval: 1 minute
- Scripts timeout after 5 minutes by default
- Time zone is server's local time

## Monitoring

Scheduled tasks are logged automatically:
- Execution start/end times
- Success/failure status
- Error messages if any
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
- Uses a fast rule-based natural language parser (no LLM required)
- Falls back to raw cron if input is already in cron format
- Validates expressions before saving
- Integrates with system scheduler
- Persists schedules in database
- Handles concurrent execution
- Provides retry logic for failures