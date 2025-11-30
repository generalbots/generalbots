# Docker Deployment

General Bots supports multiple Docker deployment strategies to fit your infrastructure needs. This guide covers all available options from single-container deployments to full orchestrated environments.

> **Note**: Docker support is currently **experimental**. While functional, some features may change in future releases.

## Deployment Options Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      DEPLOYMENT OPTIONS                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Option 1: All-in-One Container                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  botserver container                                             │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │   │
│  │  │PostgreSQL│ │  MinIO  │ │ Qdrant  │ │  Vault  │ │BotServer│  │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Option 2: Microservices (Separate Containers)                         │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐              │
│  │ PostgreSQL│ │   MinIO   │ │  Qdrant   │ │   Vault   │              │
│  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘              │
│        │             │             │             │                      │
│        └─────────────┴──────┬──────┴─────────────┘                      │
│                             │                                           │
│                      ┌──────┴──────┐                                    │
│                      │  BotServer  │                                    │
│                      └─────────────┘                                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Option 1: All-in-One Container

The simplest deployment option runs everything inside a single container. This is ideal for:
- Development environments
- Small deployments
- Quick testing
- Resource-constrained environments

### Quick Start

```bash
docker run -d \
  --name botserver \
  -p 8000:8000 \
  -p 9000:9000 \
  -v botserver-data:/opt/gbo/data \
  -e ADMIN_PASS=your-secure-password \
  pragmatismo/botserver:latest
```

### Docker Compose (All-in-One)

```yaml
version: '3.8'

services:
  botserver:
    image: pragmatismo/botserver:latest
    container_name: botserver
    restart: unless-stopped
    ports:
      - "8000:8000"   # Main API
      - "9000:9000"   # MinIO/Drive
      - "9001:9001"   # MinIO Console
    volumes:
      - botserver-data:/opt/gbo/data
      - botserver-conf:/opt/gbo/conf
      - botserver-logs:/opt/gbo/logs
      - ./work:/opt/gbo/work  # Your bot packages
    environment:
      - ADMIN_PASS=${ADMIN_PASS:-changeme}
      - DOMAIN=${DOMAIN:-localhost}
      - TZ=UTC
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  botserver-data:
  botserver-conf:
  botserver-logs:
```

### Resource Requirements (All-in-One)

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4GB | 8GB+ |
| Storage | 20GB | 50GB+ |

## Option 2: Microservices Deployment

For production environments, we recommend running each component as a separate container. This provides:
- Independent scaling
- Better resource allocation
- Easier maintenance and updates
- High availability options

### Docker Compose (Microservices)

```yaml
version: '3.8'

services:
  # PostgreSQL - Primary Database
  postgres:
    image: postgres:16-alpine
    container_name: gb-postgres
    restart: unless-stopped
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      POSTGRES_USER: botserver
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: botserver
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U botserver"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - gb-network

  # MinIO - Object Storage / Drive
  minio:
    image: minio/minio:latest
    container_name: gb-minio
    restart: unless-stopped
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio-data:/data
    environment:
      MINIO_ROOT_USER: ${DRIVE_ACCESSKEY}
      MINIO_ROOT_PASSWORD: ${DRIVE_SECRET}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 30s
      timeout: 20s
      retries: 3
    networks:
      - gb-network

  # Qdrant - Vector Database
  qdrant:
    image: qdrant/qdrant:latest
    container_name: gb-qdrant
    restart: unless-stopped
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant-data:/qdrant/storage
    environment:
      QDRANT__SERVICE__GRPC_PORT: 6334
    networks:
      - gb-network

  # Vault - Secrets Management
  vault:
    image: hashicorp/vault:latest
    container_name: gb-vault
    restart: unless-stopped
    cap_add:
      - IPC_LOCK
    ports:
      - "8200:8200"
    volumes:
      - vault-data:/vault/data
      - ./vault-config:/vault/config
    environment:
      VAULT_ADDR: http://127.0.0.1:8200
    command: server -config=/vault/config/config.hcl
    networks:
      - gb-network

  # Redis - Caching (Optional but recommended)
  redis:
    image: redis:7-alpine
    container_name: gb-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    networks:
      - gb-network

  # InfluxDB - Time Series (Optional - for analytics)
  influxdb:
    image: influxdb:2.7-alpine
    container_name: gb-influxdb
    restart: unless-stopped
    ports:
      - "8086:8086"
    volumes:
      - influxdb-data:/var/lib/influxdb2
    environment:
      DOCKER_INFLUXDB_INIT_MODE: setup
      DOCKER_INFLUXDB_INIT_USERNAME: admin
      DOCKER_INFLUXDB_INIT_PASSWORD: ${INFLUX_PASSWORD}
      DOCKER_INFLUXDB_INIT_ORG: pragmatismo
      DOCKER_INFLUXDB_INIT_BUCKET: metrics
    networks:
      - gb-network

  # BotServer - Main Application
  botserver:
    image: pragmatismo/botserver:latest
    container_name: gb-botserver
    restart: unless-stopped
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy
      qdrant:
        condition: service_started
    ports:
      - "8000:8000"
    volumes:
      - ./work:/opt/gbo/work
      - botserver-logs:/opt/gbo/logs
    environment:
      # Database
      DATABASE_URL: postgres://botserver:${DB_PASSWORD}@postgres:5432/botserver
      
      # Drive/Storage
      DRIVE_URL: http://minio:9000
      DRIVE_ACCESSKEY: ${DRIVE_ACCESSKEY}
      DRIVE_SECRET: ${DRIVE_SECRET}
      
      # Vector DB
      QDRANT_URL: http://qdrant:6333
      
      # Vault
      VAULT_ADDR: http://vault:8200
      VAULT_TOKEN: ${VAULT_TOKEN}
      
      # Redis
      REDIS_URL: redis://redis:6379
      
      # InfluxDB
      INFLUX_URL: http://influxdb:8086
      INFLUX_TOKEN: ${INFLUX_TOKEN}
      INFLUX_ORG: pragmatismo
      INFLUX_BUCKET: metrics
      
      # General
      ADMIN_PASS: ${ADMIN_PASS}
      DOMAIN: ${DOMAIN}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - gb-network

networks:
  gb-network:
    driver: bridge

volumes:
  postgres-data:
  minio-data:
  qdrant-data:
  vault-data:
  redis-data:
  influxdb-data:
  botserver-logs:
```

