# Troubleshooting

This guide covers common issues you may encounter with botserver and their solutions.

---

## Quick Diagnostics

### Check Overall Status

```bash
# View all service status
./botserver status

# Check specific service
./botserver status llm
./botserver status tables
./botserver status vault
```

### View Logs

```bash
# All logs
tail -f botserver-stack/logs/*.log

# Specific service
tail -100 botserver-stack/logs/llm.log
tail -100 botserver-stack/logs/postgres.log
tail -100 botserver-stack/logs/vault.log

# With filtering
grep -i error botserver-stack/logs/*.log
grep -i "failed\|error\|panic" botserver-stack/logs/*.log
```

### System Resources

```bash
# Memory usage
free -h

# Disk space
df -h botserver-stack/

# Process list
ps aux | grep -E "llama|postgres|minio|vault|valkey"

# Open ports
ss -tlnp | grep LISTEN
```

---

## Startup Issues

### Bootstrap Fails

**Symptom:** `./botserver` fails during initial setup

**Common Causes & Solutions:**

1. **Port already in use**
   ```bash
   # Find what's using the port
   lsof -i :9000
   lsof -i :5432
   
   # Kill conflicting process
   kill -9 <PID>
   
   # Or change port in config
   ```

2. **Insufficient disk space**
   ```bash
   # Check available space
   df -h
   
   # Clean up old installers
   rm -rf botserver-installers/*.old
   
   # Clean logs
   rm -f botserver-stack/logs/*.log.old
   ```

3. **Download failure**
   ```bash
   # Clear cache and retry
   rm -rf botserver-installers/component-name*
   ./botserver bootstrap
   
   # Manual download
   curl -L -o botserver-installers/file.zip "URL"
   ```

4. **Permission denied**
   ```bash
   # Fix permissions
   chmod +x botserver
   chmod -R u+rwX botserver-stack/
   ```

### Vault Won't Start

**Symptom:** Vault fails to initialize or unseal

**Solutions:**

1. **First-time setup failed**
   ```bash
   # Reset Vault completely
   rm -rf botserver-stack/data/vault/*
   rm -f botserver-stack/conf/vault/init.json
   ./botserver bootstrap
   ```

2. **Vault is sealed**
   ```bash
   # Check seal status
   curl http://localhost:8200/v1/sys/seal-status
   
   # Unseal manually
   ./botserver unseal
   ```

3. **Lost unseal keys**
   ```bash
   # Check init.json exists
   cat botserver-stack/conf/vault/init.json
   
   # If lost, must reset Vault (DATA LOSS)
   ./botserver reset vault
   ```

### Database Won't Start

**Symptom:** PostgreSQL fails to start

**Solutions:**

1. **Corrupted data directory**
   ```bash
   # Check PostgreSQL logs
   tail -50 botserver-stack/logs/postgres.log
   
   # Try recovery
   ./botserver-stack/bin/tables/bin/pg_resetwal -f botserver-stack/data/tables/
   ```

2. **Port conflict**
   ```bash
   # Check if another PostgreSQL is running
   lsof -i :5432
   
   # Stop system PostgreSQL
   sudo systemctl stop postgresql
   ```

3. **Incorrect permissions**
   ```bash
   chmod 700 botserver-stack/data/tables/
   ```

---

## Service Issues

### LLM Server Not Responding

**Symptom:** Requests to port 8081/8082 fail

**Solutions:**

1. **Check if running**
   ```bash
   pgrep llama-server
   curl -k https://localhost:8081/health
   ```

2. **Model not found**
   ```bash
   # Verify model exists
   ls -la botserver-stack/data/llm/
   
   # Re-download model
   ./botserver update llm
   ```

3. **Out of memory**
   ```bash
   # Check memory usage
   free -h
   
   # Use smaller model or reduce context
   # Edit config.csv:
   # llm-server-ctx-size,2048
   ```

4. **GPU issues**
   ```bash
   # Check CUDA
   nvidia-smi
   
   # Fall back to CPU
   # Edit config.csv:
   # llm-server-gpu-layers,0
   ```

