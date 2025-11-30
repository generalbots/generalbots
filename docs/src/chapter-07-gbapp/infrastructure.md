# Infrastructure Design

This chapter covers the complete infrastructure design for General Bots, including scaling, security, secrets management, observability, and high availability.

## Architecture Overview

General Bots uses a modular architecture where each component runs in isolated LXC containers. This provides:

- **Isolation**: Each service has its own filesystem and process space
- **Scalability**: Add more containers to handle increased load
- **Security**: Compromised components cannot affect others
- **Portability**: Move containers between hosts easily

## Component Diagram
## High Availability Architecture

![Infrastructure Architecture](../assets/infrastructure-architecture.svg)

*Production-ready infrastructure with automatic scaling, load balancing, and multi-tenant isolation.*

## Encryption at Rest

All data stored by General Bots is encrypted at rest using AES-256-GCM.

### Database Encryption

PostgreSQL uses Transparent Data Encryption (TDE):

```csv
# config.csv
encryption-at-rest,true
encryption-algorithm,aes-256-gcm
encryption-key-source,vault
```

Enable in PostgreSQL:

```sql
-- Enable pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Encrypted columns use pgp_sym_encrypt
ALTER TABLE bot_memories 
ADD COLUMN value_encrypted bytea;

UPDATE bot_memories 
SET value_encrypted = pgp_sym_encrypt(value, current_setting('app.encryption_key'));
```

### File Storage Encryption

MinIO server-side encryption:

```bash
# Enable SSE-S3 encryption
mc encrypt set sse-s3 local/gbo-bucket

# Or use customer-managed keys (SSE-C)
mc encrypt set sse-c local/gbo-bucket
```

Configuration:

```csv
# config.csv
drive-encryption,true
drive-encryption-type,sse-s3
drive-encryption-key,vault:gbo/encryption/drive_key
```

### Redis Encryption

Redis with TLS and encrypted RDB:

```conf
# redis.conf
tls-port 6379
port 0
tls-cert-file /opt/gbo/conf/certificates/redis/server.crt
tls-key-file /opt/gbo/conf/certificates/redis/server.key
tls-ca-cert-file /opt/gbo/conf/certificates/ca.crt

# Enable RDB encryption (Redis 7.2+)
rdb-save-incremental-fsync yes
```

### Vector Database Encryption

Qdrant with encrypted storage:

```yaml
# qdrant/config.yaml
storage:
  storage_path: /opt/gbo/data/qdrant
  on_disk_payload: true
  
service:
  enable_tls: true
  
# Disk encryption handled at filesystem level
```

### Filesystem-Level Encryption

For comprehensive encryption, use LUKS on the data partition:

```bash
# Create encrypted partition for /opt/gbo/data
cryptsetup luksFormat /dev/sdb1
cryptsetup open /dev/sdb1 gbo-data
mkfs.ext4 /dev/mapper/gbo-data
mount /dev/mapper/gbo-data /opt/gbo/data
```

## Media Processing: LiveKit vs GStreamer

### Do You Need GStreamer with LiveKit?

**Short answer: No.** LiveKit handles most media processing needs.

| Feature | LiveKit | GStreamer | Need Both? |
|---------|---------|-----------|------------|
| WebRTC | Native | Plugin | No |
| Recording | Built-in | External | No |
| Transcoding | Egress service | Full control | Rarely |
| Streaming | Native | Full control | Rarely |
| Custom filters | Limited | Extensive | Sometimes |
| AI integration | Built-in | Manual | No |

**Use GStreamer only if you need:**
- Custom video/audio filters
- Unusual codec support
- Complex media pipelines
- Broadcast streaming (RTMP/HLS)

LiveKit's Egress service handles:
- Room recording
- Participant recording
- Livestreaming to YouTube/Twitch
- Track composition

### LiveKit Configuration

```csv
# config.csv
meet-provider,livekit
meet-server-url,wss://localhost:7880
meet-api-key,vault:gbo/meet/api_key
meet-api-secret,vault:gbo/meet/api_secret
meet-recording-enabled,true
meet-transcription-enabled,true
```

## Message Queues: Kafka vs RabbitMQ

### Do You Need Kafka or RabbitMQ?

**For most deployments: No.** Redis PubSub handles messaging needs.

| Scale | Recommendation |
|-------|----------------|
| < 1,000 concurrent users | Redis PubSub |
| 1,000 - 10,000 users | Redis Streams |
| 10,000 - 100,000 users | RabbitMQ |
| > 100,000 users | Kafka |

### When to Add Message Queues

**Add RabbitMQ when you need:**
- Message persistence/durability
- Complex routing patterns
- Multiple consumer groups
- Dead letter queues

**Add Kafka when you need:**
- Event sourcing
- Stream processing
- Multi-datacenter replication
- High throughput (millions/sec)

