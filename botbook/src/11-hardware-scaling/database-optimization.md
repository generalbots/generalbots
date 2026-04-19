# Database Optimization

This document covers database schema design and optimization strategies for billion-user scale deployments.

## Schema Design Principles

### Use SMALLINT Enums Instead of VARCHAR

One of the most impactful optimizations is using integer enums instead of string-based status fields.

**Before (inefficient):**
```sql
CREATE TABLE auto_tasks (
    id UUID PRIMARY KEY,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority VARCHAR(20) NOT NULL DEFAULT 'normal',
    execution_mode VARCHAR(50) NOT NULL DEFAULT 'supervised',
    CONSTRAINT check_status CHECK (status IN ('pending', 'ready', 'running', 'paused', 'waiting_approval', 'completed', 'failed', 'cancelled'))
);
```

**After (optimized):**
```sql
CREATE TABLE auto_tasks (
    id UUID PRIMARY KEY,
    status SMALLINT NOT NULL DEFAULT 0,        -- 2 bytes
    priority SMALLINT NOT NULL DEFAULT 1,      -- 2 bytes
    execution_mode SMALLINT NOT NULL DEFAULT 1 -- 2 bytes
);
```

### Storage Comparison

| Field Type | Storage | Example Values |
|------------|---------|----------------|
| VARCHAR(50) | 1-51 bytes | 'waiting_approval' = 17 bytes |
| TEXT | 1+ bytes | 'completed' = 10 bytes |
| SMALLINT | 2 bytes | 4 = 2 bytes (always) |
| INTEGER | 4 bytes | 4 = 4 bytes (always) |

**Savings per row with 5 enum fields:**
- VARCHAR: ~50 bytes average
- SMALLINT: 10 bytes fixed
- **Savings: 40 bytes per row = 40GB per billion rows**

## Enum Value Reference

All domain values in General Bots use SMALLINT. Reference table:

### Channel Types
| Value | Name | Description |
|-------|------|-------------|
| 0 | web | Web chat interface |
| 1 | whatsapp | WhatsApp Business |
| 2 | telegram | Telegram Bot |
| 3 | msteams | Microsoft Teams |
| 4 | slack | Slack |
| 5 | email | Email channel |
| 6 | sms | SMS/Text messages |
| 7 | voice | Voice/Phone |
| 8 | instagram | Instagram DM |
| 9 | api | Direct API |

### Message Role
| Value | Name | Description |
|-------|------|-------------|
| 1 | user | User message |
| 2 | assistant | Bot response |
| 3 | system | System prompt |
| 4 | tool | Tool call/result |
| 9 | episodic | Episodic memory summary |
| 10 | compact | Compacted conversation |

### Message Type
| Value | Name | Description |
|-------|------|-------------|
| 0 | text | Plain text |
| 1 | image | Image attachment |
| 2 | audio | Audio file |
| 3 | video | Video file |
| 4 | document | Document/PDF |
| 5 | location | GPS location |
| 6 | contact | Contact card |
| 7 | sticker | Sticker |
| 8 | reaction | Message reaction |

### LLM Provider
| Value | Name | Description |
|-------|------|-------------|
| 0 | openai | OpenAI API |
| 1 | anthropic | Anthropic Claude |
| 2 | azure_openai | Azure OpenAI |
| 3 | azure_claude | Azure Claude |
| 4 | google | Google AI |
| 5 | local | Local llama.cpp |
| 6 | ollama | Ollama |
| 7 | groq | Groq |
| 8 | mistral | Mistral AI |
| 9 | cohere | Cohere |

### Task Status
| Value | Name | Description |
|-------|------|-------------|
| 0 | pending | Waiting to start |
| 1 | ready | Ready to execute |
| 2 | running | Currently executing |
| 3 | paused | Paused by user |
| 4 | waiting_approval | Needs approval |
| 5 | completed | Successfully finished |
| 6 | failed | Failed with error |
| 7 | cancelled | Cancelled by user |

### Task Priority
| Value | Name | Description |
|-------|------|-------------|
| 0 | low | Low priority |
| 1 | normal | Normal priority |
| 2 | high | High priority |
| 3 | urgent | Urgent |
| 4 | critical | Critical |