### Environment File (.env)

Create a `.env` file with your configuration:

```bash
# Database
DB_PASSWORD=your-secure-db-password

# Drive/MinIO
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=your-minio-secret

# Vault
VAULT_TOKEN=your-vault-token

# InfluxDB
INFLUX_PASSWORD=your-influx-password
INFLUX_TOKEN=your-influx-token

# General
ADMIN_PASS=your-admin-password
DOMAIN=your-domain.com
```

## Building Custom Images

### Dockerfile for BotServer

```dockerfile
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/gbo

COPY --from=builder /app/target/release/botserver /opt/gbo/bin/
COPY --from=builder /app/templates /opt/gbo/templates/
COPY --from=builder /app/ui /opt/gbo/ui/

ENV PATH="/opt/gbo/bin:${PATH}"

EXPOSE 8000

CMD ["botserver"]
```

### Multi-Architecture Build

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t pragmatismo/botserver:latest \
  --push .
```

## Kubernetes Deployment

For large-scale deployments, use Kubernetes:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: botserver
  labels:
    app: botserver
spec:
  replicas: 3
  selector:
    matchLabels:
      app: botserver
  template:
    metadata:
      labels:
        app: botserver
    spec:
      containers:
      - name: botserver
        image: pragmatismo/botserver:latest
        ports:
        - containerPort: 8000
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: botserver-secrets
              key: database-url
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: botserver
spec:
  selector:
    app: botserver
  ports:
  - port: 80
    targetPort: 8000
  type: LoadBalancer
```

## Health Checks and Monitoring

All containers expose health endpoints:

| Service | Health Endpoint |
|---------|-----------------|
| BotServer | `GET /health` |
| PostgreSQL | `pg_isready` command |
| MinIO | `GET /minio/health/live` |
| Qdrant | `GET /health` |
| Vault | `GET /v1/sys/health` |
| Redis | `redis-cli ping` |
| InfluxDB | `GET /health` |

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs gb-botserver

# Check if dependencies are running
docker ps

# Verify network connectivity
docker network inspect gb-network
```

### Database Connection Issues

```bash
# Test database connection from botserver container
docker exec -it gb-botserver psql $DATABASE_URL -c "SELECT 1"

# Check PostgreSQL logs
docker logs gb-postgres
```

### Storage Issues

```bash
# Check MinIO status
docker exec -it gb-minio mc admin info local

# Check volume mounts
docker inspect gb-botserver | jq '.[0].Mounts'
```

### Memory Issues

If containers are being killed due to OOM:

```yaml
# Increase memory limits in docker-compose.yml
services:
  botserver:
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G
```

## Migration from Non-Docker

To migrate an existing installation to Docker:

1. **Backup your data**:
   ```bash
   pg_dump botserver > backup.sql
   mc cp --recursive /path/to/drive minio/backup/
   ```

2. **Start Docker containers**

3. **Restore data**:
   ```bash
   docker exec -i gb-postgres psql -U botserver < backup.sql
   docker exec -it gb-minio mc cp --recursive /backup minio/drive/
   ```

4. **Copy bot packages** to the `work` volume

5. **Verify** everything works via the health endpoints

## Next Steps

- [Scaling and Load Balancing](./scaling.md)
- [Infrastructure Design](./infrastructure.md)
- [Observability](./observability.md)