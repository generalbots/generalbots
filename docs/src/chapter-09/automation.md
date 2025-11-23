# Automation

BotServer provides automation capabilities through scheduled tasks and event triggers, allowing bots to perform actions automatically without user interaction.

## Overview

Automation in BotServer is implemented through:
- **SET_SCHEDULE**: Cron-based task scheduling
- **ON Triggers**: Event-driven automation (database triggers)
- **System Automations**: Background task execution

## Scheduled Tasks with SET_SCHEDULE

### Basic Usage

Schedule a BASIC script to run periodically using cron expressions:

```basic
SET_SCHEDULE "0 9 * * *"  # Daily at 9:00 AM

# Rest of the script executes on schedule
let data = GET "reports/daily.csv"
let summary = LLM "Summarize: " + data
SET_BOT_MEMORY "daily_summary", summary
```

### Cron Expression Format

The cron format follows standard Unix conventions:
```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

Common patterns:
- `"0 * * * *"` - Every hour
- `"*/30 * * * *"` - Every 30 minutes
- `"0 9 * * 1-5"` - Weekdays at 9 AM
- `"0 0 1 * *"` - First day of month at midnight

### How Scheduling Works

1. **Script Detection**: Compiler finds SET_SCHEDULE in .bas files
2. **Database Registration**: Schedule stored in system_automations table
3. **Cron Execution**: Background service runs scripts on schedule
4. **Script Context**: Runs with bot's full context and permissions

### Example: Daily Report Generation

```basic
# daily-report.bas
SET_SCHEDULE "0 8 * * *"  # Every day at 8 AM

let yesterday = FORMAT(NOW() - 86400, "YYYY-MM-DD")
let data = GET "data/sales-" + yesterday + ".json"

let analysis = LLM "Analyze yesterday's sales: " + data
SET_BOT_MEMORY "latest_report", analysis

# Optional: Send notification
TALK "Daily report ready: " + analysis
```

## Event Triggers with ON

The ON keyword creates database triggers for events:

```basic
ON "user_registration" {
    # Code to execute when event occurs
    TALK "Welcome new user!"
}
```

### Trigger Types

Currently supported trigger kinds:
- **Scheduled**: Cron-based execution
- **Database Events**: Table insert/update triggers

### Implementation Details

Triggers are stored in the system_automations table with:
- `kind`: Type of trigger (TriggerKind enum)
- `target`: Target table or resource
- `param`: Additional parameters (script name for schedules)
- `bot_id`: Associated bot
- `is_active`: Enable/disable flag

## System Automations Table

The `system_automations` table manages all automation rules:

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| bot_id | UUID | Bot that owns the automation |
| kind | INT | TriggerKind (0=Scheduled, etc.) |
| schedule | TEXT | Cron expression for scheduled tasks |
| param | TEXT | Script name or parameters |
| is_active | BOOL | Whether automation is enabled |
| last_triggered | TIMESTAMP | Last execution time |

## Automation Lifecycle

### Creation

1. Script with SET_SCHEDULE compiled
2. Schedule extracted during preprocessing
3. Entry created/updated in system_automations
4. Background scheduler picks up new schedule

### Execution

1. Scheduler checks for due tasks
2. Loads bot context
3. Executes BASIC script
4. Updates last_triggered timestamp
5. Logs execution result

### Modification

When a script changes:
1. Old schedule deleted if SET_SCHEDULE removed
2. New schedule registered if SET_SCHEDULE added
3. Schedule updated if cron expression changed

### Deletion

Automations removed when:
- Script deleted
- SET_SCHEDULE line removed
- Bot deleted (cascade)

## Use Cases

### Content Updates

```basic
# update-news.bas
SET_SCHEDULE "0 */6 * * *"  # Every 6 hours

let news = GET "https://api.news.com/latest"
let summary = LLM "Summarize top stories: " + news
SET_BOT_MEMORY "latest_news", summary
```

### Data Processing

```basic
# process-orders.bas
SET_SCHEDULE "*/15 * * * *"  # Every 15 minutes

let pending = GET "orders/pending.json"
# Process pending orders
SET_BOT_MEMORY "last_check", NOW()
```

### Maintenance Tasks

```basic
# cleanup.bas
SET_SCHEDULE "0 2 * * *"  # Daily at 2 AM

# Clean old data
let cutoff = NOW() - 2592000  # 30 days ago
# Cleanup logic here
```

## Best Practices

1. **Schedule Appropriately**: Don't run heavy tasks too frequently
2. **Handle Failures**: Include error handling in scheduled scripts
3. **Log Actions**: Track what automated tasks do
4. **Test First**: Run scripts manually before scheduling
5. **Monitor Execution**: Check logs for automation failures
6. **Use Bot Memory**: Store state between executions
7. **Avoid Conflicts**: Don't schedule conflicting tasks at same time

## Limitations

- No sub-minute scheduling (minimum 1 minute)
- Scripts have timeout limits
- No dependency management between tasks
- Limited to cron expressions (no complex calendaring)
- No built-in retry on failure
- Single execution (no parallel runs of same script)

## Monitoring

### Checking Active Schedules

Active automations are stored in the database and can be queried:
```sql
SELECT * FROM system_automations 
WHERE bot_id = ? AND kind = 0 AND is_active = true;
```

### Execution Logs

Automation execution is logged:
- Success: INFO level logs
- Failure: ERROR level logs
- Schedule changes: DEBUG logs

### Debugging Failed Automations

1. Check system logs for error messages
2. Verify cron expression is valid
3. Test script manually
4. Check bot has necessary permissions
5. Verify external resources are accessible

## Security Considerations

- Scheduled tasks run with bot's permissions
- Cannot access other bots' data
- API credentials should use BOT_MEMORY
- Rate limiting applies to automated tasks
- Monitor for runaway automations

## Future Enhancements

Potential improvements to automation:
- Event-driven triggers beyond scheduling
- Webhook integration
- Task dependencies and workflows
- Retry policies
- Distributed execution
- More trigger types (file changes, API events)

## Summary

BotServer's automation features enable bots to perform scheduled and event-driven tasks without user interaction. Through SET_SCHEDULE and system_automations, bots can maintain fresh content, process data regularly, and respond to events automatically, making them more proactive and useful.