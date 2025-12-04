# Deployment Guide

Deploy General Bots in production environments with security, scalability, and reliability.

## Deployment Options

| Method | Best For | Complexity |
|--------|----------|------------|
| Single Server | Small teams, development | Low |
| Docker Compose | Medium deployments | Medium |
| LXC Containers | Isolated multi-tenant | Medium |
| Kubernetes | Large scale, high availability | High |

## Single Server Deployment

### Requirements

- **CPU**: 4+ cores
- **RAM**: 16GB minimum
- **Disk**: 100GB SSD
- **OS**: Ubuntu 22.04 LTS / Debian 12

### Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/GeneralBots/BotServer
cd BotServer
cargo build --release

# Run as service
sudo cp target/release/botserver /usr/local/bin/
sudo cp scripts/botserver.service /etc/systemd/system/
sudo systemctl enable botserver
sudo systemctl start botserver
```

### Systemd Service

```ini
# /etc/systemd/system/botserver.service
[Unit]
Description=General Bots Server
After=network.target postgresql.service

[Service]
Type=simple
User=botserver
Group=botserver
WorkingDirectory=/opt/botserver
ExecStart=/usr/local/bin/botserver --noconsole
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Docker Deployment

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  botserver:
    image: generalbots/botserver:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://bot:password@postgres/botserver
      - REDIS_URL=redis://redis:6379
      - S3_ENDPOINT=http://minio:9000
    depends_on:
      - postgres
      - redis
      - minio
    volumes:
      - ./templates:/app/templates
      - ./data:/app/data
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_USER=bot
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=botserver
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

  minio:
    image: minio/minio
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    volumes:
      - minio_data:/data
    ports:
      - "9001:9001"
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant
    volumes:
      - qdrant_data:/qdrant/storage
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  minio_data:
  qdrant_data:
```

### Start Services

```bash
docker-compose up -d
docker-compose logs -f botserver
```

## LXC Container Deployment

LXC provides lightweight isolation for multi-tenant deployments.

### Create Container

```bash
# Create container
lxc launch ubuntu:22.04 botserver

# Configure resources
lxc config set botserver limits.cpu 4
lxc config set botserver limits.memory 8GB

# Enter container
lxc exec botserver -- bash
```

### Inside Container

```bash
# Install dependencies
apt update && apt install -y curl build-essential

# Install Rust and build
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
git clone https://github.com/GeneralBots/BotServer
cd BotServer
cargo build --release

# Run
./target/release/botserver --container
```

### Container Networking

```bash
# Forward port from host
lxc config device add botserver http proxy \
  listen=tcp:0.0.0.0:8080 \
  connect=tcp:127.0.0.1:8080
```

## Reverse Proxy Setup

### Nginx

```nginx
# /etc/nginx/sites-available/botserver
upstream botserver {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    server_name bot.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name bot.example.com;

    ssl_certificate /etc/letsencrypt/live/bot.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/bot.example.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";

    location / {
        proxy_pass http://botserver;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://botserver;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }
}
```

### Enable Site

```bash
sudo ln -s /etc/nginx/sites-available/botserver /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## SSL Certificates

### Let's Encrypt

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d bot.example.com
```

### Auto-Renewal

```bash
sudo certbot renew --dry-run
```

## Environment Configuration

### Production Environment

```bash
# /opt/botserver/.env
RUST_LOG=warn,botserver=info

# Directory Service (required)
DIRECTORY_URL=https://auth.example.com
DIRECTORY_CLIENT_ID=your-client-id
DIRECTORY_CLIENT_SECRET=your-secret

# Optional overrides
DATABASE_URL=postgres://user:pass@localhost/botserver
REDIS_URL=redis://localhost:6379
S3_ENDPOINT=https://s3.example.com

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_API_RPS=100
```

## Database Setup

### PostgreSQL Production Config

```sql
-- Create database and user
CREATE USER botserver WITH PASSWORD 'secure_password';
CREATE DATABASE botserver OWNER botserver;
GRANT ALL PRIVILEGES ON DATABASE botserver TO botserver;

-- Enable extensions
\c botserver
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
```

### Connection Pooling (PgBouncer)

```ini
# /etc/pgbouncer/pgbouncer.ini
[databases]
botserver = host=localhost dbname=botserver

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = 6432
auth_type = md5
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 20
```

## Monitoring

### Health Check Endpoint

```bash
curl http://localhost:8080/api/health
```

### Prometheus Metrics

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'botserver'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: /metrics
```

### Log Aggregation

```bash
# Stream logs to file
journalctl -u botserver -f >> /var/log/botserver/app.log

# Logrotate config
# /etc/logrotate.d/botserver
/var/log/botserver/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
}
```

## Backup Strategy

### Database Backup

```bash
#!/bin/bash
# /opt/botserver/scripts/backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR=/backups/botserver

# PostgreSQL
pg_dump -U botserver botserver | gzip > $BACKUP_DIR/db_$DATE.sql.gz

# File storage
tar -czf $BACKUP_DIR/files_$DATE.tar.gz /opt/botserver/data

# Retain 30 days
find $BACKUP_DIR -mtime +30 -delete
```

### Cron Schedule

```bash
# Daily backup at 2 AM
0 2 * * * /opt/botserver/scripts/backup.sh
```

## Security Checklist

- [ ] Run as non-root user
- [ ] Enable firewall (only ports 80, 443)
- [ ] Configure SSL/TLS
- [ ] Set secure file permissions
- [ ] Enable rate limiting
- [ ] Configure authentication
- [ ] Regular security updates
- [ ] Audit logging enabled
- [ ] Backup encryption
- [ ] Network isolation for database

## Scaling

### Horizontal Scaling

```yaml
# docker-compose.scale.yml
services:
  botserver:
    deploy:
      replicas: 3
    
  nginx:
    image: nginx
    ports:
      - "80:80"
    volumes:
      - ./nginx-lb.conf:/etc/nginx/nginx.conf
```

### Load Balancer Config

```nginx
upstream botserver_cluster {
    least_conn;
    server botserver1:8080;
    server botserver2:8080;
    server botserver3:8080;
}
```

## Troubleshooting

### Check Service Status

```bash
sudo systemctl status botserver
journalctl -u botserver -n 100
```

### Database Connection Issues

```bash
psql -U botserver -h localhost -d botserver -c "SELECT 1"
```

### Memory Issues

```bash
# Check memory usage
free -h
cat /proc/meminfo | grep -E "MemTotal|MemFree|Cached"

# Increase swap if needed
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

## Updates

### Rolling Update

```bash
# Build new version
cd /opt/botserver
git pull
cargo build --release

# Graceful restart
sudo systemctl reload botserver
```

### Zero-Downtime with Docker

```bash
docker-compose pull
docker-compose up -d --no-deps --build botserver
```
