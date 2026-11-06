# Reminder Template (reminder.gbai)

A General Bots template for managing personal and team reminders with multi-channel notifications.

## Overview

The Reminder template provides a complete reminder management system with natural language scheduling, multiple notification channels, and snooze capabilities. Users can create, view, manage, and receive reminders through conversational AI.

## Features

- **Natural Language Scheduling** - Create reminders using everyday language
- **Multi-Channel Notifications** - Email, SMS, or chat notifications
- **Reminder Management** - List, snooze, and delete reminders
- **Scheduled Execution** - Background job checks and sends due reminders
- **Smart Date Parsing** - Understands "tomorrow", "next week", "in 2 hours"
- **Persistent Storage** - Reminders saved to CSV for reliability

## Package Structure

```
reminder.gbai/
├── README.md
├── reminder.gbdata/           # Data storage
│   └── reminders.csv          # Reminder records
├── reminder.gbdialog/
│   ├── start.bas              # Main entry and tool registration
│   ├── add-reminder.bas       # Create new reminders
│   └── reminder.bas           # Scheduled job to send reminders
└── reminder.gbot/
    └── config.csv             # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Registers tools, sets up suggestions, configures context |
| `add-reminder.bas` | Creates new reminder records with notifications |
| `reminder.bas` | Scheduled job that checks and sends due reminders |

## Available Tools

| Tool | Description |
|------|-------------|
| `add-reminder` | Create a new reminder |
| `list-reminders` | View all pending reminders |
| `delete-reminder` | Remove a reminder |
| `snooze-reminder` | Postpone a reminder |

## Data Schema

### Reminders Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier (REM-YYYYMMDD-XXXX) |
| `remindAt` | DateTime | When to send the reminder |
| `message` | String | Reminder content/subject |
| `notifyBy` | String | Notification channel (email, sms, chat) |
| `email` | Email | User's email address |
| `phone` | Phone | User's phone number |
| `created` | DateTime | Creation timestamp |
| `status` | String | pending, sent, snoozed, deleted |

## Usage

### Creating Reminders

**Natural Language:**
```
User: Remind me to call John tomorrow at 9am
Bot: ✅ Reminder set: Call John
     When: tomorrow at 9am
     Notification: chat
```

**With Parameters:**
```
User: Set a reminder
Bot: When should I remind you?
User: Next Monday at 2pm
Bot: What should I remind you about?
User: Submit the quarterly report
Bot: How should I notify you? (email, sms, or chat)
User: email
Bot: ✅ Reminder set: Submit the quarterly report
     When: Next Monday at 2pm
     Notification: email
```

### Viewing Reminders

```
User: Show my reminders
Bot: 📋 Your Reminders:

     1. Call John
        📅 Tomorrow at 9:00 AM
        🔔 chat
        
     2. Submit quarterly report
        📅 Monday, Jan 22 at 2:00 PM
        🔔 email
        
     3. Team meeting prep
        📅 Friday at 3:30 PM
        🔔 chat
```

### Managing Reminders

**Snooze:**
```
User: Snooze my call reminder for 1 hour
Bot: ✅ Reminder snoozed. New time: Today at 10:00 AM
```

**Delete:**
```
User: Delete the quarterly report reminder
Bot: ✅ Reminder deleted: Submit the quarterly report
```

## Scheduled Reminder Delivery

The `reminder.bas` script runs on a schedule to check and send due reminders:

```basic
REM SET SCHEDULER "1 * * * * "

data = FIND "reminder.csv", "when=" + hour

IF data THEN
    TALK TO admin, data.subject
END IF
```

### Notification Channels

| Channel | Delivery Method |
|---------|-----------------|
| `chat` | Message in bot conversation |
| `email` | Email to user's address |
| `sms` | SMS to user's phone |

## Configuration

Configure in `reminder.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Default Notification` | Default channel | `chat` |
| `Snooze Duration` | Default snooze time | `15` (minutes) |
| `Check Interval` | How often to check | `1` (minute) |
| `Timezone` | User timezone | `America/New_York` |
| `Max Reminders` | Limit per user | `100` |

## Customization

### Custom Notification Channels

Add new notification types:

```basic
' In add-reminder.bas
SWITCH notify
    CASE "chat"
        ' Default chat notification
    CASE "email"
        SEND EMAIL email, "Reminder: " + subject, message
    CASE "sms"
        SEND SMS phone, "Reminder: " + subject
    CASE "slack"
        POST "https://hooks.slack.com/...", {"text": "Reminder: " + subject}
    CASE "teams"
        POST "https://outlook.office.com/webhook/...", {"text": subject}
END SWITCH
```

### Recurring Reminders

Add support for recurring reminders:

```basic
' add-recurring-reminder.bas
PARAM subject AS STRING DESCRIPTION "What to remind about"
PARAM schedule AS STRING LIKE "daily" DESCRIPTION "Frequency: daily, weekly, monthly"
PARAM time AS STRING LIKE "9:00 AM" DESCRIPTION "Time of day"

