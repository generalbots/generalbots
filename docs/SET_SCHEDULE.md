# SET SCHEDULE Keyword Reference

> Documentation for scheduling scripts and automations in General Bots

## Overview

SET SCHEDULE has two distinct usages:

1. **File-level scheduler**: When placed at the top of a `.bas` file, defines when the script runs automatically
2. **Runtime scheduling**: When used within code, schedules another script to run at a specific time

## Usage 1: File-Level Scheduler

When SET SCHEDULE appears at the top of a file (before any executable code), it registers the entire script to run on a cron schedule.

### Syntax

```basic
SET SCHEDULE "cron_expression"

' Rest of script follows
TALK "This runs on schedule"
```

### Examples

```basic
' Run every day at 9 AM
SET SCHEDULE "0 9 * * *"

TALK "Good morning! Running daily report..."
stats = AGGREGATE "sales", "SUM", "amount", "date = TODAY()"
SEND TEMPLATE "daily-report", "email", "team@company.com", #{total: stats}
```

```basic
' Run every Monday at 10 AM
SET SCHEDULE "0 10 * * 1"

TALK "Weekly summary starting..."
' Generate weekly report
```

```basic
' Run first day of every month at midnight
SET SCHEDULE "0 0 1 * *"

TALK "Monthly billing cycle..."
' Process monthly billing
```

### Cron Expression Reference

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

| Expression | Description |
|------------|-------------|
| `0 9 * * *` | Every day at 9:00 AM |
| `0 9 * * 1-5` | Weekdays at 9:00 AM |
| `0 */2 * * *` | Every 2 hours |
| `30 8 * * 1` | Mondays at 8:30 AM |
| `0 0 1 * *` | First of each month at midnight |
| `0 12 * * 0` | Sundays at noon |
| `*/15 * * * *` | Every 15 minutes |
| `0 9,17 * * *` | At 9 AM and 5 PM daily |

## Usage 2: Runtime Scheduling

When used within code (not at file top), schedules another script to run at a specified time.

### Syntax

```basic
' Schedule with cron + date
SET SCHEDULE "cron_expression" date, "script.bas"

' Schedule for specific date/time
SET SCHEDULE datetime, "script.bas"
```

### Examples

```basic
' Schedule script to run in 3 days at 9 AM
SET SCHEDULE "0 9 * * *" DATEADD(TODAY(), 3, "day"), "followup-email.bas"
```

```basic
' Schedule for specific datetime
SET SCHEDULE "2025-02-15 10:00", "product-launch.bas"
```

```basic
' Schedule relative to now
SET SCHEDULE DATEADD(NOW(), 1, "hour"), "reminder.bas"
SET SCHEDULE DATEADD(NOW(), 30, "minute"), "check-status.bas"
```

```basic
' Campaign scheduling example
ON FORM SUBMIT "signup"
    ' Welcome email immediately
    SEND TEMPLATE "welcome", "email", fields.email, #{name: fields.name}
    
    ' Schedule follow-up sequence
    SET SCHEDULE DATEADD(NOW(), 2, "day"), "nurture-day-2.bas"
    SET SCHEDULE DATEADD(NOW(), 5, "day"), "nurture-day-5.bas"
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "nurture-day-14.bas"
END ON
```

## Passing Data to Scheduled Scripts

Use SET BOT MEMORY to pass data to scheduled scripts:

```basic
' In the scheduling script
SET BOT MEMORY "scheduled_lead_" + lead_id, lead_email
SET SCHEDULE DATEADD(NOW(), 7, "day"), "followup.bas"

' In followup.bas
PARAM lead_id AS string
lead_email = GET BOT MEMORY "scheduled_lead_" + lead_id
SEND TEMPLATE "followup", "email", lead_email, #{id: lead_id}
```

## Campaign Drip Sequence Example

```basic
' welcome-sequence.bas - File-level scheduler not used here
' This is triggered by form submission

PARAM email AS string
PARAM name AS string

DESCRIPTION "Start welcome email sequence"

' Day 0: Immediate welcome
WITH vars
    .name = name
    .date = TODAY()
END WITH

SEND TEMPLATE "welcome-1", "email", email, vars

' Store lead info for scheduled scripts
SET BOT MEMORY "welcome_" + email + "_name", name

' Schedule remaining emails
SET SCHEDULE DATEADD(NOW(), 2, "day"), "welcome-day-2.bas"
SET SCHEDULE DATEADD(NOW(), 5, "day"), "welcome-day-5.bas"
SET SCHEDULE DATEADD(NOW(), 7, "day"), "welcome-day-7.bas"

TALK "Welcome sequence started for " + email
```

## Daily Report Example

```basic
' daily-sales-report.bas
SET SCHEDULE "0 18 * * 1-5"

' This runs every weekday at 6 PM

today_sales = AGGREGATE "orders", "SUM", "total", "date = TODAY()"
order_count = AGGREGATE "orders", "COUNT", "id", "date = TODAY()"
avg_order = IIF(order_count > 0, today_sales / order_count, 0)

WITH report
    .date = TODAY()
    .total_sales = today_sales
    .order_count = order_count
    .average_order = ROUND(avg_order, 2)
END WITH

SEND TEMPLATE "daily-sales", "email", "sales@company.com", report

TALK "Daily report sent: $" + today_sales
```

## Canceling Scheduled Tasks

Scheduled tasks are stored in `system_automations` table. To cancel:

```basic
' Remove scheduled automation
DELETE "system_automations", "param = 'followup.bas' AND bot_id = '" + bot_id + "'"
```

## Best Practices

1. **Use descriptive script names**: `welcome-day-2.bas` not `email2.bas`
2. **Store context in BOT MEMORY**: Pass lead ID, not entire objects
3. **Check for unsubscribes**: Before sending scheduled emails
4. **Handle errors gracefully**: Scheduled scripts should not crash
5. **Log execution**: Track when scheduled scripts run
6. **Use appropriate times**: Consider timezone and recipient preferences
7. **Avoid overlapping schedules**: Don't schedule too many scripts at same time

## Database Schema

Scheduled tasks are stored in `system_automations`:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Automation ID |
| `bot_id` | UUID | Bot owner |
| `kind` | INT | Trigger type (Scheduled = 1) |
| `schedule` | TEXT | Cron expression |
| `param` | TEXT | Script filename |
| `is_active` | BOOL | Active status |
| `last_triggered` | TIMESTAMP | Last execution time |

## Comparison

| Feature | File-Level | Runtime |
|---------|------------|---------|
| Location | Top of file | Anywhere in code |
| Purpose | Recurring execution | One-time scheduling |
| Timing | Cron-based recurring | Specific date/time |
| Script | Self (current file) | Other script |
| Use case | Reports, cleanup | Drip campaigns, reminders |