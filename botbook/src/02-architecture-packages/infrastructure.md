# Infrastructure Design

This chapter covers the complete infrastructure design for General Bots, including scaling, security, secrets management, observability, and high availability.

## Architecture Overview

General Bots uses a modular architecture where each component runs in isolated LXC containers. This provides isolation where each service has its own filesystem and process space, scalability through adding more containers to handle increased load, security since compromised components cannot affect others, and portability allowing containers to move between hosts easily.

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

MinIO server-side encryption is enabled using SSE-S3 for automatic encryption or SSE-C for customer-managed keys:

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

Redis with TLS and encrypted RDB provides secure caching:

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

Qdrant with encrypted storage uses TLS for transport and filesystem-level encryption for data at rest:

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

## Media Processing: LiveKit

LiveKit handles all media processing needs for General Bots. WebRTC is native to LiveKit. Recording is built-in via the Egress service. Transcoding uses the Egress service. Streaming and AI integration are built into LiveKit.

LiveKit's Egress service handles room recording, participant recording, livestreaming to YouTube and Twitch, and track composition.

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

## Messaging: Redis

General Bots uses Redis for all messaging needs including session state, PubSub for real-time communication, and Streams for persistence:

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

Each tenant or organization gets isolated databases.

## Multi-Tenant Architecture

Each tenant gets isolated resources with dedicated database schemas, cache namespaces, and vector collections. The router maps tenant IDs to their respective data stores automatically.

Key isolation features include database-per-tenant or schema-per-tenant options, namespace isolation in Valkey cache, collection isolation in Qdrant vectors, and bucket isolation in SeaweedFS storage.

Configuration:

```csv
# config.csv
shard-strategy,tenant
shard-auto-provision,true
shard-isolation-level,database
```

Advantages include complete data isolation which is compliance friendly, easy backup and restore per tenant, simplicity, and no cross-tenant queries. Disadvantages include more resources per tenant, complex tenant migration, and connection pool overhead.

### Option 2: Hash-Based Sharding

Distribute by user or session ID hash. For example, a user_id of 12345 produces a hash that modulo num_shards equals 2, routing to shard-2.

Configuration:

```csv
# config.csv
shard-strategy,hash
shard-count,4
shard-key,user_id
shard-algorithm,consistent-hash
```

Advantages include even distribution, predictable routing, and good performance for high-volume single-tenant deployments. Disadvantages include complex resharding, difficult cross-shard queries, and no tenant isolation.

### Option 3: Time-Based Sharding

For time-series data like logs and analytics:

```csv
# config.csv
shard-strategy,time
shard-interval,monthly
shard-retention-months,12
shard-auto-archive,true
```

This automatically creates partitions named messages_2024_01, messages_2024_02, messages_2024_03, and so on.

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

The global router uses GeoIP to direct users to the nearest regional cluster. US-East in Virginia runs a full cluster, EU-West in Frankfurt runs a full cluster, and AP-South in Singapore runs a full cluster. Each regional cluster runs independently with data replication between regions for disaster recovery.

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

The startup flow begins with creating the LXC container from a template, then configuring resources for CPU, memory, and storage, then starting the botserver binary, and finally marking the container as ready and adding it to the load balancer pool.

The shutdown flow begins with an active container serving requests, then draining to stop accepting new connections, then stopping with a graceful botserver shutdown, and finally deleting or returning the container to the pool.

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
        reverse_proxy botserver-1:9000 botserver-2:9000 {
            lb_policy cookie
            health_uri /api/health
            health_interval 10s
        }
    }
    
    # API (round robin)
    handle /api/* {
        reverse_proxy botserver-1:9000 botserver-2:9000 {
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

The circuit breaker has three states. Closed represents normal operation while counting failures. Open means failing fast and returning errors immediately. Half-Open tests with limited requests before deciding to close or reopen.

### Database Failover

PostgreSQL with streaming replication provides high availability.

## Database Replication

PostgreSQL replication is managed by Patroni for automatic failover. The Primary serves as the write leader handling all write operations. The Replica provides synchronous replication from the primary for read scaling. Patroni acts as the failover manager performing automatic leader election on failure.

Failover happens automatically within seconds, with clients redirected via the connection pooler.

### Graceful Degradation

```csv
# config.csv - Fallbacks
fallback-llm-enabled,true
fallback-llm-provider,local
fallback-llm-model,DeepSeek-R3-Distill-Qwen-1.5B

fallback-cache-enabled,true
fallback-cache-mode,memory

fallback-vectordb-enabled,true
fallback-vectordb-mode,keyword-search
```

## Secrets Management (Vault)

### Architecture

The minimal `.env` file contains only Vault connection details. All other secrets are stored in Vault and fetched at runtime. The Vault server stores secrets organized by path including gbo/drive for access keys, gbo/tables for database credentials, gbo/cache for passwords, gbo/directory for client credentials, gbo/email for mail credentials, gbo/llm for provider API keys, gbo/encryption for master and data keys, and gbo/meet for API credentials.

### Zitadel vs Vault

Zitadel handles user authentication, OAuth/OIDC, and MFA. Vault handles service credentials, API keys, and encryption keys. Use both together where Zitadel manages user identity and SSO while Vault manages service secrets and encryption keys.

### Minimal .env with Vault

```bash
# .env - Only Vault and Directory needed
VAULT_ADDR=https://localhost:8200
VAULT_TOKEN=hvs.your-token-here

# Directory for user auth (Zitadel)
DIRECTORY_URL=https://localhost:9000
DIRECTORY_CLIENT_ID=your-client-id
DIRECTORY_CLIENT_SECRET=your-client-secret

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

Vector serves as a log and metric aggregator. botserver logs flow to Vector which pipelines them to InfluxDB for metrics storage and Grafana for dashboards.

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

Instead of replacing all log calls, configure Vector to collect logs from files, parse and enrich them, and route to appropriate sinks:

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

## Search: Qdrant

Qdrant handles all search needs in General Bots, providing both vector similarity search for semantic queries and payload filtering for keyword-like queries.

### Hybrid Search with Qdrant

Qdrant supports hybrid search combining vector similarity with keyword filters:

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

## Workflow Scheduling: SET SCHEDULE

General Bots uses the SET SCHEDULE keyword for all scheduling needs:

```basic
REM Run every day at 9 AM
SET SCHEDULE "daily-report" TO "0 9 * * *"
    TALK "Running daily report..."
    result = GET "/api/reports/daily"
    SEND MAIL "admin@example.com", "Daily Report", result
END SCHEDULE
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

In the Zitadel console, navigate to Settings then Login Behavior. Enable Multi-Factor Authentication and select allowed methods including TOTP for authenticator apps, SMS, Email, and WebAuthn/FIDO2.

### WhatsApp MFA Channel

```csv
# config.csv
auth-mfa-whatsapp-enabled,true
auth-mfa-whatsapp-provider,twilio
auth-mfa-whatsapp-template,mfa_code
```

The flow proceeds as follows: the user logs in with password, Zitadel triggers MFA, a code is sent via WhatsApp, the user enters the code, and the session is established.

## Summary: What You Need

PostgreSQL, Redis, Qdrant, MinIO, and Zitadel are required components. Vault, InfluxDB, and LiveKit are recommended for production deployments. Vector is optional for log aggregation.

## Next Steps

The Scaling and Load Balancing chapter provides a detailed scaling guide. The Container Deployment chapter covers LXC setup. The Security Features chapter offers a security deep dive. The LLM Providers appendix helps with model selection.