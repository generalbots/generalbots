# Backup and Recovery

Protecting your botserver data requires regular backups of databases, configurations, and file storage. This guide covers backup strategies, procedures, and disaster recovery.

---

## What to Backup

| Component | Data Location | Priority | Method |
|-----------|---------------|----------|--------|
| PostgreSQL | `botserver-stack/data/tables/` | **Critical** | pg_dump |
| Vault | `botserver-stack/data/vault/` | **Critical** | Vault snapshot |
| MinIO | `botserver-stack/data/drive/` | **Critical** | mc mirror |
| Configurations | `botserver-stack/conf/` | High | File copy |
| Bot Packages | S3 buckets (*.gbai) | High | mc mirror |
| Models | `botserver-stack/data/llm/` | Medium | File copy |
| Logs | `botserver-stack/logs/` | Low | Optional |

---

## Quick Backup Commands

```bash
# Full backup (all components)
./botserver backup

# Backup specific component
./botserver backup tables
./botserver backup drive
./botserver backup vault

# Backup to specific location
./botserver backup --output /mnt/backup/$(date +%Y%m%d)
```

---

## Database Backup (PostgreSQL)

### Full Database Dump

```bash
# Using pg_dump
pg_dump $DATABASE_URL > backup-$(date +%Y%m%d-%H%M%S).sql

# Compressed backup
pg_dump $DATABASE_URL | gzip > backup-$(date +%Y%m%d).sql.gz

# Custom format (faster restore)
pg_dump -Fc $DATABASE_URL > backup-$(date +%Y%m%d).dump
```

### Incremental Backups with WAL

Enable WAL archiving in `postgresql.conf`:

```ini
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backup/wal/%f'
```

### Automated Database Backup Script

```bash
#!/bin/bash
# backup-database.sh

BACKUP_DIR="/backup/postgres"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d-%H%M%S)

mkdir -p $BACKUP_DIR

# Create backup
pg_dump -Fc $DATABASE_URL > "$BACKUP_DIR/botserver-$DATE.dump"

# Remove old backups
find $BACKUP_DIR -name "*.dump" -mtime +$RETENTION_DAYS -delete

echo "Backup complete: botserver-$DATE.dump"
```

### Database Restore

```bash
# From SQL dump
psql $DATABASE_URL < backup.sql

# From custom format (faster)
pg_restore -d $DATABASE_URL backup.dump

# Drop and recreate (clean restore)
pg_restore -c -d $DATABASE_URL backup.dump
```

---

## Vault Backup

### Snapshot Method

```bash
# Create Vault snapshot
VAULT_ADDR=http://localhost:8200 vault operator raft snapshot save vault-backup-$(date +%Y%m%d).snap
```

### File-Based Backup

```bash
# Stop Vault first
./botserver stop vault

# Copy data directory
tar -czvf vault-data-$(date +%Y%m%d).tar.gz botserver-stack/data/vault/

# Copy unseal keys (store securely!)
cp botserver-stack/conf/vault/init.json /secure/location/
```

### Vault Restore

```bash
# Stop Vault
./botserver stop vault

# Restore data
rm -rf botserver-stack/data/vault/*
tar -xzvf vault-data-backup.tar.gz -C botserver-stack/data/

# Start and unseal
./botserver start vault
./botserver unseal
```

**Warning:** Keep `init.json` (unseal keys and root token) in a secure, separate location!

---

## Object Storage Backup (MinIO)

### Using MinIO Client (mc)

```bash
# Configure mc
mc alias set local http://localhost:9000 $DRIVE_ACCESS_KEY $DRIVE_SECRET_KEY

# Backup all buckets
mc mirror local/ /backup/minio/

# Backup specific bot
mc mirror local/mybot.gbai /backup/bots/mybot.gbai/
```

### Sync to Remote Storage

```bash
# Backup to S3
mc mirror local/ s3/botserver-backup/

# Backup to Backblaze B2
mc mirror local/ b2/botserver-backup/

# Backup to another MinIO
mc mirror local/ remote/botserver-backup/
```

### Restore from Backup

```bash
# Restore all buckets
mc mirror /backup/minio/ local/

# Restore specific bucket
mc mirror /backup/bots/mybot.gbai/ local/mybot.gbai/
```

---

## Configuration Backup

### Full Configuration Backup

```bash
# Backup all configs
tar -czvf config-backup-$(date +%Y%m%d).tar.gz \
    botserver-stack/conf/ \
    3rdparty.toml \
    .env

# Exclude certificates (backup separately with encryption)
tar -czvf config-backup-$(date +%Y%m%d).tar.gz \
    --exclude='certificates' \
    botserver-stack/conf/
```

### Certificate Backup (Encrypted)

```bash
# Backup certificates with encryption
tar -cz botserver-stack/conf/system/certificates/ | \
    gpg --symmetric --cipher-algo AES256 > certs-backup.tar.gz.gpg
```

### Restore Configuration

```bash
# Restore configs
tar -xzvf config-backup.tar.gz

# Restore encrypted certificates
gpg --decrypt certs-backup.tar.gz.gpg | tar -xz
```

---

## Full System Backup

### Complete Backup Script

