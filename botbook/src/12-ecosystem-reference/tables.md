# Database Tables

This section documents all database tables in General Bots, their structures, and purposes.

## Core Tables

### organizations
Stores organization/tenant information for multi-tenant deployments.

| Column | Type | Description |
|--------|------|-------------|
| org_id | UUID | Primary key |
| name | TEXT | Organization name |
| slug | TEXT | URL-friendly identifier |
| created_at | TIMESTAMPTZ | Creation timestamp |

### bots
Bot instances and their basic configuration.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| name | TEXT | Bot name |
| org_id | UUID | Foreign key to organizations |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

### bot_configuration
Stores bot-specific configuration parameters from config.csv.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| bot_id | UUID | Foreign key to bots |
| key | TEXT | Configuration key |
| value | TEXT | Configuration value |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

### bot_memories
Persistent key-value storage for bots (used by GET BOT MEMORY/SET BOT MEMORY).

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| bot_id | UUID | Foreign key to bots |
| key | TEXT | Memory key |
| value | TEXT | Memory value |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

## User Management Tables

### users
User accounts with authentication credentials.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| username | TEXT | Unique username |
| email | TEXT | Email address |
| password_hash | TEXT | Argon2 hashed password |
| active | BOOLEAN | Account status |
| created_at | TIMESTAMPTZ | Registration timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

### user_sessions
Active user sessions for authentication and state management.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| bot_id | UUID | Foreign key to bots |
| session_token | TEXT | Unique session identifier |
| expires_at | TIMESTAMPTZ | Session expiration |
| created_at | TIMESTAMPTZ | Session start |
| updated_at | TIMESTAMPTZ | Last activity |

### user_login_tokens
Authentication tokens for login flows.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| token | TEXT | Login token |
| expires_at | TIMESTAMPTZ | Token expiration |
| used | BOOLEAN | Whether token was used |
| created_at | TIMESTAMPTZ | Token creation |

### user_preferences
User-specific settings and preferences.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| preferences | JSONB | Preferences data |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update |

## Conversation Tables

### message_history
Complete conversation history between users and bots.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| session_id | UUID | Foreign key to user_sessions |
| user_id | UUID | Foreign key to users |
| bot_id | UUID | Foreign key to bots |
| message | TEXT | Message content |
| sender | TEXT | 'user' or 'bot' |
| created_at | TIMESTAMPTZ | Message timestamp |

### clicks
Tracks user interactions with UI elements.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| session_id | UUID | Foreign key to user_sessions |
| element_id | TEXT | UI element identifier |
| timestamp | TIMESTAMPTZ | Click timestamp |

### system_automations
Scheduled tasks and automation rules.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| bot_id | UUID | Foreign key to bots |
| name | TEXT | Automation name |
| schedule | TEXT | Cron expression |
| script | TEXT | BASIC script to execute |
| active | BOOLEAN | Whether automation is active |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update |

## Knowledge Base Tables

### kb_collections
Knowledge base collection definitions.

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key (collection name) |
| bot_id | UUID | Foreign key to bots |
| name | TEXT | Collection display name |
| description | TEXT | Collection description |
| metadata | JSONB | Additional metadata |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update |

### kb_documents
Documents stored in knowledge base collections.

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key (document ID) |
| collection_id | TEXT | Foreign key to kb_collections |
| bot_id | UUID | Foreign key to bots |
| name | TEXT | Document name |
| content | TEXT | Document content |
| metadata | JSONB | Document metadata |
| embedding_id | TEXT | Vector embedding reference |
| indexed | BOOLEAN | Whether document is indexed |
| created_at | TIMESTAMPTZ | Upload timestamp |
| updated_at | TIMESTAMPTZ | Last update |

### user_kb_associations
Links user sessions to available knowledge bases.

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key |
| session_id | UUID | Foreign key to user_sessions |
| collection_id | TEXT | Foreign key to kb_collections |
| created_at | TIMESTAMPTZ | Association timestamp |

## Tool Tables

### basic_tools
BASIC script tool definitions.

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key (tool name) |
| bot_id | UUID | Foreign key to bots |
| name | TEXT | Tool display name |
| description | TEXT | Tool description |
| parameters | JSONB | Parameter definitions |
| script | TEXT | BASIC script implementation |
| metadata | JSONB | Additional metadata |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update |

### session_tool_associations
Links sessions to available tools.

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Primary key |
| session_id | UUID | Foreign key to user_sessions |
| tool_id | TEXT | Foreign key to basic_tools |
| created_at | TIMESTAMPTZ | Association timestamp |

## Email Integration Tables

### user_email_accounts
Email accounts configured for users.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| email_address | TEXT | Email address |
| imap_server | TEXT | IMAP server address |
| imap_port | INTEGER | IMAP port |
| smtp_server | TEXT | SMTP server address |
| smtp_port | INTEGER | SMTP port |
| encrypted_password | TEXT | Encrypted email password |
| active | BOOLEAN | Account status |
| created_at | TIMESTAMPTZ | Configuration timestamp |
| updated_at | TIMESTAMPTZ | Last update |

### email_drafts
Draft emails created by users or bots.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| account_id | UUID | Foreign key to user_email_accounts |
| to_addresses | TEXT[] | Recipient addresses |
| cc_addresses | TEXT[] | CC addresses |
| bcc_addresses | TEXT[] | BCC addresses |
| subject | TEXT | Email subject |
| body | TEXT | Email body |
| attachments | JSONB | Attachment metadata |
| created_at | TIMESTAMPTZ | Draft creation |
| updated_at | TIMESTAMPTZ | Last edit |

### email_folders
Email folder organization.

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| account_id | UUID | Foreign key to user_email_accounts |
| name | TEXT | Folder name |
| path | TEXT | IMAP folder path |
| parent_id | UUID | Parent folder ID |
| message_count | INTEGER | Number of messages |
| unread_count | INTEGER | Unread messages |
| created_at | TIMESTAMPTZ | Folder creation |
| updated_at | TIMESTAMPTZ | Last sync |

## Indexes

### Primary Indexes
- All `id` columns have primary key indexes
- All foreign key columns have indexes for joins

### Performance Indexes
- `user_sessions.session_token` - for session lookup
- `message_history.created_at` - for time-based queries
- `kb_documents.collection_id` - for collection queries
- `bot_memories(bot_id, key)` - composite for memory lookup

### Full-Text Search Indexes
- `kb_documents.content` - for document search (when enabled)
- `message_history.message` - for conversation search (when enabled)