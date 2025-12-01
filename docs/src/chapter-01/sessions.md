# Sessions and Channels

Every conversation has memory. Sessions are the beating heart of BotServer because they remember who you are, what you have said, and where you left off. Even if you close your browser and come back tomorrow, your conversation continues right where it paused.

<img src="../assets/chapter-01/session-manager.svg" alt="Session Manager" style="max-height: 400px; width: 100%; object-fit: contain;">


## What Is a Session?

A session is a persistent conversation container that tracks everything about an ongoing interaction. This includes who is talking through user identity, what has been said through message history, the current state including variables and context, any active tools and knowledge bases, and the bot configuration in use. Think of it like a phone call that can pause and resume anytime without losing the thread of conversation.


## How Sessions Start

### UI Interface

When a user opens `http://localhost:8080`, the browser receives a session token in the form of a UUID. This token is stored in localStorage for persistence across page loads. The session itself is created in PostgreSQL for durability and cached for fast access during active conversations.

### API Access

Programmatic access to sessions uses the REST API. A POST request to `/api/session` returns a session ID and secret token. Subsequent requests include the token in the Authorization header as a Bearer token to maintain the session context.

```bash
# Get new session
curl -X POST http://localhost:8080/api/session
# Returns: {"session_id": "uuid-here", "token": "secret-token"}

# Use session
curl -H "Authorization: Bearer secret-token" \
     http://localhost:8080/api/chat
```

### Anonymous vs Authenticated

Sessions come in two flavors depending on user identity. Anonymous sessions are auto-created with temporary identities for users who have not logged in. Authenticated sessions link to a user account and maintain permanent history that persists indefinitely.


## Session Lifecycle

<img src="../assets/chapter-01/session-states.svg" alt="Session States" style="max-height: 300px; width: 100%; object-fit: contain;">

Sessions move through several states during their existence. Active sessions have no timeout while the user is actively chatting. Idle sessions timeout after 30 minutes by default, though this is configurable. Expired sessions are removed after 7 days for anonymous users, while authenticated sessions never expire automatically.


## What Gets Stored

### PostgreSQL (Permanent Storage)

The database stores the authoritative session record. The sessions table tracks the unique ID, optional user reference, which bot is being used, creation timestamp, and last activity time. The messages table stores each message with its session reference, role (user, assistant, or system), content, and timestamp. The session_state table holds variables as JSONB data and tracks the current knowledge base and tool context.

### Cache (Fast Access)

The cache layer provides rapid access to active session data. Recent messages, current variables, active knowledge bases and tools, and last activity timestamps are all cached under keys prefixed with the session UUID. This caching ensures responsive conversations without constant database queries.


## Session Variables

Variables set in BASIC scripts persist across messages automatically. When you store a variable in one message, you can retrieve it in a later message whether that is minutes or days later.

```basic
' First message
name = HEAR
SET user_name = name

' Later message (minutes or days later)
GET user_name
TALK "Welcome back, " + user_name
```

Storage happens automatically through several layers. Writes go to cache immediately for fast access. Every message triggers persistence to PostgreSQL for durability. If the cache misses, data restores automatically from the database.


## Context Management

Each session maintains its own isolated context. When one session loads a knowledge base, other sessions remain unaffected. This isolation ensures users see only the information relevant to their conversation.

```basic
' Session A
USE KB "policies"
' Only this session sees policies

' Session B (different user)
USE KB "products"  
' This session only sees products
```

Session contexts include active knowledge bases, loaded tools, LLM configuration overrides, and custom prompts. All of these are scoped to the individual session and do not leak between users.


## Multi-Bot Sessions

Different bots create entirely separate sessions. A user visiting `/default` gets one session connected to the default bot, while visiting `/support` creates a different session for the support bot. Each bot session is completely independent with its own conversation history, knowledge bases, configuration, and no data sharing between them.


## Session Security

### Token Generation

Session tokens use cryptographically secure random generation with 256-bit entropy. Tokens are encoded in URL-safe base64 format and are unique per session. This makes tokens effectively impossible to guess or predict.

### Token Validation

Every request undergoes validation to ensure security. The system verifies that the token exists, has not expired, matches the claimed session, and that the session is still active. Any failure in this chain rejects the request.

### Security Features

Multiple security measures protect sessions. Unguessable tokens prevent session hijacking. New tokens for each session prevent session fixation attacks. Automatic cleanup removes old sessions to prevent accumulation. Rate limiting per session prevents abuse.


## Debugging Sessions

### View Current Session

Within a BASIC script, you can access session information directly.

```basic
session_id = GET "session.id"
TALK "Session: " + session_id
```

### Database Inspection

Direct database queries help debug session issues. You can find all active sessions by querying for recent activity, or view message history for a specific session ordered by timestamp.

### Cache Inspection

The cache contents can be examined using the valkey-cli tool. List all session keys or retrieve specific session data like variables or context directly from the cache.


## Session Limits

Default limits control resource usage, though all are configurable. Message history keeps the last 50 messages in context. Variable storage allows up to 1MB per session. File uploads accept up to 10MB per file. Each server handles up to 1000 concurrent sessions. Rate limiting restricts each session to 60 messages per minute.


## Advanced Features

### Session Persistence

Sessions persist across server restarts through the cache and database layers. When users reconnect after a restart, their session state restores automatically. This happens transparently without any action required from users or bot developers.

### Session Context Isolation

Each session maintains its own context for knowledge base and tool usage. When you load a knowledge base or enable a tool, the change affects only the current session. Other users in other sessions remain unaffected by your context changes.

```basic
' Each session has isolated context
USE KB "docs"
' Only affects current session
```


## How It Works Automatically

Sessions require zero configuration from bot developers. Creation happens automatically on the first request from any client. Storage to database and cache happens automatically as conversations progress. Cleanup runs automatically after sessions expire. Security through token generation happens automatically without any setup. Multi-channel support through automatic adapter selection means the same session infrastructure works across all platforms.

You never need to manage sessions directly. Just use the conversation keywords like TALK, HEAR, SET, and GET. Everything else happens behind the scenes.


## Common Patterns

### Welcome Back

Personalize greetings by remembering when users last visited. Store the last visit timestamp and check for it on subsequent sessions to customize the welcome message.

```basic
last_visit = GET BOT MEMORY "last_visit_" + session_id
IF last_visit THEN
  TALK "Welcome back! Last seen: " + last_visit
ELSE
  TALK "Welcome! First time here?"
END IF
SET BOT MEMORY "last_visit_" + session_id, NOW()
```

### Progressive Disclosure

Reveal more features as users become more engaged by tracking message count and adjusting guidance accordingly.

```basic
msg_count = GET "session.message_count"
IF msg_count < 3 THEN
  TALK "I can help with basic questions"
ELSE IF msg_count < 10 THEN
  TALK "Try our advanced features!"
ELSE
  TALK "You're a power user! Check tools menu"
END IF
```

### Multi-User Support

Each user automatically receives their own isolated session. The system handles user separation without any explicit code required. Simply write your dialog logic and trust that each user's data remains private to their session.


## Troubleshooting

If sessions are not persisting, check that PostgreSQL is running and accessible. Verify that the cache server is reachable. Look for disk space issues that might prevent database writes.

If sessions expire too soon, adjust the timeout setting in config.csv. Check that server clocks are synchronized. Monitor for memory pressure that might cause early cache eviction.

If you cannot resume a session, the token might have become invalid through expiration or corruption. The session could have passed its expiration window. Database connection issues can also prevent session restoration.


## Write Once, Run Everywhere

The same BASIC script runs across all channels including the UI interface, mobile apps, WhatsApp, Microsoft Teams, email conversations, and voice assistants. Your investment in dialog development pays off everywhere because each channel adapter handles the platform specifics while you focus on conversation logic.

```basic
' This same script works everywhere

TALK "Hello! How can I help?"
answer = HEAR
TALK "I understand you need help with: " + answer
```


## Summary

Sessions and channels work together seamlessly in BotServer. Sessions handle state management automatically across any channel, persist data reliably through cache and database layers, and scale efficiently to thousands of concurrent conversations. You focus on writing the conversation flow while the system handles memory management and multi-channel delivery transparently.