```bash
#!/bin/bash
# full-backup.sh

set -e

BACKUP_DIR="/backup/botserver/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

echo "Starting full backup to $BACKUP_DIR"

# 1. Database
echo "Backing up database..."
pg_dump -Fc $DATABASE_URL > "$BACKUP_DIR/database.dump"

# 2. Vault snapshot
echo "Backing up Vault..."
VAULT_ADDR=http://localhost:8200 vault operator raft snapshot save "$BACKUP_DIR/vault.snap" 2>/dev/null || \
    tar -czvf "$BACKUP_DIR/vault-data.tar.gz" botserver-stack/data/vault/

# 3. Object storage
echo "Backing up drive..."
mc mirror local/ "$BACKUP_DIR/drive/" --quiet

# 4. Configurations
echo "Backing up configurations..."
tar -czvf "$BACKUP_DIR/config.tar.gz" \
    botserver-stack/conf/ \
    3rdparty.toml \
    .env \
    config/

# 5. Models (optional, large files)
if [ "$1" == "--include-models" ]; then
    echo "Backing up models..."
    tar -czvf "$BACKUP_DIR/models.tar.gz" botserver-stack/data/llm/
fi

# Create manifest
echo "Creating manifest..."
cat > "$BACKUP_DIR/manifest.txt" << EOF
botserver Backup
Date: $(date)
Host: $(hostname)

Contents:
- database.dump: PostgreSQL database
- vault.snap: Vault secrets
- drive/: Object storage contents
- config.tar.gz: Configuration files
EOF

echo "Backup complete: $BACKUP_DIR"
du -sh "$BACKUP_DIR"
```

### Scheduled Backups

Add to crontab:

```bash
# Daily database backup at 2 AM
0 2 * * * /opt/botserver/scripts/backup-database.sh

# Weekly full backup on Sunday at 3 AM
0 3 * * 0 /opt/botserver/scripts/full-backup.sh

# Monthly backup with models
0 4 1 * * /opt/botserver/scripts/full-backup.sh --include-models
```

---

## Disaster Recovery

### Recovery Procedure

1. **Install fresh botserver**
   ```bash
   ./botserver --skip-bootstrap
   ```

2. **Restore configurations**
   ```bash
   tar -xzvf config-backup.tar.gz
   ```

3. **Restore Vault**
   ```bash
   tar -xzvf vault-data.tar.gz
   ./botserver start vault
   ./botserver unseal
   ```

4. **Restore database**
   ```bash
   ./botserver start tables
   pg_restore -d $DATABASE_URL database.dump
   ```

5. **Restore object storage**
   ```bash
   ./botserver start drive
   mc mirror /backup/drive/ local/
   ```

6. **Start remaining services**
   ```bash
   ./botserver start
   ```

7. **Verify**
   ```bash
   ./botserver status
   ./botserver test
   ```

### Recovery Time Objectives

| Scenario | RTO Target | Method |
|----------|------------|--------|
| Single component failure | < 15 min | Restart/restore component |
| Database corruption | < 1 hour | pg_restore from backup |
| Full server failure | < 4 hours | Full restore procedure |
| Data center failure | < 24 hours | Geo-replicated restore |

---

## Backup Verification

### Test Restore Regularly

```bash
# Restore to test environment
./botserver --test-restore /backup/latest/

# Verify database integrity
pg_restore --list database.dump
psql $DATABASE_URL -c "SELECT COUNT(*) FROM bots;"

# Verify drive contents
mc ls local/
```

### Backup Integrity Checks

```bash
# Verify backup file integrity
sha256sum /backup/*/database.dump > /backup/checksums.txt

# Verify on restore
sha256sum -c /backup/checksums.txt
```

---

## Cloud Backup Integration

### AWS S3

```bash
# Configure AWS CLI
aws configure

# Sync backups to S3
aws s3 sync /backup/botserver/ s3://my-backup-bucket/botserver/

# Enable versioning for point-in-time recovery
aws s3api put-bucket-versioning \
    --bucket my-backup-bucket \
    --versioning-configuration Status=Enabled
```

### Backblaze B2

```bash
# Configure rclone
rclone config

# Sync backups
rclone sync /backup/botserver/ b2:my-backup-bucket/botserver/
```

### Encrypted Remote Backup

```bash
# Encrypt before upload
tar -cz /backup/botserver/ | \
    gpg --symmetric --cipher-algo AES256 | \
    aws s3 cp - s3://my-backup-bucket/botserver-$(date +%Y%m%d).tar.gz.gpg
```

---

## Retention Policy

| Backup Type | Retention | Storage |
|-------------|-----------|---------|
| Hourly snapshots | 24 hours | Local |
| Daily backups | 30 days | Local + Remote |
| Weekly backups | 12 weeks | Remote |
| Monthly backups | 12 months | Remote (cold) |
| Yearly backups | 7 years | Archive |

### Cleanup Script

```bash
#!/bin/bash
# cleanup-backups.sh

BACKUP_DIR="/backup/botserver"

# Remove daily backups older than 30 days
find $BACKUP_DIR/daily -mtime +30 -delete

# Remove weekly backups older than 12 weeks
find $BACKUP_DIR/weekly -mtime +84 -delete

# Remove monthly backups older than 12 months
find $BACKUP_DIR/monthly -mtime +365 -delete
```

---

## See Also

- [Updating Components](./updating-components.md) - Safe update procedures
- [Troubleshooting](./troubleshooting.md) - Recovery from common issues
- [Security Auditing](./security-auditing.md) - Protecting backup data