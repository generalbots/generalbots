# Sessions

Understanding how BotServer manages conversational sessions is crucial for building effective bots.

## What is a Session?

A session represents a single conversation between a user and a bot. It maintains:
- User identity
- Conversation state
- Context and memory
- Active knowledge bases
- Loaded tools

## Session Lifecycle

### 1. Session Creation

Sessions are created when:
- A user visits the web interface (cookie-based)
- A message arrives from a messaging channel
- An API call includes a new session ID

```basic
' Sessions start automatically when user connects
' The start.bas script runs for each new session
TALK "Welcome! This is a new session."
```

### 2. Session Persistence

Sessions persist:
- **Web**: Via browser cookies (30-day default)
- **WhatsApp**: Phone number as session ID
- **Teams**: User ID from Microsoft Graph
- **API**: Client-provided session token

### 3. Session Termination

Sessions end when:
- User explicitly ends conversation
- Timeout period expires (configurable)
- Server restarts (optional persistence)
- Memory limit reached

## Session Storage

### Database Tables

Sessions use these primary tables:
- `users`: User profiles and authentication
- `user_sessions`: Active session records
- `conversations`: Message history
- `bot_memories`: Persistent bot data

### Memory Management

Each session maintains:
```
Session Memory
├── User Variables (SET/GET)
├── Context Strings (SET CONTEXT)
├── Active KBs (USE KB)
├── Loaded Tools (USE TOOL)
├── Suggestions (ADD SUGGESTION)
└── Temporary Data
```

## Session Variables

### User Variables
```basic
' Set a variable for this session
SET "user_name", "John"
SET "preference", "email"

' Retrieve variables
name = GET "user_name"
TALK "Hello, " + name
```

### Bot Memory
```basic
' Bot memory persists across all sessions
SET BOT MEMORY "company_name", "ACME Corp"

' Available to all users
company = GET BOT MEMORY "company_name"
```

## Session Context

Context provides information to the LLM:

```basic
' Add context for better responses
SET CONTEXT "user_profile" AS "Premium customer since 2020"
SET CONTEXT "preferences" AS "Prefers technical documentation"

' Context is automatically included in LLM prompts
response = LLM "What products should I recommend?"
```

## Multi-Channel Sessions

### Channel Identification

Sessions track their origin channel:
```basic
channel = GET SESSION "channel"
IF channel = "whatsapp" THEN
    ' WhatsApp-specific features
    ADD SUGGESTION "Call Support" AS "phone"
ELSE IF channel = "web" THEN
    ' Web-specific features
    SHOW IMAGE "dashboard.png"
END IF
```

### Channel-Specific Data

Each channel provides different session data:

| Channel | Session ID | User Info | Metadata |
|---------|------------|-----------|----------|
| Web | Cookie UUID | IP, Browser | Page URL |
| WhatsApp | Phone Number | Name, Profile | Message Type |
| Teams | User ID | Email, Tenant | Organization |
| Email | Email Address | Name | Subject |

## Session Security

### Authentication States

Sessions can be:
- **Anonymous**: No authentication required
- **Authenticated**: User logged in via directory service
- **Elevated**: Additional verification completed

```basic
auth_level = GET SESSION "auth_level"
IF auth_level <> "authenticated" THEN
    TALK "Please log in to continue"
    RUN "auth.bas"
END IF
```

### Session Tokens

Secure token generation:
- UUID v4 for session IDs
- Signed JWTs for API access
- Refresh tokens for long-lived sessions

## Session Limits

### Resource Constraints

| Resource | Default Limit | Configurable |
|----------|--------------|--------------|
| Memory per session | 10MB | Yes |
| Context size | 4096 tokens | Yes |
| Active KBs | 10 | Yes |
| Variables | 100 | Yes |
| Message history | 50 messages | Yes |

### Concurrent Sessions

- Server supports 1000+ concurrent sessions
- Database connection pooling
- Redis caching for performance
- Automatic cleanup of stale sessions

## Session Recovery

### Automatic Recovery

If a session disconnects:
1. State preserved for timeout period
2. User can reconnect with same session ID
3. Conversation continues from last point

```basic
last_message = GET SESSION "last_interaction"
IF last_message <> "" THEN
    TALK "Welcome back! We were discussing: " + last_message
END IF
```

### Manual Save/Restore

```basic
' Save session state
state = SAVE SESSION STATE
SET BOT MEMORY "saved_session_" + user_id, state

' Restore later
saved = GET BOT MEMORY "saved_session_" + user_id
RESTORE SESSION STATE saved
```

## Session Analytics

Track session metrics:
- Duration
- Message count
- User satisfaction
- Completion rate
- Error frequency

```basic
' Log session events
LOG SESSION "milestone", "order_completed"
LOG SESSION "error", "payment_failed"
```

## Best Practices

### 1. Session Initialization
```basic
' start.bas - Initialize every session properly
user_id = GET SESSION "user_id"
IF user_id = "" THEN
    ' First time user
    TALK "Welcome! Let me help you get started."
    RUN "onboarding.bas"
ELSE
    ' Returning user
    TALK "Welcome back!"
END IF
```

### 2. Session Cleanup
```basic
' Clean up before session ends
ON SESSION END
    CLEAR KB ALL
    CLEAR SUGGESTIONS
    LOG "Session ended: " + SESSION_ID
END ON
```

### 3. Session Handoff
```basic
' Transfer session to human agent
FUNCTION HandoffToAgent()
    agent_id = GET AVAILABLE AGENT
    TRANSFER SESSION agent_id
    TALK "Connecting you to an agent..."
END FUNCTION
```

### 4. Session Persistence
```basic
' Save important data beyond session
important_data = GET "order_details"
SET BOT MEMORY "user_" + user_id + "_last_order", important_data
```

## Debugging Sessions

### Session Inspection

View session data:
```basic
' Debug session information
DEBUG SHOW SESSION
DEBUG SHOW CONTEXT
DEBUG SHOW VARIABLES
```

### Session Logs

All sessions are logged:
- Start/end timestamps
- Messages exchanged
- Errors encountered
- Performance metrics

## Advanced Session Features

### Session Branching
```basic
' Create sub-session for specific task
sub_session = CREATE SUB SESSION
RUN IN SESSION sub_session, "specialized_task.bas"
MERGE SESSION sub_session
```

### Session Templates
```basic
' Apply template to session
APPLY SESSION TEMPLATE "support_agent"
' Automatically loads KBs, tools, and context
```

### Cross-Session Communication
```basic
' Send message to another session
SEND TO SESSION other_session_id, "Notification: Your order is ready"
```

## Summary

Sessions are the foundation of conversational state in BotServer. They:
- Maintain conversation continuity
- Store user-specific data
- Manage resources efficiently
- Enable multi-channel support
- Provide security boundaries

Understanding sessions helps you build bots that feel natural, remember context, and provide personalized experiences across any channel.