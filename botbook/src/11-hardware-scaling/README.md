# Chapter 11: Hardware & Scaling

This chapter covers hardware requirements and horizontal scaling strategies for General Bots.

## Overview

General Bots is designed from the ground up to scale horizontally. The architecture supports:

- **Multi-tenancy**: Complete isolation between organizations
- **Regional sharding**: Data locality for compliance and performance
- **Database partitioning**: Efficient handling of high-volume tables
- **Stateless services**: Easy horizontal pod autoscaling

## Chapter Contents

- [Sharding Architecture](./sharding.md) - How data is distributed across shards
- [Database Optimization](./database-optimization.md) - Schema design for billion-scale
- [Regional Deployment](./regional-deployment.md) - Multi-region setup
- [Performance Tuning](./performance-tuning.md) - Optimization strategies

## Key Concepts

### Tenant Isolation

Every piece of data in General Bots is associated with a `tenant_id`. This enables:

1. Complete data isolation between organizations
2. Per-tenant resource limits and quotas
3. Tenant-specific configurations
4. Easy data export/deletion for compliance

### Shard Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Load Balancer                          │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
   ┌─────────┐       ┌─────────┐       ┌─────────┐
   │ Region  │       │ Region  │       │ Region  │
   │   USA   │       │   EUR   │       │   APAC  │
   └────┬────┘       └────┬────┘       └────┬────┘
        │                 │                 │
   ┌────┴────┐       ┌────┴────┐       ┌────┴────┐
   │ Shard 1 │       │ Shard 2 │       │ Shard 3 │
   │ Shard 4 │       │ Shard 5 │       │ Shard 6 │
   └─────────┘       └─────────┘       └─────────┘
```

### Database Design Principles

1. **SMALLINT enums** instead of VARCHAR for domain values (2 bytes vs 20+ bytes)
2. **Partitioned tables** for high-volume data (messages, sessions, analytics)
3. **Composite primary keys** including `shard_id` for distributed queries
4. **Snowflake-like IDs** for globally unique, time-sortable identifiers

## When to Scale

| Users | Sessions/day | Messages/day | Recommended Setup |
|-------|--------------|--------------|-------------------|
| < 10K | < 100K | < 1M | Single node |
| 10K-100K | 100K-1M | 1M-10M | 2-3 nodes, single region |
| 100K-1M | 1M-10M | 10M-100M | Multi-node, consider sharding |
| 1M-10M | 10M-100M | 100M-1B | Regional shards |
| > 10M | > 100M | > 1B | Global shards with Citus/CockroachDB |

## Quick Start

To enable sharding in your deployment:

1. Configure shard mapping in `shard_config` table
2. Set `SHARD_ID` environment variable per instance
3. Deploy region-specific instances
4. Configure load balancer routing rules

See [Sharding Architecture](./sharding.md) for detailed setup instructions.