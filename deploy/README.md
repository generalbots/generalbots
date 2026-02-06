# Deployment Guide

## Overview

This directory contains deployment configurations and scripts for General Bots in production environments.

## Deployment Methods

### 1. Traditional Server Deployment

#### Prerequisites
- Server with Linux (Ubuntu 20.04+ recommended)
- Rust 1.70+ toolchain
- PostgreSQL, Redis, Qdrant installed or managed by botserver
- At least 4GB RAM, 2 CPU cores

#### Steps

1. **Build Release Binaries:**
```bash
cargo build --release -p botserver -p botui
```

2. **Deploy to Production:**
```bash
# Copy binaries
sudo cp target/release/botserver /opt/gbo/bin/
sudo cp target/release/botui /opt/gbo/bin/

# Deploy UI files
./botserver/deploy/deploy-ui.sh /opt/gbo

# Set permissions
sudo chmod +x /opt/gbo/bin/botserver
sudo chmod +x /opt/gbo/bin/botui
```

3. **Configure Environment:**
```bash
# Copy and edit environment file
cp botserver/.env.example /opt/gbo/.env
nano /opt/gbo/.env
```

4. **Start Services:**
```bash
# Using systemd (recommended)
sudo systemctl start botserver
sudo systemctl start botui

# Or manually
/opt/gbo/bin/botserver --noconsole
/opt/gbo/bin/botui
```

### 2. Kubernetes Deployment

#### Prerequisites
- Kubernetes cluster 1.24+
- kubectl configured
- Persistent volumes provisioned

#### Steps

1. **Create Namespace:**
```bash
kubectl create namespace generalbots
```

2. **Deploy UI Files:**
```bash
# Create ConfigMap with UI files
kubectl create configmap botui-files \
  --from-file=botui/ui/suite/ \
  -n generalbots
```

3. **Apply Deployment:**
```bash
kubectl apply -f botserver/deploy/kubernetes/deployment.yaml
```

4. **Verify Deployment:**
```bash
kubectl get pods -n generalbots
kubectl logs -f deployment/botserver -n generalbots
```

## Troubleshooting

### UI Files Not Found Error

**Symptom:**
```
Asset 'suite/index.html' not found in embedded binary, falling back to filesystem
Failed to load suite UI: No such file or directory
```

**Solution:**

**For Traditional Deployment:**
```bash
# Run the deployment script
./botserver/deploy/deploy-ui.sh /opt/gbo

# Verify files exist
ls -la /opt/gbo/bin/ui/suite/index.html
```

**For Kubernetes:**
```bash
# Recreate UI ConfigMap
kubectl delete configmap botui-files -n generalbots
kubectl create configmap botui-files \
  --from-file=botui/ui/suite/ \
  -n generalbots

# Restart pods
kubectl rollout restart deployment/botserver -n generalbots
```

### Port Already in Use

```bash
# Find process using port
lsof -ti:8088 | xargs kill -9
lsof -ti:3000 | xargs kill -9
```

### Permission Denied

```bash
# Fix ownership and permissions
sudo chown -R gbo:gbo /opt/gbo
sudo chmod -R 755 /opt/gbo/bin
```

## Maintenance

### Update UI Files

**Traditional:**
```bash
./botserver/deploy/deploy-ui.sh /opt/gbo
sudo systemctl restart botui
```

**Kubernetes:**
```bash
kubectl create configmap botui-files \
  --from-file=botui/ui/suite/ \
  -n generalbots \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl rollout restart deployment/botserver -n generalbots
```

### Update Binaries

1. Build new release
2. Stop services
3. Replace binaries
4. Start services

### Backup

```bash
# Backup database
pg_dump -U postgres -d gb > backup.sql

# Backup UI files (if customized)
tar -czf ui-backup.tar.gz /opt/gbo/bin/ui/

# Backup configuration
cp /opt/gbo/.env /opt/gbo/.env.backup
```

## Monitoring

### Check Logs

**Traditional:**
```bash
tail -f /opt/gbo/logs/botserver.log
tail -f /opt/gbo/logs/botui.log
```

**Kubernetes:**
```bash
kubectl logs -f deployment/botserver -n generalbots
```

### Health Checks

```bash
# Check server health
curl http://localhost:8088/health

# Check botui health
curl http://localhost:3000/health
```

## Security

- Always use HTTPS in production
- Rotate secrets regularly
- Update dependencies monthly
- Review logs for suspicious activity
- Use firewall to restrict access

## Support

For issues or questions:
- Documentation: https://docs.pragmatismo.com.br
- GitHub Issues: https://github.com/GeneralBots/BotServer/issues