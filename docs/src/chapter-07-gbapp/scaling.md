# Scaling and Load Balancing

General Bots is designed to scale from a single instance to a distributed cluster using LXC containers. This chapter covers auto-scaling, load balancing, sharding strategies, and failover systems.

## Scaling Architecture

General Bots uses a **horizontal scaling** approach with LXC containers:

```
                    ┌─────────────────┐
                    │   Caddy Proxy   │
                    │  (Load Balancer)│
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  LXC Container  │ │  LXC Container  │ │  LXC Container  │
│   botserver-1   │ │   botserver-2   │ │   botserver-3   │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                   │                   │
         └───────────────────┼───────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   PostgreSQL    │ │     Redis       │ │     Qdrant      │
│   (Primary)     │ │   (Cluster)     │ │   (Cluster)     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## Auto-Scaling Configuration

### config.csv Parameters

Configure auto-scaling behavior in your bot's `config.csv`:

```csv
# Auto-scaling settings
scale-enabled,true
scale-min-instances,1
scale-max-instances,10
scale-cpu-threshold,70
scale-memory-threshold,80
scale-request-threshold,1000
scale-cooldown-seconds,300
scale-check-interval,30
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `scale-enabled` | Enable auto-scaling | `false` |
| `scale-min-instances` | Minimum container count | `1` |
| `scale-max-instances` | Maximum container count | `10` |
| `scale-cpu-threshold` | CPU % to trigger scale-up | `70` |
| `scale-memory-threshold` | Memory % to trigger scale-up | `80` |
| `scale-request-threshold` | Requests/min to trigger scale-up | `1000` |
| `scale-cooldown-seconds` | Wait time between scaling events | `300` |
| `scale-check-interval` | Seconds between metric checks | `30` |

### Scaling Rules

Define custom scaling rules:

```csv
# Scale up when average response time exceeds 2 seconds
scale-rule-response-time,2000
scale-rule-response-action,up

# Scale down when CPU drops below 30%
scale-rule-cpu-low,30
scale-rule-cpu-low-action,down

# Scale up on queue depth
scale-rule-queue-depth,100
scale-rule-queue-action,up
```

## LXC Container Management

### Creating Scaled Instances

```bash
# Create additional botserver containers
for i in {2..5}; do
  lxc launch images:debian/12 botserver-$i
  lxc config device add botserver-$i port-$((8080+i)) proxy \
    listen=tcp:0.0.0.0:$((8080+i)) connect=tcp:127.0.0.1:8080
done
```

### Container Resource Limits

Set resource limits per container:

```bash
# CPU limits (number of cores)
lxc config set botserver-1 limits.cpu 4

# Memory limits
lxc config set botserver-1 limits.memory 8GB

# Disk I/O priority (0-10)
lxc config set botserver-1 limits.disk.priority 5

# Network bandwidth (ingress/egress)
lxc config device set botserver-1 eth0 limits.ingress 100Mbit
lxc config device set botserver-1 eth0 limits.egress 100Mbit
```

### Auto-Scaling Script

Create `/opt/gbo/scripts/autoscale.sh`:

