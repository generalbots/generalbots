# Sharding Architecture

This document describes how General Bots distributes data across multiple database shards for horizontal scaling.

## Overview

Sharding enables General Bots to scale beyond single-database limits by distributing data across multiple database instances. Each shard contains a subset of tenants, and data never crosses shard boundaries during normal operations.

## Shard Configuration

### Shard Config Table

The `shard_config` table defines all available shards:

```sql
CREATE TABLE shard_config (
    shard_id SMALLINT PRIMARY KEY,
    region_code CHAR(3) NOT NULL,        -- ISO 3166-1 alpha-3: USA, BRA, DEU
    datacenter VARCHAR(32) NOT NULL,      -- e.g., 'us-east-1', 'eu-west-1'
    connection_string TEXT NOT NULL,      -- Encrypted connection string
    is_primary BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    min_tenant_id BIGINT NOT NULL,
    max_tenant_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Example Configuration

```sql
-- Americas
INSERT INTO shard_config VALUES 
(1, 'USA', 'us-east-1', 'postgresql://shard1.db:5432/gb', true, true, 1, 1000000),
(2, 'USA', 'us-west-2', 'postgresql://shard2.db:5432/gb', false, true, 1000001, 2000000),
(3, 'BRA', 'sa-east-1', 'postgresql://shard3.db:5432/gb', false, true, 2000001, 3000000);

-- Europe
INSERT INTO shard_config VALUES 
(4, 'DEU', 'eu-central-1', 'postgresql://shard4.db:5432/gb', false, true, 3000001, 4000000),
(5, 'GBR', 'eu-west-2', 'postgresql://shard5.db:5432/gb', false, true, 4000001, 5000000);