### Execution Mode
| Value | Name | Description |
|-------|------|-------------|
| 0 | manual | Manual execution only |
| 1 | supervised | Requires approval |
| 2 | autonomous | Fully automatic |

### Risk Level
| Value | Name | Description |
|-------|------|-------------|
| 0 | none | No risk |
| 1 | low | Low risk |
| 2 | medium | Medium risk |
| 3 | high | High risk |
| 4 | critical | Critical risk |

### Approval Status
| Value | Name | Description |
|-------|------|-------------|
| 0 | pending | Awaiting decision |
| 1 | approved | Approved |
| 2 | rejected | Rejected |
| 3 | expired | Timed out |
| 4 | skipped | Skipped |

### Intent Type
| Value | Name | Description |
|-------|------|-------------|
| 0 | unknown | Unclassified |
| 1 | app_create | Create application |
| 2 | todo | Create task/reminder |
| 3 | monitor | Set up monitoring |
| 4 | action | Execute action |
| 5 | schedule | Create schedule |
| 6 | goal | Set goal |
| 7 | tool | Create tool |
| 8 | query | Query/search |

### Memory Type
| Value | Name | Description |
|-------|------|-------------|
| 0 | short | Short-term |
| 1 | long | Long-term |
| 2 | episodic | Episodic |
| 3 | semantic | Semantic |
| 4 | procedural | Procedural |

### Sync Status
| Value | Name | Description |
|-------|------|-------------|
| 0 | synced | Fully synced |
| 1 | pending | Sync pending |
| 2 | conflict | Conflict detected |
| 3 | error | Sync error |
| 4 | deleted | Marked for deletion |

## Indexing Strategies

### Composite Indexes for Common Queries

```sql
-- Session lookup by user
CREATE INDEX idx_sessions_user ON user_sessions(user_id, created_at DESC);

-- Messages by session (most common query)
CREATE INDEX idx_messages_session ON message_history(session_id, message_index);

-- Active tasks by status and priority
CREATE INDEX idx_tasks_status ON auto_tasks(status, priority) WHERE status < 5;

-- Tenant-scoped queries
CREATE INDEX idx_sessions_tenant ON user_sessions(tenant_id, created_at DESC);
```

### Partial Indexes for Active Records

```sql
-- Only index active bots (saves space)
CREATE INDEX idx_bots_active ON bots(tenant_id, is_active) WHERE is_active = true;

-- Only index pending approvals
CREATE INDEX idx_approvals_pending ON task_approvals(task_id, expires_at) WHERE status = 0;

-- Only index unread messages
CREATE INDEX idx_messages_unread ON message_history(user_id, created_at) WHERE is_read = false;
```

### BRIN Indexes for Time-Series Data

```sql
-- BRIN index for time-ordered data (much smaller than B-tree)
CREATE INDEX idx_messages_created_brin ON message_history USING BRIN (created_at);
CREATE INDEX idx_analytics_date_brin ON analytics_events USING BRIN (created_at);
```

## Table Partitioning

### Partition High-Volume Tables by Time

```sql
-- Partitioned messages table
CREATE TABLE message_history (
    id UUID NOT NULL,
    session_id UUID NOT NULL,
    tenant_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    -- other columns...
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Monthly partitions
CREATE TABLE message_history_2025_01 PARTITION OF message_history
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
CREATE TABLE message_history_2025_02 PARTITION OF message_history
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');
-- ... continue for each month

-- Default partition for future data
CREATE TABLE message_history_default PARTITION OF message_history DEFAULT;
```

### Automatic Partition Management

```sql
-- Function to create next month's partition
CREATE OR REPLACE FUNCTION create_monthly_partition(
    table_name TEXT,
    partition_date DATE
) RETURNS VOID AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    partition_name := table_name || '_' || to_char(partition_date, 'YYYY_MM');
    start_date := date_trunc('month', partition_date);
    end_date := start_date + INTERVAL '1 month';
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF %I FOR VALUES FROM (%L) TO (%L)',
        partition_name, table_name, start_date, end_date
    );
END;
$$ LANGUAGE plpgsql;

-- Create partitions for next 3 months
SELECT create_monthly_partition('message_history', NOW() + INTERVAL '1 month');
SELECT create_monthly_partition('message_history', NOW() + INTERVAL '2 months');
SELECT create_monthly_partition('message_history', NOW() + INTERVAL '3 months');
```

