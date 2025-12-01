# User Context vs System Context

This chapter explains the two execution contexts in General Bots: User Context and System Context. Understanding these contexts is essential for building secure, properly scoped bot interactions.

## Overview

Every API call and BASIC script execution happens in one of two contexts:

| Context | Identity | Use Case |
|---------|----------|----------|
| **User Context** | Logged-in user | Interactive operations on user's behalf |
| **System Context** | Bot service account | Automated/scheduled operations |

## User Context

### Definition

User Context means the operation is performed **as** the authenticated user, using their identity and permissions.

### Characteristics

- **Identity**: The logged-in user's ID
- **Permissions**: Limited to what the user can access
- **Scope**: Only user's own resources
- **Token**: User's OAuth access token

### When User Context Applies

1. **Interactive Chat**: User sends a message
2. **File Operations**: User uploads/downloads files
3. **Email Access**: User reads their inbox
4. **Calendar**: User views their schedule
5. **Tasks**: User manages their task list

### Example Flow

```
User logs in → OAuth token issued → User asks bot to send email
                                           ↓
                               Bot sends email AS the user
                                           ↓
                               Email "From:" shows user's address
```

### BASIC Script Example

```basic
' This runs in User Context when triggered by user interaction
' The email is sent from the logged-in user's account

TALK "Who should I email?"
recipient = HEAR

TALK "What's the subject?"
subject = HEAR

TALK "What's the message?"
body = HEAR

SEND MAIL recipient, subject, body
TALK "Email sent from your account to " + recipient
```

### Access Boundaries

In User Context, the bot can only access:

| Resource | Access Level |
|----------|--------------|
| Files | User's files and shared files |
| Email | User's mailbox only |
| Calendar | User's calendar only |
| Tasks | User's tasks only |
| Contacts | User's contacts |
| Meet | Meetings user is invited to |

## System Context

### Definition

System Context means the operation is performed **by** the bot system itself, using a service account with elevated permissions.

### Characteristics

- **Identity**: Bot's service account
- **Permissions**: Defined by admin configuration
- **Scope**: Cross-user or system-wide resources
- **Token**: Service account credentials

### When System Context Applies

1. **Scheduled Tasks**: Cron-based script execution via SET SCHEDULE
2. **Event Handlers**: ON keyword triggers
3. **Admin Operations**: User management
4. **Analytics**: Cross-user reporting
5. **Backups**: System-wide data export
6. **Bot-Initiated Messages**: Proactive notifications

### Example Flow

```
Schedule triggers at 9:00 AM → System context activated
                                       ↓
                            Bot sends summary to all managers
                                       ↓
                            Email "From:" shows bot's address
```

### BASIC Script Example

```basic
' This runs in System Context (scheduled task)
' The bot sends emails from its own account

SET SCHEDULE "0 9 * * 1"  ' Every Monday at 9 AM

' Bot processes data and sends notifications
summary = LLM "Generate weekly summary"
SEND MAIL "team@example.com", "Weekly Summary", summary

PRINT "Weekly summary sent"
```

### Access Boundaries

In System Context, the bot can access:

| Resource | Access Level |
|----------|--------------|
| Files | All bot storage |
| Email | Send as bot identity |
| Calendar | Bot's calendar, create events |
| Tasks | Create/assign to any user |
| Users | Read user directory |
| Meet | Join any meeting (if configured) |
| Config | Read bot configuration |

## Determining Context

### Automatic Detection

General Bots automatically determines context based on how the script is triggered:

| Trigger | Context |
|---------|---------|
| User sends message | User Context |
| SET SCHEDULE execution | System Context |
| ON event handler | System Context |
| HTTP API with user token | User Context |
| Internal service call | System Context |

### Context in Scripts

The context is determined by the trigger, not by keywords in the script:

```basic
' User-triggered script (User Context)
' - Runs when user interacts
' - Uses user's permissions

name = HEAR "What's your name?"
TALK "Hello, " + name
```

```basic
' Scheduled script (System Context)
' - Runs on schedule
' - Uses bot's permissions

SET SCHEDULE "0 8 * * *"  ' Daily at 8 AM
TALK "Good morning! Here's your daily briefing."
```

## Security Implications

### User Context Security

| Benefit | Consideration |
|---------|---------------|
| Limited blast radius | Cannot access others' data |
| Audit trail to user | User responsible for actions |
| Respects user permissions | May limit bot functionality |

### System Context Security

| Benefit | Consideration |
|---------|---------------|
| Full bot capabilities | Must be carefully controlled |
| Cross-user operations | Audit critical for compliance |
| Scheduled automation | Service account must be secured |

## Configuration

### Service Account Setup

The bot's system identity is managed through the Directory service (Zitadel). Configure in `config.csv`:

```csv
key,value
system-account-email,bot@yourdomain.com
system-context-permissions,files:read|email:send|calendar:write
```

### Context Restrictions

Limit what System Context can do:

```csv
key,value
system-allow-email,true
system-allow-file-delete,false
system-allow-user-create,false
system-allow-config-change,false
```

## Audit Logging

All operations are logged with context:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "context": "user",
  "user_id": "user-123",
  "action": "email:send",
  "resource": "email to client@example.com",
  "result": "success"
}
```

```json
{
  "timestamp": "2024-01-15T09:00:00Z",
  "context": "system",
  "service_account": "bot-service-account",
  "action": "email:send",
  "resource": "weekly-summary to 47 recipients",
  "trigger": "schedule:weekly-summary",
  "result": "success"
}
```

## Best Practices

### Use User Context When

- User initiates the action
- Operation affects only the user
- Audit trail should point to user
- Respecting user permissions is required

### Use System Context When

- Scheduled or automated tasks
- Cross-user operations needed
- Bot needs elevated permissions
- System-wide actions required

### Security Guidelines

1. **Minimize System Context**: Use only when necessary
2. **Audit Everything**: Log all system context operations
3. **Rotate Credentials**: Change service account tokens regularly
4. **Limit Scope**: Grant minimal permissions to service account
5. **Review Access**: Periodically audit system context usage

## Troubleshooting

### "Permission Denied" Errors

Check if the operation is running in the expected context:

- **User-triggered actions** run in User Context with user permissions
- **Scheduled actions** run in System Context with bot permissions

If a scheduled task fails with permission errors, verify the bot's service account has the required permissions in Zitadel.

### Unexpected "From" Address in Emails

The sender depends on context:

- **User Context**: Sends as logged-in user
- **System Context**: Sends as bot account

Ensure your script is triggered in the intended way for the correct sender.

## See Also

- [Permissions Matrix](./permissions-matrix.md) - Full permission reference
- [Bot Authentication](./bot-auth.md) - Service account setup
- [Security Policy](./security-policy.md) - Security guidelines
- [SET SCHEDULE](../chapter-06-gbdialog/keyword-set-schedule.md) - Scheduled execution