-- Asia Pacific
INSERT INTO shard_config VALUES 
(6, 'SGP', 'ap-southeast-1', 'postgresql://shard6.db:5432/gb', false, true, 5000001, 6000000),
(7, 'JPN', 'ap-northeast-1', 'postgresql://shard7.db:5432/gb', false, true, 6000001, 7000000);
```

## Tenant-to-Shard Mapping

### Mapping Table

```sql
CREATE TABLE tenant_shard_map (
    tenant_id BIGINT PRIMARY KEY,
    shard_id SMALLINT NOT NULL REFERENCES shard_config(shard_id),
    region_code CHAR(3) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Routing Logic

When a request comes in, the system:

1. Extracts `tenant_id` from the request context
2. Looks up `shard_id` from `tenant_shard_map`
3. Routes the query to the appropriate database connection

```rust
// Rust routing example
pub fn get_shard_connection(tenant_id: i64) -> Result<DbConnection> {
    let shard_id = SHARD_MAP.get(&tenant_id)
        .ok_or_else(|| Error::TenantNotFound(tenant_id))?;
    
    CONNECTION_POOLS.get(shard_id)
        .ok_or_else(|| Error::ShardNotAvailable(*shard_id))
}
```

## Data Model Requirements

### Every Table Includes Shard Keys

All tables must include `tenant_id` and `shard_id` columns:

```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id BIGINT NOT NULL,           -- Required for routing
    shard_id SMALLINT NOT NULL,          -- Denormalized for queries
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    -- ... other columns
);
```

### Foreign Keys Within Shard Only

Foreign keys only reference tables within the same shard:

```sql
-- Good: Same shard reference
ALTER TABLE message_history 
ADD CONSTRAINT fk_session 
FOREIGN KEY (session_id) REFERENCES user_sessions(id);

-- Bad: Cross-shard reference (never do this)
-- FOREIGN KEY (other_tenant_data) REFERENCES other_shard.table(id)
```

## Snowflake ID Generation

For globally unique, time-sortable IDs across shards:

```sql
CREATE OR REPLACE FUNCTION generate_snowflake_id(p_shard_id SMALLINT)
RETURNS BIGINT AS $$
DECLARE
    epoch BIGINT := 1704067200000;  -- 2024-01-01 00:00:00 UTC
    ts BIGINT;
    seq BIGINT;
BEGIN
    -- 41 bits: timestamp (milliseconds since epoch)
    ts := (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT - epoch;
    
    -- 10 bits: shard_id (0-1023)
    -- 12 bits: sequence (0-4095)
    seq := nextval('global_seq') & 4095;
    
    RETURN (ts << 22) | ((p_shard_id & 1023) << 12) | seq;
END;
$$ LANGUAGE plpgsql;
```

### ID Structure

```
 64-bit Snowflake ID
┌─────────────────────────────────────────────────────────────────┐
│  41 bits timestamp  │  10 bits shard  │  12 bits sequence      │
│  (69 years range)   │  (1024 shards)  │  (4096/ms/shard)       │
└─────────────────────────────────────────────────────────────────┘
```

## Shard Operations

### Creating a New Shard

1. Provision new database instance
2. Run migrations
3. Add to `shard_config`
4. Update routing configuration
5. Begin assigning new tenants

```bash
# 1. Run migrations on new shard
DATABASE_URL="postgresql://new-shard:5432/gb" diesel migration run

# 2. Add shard config
psql -c "INSERT INTO shard_config VALUES (8, 'AUS', 'ap-southeast-2', '...', false, true, 7000001, 8000000);"

# 3. Reload routing
curl -X POST http://localhost:3000/api/admin/reload-shard-config
```

### Tenant Migration Between Shards

Moving a tenant to a different shard (e.g., for data locality):

```sql
-- 1. Set tenant to read-only mode
UPDATE tenants SET settings = settings || '{"read_only": true}' WHERE id = 12345;

-- 2. Export tenant data
pg_dump -t 'user_sessions' -t 'message_history' --where="tenant_id=12345" source_db > tenant_12345.sql

-- 3. Import to new shard
psql target_db < tenant_12345.sql

-- 4. Update routing
UPDATE tenant_shard_map SET shard_id = 5, region_code = 'DEU' WHERE tenant_id = 12345;

-- 5. Remove read-only mode
UPDATE tenants SET settings = settings - 'read_only' WHERE id = 12345;

-- 6. Clean up source shard (after verification)
DELETE FROM user_sessions WHERE tenant_id = 12345;
DELETE FROM message_history WHERE tenant_id = 12345;
```

## Query Patterns

### Single-Tenant Queries (Most Common)

```sql
-- Efficient: Uses shard routing
SELECT * FROM user_sessions 
WHERE tenant_id = 12345 AND user_id = 'abc-123';
```

### Cross-Shard Queries (Admin Only)

For global analytics, use a federation layer:

```sql
-- Using postgres_fdw for cross-shard reads
SELECT shard_id, COUNT(*) as session_count
FROM all_shards.user_sessions
WHERE created_at > NOW() - INTERVAL '1 day'
GROUP BY shard_id;
```

### Scatter-Gather Pattern

For queries that must touch multiple shards:

```rust
async fn get_global_stats() -> Stats {
    let futures: Vec<_> = SHARDS.iter()
        .map(|shard| get_shard_stats(shard.id))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    results.into_iter().fold(Stats::default(), |acc, s| acc.merge(s))
}
```

## High Availability

### Per-Shard Replication

Each shard should have:

- 1 Primary (read/write)
- 1-2 Replicas (read-only, failover)
- Async replication with < 1s lag

```
Shard 1 Architecture:
┌─────────────┐
│   Primary   │◄──── Writes
└──────┬──────┘
       │ Streaming Replication
   ┌───┴───┐
   ▼       ▼
┌──────┐ ┌──────┐
│Rep 1 │ │Rep 2 │◄──── Reads
└──────┘ └──────┘
```

### Failover Configuration

```yaml
# config.csv
shard-1-primary,postgresql://shard1-primary:5432/gb
shard-1-replica-1,postgresql://shard1-replica1:5432/gb
shard-1-replica-2,postgresql://shard1-replica2:5432/gb
shard-1-failover-priority,replica-1,replica-2
```

## Monitoring

### Key Metrics Per Shard

| Metric | Warning | Critical |
|--------|---------|----------|
| Connection pool usage | > 70% | > 90% |
| Query latency p99 | > 100ms | > 500ms |
| Replication lag | > 1s | > 10s |
| Disk usage | > 70% | > 85% |
| Tenant count | > 80% capacity | > 95% capacity |

### Shard Health Check

```sql
-- Run on each shard
SELECT 
    current_setting('cluster_name') as shard,
    pg_is_in_recovery() as is_replica,
    pg_last_wal_receive_lsn() as wal_position,
    pg_postmaster_start_time() as uptime_since,
    (SELECT count(*) FROM pg_stat_activity) as connections,
    (SELECT count(DISTINCT tenant_id) FROM tenants) as tenant_count;
```

## Best Practices

1. **Shard by tenant, not by table** - Keep all tenant data together
2. **Avoid cross-shard transactions** - Design for eventual consistency where needed
3. **Pre-allocate tenant ranges** - Leave room for growth in each shard
4. **Monitor shard hotspots** - Rebalance if one shard gets too busy
5. **Test failover regularly** - Ensure replicas can be promoted
6. **Use connection pooling** - PgBouncer or similar for each shard
7. **Cache shard routing** - Don't query `tenant_shard_map` on every request

## Migration from Single Database

To migrate an existing single-database deployment to sharded:

1. Add `shard_id` column to all tables (default to 1)
2. Deploy shard routing code (disabled)
3. Set up additional shard databases
4. Enable routing for new tenants only
5. Gradually migrate existing tenants during low-traffic windows
6. Decommission original database when empty

See [Regional Deployment](./regional-deployment.md) for multi-region considerations.