## Connection Pooling

### PgBouncer Configuration

```ini
; pgbouncer.ini
[databases]
gb_shard1 = host=shard1.db port=5432 dbname=gb
gb_shard2 = host=shard2.db port=5432 dbname=gb

[pgbouncer]
listen_port = 6432
listen_addr = *
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt

; Pool settings
pool_mode = transaction
max_client_conn = 10000
default_pool_size = 50
min_pool_size = 10
reserve_pool_size = 25
reserve_pool_timeout = 3

; Timeouts
server_connect_timeout = 3
server_idle_timeout = 600
server_lifetime = 3600
client_idle_timeout = 0
```

### Application Connection Settings

```toml
# config.toml
[database]
max_connections = 100
min_connections = 10
connection_timeout_secs = 5
idle_timeout_secs = 300
max_lifetime_secs = 1800
```

## Query Optimization

### Use EXPLAIN ANALYZE

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM message_history
WHERE session_id = 'abc-123'
ORDER BY message_index;
```

### Avoid N+1 Queries

**Bad:**
```sql
-- 1 query for sessions
SELECT * FROM user_sessions WHERE user_id = 'xyz';
-- N queries for messages (one per session)
SELECT * FROM message_history WHERE session_id = ?;
```

**Good:**
```sql
-- Single query with JOIN
SELECT s.*, m.*
FROM user_sessions s
LEFT JOIN message_history m ON m.session_id = s.id
WHERE s.user_id = 'xyz'
ORDER BY s.created_at DESC, m.message_index;
```

### Use Covering Indexes

```sql
-- Index includes all needed columns (no table lookup)
CREATE INDEX idx_sessions_covering ON user_sessions(user_id, created_at DESC)
INCLUDE (title, message_count, last_activity_at);
```

## Vacuum and Maintenance

### Aggressive Autovacuum for High-Churn Tables

```sql
ALTER TABLE message_history SET (
    autovacuum_vacuum_scale_factor = 0.01,
    autovacuum_analyze_scale_factor = 0.005,
    autovacuum_vacuum_cost_delay = 2
);

ALTER TABLE user_sessions SET (
    autovacuum_vacuum_scale_factor = 0.02,
    autovacuum_analyze_scale_factor = 0.01
);
```

### Regular Maintenance Tasks

```sql
-- Weekly: Reindex bloated indexes
REINDEX INDEX CONCURRENTLY idx_messages_session;

-- Monthly: Update statistics
ANALYZE VERBOSE message_history;

-- Quarterly: Cluster heavily-queried tables
CLUSTER message_history USING idx_messages_session;
```

## Monitoring Queries

### Table Bloat Check

```sql
SELECT
    schemaname || '.' || tablename AS table,
    pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname || '.' || tablename)) AS table_size,
    pg_size_pretty(pg_indexes_size(schemaname || '.' || tablename)) AS index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC
LIMIT 20;
```

### Slow Query Log

```sql
-- postgresql.conf
log_min_duration_statement = 100  -- Log queries > 100ms
log_statement = 'none'
log_lock_waits = on
```

### Index Usage Statistics

```sql
SELECT
    schemaname || '.' || relname AS table,
    indexrelname AS index,
    idx_scan AS scans,
    idx_tup_read AS tuples_read,
    idx_tup_fetch AS tuples_fetched,
    pg_size_pretty(pg_relation_size(indexrelid)) AS size
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC
LIMIT 20;
```

## Best Practices Summary

1. **Use SMALLINT for enums** - 2 bytes vs 10-50 bytes per field
2. **Partition time-series tables** - Monthly partitions for messages/analytics
3. **Create partial indexes** - Only index active/relevant rows
4. **Use connection pooling** - PgBouncer with transaction mode
5. **Enable aggressive autovacuum** - For high-churn tables
6. **Monitor query performance** - Log slow queries, check EXPLAIN plans
7. **Use covering indexes** - Include frequently-accessed columns
8. **Avoid cross-shard queries** - Keep tenant data together
9. **Regular maintenance** - Reindex, analyze, cluster as needed
10. **Test at scale** - Use production-like data volumes in staging