5. **Restart LLM server**
   ```bash
   pkill llama-server
   ./botserver start llm
   ```

### Drive (MinIO) Issues

**Symptom:** File uploads/downloads fail

**Solutions:**

1. **Check MinIO status**
   ```bash
   curl http://localhost:9000/minio/health/live
   ```

2. **Credential issues**
   ```bash
   # Verify credentials from Vault
   ./botserver show-secret drive
   
   # Test with mc client
   mc alias set local http://localhost:9000 ACCESS_KEY SECRET_KEY
   mc ls local/
   ```

3. **Disk full**
   ```bash
   df -h botserver-stack/data/drive/
   
   # Clean old versions
   mc rm --recursive --force local/bucket/.minio.sys/
   ```

### Cache (Valkey) Issues

**Symptom:** Session errors, slow responses

**Solutions:**

1. **Check Valkey status**
   ```bash
   ./botserver-stack/bin/cache/valkey-cli ping
   # Expected: PONG
   ```

2. **Memory issues**
   ```bash
   ./botserver-stack/bin/cache/valkey-cli info memory
   
   # Flush cache if needed
   ./botserver-stack/bin/cache/valkey-cli FLUSHALL
   ```

3. **Connection refused**
   ```bash
   # Check if running
   pgrep valkey-server
   
   # Restart
   ./botserver restart cache
   ```

### Directory (Zitadel) Issues

**Symptom:** Login fails, authentication errors

**Solutions:**

1. **Check Zitadel logs**
   ```bash
   tail -100 botserver-stack/logs/zitadel.log
   ```

2. **Database connection**
   ```bash
   # Zitadel uses PostgreSQL
   psql $DATABASE_URL -c "SELECT 1;"
   ```

3. **Certificate issues**
   ```bash
   # Regenerate certificates
   ./botserver regenerate-certs
   ```

---

## Connection Issues

### Cannot Connect to Database

**Error:** `connection refused` or `authentication failed`

**Solutions:**

1. **Verify DATABASE_URL**
   ```bash
   echo $DATABASE_URL
   # Should be: postgres://user:pass@localhost:5432/dbname
   ```

2. **Check PostgreSQL is running**
   ```bash
   pgrep postgres
   ./botserver status tables
   ```

3. **Test connection**
   ```bash
   psql $DATABASE_URL -c "SELECT 1;"
   ```

4. **Check pg_hba.conf**
   ```bash
   cat botserver-stack/conf/tables/pg_hba.conf
   # Ensure local connections are allowed
   ```

### SSL/TLS Certificate Errors

**Error:** `certificate verify failed` or `SSL handshake failed`

**Solutions:**

1. **Regenerate certificates**
   ```bash
   ./botserver regenerate-certs
   ```

2. **Check certificate validity**
   ```bash
   openssl x509 -in botserver-stack/conf/system/certificates/api/server.crt -noout -dates
   ```

3. **Skip verification (development only)**
   ```bash
   curl -k https://localhost:8081/health
   ```

### Network Timeouts

**Error:** Requests timeout after waiting

**Solutions:**

1. **Check DNS resolution**
   ```bash
   nslookup api.botserver.local
   ```

2. **Verify firewall rules**
   ```bash
   sudo ufw status
   sudo iptables -L
   ```

3. **Check service is listening**
   ```bash
   ss -tlnp | grep 8080
   ```

---

## Performance Issues

### Slow Response Times

**Solutions:**

1. **Check system resources**
   ```bash
   top -b -n 1 | head -20
   iostat -x 1 3
   ```

2. **Database performance**
   ```bash
   psql $DATABASE_URL -c "SELECT * FROM pg_stat_activity;"
   
   # Vacuum database
   psql $DATABASE_URL -c "VACUUM ANALYZE;"
   ```

3. **LLM performance**
   ```bash
   # Reduce context size
   # config.csv: llm-server-ctx-size,2048
   
   # Use GPU layers
   # config.csv: llm-server-gpu-layers,35
   ```