### Current Redis-Based Messaging

General Bots uses Redis for:

```rust
// Session state
redis::cmd("SET").arg("session:123").arg(state_json)

// PubSub for real-time
redis::cmd("PUBLISH").arg("channel:bot-1").arg(message)

// Streams for persistence (optional)
redis::cmd("XADD").arg("stream:events").arg("*").arg("event").arg(data)
```

Configuration:

```csv
# config.csv
messaging-provider,redis
messaging-persistence,streams
messaging-retention-hours,24
```

## Sharding Strategies

### Option 1: Tenant-Based Sharding (Recommended)

Each tenant/organization gets isolated databases:
## Multi-Tenant Architecture

Each tenant gets isolated resources with dedicated database schemas, cache namespaces, and vector collections. The router maps tenant IDs to their respective data stores automatically.

**Key isolation features:**
- Database-per-tenant or schema-per-tenant options
- Namespace isolation in Valkey cache
- Collection isolation in Qdrant vectors
- Bucket isolation in SeaweedFS storage

Configuration:

```csv
# config.csv
shard-strategy,tenant
shard-auto-provision,true
shard-isolation-level,database
```

**Advantages:**
- Complete data isolation (compliance friendly)
- Easy backup/restore per tenant
- Simple to understand
- No cross-tenant queries

**Disadvantages:**
- More resources per tenant
- Complex tenant migration
- Connection pool overhead

### Option 2: Hash-Based Sharding

Distribute by user/session ID hash:

```
user_id = 12345
shard = hash(12345) % num_shards = 2
→ Route to shard-2
```

Configuration:

```csv
# config.csv
shard-strategy,hash
shard-count,4
shard-key,user_id
shard-algorithm,consistent-hash
```

**Advantages:**
- Even distribution
- Predictable routing
- Good for high-volume single-tenant

**Disadvantages:**
- Resharding is complex
- Cross-shard queries difficult
- No tenant isolation

### Option 3: Time-Based Sharding

For time-series data (logs, analytics):

```csv
# config.csv
shard-strategy,time
shard-interval,monthly
shard-retention-months,12
shard-auto-archive,true
```

Automatically creates partitions:

```
messages_2024_01
messages_2024_02
messages_2024_03
...
```

### Option 4: Geographic Sharding

Route by user location:

```csv
# config.csv
shard-strategy,geo
shard-regions,us-east,eu-west,ap-south
shard-default,us-east
shard-detection,ip
```
## Geographic Distribution

Global router uses GeoIP to direct users to the nearest regional cluster:

| Region | Location | Services |
|--------|----------|----------|
| US-East | Virginia | Full cluster |
| EU-West | Frankfurt | Full cluster |
| AP-South | Singapore | Full cluster |

Each regional cluster runs independently with data replication between regions for disaster recovery.

## Auto-Scaling with LXC

### Configuration

```csv
# config.csv - Auto-scaling settings
scale-enabled,true
scale-min-instances,1
scale-max-instances,10
scale-cpu-threshold,70
scale-memory-threshold,80
scale-request-threshold,1000
scale-cooldown-seconds,300
scale-check-interval,30
```

### Scaling Rules

| Metric | Scale Up | Scale Down |
|--------|----------|------------|
| CPU | > 70% for 2 min | < 30% for 5 min |
| Memory | > 80% for 2 min | < 40% for 5 min |
| Requests/sec | > 1000 | < 200 |
| Response time | > 2000ms | < 500ms |
| Queue depth | > 100 | < 10 |

### Auto-Scale Service

The auto-scaler runs as a systemd service:

```ini
# /etc/systemd/system/gbo-autoscale.service
[Unit]
Description=General Bots Auto-Scaler
After=network.target

[Service]
Type=simple
ExecStart=/opt/gbo/scripts/autoscale.sh
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

## Container Lifecycle

**Startup Flow:**
1. **Create** → LXC container created from template
2. **Configure** → Resources allocated (CPU, memory, storage)
3. **Start** → BotServer binary launched
4. **Ready** → Added to load balancer pool

**Shutdown Flow:**
1. **Active** → Container serving requests
2. **Drain** → Stop accepting new connections
3. **Stop** → Graceful BotServer shutdown
4. **Delete** → Container removed (or returned to pool)

## Load Balancing

### Caddy Configuration

```caddyfile
{
    admin off
    auto_https on
}