```bash
#!/bin/bash

# Configuration
MIN_INSTANCES=1
MAX_INSTANCES=10
CPU_THRESHOLD=70
SCALE_COOLDOWN=300
LAST_SCALE_FILE="/tmp/last_scale_time"

get_avg_cpu() {
    local total=0
    local count=0
    for container in $(lxc list -c n --format csv | grep "^botserver-"); do
        cpu=$(lxc exec $container -- cat /proc/loadavg | awk '{print $1}')
        total=$(echo "$total + $cpu" | bc)
        count=$((count + 1))
    done
    echo "scale=2; $total / $count * 100" | bc
}

get_instance_count() {
    lxc list -c n --format csv | grep -c "^botserver-"
}

can_scale() {
    if [ ! -f "$LAST_SCALE_FILE" ]; then
        return 0
    fi
    last_scale=$(cat "$LAST_SCALE_FILE")
    now=$(date +%s)
    diff=$((now - last_scale))
    [ $diff -gt $SCALE_COOLDOWN ]
}

scale_up() {
    current=$(get_instance_count)
    if [ $current -ge $MAX_INSTANCES ]; then
        echo "Already at max instances ($MAX_INSTANCES)"
        return 1
    fi
    
    new_id=$((current + 1))
    echo "Scaling up: creating botserver-$new_id"
    
    lxc launch images:debian/12 botserver-$new_id
    lxc config set botserver-$new_id limits.cpu 4
    lxc config set botserver-$new_id limits.memory 8GB
    
    # Copy configuration
    lxc file push /opt/gbo/conf/botserver.env botserver-$new_id/opt/gbo/conf/
    
    # Start botserver
    lxc exec botserver-$new_id -- /opt/gbo/bin/botserver &
    
    # Update load balancer
    update_load_balancer
    
    date +%s > "$LAST_SCALE_FILE"
    echo "Scale up complete"
}

scale_down() {
    current=$(get_instance_count)
    if [ $current -le $MIN_INSTANCES ]; then
        echo "Already at min instances ($MIN_INSTANCES)"
        return 1
    fi
    
    # Remove highest numbered instance
    target="botserver-$current"
    echo "Scaling down: removing $target"
    
    # Drain connections
    lxc exec $target -- /opt/gbo/bin/botserver drain
    sleep 30
    
    # Stop and delete
    lxc stop $target
    lxc delete $target
    
    # Update load balancer
    update_load_balancer
    
    date +%s > "$LAST_SCALE_FILE"
    echo "Scale down complete"
}

update_load_balancer() {
    # Generate upstream list
    upstreams=""
    for container in $(lxc list -c n --format csv | grep "^botserver-"); do
        ip=$(lxc list $container -c 4 --format csv | cut -d' ' -f1)
        upstreams="$upstreams\n        to $ip:8080"
    done
    
    # Update Caddy config
    cat > /opt/gbo/conf/caddy/upstream.conf << EOF
upstream botserver {
    $upstreams
    lb_policy round_robin
    health_uri /api/health
    health_interval 10s
}
EOF
    
    # Reload Caddy
    lxc exec proxy-1 -- caddy reload --config /etc/caddy/Caddyfile
}

# Main loop
while true; do
    avg_cpu=$(get_avg_cpu)
    echo "Average CPU: $avg_cpu%"
    
    if can_scale; then
        if (( $(echo "$avg_cpu > $CPU_THRESHOLD" | bc -l) )); then
            scale_up
        elif (( $(echo "$avg_cpu < 30" | bc -l) )); then
            scale_down
        fi
    fi
    
    sleep 30
done
```

## Load Balancing

### Caddy Configuration

Primary load balancer configuration (`/opt/gbo/conf/caddy/Caddyfile`):

```caddyfile
{
    admin off
    auto_https on
}

(common) {
    encode gzip zstd
    header {
        -Server
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
}

bot.example.com {
    import common
    
    # Health check endpoint (no load balancing)
    handle /api/health {
        reverse_proxy localhost:8080
    }
    
    # WebSocket connections (sticky sessions)
    handle /ws* {
        reverse_proxy botserver-1:8080 botserver-2:8080 botserver-3:8080 {
            lb_policy cookie
            lb_try_duration 5s
            health_uri /api/health
            health_interval 10s
            health_timeout 5s
        }
    }
    
    # API requests (round robin)
    handle /api/* {
        reverse_proxy botserver-1:8080 botserver-2:8080 botserver-3:8080 {
            lb_policy round_robin
            lb_try_duration 5s
            health_uri /api/health
            health_interval 10s
            fail_duration 30s
        }
    }
    
    # Static files (any instance)
    handle {
        reverse_proxy botserver-1:8080 botserver-2:8080 botserver-3:8080 {
            lb_policy first
        }
    }
}
```