4. **Enable caching**
   ```bash
   # Verify cache is working
   ./botserver-stack/bin/cache/valkey-cli info stats
   ```

### High Memory Usage

**Solutions:**

1. **Identify memory hogs**
   ```bash
   ps aux --sort=-%mem | head -10
   ```

2. **Reduce LLM memory**
   ```bash
   # Use quantized model (Q3_K_M instead of F16)
   # Reduce context: llm-server-ctx-size,1024
   # Reduce batch: llm-server-batch-size,256
   ```

3. **Limit PostgreSQL memory**
   ```bash
   # Edit postgresql.conf
   shared_buffers = 256MB
   work_mem = 64MB
   ```

### High Disk Usage

**Solutions:**

1. **Find large files**
   ```bash
   du -sh botserver-stack/*
   du -sh botserver-stack/data/*
   ```

2. **Clean logs**
   ```bash
   truncate -s 0 botserver-stack/logs/*.log
   ```

3. **Clean old installers**
   ```bash
   # Keep only latest versions
   ls -la botserver-installers/
   rm botserver-installers/old-*
   ```

4. **Prune drive storage**
   ```bash
   mc rm --recursive --older-than 30d local/bucket/
   ```

---

## Update Issues

### Component Update Failed

**Symptom:** Update command fails or service won't start after update

**Solutions:**

1. **Clear cache and retry**
   ```bash
   rm botserver-installers/component-name*
   ./botserver update component-name
   ```

2. **Checksum mismatch**
   ```bash
   # Verify checksum
   sha256sum botserver-installers/file.zip
   
   # Compare with 3rdparty.toml
   grep sha256 3rdparty.toml | grep component
   
   # Update checksum if release changed
   ```

3. **Rollback to previous version**
   ```bash
   # If old version cached
   ls botserver-installers/
   
   # Restore old binary
   cp botserver-installers/old-version.zip /tmp/
   unzip /tmp/old-version.zip -d botserver-stack/bin/component/
   ```

### Database Migration Failed

**Solutions:**

1. **Check migration status**
   ```bash
   ./botserver migrate --status
   ```

2. **Run migrations manually**
   ```bash
   ./botserver migrate
   ```

3. **Rollback migration**
   ```bash
   ./botserver migrate --rollback
   ```

4. **Reset from backup**
   ```bash
   pg_restore -c -d $DATABASE_URL backup.dump
   ```

---

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `connection refused` | Service not running | Start the service |
| `permission denied` | File permissions | `chmod +x` on binary |
| `address already in use` | Port conflict | Kill conflicting process |
| `out of memory` | Insufficient RAM | Reduce model/context size |
| `no such file or directory` | Missing binary/config | Re-run bootstrap |
| `certificate verify failed` | SSL issues | Regenerate certificates |
| `authentication failed` | Wrong credentials | Check Vault secrets |
| `disk quota exceeded` | Disk full | Clean logs/old files |
| `too many open files` | ulimit too low | `ulimit -n 65536` |
| `connection timed out` | Network/firewall | Check firewall rules |

---

## Getting Help

### Collect Diagnostics

```bash
# Generate diagnostic report
./botserver diagnose > diagnostics-$(date +%Y%m%d).txt

# Include in bug reports:
# - botserver version
# - OS and architecture
# - Error messages
# - Relevant logs
```

### Debug Logging

```bash
# Enable verbose logging
RUST_LOG=debug ./botserver

# Trace level (very verbose)
RUST_LOG=trace ./botserver
```

### Community Support

- GitHub Issues: [github.com/GeneralBots/botserver/issues](https://github.com/GeneralBots/botserver/issues)
- Documentation: [docs.generalbots.ai](https://docs.generalbots.ai)

---

## See Also

- [Updating Components](./updating-components.md) - Safe update procedures
- [Backup and Recovery](./backup-recovery.md) - Data protection
- [Security Auditing](./security-auditing.md) - Security checks