bot.example.com {
    # Rate limiting
    rate_limit {
        zone api {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # WebSocket (sticky sessions)
    handle /ws* {
        reverse_proxy botserver-1:8080 botserver-2:8080 {
            lb_policy cookie
            health_uri /api/health
            health_interval 10s
        }
    }
    
    # API (round robin)
    handle /api/* {
        reverse_proxy botserver-1:8080 botserver-2:8080 {
            lb_policy round_robin
            fail_duration 30s
        }
    }
}
```

### Rate Limiting Configuration

```csv
# config.csv - Rate limiting
rate-limit-enabled,true
rate-limit-requests,100
rate-limit-window,60
rate-limit-burst,20
rate-limit-by,ip

# Per-endpoint limits
rate-limit-api-chat,30
rate-limit-api-files,50
rate-limit-api-auth,10
rate-limit-api-llm,20
```

## Failover Systems

### Health Checks

Every service exposes `/health`:

```json
{
  "status": "healthy",
  "version": "6.1.0",
  "checks": {
    "database": {"status": "ok", "latency_ms": 5},
    "cache": {"status": "ok", "latency_ms": 2},
    "vectordb": {"status": "ok", "latency_ms": 10},
    "llm": {"status": "ok", "latency_ms": 50}
  }
}
```

### Circuit Breaker

```csv
# config.csv
circuit-breaker-enabled,true
circuit-breaker-threshold,5
circuit-breaker-timeout,30
circuit-breaker-half-open-requests,3
```

States:
- **Closed**: Normal operation, counting failures
- **Open**: Failing fast, returning errors immediately
- **Half-Open**: Testing with limited requests

### Database Failover

PostgreSQL with streaming replication:
## Database Replication

PostgreSQL replication is managed by Patroni for automatic failover:

| Component | Role | Description |
|-----------|------|-------------|
| Primary | Write leader | Handles all write operations |
| Replica | Read replica | Synchronous replication from primary |
| Patroni | Failover manager | Automatic leader election on failure |

Failover happens automatically within seconds, with clients redirected via connection pooler.

### Graceful Degradation

```csv
# config.csv - Fallbacks
fallback-llm-enabled,true
fallback-llm-provider,local
fallback-llm-model,DeepSeek-R1-Distill-Qwen-1.5B

fallback-cache-enabled,true
fallback-cache-mode,memory

fallback-vectordb-enabled,true
fallback-vectordb-mode,keyword-search
```

## Secrets Management (Vault)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         .env (minimal)                          │
│         VAULT_ADDR=https://localhost:8200                       │
│         VAULT_TOKEN=hvs.xxxxxxxxxxxxx                           │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Vault Server                            │
│                                                                 │
│  gbo/drive        → accesskey, secret                           │
│  gbo/tables       → username, password                          │
│  gbo/cache        → password                                    │
│  gbo/directory    → client_id, client_secret                    │
│  gbo/email        → username, password                          │
│  gbo/llm          → openai_key, anthropic_key, groq_key         │
│  gbo/encryption   → master_key, data_key                        │
│  gbo/meet         → api_key, api_secret                         │
│  gbo/alm          → admin_password, runner_token                │
└─────────────────────────────────────────────────────────────────┘
```

### Zitadel vs Vault

| Purpose | Zitadel | Vault |
|---------|---------|-------|
| User authentication | Yes | No |
| Service credentials | No | Yes |
| API keys | No | Yes |
| Encryption keys | No | Yes |
| OAuth/OIDC | Yes | No |
| MFA | Yes | No |

**Use both:**
- Zitadel: User identity, SSO, MFA
- Vault: Service secrets, encryption keys

### Minimal .env with Vault

```bash
# .env - Only Vault and Directory needed
VAULT_ADDR=https://localhost:8200
VAULT_TOKEN=hvs.your-token-here

# Directory for user auth (Zitadel)
DIRECTORY_URL=https://localhost:8080
DIRECTORY_PROJECT_ID=your-project-id

# All other secrets fetched from Vault at runtime
```

## Observability

### Option 1: InfluxDB + Grafana (Current)

For time-series metrics:

```csv
# config.csv
observability-provider,influxdb
observability-url,http://localhost:8086
observability-org,pragmatismo
observability-bucket,metrics
```

### Option 2: Vector + InfluxDB (Recommended)

Vector as log/metric aggregator:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  BotServer  │ ──▶ │   Vector    │ ──▶ │  InfluxDB   │
│    Logs     │     │  (Pipeline) │     │  (Metrics)  │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │   Grafana   │
                    │ (Dashboard) │
                    └─────────────┘
```

Vector configuration:

```toml
# vector.toml
[sources.botserver_logs]
type = "file"
include = ["/opt/gbo/logs/*.log"]

[transforms.parse_logs]
type = "remap"
inputs = ["botserver_logs"]
source = '''
. = parse_json!(.message)
'''

[sinks.influxdb]
type = "influxdb_metrics"
inputs = ["parse_logs"]
endpoint = "http://localhost:8086"
org = "pragmatismo"
bucket = "metrics"
```

### Replacing log.* Calls with Vector

Instead of replacing all log calls, configure Vector to:

1. Collect logs from files
2. Parse and enrich
3. Route to appropriate sinks

```toml
# Route errors to alerts
[transforms.filter_errors]
type = "filter"
inputs = ["parse_logs"]
condition = '.level == "error"'

[sinks.alertmanager]
type = "http"
inputs = ["filter_errors"]
uri = "http://alertmanager:9093/api/v1/alerts"
```

## Full-Text Search: Tantivy vs Qdrant

### Comparison

| Feature | Tantivy | Qdrant |
|---------|---------|--------|
| Type | Full-text search | Vector search |
| Query | Keywords, boolean | Semantic similarity |
| Results | Exact matches | Similar meanings |
| Speed | Very fast | Fast |
| Use case | Known keywords | Natural language |

### Do You Need Tantivy?

**Usually no.** Qdrant handles both:
- Vector similarity search (semantic)
- Payload filtering (keyword-like)

Use Tantivy only if you need:
- BM25 ranking
- Complex boolean queries
- Phrase matching
- Faceted search

### Hybrid Search with Qdrant

Qdrant supports hybrid search:

```rust
// Combine vector similarity + keyword filter
let search_request = SearchPoints {
    collection_name: "kb".to_string(),
    vector: query_embedding,
    limit: 10,
    filter: Some(Filter {
        must: vec![
            Condition::Field(FieldCondition {
                key: "content".to_string(),
                r#match: Some(Match::Text("keyword".to_string())),
            }),
        ],
        ..Default::default()
    }),
    ..Default::default()
};
```

## Workflow Scheduling: Temporal

### When to Use Temporal

Temporal is useful for:
- Long-running workflows (hours/days)
- Retry logic with exponential backoff
- Workflow versioning
- Distributed transactions

### Current Alternative: SET SCHEDULE

For simple scheduling, General Bots uses:

```basic
REM Run every day at 9 AM
SET SCHEDULE "daily-report" TO "0 9 * * *"
    TALK "Running daily report..."
    result = GET "/api/reports/daily"
    SEND MAIL "admin@example.com", "Daily Report", result
END SCHEDULE
```

### Adding Temporal

If you need complex workflows:

```csv
# config.csv
workflow-provider,temporal
workflow-server,localhost:7233
workflow-namespace,botserver
```

Example workflow:

```basic
REM Temporal workflow
START WORKFLOW "onboarding"
    STEP "welcome"
        SEND MAIL user_email, "Welcome!", "Welcome to our service"
        WAIT 1, "day"
    
    STEP "followup"
        IF NOT user_completed_profile THEN
            SEND MAIL user_email, "Complete Profile", "..."
            WAIT 3, "days"
        END IF
    
    STEP "activation"
        IF user_completed_profile THEN
            CALL activate_user(user_id)
        END IF
END WORKFLOW
```

## MFA with Zitadel

### Configuration

MFA is handled transparently by Zitadel:

```csv
# config.csv
auth-mfa-enabled,true
auth-mfa-methods,totp,sms,email,whatsapp
auth-mfa-required-for,admin,sensitive-operations
auth-mfa-grace-period-days,7
```

### Zitadel MFA Settings

In Zitadel console:
1. Go to Settings → Login Behavior
2. Enable "Multi-Factor Authentication"
3. Select allowed methods:
   - TOTP (authenticator apps)
   - SMS
   - Email
   - WebAuthn/FIDO2

### WhatsApp MFA Channel

```csv
# config.csv
auth-mfa-whatsapp-enabled,true
auth-mfa-whatsapp-provider,twilio
auth-mfa-whatsapp-template,mfa_code
```

Flow:
1. User logs in with password
2. Zitadel triggers MFA
3. Code sent via WhatsApp
4. User enters code
5. Session established

## Summary: What You Need

| Component | Required | Recommended | Optional |
|-----------|----------|-------------|----------|
| PostgreSQL | Yes | - | - |
| Redis | Yes | - | - |
| Qdrant | Yes | - | - |
| MinIO | Yes | - | - |
| Zitadel | Yes | - | - |
| Vault | - | Yes | - |
| InfluxDB | - | Yes | - |
| LiveKit | - | Yes | - |
| Vector | - | - | Yes |
| Kafka | - | - | Yes |
| RabbitMQ | - | - | Yes |
| Temporal | - | - | Yes |
| GStreamer | - | - | Yes |
| Tantivy | - | - | Yes |

## Next Steps

- [Scaling and Load Balancing](./scaling.md) - Detailed scaling guide
- [Container Deployment](./containers.md) - LXC setup
- [Security Features](../chapter-12-auth/security-features.md) - Security deep dive
- [LLM Providers](../appendix-external-services/llm-providers.md) - Model selection