### Load Balancing Policies

| Policy | Description | Use Case |
|--------|-------------|----------|
| `round_robin` | Rotate through backends | General API requests |
| `first` | Use first available | Static content |
| `least_conn` | Fewest active connections | Long-running requests |
| `ip_hash` | Consistent by client IP | Session affinity |
| `cookie` | Sticky sessions via cookie | WebSocket, stateful |
| `random` | Random selection | Testing |

### Rate Limiting

Configure rate limits in `config.csv`:

```csv
# Rate limiting
rate-limit-enabled,true
rate-limit-requests,100
rate-limit-window,60
rate-limit-burst,20
rate-limit-by,ip

# Per-endpoint limits
rate-limit-api-chat,30
rate-limit-api-files,50
rate-limit-api-auth,10
```

Rate limiting in Caddy:

```caddyfile
bot.example.com {
    # Global rate limit
    rate_limit {
        zone global {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # Stricter limit for auth endpoints
    handle /api/auth/* {
        rate_limit {
            zone auth {
                key {remote_host}
                events 10
                window 1m
            }
        }
        reverse_proxy botserver:8080
    }
}
```

## Sharding Strategies

### Database Sharding Options

#### Option 1: Tenant-Based Sharding

Each tenant gets their own database:

```
┌─────────────────┐
│   Router/Proxy  │
└────────┬────────┘
         │
    ┌────┴────┬──────────┐
    │         │          │
    ▼         ▼          ▼
┌───────┐ ┌───────┐ ┌───────┐
│Tenant1│ │Tenant2│ │Tenant3│
│  DB   │ │  DB   │ │  DB   │
└───────┘ └───────┘ └───────┘
```

Configuration:

```csv
# Tenant sharding
shard-strategy,tenant
shard-tenant-db-prefix,gb_tenant_
shard-auto-create,true
```

#### Option 2: Hash-Based Sharding

Distribute data by hash of primary key:

```
User ID: 12345
Hash: 12345 % 4 = 1
Shard: shard-1
```

Configuration:

```csv
# Hash sharding
shard-strategy,hash
shard-count,4
shard-key,user_id
shard-algorithm,modulo
```

#### Option 3: Range-Based Sharding

Partition by ID ranges:

```csv
# Range sharding
shard-strategy,range
shard-ranges,0-999999:shard1,1000000-1999999:shard2,2000000-:shard3
```

#### Option 4: Geographic Sharding

Route by user location:

```csv
# Geographic sharding
shard-strategy,geo
shard-geo-us,postgres-us.example.com
shard-geo-eu,postgres-eu.example.com
shard-geo-asia,postgres-asia.example.com
shard-default,postgres-us.example.com
```

### Vector Database Sharding (Qdrant)

Qdrant supports automatic sharding:

```csv
# Qdrant sharding
qdrant-shard-count,4
qdrant-replication-factor,2
qdrant-write-consistency,majority
```

Collection creation with sharding:

```rust
// In vectordb code
let collection_config = CreateCollection {
    collection_name: format!("kb_{}", bot_id),
    vectors_config: VectorsConfig::Single(VectorParams {
        size: 384,
        distance: Distance::Cosine,
    }),
    shard_number: Some(4),
    replication_factor: Some(2),
    write_consistency_factor: Some(1),
    ..Default::default()
};
```

### Redis Cluster

For high-availability caching:

```csv
# Redis cluster
cache-mode,cluster
cache-nodes,redis-1:6379,redis-2:6379,redis-3:6379
cache-replicas,1
```

## Failover Systems

### Health Checks

Configure health check endpoints:

```csv
# Health check configuration
health-enabled,true
health-endpoint,/api/health
health-interval,10
health-timeout,5
health-retries,3
```

Health check response:

```json
{
  "status": "healthy",
  "version": "6.1.0",
  "uptime": 86400,
  "checks": {
    "database": "ok",
    "cache": "ok",
    "vectordb": "ok",
    "llm": "ok"
  },
  "metrics": {
    "cpu": 45.2,
    "memory": 62.1,
    "connections": 150
  }
}
```

### Automatic Failover

#### Database Failover (PostgreSQL)

Using Patroni for PostgreSQL HA:

```yaml
# patroni.yml
scope: botserver-cluster
name: postgres-1

restapi:
  listen: 0.0.0.0:8008
  connect_address: postgres-1:8008

etcd:
  hosts: etcd-1:2379,etcd-2:2379,etcd-3:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576
    postgresql:
      use_pg_rewind: true
      parameters:
        max_connections: 200
        shared_buffers: 2GB

postgresql:
  listen: 0.0.0.0:5432
  connect_address: postgres-1:5432
  data_dir: /var/lib/postgresql/data
  authentication:
    superuser:
      username: postgres
      password: ${POSTGRES_PASSWORD}
    replication:
      username: replicator
      password: ${REPLICATION_PASSWORD}
```

#### Cache Failover (Redis Sentinel)

```csv
# Redis Sentinel configuration
cache-mode,sentinel
cache-sentinel-master,mymaster
cache-sentinel-nodes,sentinel-1:26379,sentinel-2:26379,sentinel-3:26379
```

### Circuit Breaker

Prevent cascade failures:

```csv
# Circuit breaker settings
circuit-breaker-enabled,true
circuit-breaker-threshold,5
circuit-breaker-timeout,30
circuit-breaker-half-open-requests,3
```

States:
- **Closed**: Normal operation
- **Open**: Failing, reject requests immediately
- **Half-Open**: Testing if service recovered

### Graceful Degradation

Configure fallback behavior:

```csv
# Fallback configuration
fallback-llm-enabled,true
fallback-llm-provider,local
fallback-llm-model,DeepSeek-R1-Distill-Qwen-1.5B

fallback-cache-enabled,true
fallback-cache-mode,memory

fallback-vectordb-enabled,true
fallback-vectordb-mode,keyword-search
```

## Monitoring Scaling

### Metrics Collection

Key metrics to monitor:

```csv
# Scaling metrics
metrics-scaling-enabled,true
metrics-container-count,true
metrics-scaling-events,true
metrics-load-distribution,true
```

### Alerting Rules

Configure alerts for scaling issues:

```yaml
# alerting-rules.yml
groups:
  - name: scaling
    rules:
      - alert: HighCPUUsage
        expr: avg(cpu_usage) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          
      - alert: MaxInstancesReached
        expr: container_count >= max_instances
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Maximum instances reached, cannot scale up"
          
      - alert: ScalingFailed
        expr: scaling_errors > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Scaling operation failed"
```

## Best Practices

### Scaling

1. **Start small** - Begin with auto-scaling disabled, monitor patterns first
2. **Set appropriate thresholds** - Too low causes thrashing, too high causes poor performance
3. **Use cooldown periods** - Prevent rapid scale up/down cycles
4. **Test failover** - Regularly test your failover procedures
5. **Monitor costs** - More instances = higher infrastructure costs

### Load Balancing

1. **Use sticky sessions for WebSockets** - Required for real-time features
2. **Enable health checks** - Remove unhealthy instances automatically
3. **Configure timeouts** - Prevent hanging connections
4. **Use connection pooling** - Reduce connection overhead

### Sharding

1. **Choose the right strategy** - Tenant-based is simplest for SaaS
2. **Plan for rebalancing** - Have procedures to move data between shards
3. **Avoid cross-shard queries** - Design to minimize these
4. **Monitor shard balance** - Uneven distribution causes hotspots

## Next Steps

- [Container Deployment](./containers.md) - LXC container basics
- [Architecture Overview](./architecture.md) - System design
- [Monitoring Dashboard](../chapter-04-gbui/monitoring.md) - Observe your cluster