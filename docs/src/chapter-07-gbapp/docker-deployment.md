# Docker Deployment

> **Note**: Docker support is currently **experimental**.

## Deployment Options

| Option | Description | Best For |
|--------|-------------|----------|
| **All-in-One** | Single container with all components | Development, testing |
| **Microservices** | Separate containers per component | Production, scaling |

## Option 1: All-in-One Container

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

### Docker Compose

```yaml
version: '3.8'

services:
  botserver:
    image: pragmatismo/botserver:latest
    restart: unless-stopped
    ports:
      - "8000:8000"
      - "9000:9000"
      - "9001:9001"
    volumes:
      - botserver-data:/opt/gbo/data
      - ./work:/opt/gbo/work
    environment:
      - ADMIN_PASS=${ADMIN_PASS:-changeme}
      - DOMAIN=${DOMAIN:-localhost}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  botserver-data:
```

**Resources:** 2 CPU cores, 4GB RAM minimum

## Option 2: Microservices

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      POSTGRES_USER: botserver
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: botserver
    networks:
      - gb-network

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio-data:/data
    environment:
      MINIO_ROOT_USER: ${DRIVE_ACCESSKEY}
      MINIO_ROOT_PASSWORD: ${DRIVE_SECRET}
    networks:
      - gb-network

  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
    volumes:
      - qdrant-data:/qdrant/storage
    networks:
      - gb-network

  botserver:
    image: pragmatismo/botserver:latest
    depends_on:
      - postgres
      - minio
      - qdrant
    ports:
      - "8000:8000"
    volumes:
      - ./work:/opt/gbo/work
    environment:
      DATABASE_URL: postgres://botserver:${DB_PASSWORD}@postgres:5432/botserver
      DRIVE_URL: http://minio:9000
      DRIVE_ACCESSKEY: ${DRIVE_ACCESSKEY}
      DRIVE_SECRET: ${DRIVE_SECRET}
      QDRANT_URL: http://qdrant:6333
      ADMIN_PASS: ${ADMIN_PASS}
    networks:
      - gb-network

networks:
  gb-network:

volumes:
  postgres-data:
  minio-data:
  qdrant-data:
```

### Environment File (.env)

```bash
DB_PASSWORD=secure-db-password
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=secure-minio-secret
ADMIN_PASS=admin-password
DOMAIN=your-domain.com
```

## Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: botserver
spec:
  replicas: 3
  selector:
    matchLabels:
      app: botserver
  template:
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
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
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

## Health Endpoints

| Service | Endpoint |
|---------|----------|
| BotServer | `GET /health` |
| PostgreSQL | `pg_isready` |
| MinIO | `GET /minio/health/live` |
| Qdrant | `GET /health` |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Container won't start | `docker logs gb-botserver` |
| DB connection failed | `docker exec -it gb-botserver psql $DATABASE_URL -c "SELECT 1"` |
| Memory issues | Increase limits in compose or add `deploy.resources.limits.memory` |

## Migration from Non-Docker

```bash
# 1. Backup data
pg_dump botserver > backup.sql
mc cp --recursive /path/to/drive minio/backup/

# 2. Start Docker containers

# 3. Restore
docker exec -i gb-postgres psql -U botserver < backup.sql
docker exec gb-minio mc cp --recursive /backup minio/drive/
```

## See Also

- [Installation](../chapter-01/installation.md) - Local installation
- [Container Deployment (LXC)](./containers.md) - Linux containers
- [Scaling](./scaling.md) - Load balancing and scaling