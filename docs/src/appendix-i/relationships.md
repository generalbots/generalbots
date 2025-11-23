# Database Relationships

This document describes the relationships between tables in the BotServer database schema.

## Entity Relationship Overview

The database follows a hierarchical structure with organizations at the top, containing bots, which in turn manage users, sessions, and content.

## Primary Relationships

### Organization Hierarchy

```
organizations
    ├── bots (1:N)
        ├── bot_configuration (1:N)
        ├── bot_memories (1:N)
        ├── kb_collections (1:N)
        │   └── kb_documents (1:N)
        ├── basic_tools (1:N)
        └── system_automations (1:N)
```

**Key Relationships:**
- Each organization can have multiple bots
- Each bot has its own configuration, memories, knowledge bases, tools, and automations
- Cascade delete: Deleting an organization removes all associated bots and their data

### User and Session Management

```
users
    ├── user_sessions (1:N)
    │   ├── message_history (1:N)
    │   ├── clicks (1:N)
    │   ├── user_kb_associations (1:N)
    │   └── session_tool_associations (1:N)
    ├── user_login_tokens (1:N)
    ├── user_preferences (1:1)
    └── user_email_accounts (1:N)
        ├── email_drafts (1:N)
        └── email_folders (1:N)
```

**Key Relationships:**
- Users can have multiple active sessions across different bots
- Each session maintains its own message history and associations
- Sessions link to both users and bots (many-to-many through sessions)

### Bot-User Interaction

```
bots ←→ user_sessions ←→ users
  │           │
  │           ├── message_history
  │           ├── user_kb_associations → kb_collections
  │           └── session_tool_associations → basic_tools
  │
  ├── kb_collections
  └── basic_tools
```

**Key Relationships:**
- Users interact with bots through sessions
- Sessions dynamically associate with knowledge bases and tools
- Message history preserves the conversation context

## Foreign Key Constraints

### Strong Relationships (CASCADE DELETE)

These relationships enforce referential integrity with cascade deletion:

1. **organizations → bots**
   - Deleting an organization removes all its bots
   - `bots.org_id` references `organizations.org_id`

2. **bots → bot_configuration**
   - Deleting a bot removes all its configuration
   - `bot_configuration.bot_id` references `bots.id`

3. **bots → bot_memories**
   - Deleting a bot removes all its memories
   - `bot_memories.bot_id` references `bots.id`

4. **user_sessions → message_history**
   - Ending a session removes its message history
   - `message_history.session_id` references `user_sessions.id`

### Weak Relationships (SET NULL/RESTRICT)

These relationships maintain data integrity without cascade deletion:

1. **users → user_sessions**
   - Deleting a user sets session.user_id to NULL (anonymous)
   - Preserves conversation history for audit

2. **kb_collections → kb_documents**
   - Deleting a collection restricts if documents exist
   - Requires explicit document deletion first

3. **user_email_accounts → email_drafts**
   - Deleting an email account preserves drafts
   - Allows draft recovery or reassignment

## Many-to-Many Relationships

### Sessions ↔ Knowledge Bases

```
user_sessions ←→ user_kb_associations ←→ kb_collections
```

- Junction table: `user_kb_associations`
- Allows dynamic KB activation per session
- Multiple KBs can be active simultaneously

### Sessions ↔ Tools

```
user_sessions ←→ session_tool_associations ←→ basic_tools
```

- Junction table: `session_tool_associations`
- Tools are loaded per session as needed
- Supports dynamic tool discovery

## Relationship Cardinality

### One-to-One (1:1)
- users : user_preferences
- Each user has exactly one preferences record

### One-to-Many (1:N)
- organizations : bots
- bots : bot_configuration
- bots : kb_collections
- kb_collections : kb_documents
- users : user_sessions
- user_sessions : message_history
- user_email_accounts : email_drafts

### Many-to-Many (M:N)
- user_sessions : kb_collections (through user_kb_associations)
- user_sessions : basic_tools (through session_tool_associations)
- users : bots (through user_sessions)

## Referential Integrity Rules

### Insert Order
1. organizations → bots → bot_configuration
2. users → user_sessions → message_history
3. kb_collections → kb_documents
4. basic_tools → session_tool_associations

### Delete Order (reverse of insert)
1. message_history → user_sessions → users
2. session_tool_associations → basic_tools
3. kb_documents → kb_collections
4. bot_configuration → bots → organizations

## Orphan Prevention

### Automatic Cleanup
- Sessions expire based on `expires_at` timestamp
- Orphaned associations cleaned by background jobs
- Temporary data has TTL settings

### Manual Cleanup Required
- Unused kb_documents
- Old message_history (based on retention policy)
- Expired user_login_tokens

## Performance Implications

### Hot Paths
These relationships are frequently traversed and should be optimized:

1. **user_sessions → message_history**
   - Index: (session_id, created_at DESC)
   - Used for conversation display

2. **bots → bot_memories**
   - Index: (bot_id, key)
   - Used by GET_BOT_MEMORY/SET_BOT_MEMORY

3. **kb_collections → kb_documents**
   - Index: (collection_id, indexed)
   - Used for semantic search

### Join Optimization
Common join patterns that benefit from composite indexes:

1. **User Session Context**
   ```sql
   user_sessions 
   JOIN users ON user_sessions.user_id = users.id
   JOIN bots ON user_sessions.bot_id = bots.id
   ```

2. **Knowledge Base Loading**
   ```sql
   user_kb_associations
   JOIN kb_collections ON user_kb_associations.collection_id = kb_collections.id
   JOIN kb_documents ON kb_collections.id = kb_documents.collection_id
   ```

3. **Tool Discovery**
   ```sql
   session_tool_associations
   JOIN basic_tools ON session_tool_associations.tool_id = basic_tools.id
   WHERE session_id = ? AND basic_tools.bot_id = ?
   ```

## Data Consistency Patterns

### Transaction Boundaries
Operations that must be atomic:

1. **Session Creation**
   - Insert user_session
   - Initialize default associations
   - Create initial message

2. **Tool Registration**
   - Insert basic_tool
   - Update bot_configuration
   - Refresh active sessions

3. **Document Upload**
   - Insert kb_document
   - Trigger indexing job
   - Update collection metadata

### Eventual Consistency
Operations that can be eventually consistent:

1. **Vector Embeddings**
   - Document upload completes
   - Async indexing creates embeddings
   - Search available after processing

2. **Email Sync**
   - Account configuration saved
   - Background sync fetches emails
   - Folders and counts update async

## Best Practices

1. **Always use foreign keys** for data integrity
2. **Index foreign key columns** for join performance
3. **Use transactions** for related updates
4. **Implement soft deletes** for audit trails
5. **Monitor constraint violations** in logs
6. **Plan cascade paths** carefully
7. **Document relationship changes** in migrations