DESCRIPTION "Create a recurring reminder"

SET SCHEDULE cron_expression, "send-recurring.bas"

WITH reminder
    id = "REC-" + FORMAT(GUID())
    message = subject
    frequency = schedule
    remindTime = time
    status = "active"
END WITH

SAVE "recurring_reminders.csv", reminder
```

### Priority Levels

Add priority support:

```basic
PARAM priority AS STRING LIKE "high" DESCRIPTION "Priority: low, medium, high" OPTIONAL

IF priority = "high" THEN
    ' Send via multiple channels
    SEND EMAIL email, "🔴 URGENT: " + subject, message
    SEND SMS phone, "URGENT: " + subject
END IF
```

### Smart Suggestions

Add context-aware suggestions based on time:

```basic
hour = HOUR(NOW())

IF hour < 12 THEN
    ADD SUGGESTION "today" AS "Remind me today"
    ADD SUGGESTION "afternoon" AS "This afternoon"
ELSE
    ADD SUGGESTION "tomorrow" AS "Tomorrow morning"
    ADD SUGGESTION "evening" AS "This evening"
END IF

ADD SUGGESTION "week" AS "Next week"
ADD SUGGESTION "list" AS "My reminders"
```

## Integration Examples

### With Calendar

```basic
' Sync reminder to calendar
IF reminder.notifyBy = "calendar" THEN
    CREATE CALENDAR EVENT reminder.message, reminder.remindAt, 15
END IF
```

### With Tasks

```basic
' Convert reminder to task when due
IF reminder.status = "sent" THEN
    CREATE TASK reminder.message, "medium", user_email
END IF
```

### With CRM

```basic
' Add follow-up reminder from CRM
PARAM contact_id AS STRING DESCRIPTION "Contact to follow up with"
PARAM days AS INTEGER LIKE 7 DESCRIPTION "Days until follow-up"

contact = FIND "contacts.csv", "id = " + contact_id

WITH reminder
    id = FORMAT(GUID())
    message = "Follow up with " + contact.name
    remindAt = DATEADD(NOW(), days, "day")
    notifyBy = "chat"
    relatedTo = contact_id
END WITH

SAVE "reminders.csv", reminder
```

## Example Conversations

### Quick Reminder

```
User: remind me in 30 minutes to take a break
Bot: ✅ Reminder set: Take a break
     When: in 30 minutes (2:45 PM)
     Notification: chat
```

### Work Reminder

```
User: I need to remember to review the PR before EOD
Bot: I'll set a reminder. What time should I remind you?
User: 4:30pm
Bot: ✅ Reminder set: Review the PR before EOD
     When: Today at 4:30 PM
     Notification: chat
```

### Today's Reminders

```
User: What's on my plate today?
Bot: 📋 Today's Reminders:

     ⏰ 10:00 AM - Team standup
     ⏰ 2:00 PM - Client call
     ⏰ 4:30 PM - Review the PR
     
     You have 3 reminders for today.
```

## Best Practices

1. **Be specific** - Include enough detail in reminder messages
2. **Set appropriate times** - Don't set reminders for odd hours
3. **Use the right channel** - Critical reminders via multiple channels
4. **Clean up** - Delete completed reminders regularly
5. **Review regularly** - Check reminder list to stay organized
6. **Test notifications** - Verify each channel works before relying on it

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Reminder not sent | Scheduler not running | Verify cron job is active |
| Wrong time | Timezone mismatch | Configure correct timezone |
| No notification | Missing contact info | Ensure email/phone is set |
| Duplicate reminders | Created multiple times | Check for existing before adding |
| Past date accepted | No validation | Add date validation logic |

## Date Parsing Examples

The LLM understands various date formats:

| Input | Parsed As |
|-------|-----------|
| "tomorrow" | Next day, 9:00 AM |
| "tomorrow at 3pm" | Next day, 3:00 PM |
| "in 2 hours" | Current time + 2 hours |
| "next Monday" | Coming Monday, 9:00 AM |
| "end of day" | Today, 5:00 PM |
| "next week" | 7 days from now |
| "January 15" | Jan 15, current year |
| "1/15 at noon" | Jan 15, 12:00 PM |

## Related Templates

- `office.gbai` - Office productivity with task management
- `crm.gbai` - CRM with follow-up reminders
- `contacts.gbai` - Contact management with activity tracking
- `marketing.gbai` - Marketing with scheduled broadcasts

## Use Cases

- **Personal Productivity** - Don't forget important tasks
- **Team Coordination** - Remind team members of deadlines
- **Customer Follow-ups** - Schedule sales and support follow-ups
- **Meeting Prep** - Get reminded before meetings
- **Health & Wellness** - Regular break and wellness reminders

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots