# Database Relationships

This document describes the relationships between tables in the General Bots database schema.

## Entity Relationship Overview

The database follows a hierarchical structure with organizations at the top, containing bots, which in turn manage users, sessions, and content.

## Primary Relationships

### Organization Hierarchy

```
organizations
    bots (1:N)
        bot_configuration (1:N)
        bot_memories (1:N)
        kb_collections (1:N)
            kb_documents (1:N)
        basic_tools (1:N)
        system_automations (1:N)
```

Each organization can have multiple bots, and each bot has its own configuration, memories, knowledge bases, tools, and automations. Cascade delete behavior means that deleting an organization removes all associated bots and their data.

### User and Session Management

```
users
    user_sessions (1:N)
        message_history (1:N)
        clicks (1:N)
        user_kb_associations (1:N)
        session_tool_associations (1:N)
    user_login_tokens (1:N)
    user_preferences (1:1)
    user_email_accounts (1:N)
        email_drafts (1:N)
        email_folders (1:N)
            folder_messages (1:N)
```

Users can have multiple active sessions across different bots. Each session maintains its own message history and associations. Sessions link to both users and bots, forming a many-to-many relationship through the sessions table.

### Bot-User Interaction

```
bots ←→ user_sessions ←→ users
              
    user_sessions:
        message_history
        user_kb_associations → kb_collections
        session_tool_associations → basic_tools
    
    bots:
        kb_collections
        basic_tools
```

Users interact with bots through sessions. Sessions dynamically associate with knowledge bases and tools as needed. Message history preserves the conversation context for continuity across interactions.

## Foreign Key Constraints

### Strong Relationships (CASCADE DELETE)

These relationships enforce referential integrity with cascade deletion.

The organizations to bots relationship means deleting an organization removes all its bots, with `bots.org_id` referencing `organizations.org_id`.

The bots to bot_configuration relationship means deleting a bot removes all its configuration, with `bot_configuration.bot_id` referencing `bots.id`.

The bots to bot_memories relationship means deleting a bot removes all its memories, with `bot_memories.bot_id` referencing `bots.id`.

The user_sessions to message_history relationship means ending a session removes its message history, with `message_history.session_id` referencing `user_sessions.id`.

### Weak Relationships (SET NULL/RESTRICT)

These relationships maintain data integrity without cascade deletion.

The users to user_sessions relationship sets `session.user_id` to NULL when a user is deleted, preserving conversation history for audit purposes while making the session anonymous.

The kb_collections to kb_documents relationship restricts deletion if documents exist, requiring explicit document deletion first to prevent accidental data loss.

The user_email_accounts to email_drafts relationship preserves drafts when an email account is deleted, allowing draft recovery or reassignment to other accounts.

## Many-to-Many Relationships

### Sessions ↔ Knowledge Bases

```
user_sessions ←→ user_kb_associations ←→ kb_collections
```

The `user_kb_associations` junction table allows dynamic KB activation per session. Multiple knowledge bases can be active simultaneously, enabling conversations that draw from several information sources.

### Sessions ↔ Tools

```
user_sessions ←→ session_tool_associations ←→ basic_tools
```

The `session_tool_associations` junction table enables tools to be loaded per session as needed. This supports dynamic tool discovery where available capabilities vary based on context.

## Relationship Cardinality

One-to-one relationships exist between users and user_preferences, where each user has exactly one preferences record.

One-to-many relationships include organizations to bots, bots to bot_configuration, bots to kb_collections, kb_collections to kb_documents, users to user_sessions, user_sessions to message_history, and user_email_accounts to email_drafts.

Many-to-many relationships exist between user_sessions and kb_collections through user_kb_associations, between user_sessions and basic_tools through session_tool_associations, and between users and bots through user_sessions.

## Referential Integrity Rules

### Insert Order

When inserting data, follow this sequence: organizations first, then bots, then bot_configuration. For user data, insert users first, then user_sessions, then message_history. Knowledge base data requires kb_collections before kb_documents. Tools require basic_tools before session_tool_associations.

### Delete Order (reverse of insert)

When deleting data, reverse the insert order: message_history first, then user_sessions, then users. For tools, delete session_tool_associations before basic_tools. For knowledge bases, delete kb_documents before kb_collections. For organizational data, delete bot_configuration, then bots, then organizations.

## Orphan Prevention

### Automatic Cleanup

Sessions expire based on the `expires_at` timestamp. Orphaned associations are cleaned by background jobs that run periodically. Temporary data has TTL settings that trigger automatic removal.

### Manual Cleanup Required

Some data requires manual cleanup. Unused kb_documents should be periodically reviewed and removed. Old message_history should be cleared based on retention policy. Expired user_login_tokens should be purged.

## Performance Implications

### Hot Paths

These relationships are frequently traversed and should be optimized.

The user_sessions to message_history path benefits from an index on `(session_id, created_at DESC)` and is used for conversation display.

The bots to bot_memories path benefits from an index on `(bot_id, key)` and is used by GET BOT MEMORY and SET BOT MEMORY operations.

The kb_collections to kb_documents path benefits from an index on `(collection_id, indexed)` and is used for semantic search.

### Join Optimization

Common join patterns benefit from composite indexes.

User session context queries join user_sessions with users on `user_sessions.user_id = users.id` and with bots on `user_sessions.bot_id = bots.id`.

Knowledge base loading joins user_kb_associations with kb_collections on `user_kb_associations.collection_id = kb_collections.id` and kb_documents on `kb_collections.id = kb_documents.collection_id`.

Tool discovery joins session_tool_associations with basic_tools on `session_tool_associations.tool_id = basic_tools.id` filtered by session_id and bot_id.

## Data Consistency Patterns

### Transaction Boundaries

Certain operations must be atomic.

Session creation requires inserting the user_session record, initializing default associations, and creating the initial message all within a single transaction.

Tool registration requires inserting the basic_tool record, updating bot_configuration, and refreshing active sessions together.

Document upload requires inserting the kb_document record, triggering the indexing job, and updating collection metadata atomically.

### Eventual Consistency

Some operations can be eventually consistent.

Vector embeddings allow document upload to complete first, with asynchronous indexing creating embeddings afterward. Search becomes available after processing completes.

Email synchronization saves account configuration immediately, then background sync fetches emails asynchronously. Folders and counts update as sync progresses.

## Best Practices

Always use foreign keys for data integrity to catch relationship violations at the database level. Index foreign key columns for join performance to avoid full table scans on relationship traversals. Use transactions for related updates to maintain consistency across multiple tables.

Implement soft deletes for audit trails where regulations require historical data retention. Monitor constraint violations in logs to catch application bugs early. Plan cascade paths carefully to avoid unintended data deletion.

Document relationship changes in migrations so the team understands schema evolution over time.