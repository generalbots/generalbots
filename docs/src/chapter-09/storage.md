# Storage and Data

BotServer uses multiple storage layers to handle different types of data, from structured user information to unstructured documents and vector embeddings.

## Overview

Storage in BotServer is organized into:
- **PostgreSQL** - Structured data and metadata
- **MinIO/S3** - Object storage for files and documents
- **Redis/Valkey** - Session cache and temporary data
- **Qdrant** - Vector embeddings for semantic search
- **Local filesystem** - Working directories and cache

## Storage Architecture

### Data Flow

```
User Upload → MinIO Storage → Processing → Database Metadata
                    ↓                           ↓
              Vector Database            PostgreSQL Tables
                    ↓                           ↓
              Semantic Search            Structured Queries
```

## PostgreSQL Database

### Primary Data Store

PostgreSQL stores all structured data:
- User accounts and sessions
- Bot configurations
- Message history
- System automations
- Knowledge base metadata

### Schema Management

- Migrations in `migrations/` directory
- Diesel ORM for type-safe queries
- Automatic migration on bootstrap
- Version tracking in database

### Connection Pooling

```
DATABASE_URL=postgres://gbuser:password@localhost:5432/botserver
DB_POOL_SIZE=10
```

Connection pool managed by Diesel:
- Default 10 connections
- Automatic retry on failure
- Connection recycling
- Timeout protection

## MinIO/S3 Object Storage

### File Organization

MinIO stores unstructured data:

```
minio/
├── bot-name.gbai/           # Bot-specific bucket
│   ├── bot-name.gbdialog/  # BASIC scripts
│   ├── bot-name.gbkb/       # Knowledge base documents
│   └── bot-name.gbot/       # Configuration files
└── botserver-media/         # Shared media files
```

### Storage Operations

- **Upload**: Files uploaded via PUT operations
- **Retrieval**: GET operations with bucket/key
- **Listing**: Browse bucket contents
- **Deletion**: Remove objects (rarely used)

### Configuration

```bash
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin
```

## Redis/Valkey Cache

### Cached Data

Redis stores temporary and cached data:
- Session tokens
- Temporary conversation state
- API response cache
- Rate limiting counters
- Lock mechanisms

### Cache Patterns

```
# Session cache
session:{session_id} → session_data (TTL: 24 hours)

# Rate limiting
rate:{user_id}:{endpoint} → request_count (TTL: 1 hour)

# Temporary data
temp:{key} → data (TTL: varies)
```

### Configuration

```bash
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=5
REDIS_TTL_SECONDS=86400
```

## Qdrant Vector Database

### Vector Storage

Qdrant stores embedding vectors:
- Document embeddings
- Search indices
- Semantic relationships
- Similarity scores

### Collection Structure

```
Collections:
├── {bot_id}_documents     # Document embeddings
├── {bot_id}_conversations # Conversation embeddings
└── {bot_id}_cache        # Cached query results
```

### Vector Operations

- **Insert**: Add new embeddings
- **Search**: Find similar vectors
- **Update**: Modify metadata
- **Delete**: Remove outdated vectors

## Local Storage

### Working Directories

```
botserver/
├── work/               # Temporary processing
│   └── bot.gbai/      # Bot working files
├── logs/              # Application logs
├── cache/             # Local cache files
└── uploads/           # Temporary uploads
```

### File Management

- Automatic cleanup of old files
- Size limits on uploads
- Temp file rotation
- Log file management

## Data Persistence

### Backup Strategy

1. **Database Backups**
   - Daily PostgreSQL dumps
   - Point-in-time recovery
   - Automated backup scripts

2. **Object Storage**
   - MinIO replication
   - Versioning enabled
   - Cross-region backup

3. **Configuration**
   - Version controlled
   - Environment-specific
   - Encrypted secrets

### Data Retention

- Message history: 90 days default
- Session data: 30 days
- Temporary files: 24 hours
- Logs: 7 days rolling
- Backups: 30 days

## BASIC Script Storage Operations

### Saving Data

```basic
# Save to CSV file
SAVE "data/results.csv", column1, column2, column3

# Save with timestamp
let filename = "backup_" + FORMAT(NOW(), "YYYYMMDD") + ".txt"
SAVE filename, data
```

### Reading Data

```basic
# Read from storage
let content = GET "documents/report.pdf"

# Read configuration
let config = GET "settings/config.json"
```

## Storage Optimization

### Performance Tips

1. **Use appropriate storage**
   - PostgreSQL for structured data
   - MinIO for files
   - Redis for cache
   - Qdrant for vectors

2. **Implement caching**
   - Cache frequent queries
   - Use Redis for sessions
   - Local cache for static files

3. **Batch operations**
   - Bulk inserts
   - Batch file uploads
   - Grouped queries

### Resource Management

- Monitor disk usage
- Set storage quotas
- Implement cleanup jobs
- Compress old data

## Security

### Data Encryption

- **At Rest**: Database encryption
- **In Transit**: TLS/SSL connections
- **Sensitive Data**: AES-GCM encryption
- **Passwords**: Never stored (Zitadel handles)

### Access Control

- Role-based permissions
- Bot isolation
- User data segregation
- Audit logging

## Monitoring

### Storage Metrics

Monitor these metrics:
- Database size and growth
- MinIO bucket usage
- Redis memory usage
- Qdrant index size
- Disk space available

### Health Checks

- Database connectivity
- MinIO availability
- Redis response time
- Qdrant query performance
- Disk space warnings

## Troubleshooting

### Common Issues

1. **Out of Space**
   - Clean temporary files
   - Archive old data
   - Increase storage allocation

2. **Slow Queries**
   - Add database indexes
   - Optimize query patterns
   - Increase cache size

3. **Connection Failures**
   - Check service status
   - Verify credentials
   - Review network configuration

## Best Practices

1. **Regular Maintenance**
   - Vacuum PostgreSQL
   - Clean MinIO buckets
   - Flush Redis cache
   - Reindex Qdrant

2. **Monitor Growth**
   - Track storage trends
   - Plan capacity
   - Set up alerts

3. **Data Hygiene**
   - Remove orphaned data
   - Archive old records
   - Validate integrity

## Configuration Reference

### Storage Limits

```
# Database
MAX_CONNECTIONS=100
STATEMENT_TIMEOUT=30s

# MinIO
MAX_OBJECT_SIZE=5GB
BUCKET_QUOTA=100GB

# Redis
MAX_MEMORY=2GB
EVICTION_POLICY=allkeys-lru

# Filesystem
UPLOAD_SIZE_LIMIT=100MB
TEMP_DIR_SIZE=10GB
```

## Future Enhancements

Planned storage improvements:
- Distributed storage support
- Advanced caching strategies
- Data lake integration
- Time-series optimization
- GraphQL API for queries
- Real-time synchronization

## Summary

BotServer's multi-layered storage architecture provides flexibility, performance, and reliability. By using the right storage solution for each data type and implementing proper caching and optimization strategies, the system can handle large-scale deployments while maintaining responsiveness.