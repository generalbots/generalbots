# Database Schema Overview

BotServer uses PostgreSQL as its primary database with Diesel ORM for type-safe database operations. The schema is designed to support multi-tenant bot hosting with comprehensive session management, user authentication, and content storage.

## Core Architecture

The database schema follows these design principles:
- **UUID Primary Keys**: All tables use UUIDs for globally unique identifiers
- **Timestamps**: Created/updated timestamps for audit trails
- **Foreign Key Relationships**: Referential integrity between related entities
- **JSON Fields**: Flexible storage for dynamic configuration and metadata

## Schema Categories

### 1. Organization & Bot Management
- **organizations**: Multi-tenant organization support
- **bots**: Bot instances and configurations
- **bot_configuration**: Bot-specific settings and parameters
- **bot_memories**: Persistent key-value storage for bots

### 2. User & Authentication
- **users**: User accounts with secure password storage
- **user_sessions**: Active user sessions with tokens
- **user_login_tokens**: Authentication tokens for login
- **user_preferences**: User-specific settings

### 3. Conversation & Messaging
- **message_history**: Complete conversation history
- **clicks**: User interaction tracking
- **system_automations**: Scheduled tasks and automation rules

### 4. Knowledge Base
- **kb_collections**: Knowledge base collection definitions
- **kb_documents**: Documents stored in collections
- **user_kb_associations**: User access to knowledge bases
- **session_tool_associations**: Tools available in sessions

### 5. Tools & Integration
- **basic_tools**: BASIC script tool definitions
- **user_email_accounts**: Email integration accounts
- **email_drafts**: Draft emails
- **email_folders**: Email folder organization

## Table Relationships

### Primary Relationships

1. **Bot Hierarchy**
   - organizations (1) → (N) bots
   - bots (1) → (N) bot_configuration
   - bots (1) → (N) bot_memories

2. **User Sessions**
   - users (1) → (N) user_sessions
   - user_sessions (1) → (N) message_history
   - bots (1) → (N) user_sessions

3. **Knowledge Management**
   - bots (1) → (N) kb_collections
   - kb_collections (1) → (N) kb_documents
   - user_sessions (1) → (N) user_kb_associations

4. **Tool Associations**
   - bots (1) → (N) basic_tools
   - user_sessions (1) → (N) session_tool_associations

## Data Types

### Common Field Types
- **UUID**: Primary keys and foreign key references
- **Text**: Variable-length string data
- **Varchar**: Fixed-length strings for codes and identifiers
- **Timestamptz**: Timestamps with timezone
- **Jsonb**: JSON data for flexible schemas
- **Boolean**: Binary flags and settings
- **Integer**: Counters and numeric values

## Indexing Strategy

### Primary Indexes
- All primary keys (id fields)
- Foreign key relationships
- Timestamp fields for time-based queries
- Session tokens for authentication

### Composite Indexes
- (bot_id, user_id) for session lookup
- (collection_id, document_id) for knowledge retrieval
- (user_id, created_at) for history queries

## Migration Management

Database migrations are managed through Diesel's migration system:
- Migrations stored in `migrations/` directory
- Each migration has up.sql and down.sql
- Version tracking in `__diesel_schema_migrations` table
- Automatic migration on bootstrap

## Performance Considerations

### Connection Pooling
- Default pool size: 10 connections
- Configurable via environment variables
- Automatic connection recycling

### Query Optimization
- Prepared statements for repeated queries
- Batch operations for bulk inserts
- Lazy loading for related entities
- Pagination for large result sets

### Data Retention
- Message history retention configurable
- Automatic cleanup of expired sessions
- Archival strategy for old conversations

## Security Features

### Data Protection
- Argon2 hashing for passwords
- AES-GCM encryption for sensitive fields
- Secure random tokens for sessions
- SQL injection prevention via Diesel

### Access Control
- Row-level security via application logic
- User isolation by session
- Bot isolation by organization
- Audit logging for sensitive operations

## Backup Strategy

### Backup Types
- Full database dumps
- Incremental WAL archiving
- Point-in-time recovery support
- Cross-region replication (optional)

### Restore Procedures
- Automated restore testing
- Version compatibility checks
- Data integrity validation
- Zero-downtime migration support

## Monitoring

### Key Metrics
- Connection pool usage
- Query execution time
- Table sizes and growth
- Index effectiveness
- Lock contention

### Health Checks
- Database connectivity
- Migration status
- Replication lag (if applicable)
- Storage usage

## Best Practices

1. **Always use migrations** for schema changes
2. **Never modify production data** directly
3. **Test migrations** in development first
4. **Monitor performance** metrics regularly
5. **Plan capacity** based on growth projections
6. **Document changes** in migration files
7. **Use transactions** for data consistency
8. **Implement retry logic** for transient failures

## Future Considerations

- Partitioning for large tables (message_history)
- Read replicas for scaling
- Time-series optimization for metrics
- Full-text search indexes
- Graph relationships for advanced queries