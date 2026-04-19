# Production Environment Guide

## CRITICAL RULES — READ FIRST

NEVER INCLUDE HERE CREDENTIALS OR COMPANY INFORMATION, THIS IS COMPANY AGNOSTIC.

Always manage services with `systemctl` inside the `system` Incus container. Never run `/opt/gbo/bin/botserver` or `/opt/gbo/bin/botui` directly — they will fail because they won't load the `.env` file containing Vault credentials and paths. The correct commands are `sudo incus exec system -- systemctl start|stop|restart|status botserver` and the same for `ui`. Systemctl handles environment loading, auto-restart, logging, and dependencies.

Never push secrets (API keys, passwords, tokens) to git. Never commit `init.json` (it contains Vault unseal keys). All secrets must come from Vault — only `VAULT_*` variables are allowed in `.env`. Never deploy manually via scp or ssh; always use CI/CD. Always push all submodules (botserver, botui, botlib) before or alongside the main repo. Always ask before pushing to ALM.

---

## Infrastructure Overview

The host machine is accessed via `ssh user@<hostname>`, running Incus (an LXD fork) as hypervisor. All services run inside named Incus containers. You enter containers with `sudo incus exec <container> -- <command>` and list them with `sudo incus list`.

### Container Architecture

| Container | Service | Technology | Binary Path | Logs Path | Data Path | Notes |
|-----------|---------|------------|-------------|-----------|-----------|-------|
| **system** | BotServer + BotUI | Rust/Axum | `/opt/gbo/bin/botserver`<br>`/opt/gbo/bin/botui` | `/opt/gbo/logs/out.log`<br>`/opt/gbo/logs/err.log` | `/opt/gbo/work/` | Main API + UI proxy |
| **tables** | PostgreSQL | PostgreSQL 15+ | `/usr/lib/postgresql/*/bin/postgres` | `/opt/gbo/logs/postgresql/` | `/opt/gbo/data/pgdata/` | Primary database |
| **vault** | HashiCorp Vault | Vault | `/opt/gbo/bin/vault` | `/opt/gbo/logs/vault/` | `/opt/gbo/data/vault/` | Secrets management |
| **cache** | Valkey | Valkey (Redis fork) | `/opt/gbo/bin/valkey-server` | `/opt/gbo/logs/valkey/` | `/opt/gbo/data/valkey/` | Distributed cache |
| **drive** | MinIO | MinIO | `/opt/gbo/bin/minio` | `/opt/gbo/logs/minio/` | `/opt/gbo/data/minio/` | Object storage (S3 API) |
| **directory** | Zitadel | Zitadel (Go) | `/opt/gbo/bin/zitadel` | `/opt/gbo/logs/zitadel.log` | `PROD-DIRECTORY` DB | Identity provider |
| **llm** | llama.cpp | C++/CUDA | `/opt/gbo/bin/llama-server` | `/opt/gbo/logs/llm/` | `/opt/gbo/models/` | Local LLM inference |
| **vectordb** | Qdrant | Qdrant (Rust) | `/opt/gbo/bin/qdrant` | `/opt/gbo/logs/qdrant/` | `/opt/gbo/data/qdrant/` | Vector database |
| **alm** | Forgejo | Forgejo (Go) | `/opt/gbo/bin/forgejo` | `/opt/gbo/logs/forgejo/` | `/opt/gbo/data/forgejo/` | Git server (port 4747) |
| **alm-ci** | Forgejo Runner | Docker/runner | `/opt/gbo/bin/forgejo-runner` | `/opt/gbo/logs/forgejo-runner.log` | `/opt/gbo/data/ci/` | CI/CD runner |
| **proxy** | Caddy | Caddy | `/opt/gbo/bin/caddy` | `/opt/gbo/logs/caddy/` | `/opt/gbo/conf/` | Reverse proxy |
| **email** | Stalwart | Stalwart (Rust) | `/opt/gbo/bin/stalwart` | `/opt/gbo/logs/email/` | `/opt/gbo/data/email/` | Mail server |
| **webmail** | Roundcube | PHP | `/usr/share/roundcube/` | `/var/log/php/` | `/var/lib/roundcube/` | Webmail frontend |
| **dns** | CoreDNS | CoreDNS (Go) | `/opt/gbo/bin/coredns` | `/opt/gbo/logs/dns/` | `/opt/gbo/conf/Corefile` | DNS resolution |
| **meet** | LiveKit | LiveKit (Go) | `/opt/gbo/bin/livekit-server` | `/opt/gbo/logs/meet/` | `/opt/gbo/data/meet/` | Video conferencing |
| **table-editor** | NocoDB | NocoDB | `/opt/gbo/bin/nocodb` | `/opt/gbo/logs/nocodb/` | `/opt/gbo/data/nocodb/` | Database UI |

### Network Access

Externally, services are exposed via reverse proxy (Caddy). Internally, containers communicate via private IPs:

| Service | External URL | Internal Address |
|---------|--------------|------------------|
| BotServer | `https://<system-domain>` | `http://<system-ip>:8080` |
| BotUI | `https://<chat-domain>` | `http://<system-ip>:3000` |
| Zitadel | `https://<login-domain>` | `http://<directory-ip>:8080` |
| Forgejo | `https://<alm-domain>` | `http://<alm-ip>:4747` |
| Webmail | `https://<webmail-domain>` | `http://<webmail-ip>:80` |
| Roundcube | `https://<roundcube-domain>` | `http://<webmail-ip>:80` |

**Note:** BotUI's `BOTSERVER_URL` must be `http://<system-ip>:8080` internally, NOT the external HTTPS URL.

---

## Daily Operations

### Daily Health Check (5 minutes)

Run this every morning or after any deploy:

```bash
# 1. Container status
sudo incus list

# 2. Service health - all should show "active (running)"
sudo incus exec system -- systemctl is-active botserver
sudo incus exec system -- systemctl is-active ui
sudo incus exec directory -- systemctl is-active directory 2>/dev/null || echo "Directory check failed"
sudo incus exec drive -- pgrep -f minio > /dev/null && echo "MinIO OK" || echo "MinIO DOWN"
sudo incus exec tables -- pgrep -f postgres > /dev/null && echo "PostgreSQL OK" || echo "PostgreSQL DOWN"

# 3. IPv4 connectivity check - all containers should have IPv4
sudo incus list -c n4 | grep -E "(system|tables|vault|directory|drive|cache|llm|vector_db)" | grep -v "10\." && echo "WARNING: Missing IPv4" || echo "IPv4 OK"

# 4. Application health endpoint
curl -sf https://<system-domain>/api/health && echo "Health OK" || echo "Health FAILED"

# 5. Recent errors (last 10 lines)
sudo incus exec system -- tail -10 /opt/gbo/logs/err.log | grep -i "error\|panic\|failed" | head -5
```

**Expected Result:** All services "active", all containers have IPv4, health endpoint returns 200, no critical errors.

### Weekly Deep Check (15 minutes)

Run every Monday morning:

```bash
# 1. Disk space on all containers
for c in system tables vault directory drive cache llm vector_db; do
  echo "=== $c ==="
  sudo incus exec $c -- df -h / 2>/dev/null | tail -1
done

# 2. Database connection pool status
sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c "SELECT count(*), state FROM pg_stat_activity GROUP BY state;"

# 3. Vault status (should be unsealed)
sudo incus exec vault -- curl -ksf https://localhost:8200/v1/sys/health | grep -q '"sealed":false' && echo "Vault unsealed" || echo "Vault SEALED - CRITICAL"

# 4. CI runner status
sudo incus exec alm-ci -- pgrep -f forgejo > /dev/null && echo "CI runner OK" || echo "CI runner DOWN"

# 5. MinIO buckets health
sudo incus exec drive -- bash -c 'export PATH=/opt/gbo/bin:$PATH && mc admin info local' 2>&1 | head -10

# 6. Backup verification - check latest snapshot exists
sudo incus snapshot list system | head -5
```

### Quick Status Dashboard

One-line status of everything:

```bash
echo "=== GBO Status Dashboard $(date) ==="
echo "Containers:"
sudo incus list -c n4,s | grep -E "(system|tables|vault|directory|drive|cache|llm|vector_db|alm-ci)" | awk '{print $1 ": " $3 " " $4}'
echo ""
echo "Services:"
for svc in botserver ui; do
  sudo incus exec system -- systemctl is-active $svc 2>/dev/null && echo "  $svc: ACTIVE" || echo "  $svc: DOWN"
done
echo ""
echo "Health:"
curl -s -o /dev/null -w "%{http_code}" https://<system-domain>/api/health 2>/dev/null | grep -q "200" && echo "  API: OK" || echo "  API: FAIL"
```

---

## Alert Response Playbook

### Alert: "No IPv4 on container"

**Symptoms:** Container shows empty IPV4 column in `incus list`

**Quick Fix:**
```bash
# Identify container
CONTAINER=<name>
IP=<unused-ip-in-range>  # e.g., 10.x.x.x
GATEWAY=<gateway-ip>

# Set static IP
sudo incus config device set $CONTAINER eth0 ipv4.address $IP

# Configure network inside
sudo incus exec $CONTAINER -- bash -c "cat > /etc/network/interfaces << 'EOF'
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
address $IP
netmask 255.255.255.0
gateway $GATEWAY
dns-nameservers 8.8.8.8 8.8.4.4
EOF"

# Restart
sudo incus restart $CONTAINER

# Verify
sudo incus exec $CONTAINER -- ip addr show eth0
```

**Prevention:** Always configure static IP when creating new containers.

---

### Alert: "ALM botserver problem" / CI Build Failed

**Symptoms:** Deploy not working, CI status shows failure

**Quick Diagnostics:**
```bash
# Check CI database for recent runs
sudo incus exec tables -- bash -c 'export PGPASSWORD=<password>; psql -h localhost -U postgres -d PROD-ALM -c "SELECT id, status, created FROM action_run ORDER BY id DESC LIMIT 5;"'
# Status codes: 0=pending, 1=success, 2=failure, 3=cancelled, 6=running
```

**Quick Fixes:**

1. **If stuck at status 6 (running):**
```bash
RUN_ID=<stuck-run-id>
sudo incus exec tables -- bash -c "export PGPASSWORD=<password>; psql -h localhost -U postgres -d PROD-ALM -c \"UPDATE action_task SET status = 0 WHERE id = $RUN_ID; UPDATE action_run_job SET status = 0 WHERE run_id = $RUN_ID; UPDATE action_run SET status = 0 WHERE id = $RUN_ID;\""
```

2. **If /tmp permission denied:**
```bash
sudo incus exec alm-ci -- chmod 1777 /tmp
sudo incus exec alm-ci -- touch /tmp/build.log && chmod 666 /tmp/build.log
```

3. **If CI runner down:**
```bash
sudo incus exec alm-ci -- pkill -9 forgejo
sleep 2
sudo incus exec alm-ci -- bash -c 'cd /opt/gbo/bin && nohup ./forgejo-runner daemon --config config.yaml >> /opt/gbo/logs/forgejo-runner.log 2>&1 &'
```

**After fix:** Push a trivial change to re-trigger CI.

---

### Alert: "Email container stopping reach Internet"

**Symptoms:** Email notifications failing, container cannot resolve external domains

**Quick Diagnostics:**
```bash
# Test DNS from email container
sudo incus exec email -- nslookup google.com

# Check network config
sudo incus exec email -- cat /etc/resolv.conf
sudo incus exec email -- ip route
```

**Quick Fixes:**

1. **If IPv6-only (no IPv4):** Follow "No IPv4 on container" playbook above.

2. **If DNS not working:**
```bash
# Force Google DNS
sudo incus exec email -- bash -c 'echo "nameserver 8.8.8.8" > /etc/resolv.conf'

# Or configure via interfaces file
sudo incus exec email -- bash -c "cat > /etc/network/interfaces << 'EOF'
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
address <email-container-ip>
netmask 255.255.255.0
gateway <gateway>
dns-nameservers 8.8.8.8 8.8.4.4
EOF"
sudo incus restart email
```

3. **If firewall blocking:** Check iptables rules on host for email container IP.

---

### Alert: "Vault sealed"

**Symptoms:** All services failing, Vault health shows "sealed": true

**Quick Fix:**
```bash
# Get unseal keys from secure location (not in git!)
KEY1=<key-from-secure-location>
KEY2=<key-from-secure-location>
KEY3=<key-from-secure-location>

sudo incus exec vault -- vault operator unseal $KEY1
sudo incus exec vault -- vault operator unseal $KEY2
sudo incus exec vault -- vault operator unseal $KEY3

# Verify
sudo incus exec vault -- vault status
```

---

### Alert: "Botserver not responding"

**Quick Diagnostics:**
```bash
# Check process
sudo incus exec system -- pgrep -f botserver || echo "NOT RUNNING"

# Check systemd status
sudo incus exec system -- systemctl status botserver --no-pager

# Check recent logs
sudo incus exec system -- tail -20 /opt/gbo/logs/err.log

# Check for GLIBC errors
sudo incus exec system -- ldd /opt/gbo/bin/botserver | grep "not found"
```

**Quick Fixes:**

1. **If systemd failed:**
```bash
sudo incus exec system -- systemctl restart botserver
sudo incus exec system -- systemctl restart ui
```

2. **If GLIBC mismatch:** Binary compiled with wrong glibc. Must rebuild inside system container (Debian 12, glibc 2.36).

3. **If port conflict:**
```bash
sudo incus exec system -- lsof -i :8080
sudo incus exec system -- killall botserver
sudo incus exec system -- systemctl start botserver
```

---

## Services Detail

Botserver runs as user `gbuser`, binary at `/opt/gbo/bin/botserver`, logs at `/opt/gbo/logs/out.log` and `/opt/gbo/logs/err.log`, systemd unit at `/etc/systemd/system/botserver.service`, env loaded from `/opt/gbo/bin/.env`. Bot BASIC scripts are stored in MinIO Drive under `{bot}.gbai/{bot}.gbdialog/*.bas` and are downloaded/compiled by DriveMonitor to `/opt/gbo/work/{bot}.gbai/{bot}.gbdialog/*.ast`.

The directory service runs Zitadel as user `root`, binary at `/opt/gbo/bin/zitadel`, logs at `/opt/gbo/logs/zitadel.log`, systemd unit at `/etc/systemd/system/directory.service`, and loads environment from the service configuration. Zitadel provides identity management and OAuth2 services for the platform.

Internally, Zitadel listens on port 8080 within the directory container. For external access:
- Via public domain (HTTPS): `https://<login-domain>` (configured through proxy container)
- Via host IP (HTTP): `http://<host-ip>:9000` (direct container port forwarding)
- Via container IP (HTTP): `http://<directory-container-ip>:9000` (direct container access)

Access the Zitadel console at `https://<login-domain>/ui/console` with admin credentials. Zitadel implements v1 Management API (deprecated) and v2 Organization/User services. Always use the v2 endpoints under `/v2/organizations` and `/v2/users` for all operations.

The botserver bootstrap also manages: Vault (secrets), PostgreSQL (database), Valkey (cache, password auth), MinIO (object storage), Zitadel (identity provider), and llama.cpp (LLM).

To obtain a PAT for Zitadel API access, check /opt/gbo/conf/directory/admin-pat.txt in the directory container. Use it with curl by setting the Authorization header: `Authorization: Bearer $(cat /opt/gbo/conf/directory/admin-pat.txt)` and include `-H "Host: <directory-ip> "` for correct host resolution.

---

## Directory Management (Zitadel)

### Getting Admin PAT (Personal Access Token)

```bash
# Get the admin PAT from directory container
PAT=$(ssh administrator@<hostname> "sudo incus exec directory -- cat /opt/gbo/conf/directory/admin-pat.txt")
```

### User Management via API (v2)

**Create a Human User:**
```bash
curl -X POST "http://<directory-ip>:8080/v2/users/human" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $PAT" \
-H "Host: <directory-ip>" \
-d '{
  "username": "testuser",
  "profile": {"givenName": "Test", "familyName": "User"},
  "email": {"email": "test@example.com", "isVerified": true},
  "password": {"password": "<password>", "changeRequired": false}
}'
```

**List Users:**
```bash
curl -X POST "http://<directory-ip>:8080/v2/users" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $PAT" \
-H "Host: <directory-ip>" \
-d '{"query": {"offset": 0, "limit": 100}}'
```

**Update User Password:**
```bash
curl -X POST "http://<directory-ip>:8080/v2/users/<user-id>/password" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer $PAT" \
-H "Host: <directory-ip>" \
-d '{
  "newPassword": {"password": "<password>", "changeRequired": false}
}'
```

**Delete User:**
```bash
curl -X DELETE "http://<directory-ip>:8080/v2/users/<user-id>" \
-H "Authorization: Bearer $PAT" \
-H "Host: <directory-ip>"
```

### Directory Quick Reference

| Task | Command |
|------|---------|
| Get PAT | `sudo incus exec directory -- cat /opt/gbo/conf/directory/admin-pat.txt` |
| Check health | `curl -sf http://<directory-ip>:8080/debug/healthz` |
| Console UI | `http://<host-ip>:9000/ui/console` |
| Create user | `POST /v2/users/human` |
| List users | `POST /v2/users` |
| Update password | `POST /v2/users/{id}/password` |

# /tmp permission denied for build.log
sudo incus exec alm-ci -- chmod 1777 /tmp
sudo incus exec alm-ci -- touch /tmp/build.log && chmod 666 /tmp/build.log

# Clean old CI runs (keep recent)
sudo incus exec tables -- bash -c 'export PGPASSWORD=<postgres-password>; psql -h localhost -U postgres -d PROD-ALM -c "DELETE FROM action_run WHERE id < <RECENT_ID>;"'
sudo incus exec tables -- bash -c 'export PGPASSWORD=<postgres-password>; psql -h localhost -U postgres -d PROD-ALM -c "DELETE FROM action_run_job WHERE run_id < <RECENT_ID>;"'
```

**Watch CI in real-time:**
```bash
# Tail runner logs
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log

# Check if new builds appear
watch -n 5 'sudo incus exec tables -- bash -c "export PGPASSWORD=<postgres-password>; psql -h localhost -U postgres -d PROD-ALM -c \\"SELECT id, status, created FROM action_run ORDER BY id DESC LIMIT 3;\\""'

# Verify botserver deployed correctly
sudo incus exec system -- /opt/gbo/bin/botserver --version 2>&1 | head -3
sudo incus exec system -- tail -5 /opt/gbo/logs/err.log
```

### Monitor CI/CD Build Status

**Check latest build status:**
```bash
# View latest 3 builds with status
sudo incus exec alm -- bash -c 'cd /opt/gbo/data/GeneralBots/BotServer/actions/runs && for dir in $(ls -t | head -3); do echo "=== Build $dir ==="; cat $dir/jobs/0.json 2>/dev/null | grep -E "\"status\"|\"commit\"|\"workflow\"" | head -5; done'

# Watch runner logs in real-time
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log | grep -E "Clone|Build|Deploy|Success|Failure"
```

**Understand build timing:**
- **Rust compilation**: 2-5 minutes (cold build), 30-60 seconds (incremental)
- **Dependencies**: First build downloads ~200 dependencies
- **Deploy step**: ~5 seconds
- **Total CI time**: 2-6 minutes depending on cache

**Verify binary was updated:**
```bash
# Check binary timestamp
ssh administrator@63.141.255.9 "sudo incus exec system -- stat -c '%y' /opt/gbo/bin/botserver"

# Check running version
ssh administrator@63.141.255.9 "sudo incus exec system -- /opt/gbo/bin/botserver --version"

# Check health endpoint
curl -sf https://chat.pragmatismo.com.br/api/health || echo "Health check failed"
```
```

---

## DriveMonitor & Bot Configuration

DriveMonitor is a background service inside botserver that watches MinIO buckets and syncs changes to the local filesystem and database every 10 seconds. It monitors three directory types per bot: the `.gbdialog/` folder for BASIC scripts (downloads and recompiles on change), the `.gbot/` folder for `config.csv` (syncs to the `bot_configuration` database table), and the `.gbkb/` folder for knowledge base documents (downloads and indexes for vector search).

Bot configuration is stored in two PostgreSQL tables inside the `botserver` database. The `bot_configuration` table holds key-value pairs with columns `bot_id`, `config_key`, `config_value`, `config_type`, `is_encrypted`, and `updated_at`. The `gbot_config_sync` table tracks sync state with columns `bot_id`, `config_file_path`, `last_sync_at`, `file_hash`, and `sync_count`.

The `config.csv` format is a plain CSV with no header: each line is `key,value`, for example `llm-provider,groq` or `theme-color1,#cc0000`. DriveMonitor syncs it when the file ETag changes in MinIO, on botserver startup, or after a restart.

**Check config status:** Query `bot_configuration` via `sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = (SELECT id FROM bots WHERE name = '<botname>') ORDER BY config_key;"`. Check sync state via the `gbot_config_sync` table. Inspect the bucket directly with `sudo incus exec drive -- /opt/gbo/bin/mc cat local/<botname>.gbai/<botname>.gbot/config.csv`.

**Debug DriveMonitor:** Monitor live logs with `sudo incus exec system -- tail -f /opt/gbo/logs/out.log | grep -E "(DRIVE_MONITOR|check_gbot|config)"`. An empty `gbot_config_sync` table means DriveMonitor has not synced yet. If no new log entries appear after 30 seconds, the loop may be stuck — restart botserver with systemctl to clear the state.

**Common config issues:** If config.csv is missing from the bucket, create and upload it with `mc cp`. If the database shows stale values, restart botserver to force a fresh sync, or as a temporary fix update the database directly with `UPDATE bot_configuration SET config_value = 'groq', updated_at = NOW() WHERE ...`. To force a re-sync without restarting, copy config.csv over itself with `mc cp local/... local/...` to change the ETag.

---

## MinIO (Drive) Operations

All bot files live in MinIO buckets. Use the `mc` CLI at `/opt/gbo/bin/mc` from inside the `drive` container. The bucket structure per bot is: `{bot}.gbai/` as root, `{bot}.gbai/{bot}.gbdialog/` for BASIC scripts, `{bot}.gbai/{bot}.gbot/` for config.csv, and `{bot}.gbai/{bot}.gbkb/` for knowledge base folders.

Common mc commands: `mc ls local/` lists all buckets; `mc ls local/botname.gbai/` lists a bucket; `mc cat local/.../start.bas` prints a file; `mc cp local/.../file /tmp/file` downloads; `mc cp /tmp/file local/.../file` uploads (this triggers DriveMonitor recompile); `mc stat local/.../config.csv` shows ETag and metadata; `mc mb local/newbot.gbai` creates a bucket; `mc rb local/oldbot.gbai` removes an empty bucket.

If mc is not found, use the full path `/opt/gbo/bin/mc`. If alias `local` is not configured, check with `mc config host list`. If MinIO is not running, check with `sudo incus exec drive -- systemctl status minio`.

---

## Vault Security Architecture

HashiCorp Vault is the single source of truth for all secrets. Botserver reads `VAULT_ADDR` and `VAULT_TOKEN` from `/opt/gbo/bin/.env` at startup, initializes a TLS/mTLS client, then reads credentials from Vault paths. If Vault is unavailable, it falls back to defaults. The `.env` file must only contain `VAULT_*` variables plus `PORT`, `DATA_DIR`, `WORK_DIR`, and `LOAD_ONLY`.

**Global Vault paths:** `gbo/tables` holds PostgreSQL credentials; `gbo/drive` holds MinIO access key and secret; `gbo/cache` holds Valkey password; `gbo/llm` holds LLM URL and API keys; `gbo/directory` holds Zitadel config; `gbo/email` holds SMTP credentials; `gbo/vectordb` holds Qdrant config; `gbo/jwt` holds JWT signing secret; `gbo/encryption` holds the master encryption key. Organization-scoped secrets follow patterns like `gbo/orgs/{org_id}/bots/{bot_id}` and tenant infrastructure uses `gbo/tenants/{tenant_id}/infrastructure`.

**Credential resolution:** For any service, botserver checks the most specific Vault path first (org+bot level), falls back to a default bot path, then falls back to the global path, and only uses environment variables as a last resort in development.

**Verify Vault health:** `sudo incus exec vault -- curl -k -sf https://localhost:8200/v1/sys/health` should return JSON with `"sealed":false`. To read a secret: set `VAULT_ADDR`, `VAULT_TOKEN`, and `VAULT_CACERT` then run `vault kv get secret/gbo/tables`. To test from the system container, use curl with `--cacert /opt/gbo/conf/system/certificates/ca/ca.crt` and `-H "X-Vault-Token: <token>"`.

**init.json** is stored at `/opt/gbo/bin/botserver-stack/conf/vault/vault-conf/init.json` and contains the root token and 5 unseal keys (3 needed to unseal). Never commit this file to git. Store it encrypted in a secure location.

**Vault troubleshooting — cannot connect:** Check that the vault container's systemd unit is running, verify the token in `.env` is not expired with `vault token lookup`, confirm the CA cert path in `.env` matches the actual file location, and test network connectivity from system to vault container. To generate a new token: `vault token create -policy="botserver" -ttl="8760h" -format=json` then update `.env` and restart botserver.

**Get database credentials from Vault v2 API:**
```bash
ssh user@<hostname> "sudo incus exec system -- curl -s --cacert /opt/gbo/conf/system/certificates/ca/ca.crt -H 'X-Vault-Token: <vault-token>' https://<vault-ip>:8200/v1/secret/data/gbo/tables 2>/dev/null"
```

**Vault troubleshooting — secrets missing:** Run `vault kv get secret/gbo/tables` (and other paths) to check if secrets exist. If a path returns NOT FOUND, add secrets with `vault kv put secret/gbo/tables host=<ip> port=5432 database=botserver username=gbuser password=<pw>` and similar for other paths.

**Vault sealed after restart:** Run `vault operator unseal <key1>`, repeat with key2 and key3 (3 of 5 keys from init.json), then verify with `vault status`.

**TLS certificate errors:** Confirm `/opt/gbo/conf/system/certificates/ca/ca.crt` exists in the system container. If missing, copy it from the vault container using `incus file pull vault/opt/gbo/conf/vault/ca.crt /tmp/ca.crt` then place it at the expected path.

**Vault snapshots:** Stop vault, run `sudo incus snapshot create vault backup-$(date +%Y%m%d-%H%M)`, start vault. Restore with `sudo incus snapshot restore vault <name>` while vault is stopped.

---

## DNS Management

### Updating DNS Records

**CRITICAL:** When updating DNS zone files, you MUST:

1. **Update the serial number** in the SOA record (format: YYYYMMDDNN)
2. **Run sync-zones.sh** to propagate changes to secondary nameservers
3. **Anonymize IPs and credentials** in all documentation and logs

**Workflow:**
```bash
# 1. Edit zone file
sudo incus exec dns -- nano /opt/gbo/data/pragmatismo.com.br.zone

# 2. Update serial (YYYYMMDDNN format)
# Example: 2026041801 (April 18, 2026, change #1)
sudo incus exec dns -- sed -i 's/2026041801/2026041802/' /opt/gbo/data/pragmatismo.com.br.zone

# 3. Reload CoreDNS
sudo incus exec dns -- pkill -HUP coredns

# 4. Sync to secondary NS
sudo /opt/gbo/bin/sync-zones.sh

# 5. Verify on secondary
ssh -o StrictHostKeyChecking=no -i /home/administrator/.ssh/id_ed25519 admin@<secondary-ip> 'getent hosts <domain>'
```

**Zone File Location:** `/opt/gbo/data/<domain>.zone` in the `dns` container

**Sync Script:** `/opt/gbo/bin/sync-zones.sh` - copies zone files to secondary NS (3.218.224.38)

**⚠️ Security Rules:**
- NEVER include real IPs in documentation - use `<ip>` or `10.x.x.x`
- NEVER include credentials - use `<password>` or `<token>`
- NEVER commit zone files with secrets to git

---

### Adding New Subdomains (HTTPS with Caddy)

**CRITICAL:** When adding new subdomains that need HTTPS, follow this order:

1. **Add DNS record FIRST** (see above workflow)
2. **Wait for DNS propagation** (can take up to 1 hour)
3. **Add Caddy config** - Caddy will automatically obtain Let's Encrypt certificate

**Complete Workflow:**
```bash
# 1. Add DNS record (update serial, sync zones)
sudo incus exec dns -- nano /opt/gbo/data/pragmatismo.com.br.zone
# Add: news IN A <ip>
sudo incus exec dns -- sed -i 's/2026041801/2026041802/' /opt/gbo/data/pragmatismo.com.br.zone
sudo incus exec dns -- pkill -HUP coredns
sudo /opt/gbo/bin/sync-zones.sh

# 2. Verify DNS propagation (wait until this works)
dig @9.9.9.9 news.pragmatismo.com.br A +short

# 3. Add Caddy config (AFTER DNS is working)
sudo sh -c 'cat >> /opt/gbo/conf/config << EOF

news.pragmatismo.com.br {
    import tls_config
    reverse_proxy http://<container-ip>:<port> {
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-Proto https
    }
}
EOF'

# 4. Restart Caddy
sudo incus exec proxy -- systemctl restart proxy

# 5. Wait for certificate (Caddy will auto-obtain from Let's Encrypt)
# Check logs: sudo incus exec proxy -- tail -f /opt/gbo/logs/access.log
```

**⚠️ Important:**
- Caddy will fail to obtain certificate if DNS is not propagated
- Wait up to 1 hour for DNS propagation before adding Caddy config
- Check Caddy logs for "challenge failed" errors - indicates DNS not ready
- Certificate is automatically renewed by Caddy

---

## Troubleshooting Quick Reference

**botserver won't start:** Run `sudo incus exec system -- ldd /opt/gbo/bin/botserver | grep "not found"` to check for missing libraries. Run `sudo incus exec system -- timeout 10 /opt/gbo/bin/botserver 2>&1` to see startup errors. Confirm `/opt/gbo/work/` exists and is accessible.

**botui can't reach botserver:** Check that the `ui.service` systemd file has `BOTSERVER_URL=http://localhost:5858` — not the external HTTPS URL. Fix with `sed -i 's|BOTSERVER_URL=.*|BOTSERVER_URL=http://localhost:5858|'` on the service file, then `systemctl daemon-reload` and `systemctl restart ui`.

**Suggestions not showing:** Confirm bot `.bas` files exist in MinIO Drive under `{bot}.gbai/{bot}.gbdialog/`. Check logs for compilation errors. Clear the AST cache in `/opt/gbo/work/` and restart botserver.

**IPv6 DNS timeouts on external APIs (Groq, Cloudflare):** The container's DNS may return AAAA records without IPv6 connectivity. The container should have `IPV6=no` in its network config and `gai.conf` set appropriately. Check for `RES_OPTIONS=inet4` in `botserver.service` if issues persist.

**Logs show development paths instead of Drive:** Botserver is using hardcoded dev paths. Check `.env` has `DATA_DIR=/opt/gbo/work/` and `WORK_DIR=/opt/gbo/work/`, verify the systemd unit has `EnvironmentFile=/opt/gbo/bin/.env`, and confirm Vault is reachable so service discovery works. Expected startup log lines include `info watcher:Watching data directory /opt/gbo/work` and `info botserver:BotServer started successfully on port 5858`.

**Migrations not running after push:** If `stat /opt/gbo/bin/botserver` shows old timestamp and `__diesel_schema_migrations` table has no new entries, CI did not rebuild. Make a trivial code change (e.g., add a comment) in botserver and push again to force rebuild.

---

## Drive (MinIO) File Operations Cheatsheet

All `mc` commands run inside the `drive` container with `PATH` set: `sudo incus exec drive -- bash -c 'export PATH=/opt/gbo/bin:$PATH && mc <command>'`. If `local` alias is missing, create it with credentials from Vault path `gbo/drive`.

**List bucket contents recursively:** `mc ls local/<bot>.gbai/ --recursive`

**Read a file from Drive:** `mc cat local/<bot>.gbai/<bot>.gbdialog/start.bas`

**Download a file:** `mc cp local/<bot>.gbai/<bot>.gbdialog/start.bas /tmp/start.bas`

**Upload a file to Drive (triggers DriveMonitor recompile):** Transfer file to host via `scp`, push into drive container with `sudo incus file push /tmp/file drive/tmp/file`, then `mc put /tmp/file local/<bot>.gbai/<bot>.gbdialog/start.bas`

**Full upload workflow example — updating config.csv:**
```bash
# 1. Download current config from Drive
ssh user@host "sudo incus exec drive -- bash -c 'export PATH=/opt/gbo/bin:\$PATH && mc cat local/botname.gbai/botname.gbot/config.csv'" > /tmp/config.csv

# 2. Edit locally (change model, keys, etc.)
sed -i 's/llm-model,old-model/llm-model,new-model/' /tmp/config.csv

# 3. Push edited file back to Drive
scp /tmp/config.csv user@host:/tmp/config.csv
ssh user@host "sudo incus file push /tmp/config.csv drive/tmp/config.csv"
ssh user@host "sudo incus exec drive -- bash -c 'export PATH=/opt/gbo/bin:\$PATH && mc put /tmp/config.csv local/botname.gbai/botname.gbot/config.csv'"

# 4. Wait ~15 seconds, then verify DriveMonitor picked up the change
ssh user@host "sudo incus exec system -- bash -c 'grep -i \"Model:\" /opt/gbo/logs/err.log | tail -3'"
```

**Force re-sync of config.csv** (change ETag without content change): `mc cp local/<bot>.gbai/<bot>.gbot/config.csv local/<bot>.gbai/<bot>.gbot/config.csv`

**Create a new bot bucket:** `mc mb local/newbot.gbai`

**Check MinIO health:** `sudo incus exec drive -- bash -c '/opt/gbo/bin/mc admin info local'`

---

## Logging Quick Reference

**Application logs** (searchable, timestamped, most useful): `sudo incus exec system -- tail -f /opt/gbo/logs/err.log` (errors and debug) or `/opt/gbo/logs/out.log` (stdout). The systemd journal only captures process lifecycle events, not application output.

**Search logs for specific bot activity:** `grep -i "botname\|llm\|Model:\|KB\|USE_KB\|drive_monitor" /opt/gbo/logs/err.log | tail -30`

**Check which LLM model a bot is using:** `grep "Model:" /opt/gbo/logs/err.log | tail -5`

**Check DriveMonitor config sync:** `grep "check_gbot\|config.csv\|should_sync" /opt/gbo/logs/err.log | tail -20`

**Check KB/vector operations:** `grep -i "gbkb\|qdrant\|embedding\|index" /opt/gbo/logs/err.log | tail -20`

**Live tail with filter:** `sudo incus exec system -- bash -c 'tail -f /opt/gbo/logs/err.log | grep --line-buffered -i "botname\|error\|KB"'`

---

## Program Access Cheatsheet

| Program | Container | Path | Notes |
|---------|-----------|------|-------|
| botserver | system | `/opt/gbo/bin/botserver` | Run via systemctl only |
| botui | system | `/opt/gbo/bin/botui` | Run via systemctl only |
| mc (MinIO Client) | drive | `/opt/gbo/bin/mc` | Must set `PATH=/opt/gbo/bin:$PATH` |
| psql | tables | `/usr/bin/psql` | `psql -h localhost -U postgres -d botserver` |
| vault | vault | `/opt/gbo/bin/vault` | Needs `VAULT_ADDR`, `VAULT_TOKEN`, `VAULT_CACERT` |
| zitadel | directory | `/opt/gbo/bin/zitadel` | Runs as root on port 8080 internally |

**Quick psql query — bot config:** `sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = (SELECT id FROM bots WHERE name = 'botname') ORDER BY config_key;"`

**Quick psql query — active KBs for session:** `sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c "SELECT * FROM session_kb_associations WHERE session_id = '<uuid>' AND is_active = true;"`

---

## BASIC Compilation Architecture

Compilation and runtime are now strictly separated. **Compilation** happens only in `BasicCompiler` inside DriveMonitor when it detects `.bas` file changes. The output is a fully preprocessed `.ast` file written to `work/<bot>.gbai/<bot>.gbdialog/<tool>.ast`. **Runtime** (start.bas, TOOL_EXEC, automation, schedule) loads only `.ast` files and calls `ScriptService::run()` which does `engine.compile() + eval_ast_with_scope()` on the already-preprocessed Rhai source — no preprocessing at runtime.

The `.ast` file has all transforms applied: `USE KB "cartas"` becomes `USE_KB("cartas")`, `IF/END IF` → `if/{ }`, `WHILE/WEND` → `while/{ }`, `BEGIN TALK/END TALK` → function calls, `SAVE`, `FOR EACH/NEXT`, `SELECT CASE`, `SET SCHEDULE`, `WEBHOOK`, `USE WEBSITE`, `LLM` keyword expansion, variable predeclaration, and keyword lowercasing. Runtime never calls `compile()`, `compile_tool_script()`, or `compile_preprocessed()` — those methods no longer exist.

**Tools (TOOL_EXEC) load `.ast` only** — there is no `.bas` fallback. If an `.ast` file is missing, the tool fails with "Failed to read tool .ast file". DriveMonitor must have compiled it first.

**Suggestion deduplication** uses Redis `SADD` (set) instead of `RPUSH` (list). This prevents duplicate suggestion buttons when `start.bas` runs multiple times per session. The key format is `suggestions:{bot_id}:{session_id}` and `get_suggestions` uses `SMEMBERS` to read it.

---

## Container Quick Reference

| Container | Critical | Check Command | Restart Command |
|-----------|----------|---------------|-----------------|
| system | YES | `systemctl is-active botserver` | `systemctl restart botserver` |
| tables | YES | `pgrep -f postgres` | `systemctl restart postgresql` |
| vault | YES | `curl -ksf https://localhost:8200/v1/sys/health` | `systemctl restart vault` |
| drive | YES | `pgrep -f minio` | `systemctl restart minio` |
| cache | HIGH | `pgrep -f valkey` | `systemctl restart valkey` |
| directory | HIGH | `curl -sf http://localhost:8080/debug/healthz` | `systemctl restart directory` |
| alm-ci | MED | `pgrep -f forgejo` | manual restart |
| llm | MED | `curl -sf http://localhost:8081/health` | `systemctl restart llm` |
| vector_db | LOW | `curl -sf http://localhost:6333/healthz` | `systemctl restart qdrant` |

---

## Log Tailing Commands

```bash
# Live error monitoring
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "error\|panic\|failed"

# Bot-specific activity
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "<botname>"

# DriveMonitor activity
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "drive\|config"

# LLM calls
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "model\|llm\|groq"

# CI runner
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log
```

---

## Health Endpoint Monitoring

Set up a simple cron job to alert if health fails:

```bash
# Add to host crontab (crontab -e)
*/5 * * * * curl -sf https://<system-domain>/api/health || echo "ALERT: Health check failed at $(date)" >> /var/log/gbo-health.log
```

---

## Troubleshooting Quick Reference

### Container Won't Start (No IPv4)

**Symptom:** Container shows empty IPV4 column in `sudo incus list`

**Diagnose:**
```bash
sudo incus list <container> -c n4
sudo incus exec <container> -- ip addr show eth0
```

**Fix:**
```bash
# 1. Stop container
sudo incus stop <container>

# 2. Set static IP
sudo incus config device set <container> eth0 ipv4.address <ip-address>

# 3. Configure network inside
sudo incus exec <container> -- bash -c 'cat > /etc/network/interfaces << EOF
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
address <ip-address>
netmask 255.255.255.0
gateway <gateway>
dns-nameservers 8.8.8.8 8.8.4.4
EOF'

# 4. Restart
sudo incus restart <container>

# 5. Verify
sudo incus exec <container> -- ip addr show eth0
```

---

### CI/ALM Permission Errors

**Symptom:** `/tmp permission denied` during CI build

**Fix:**
```bash
# On alm-ci container
sudo incus exec alm-ci -- chmod 1777 /tmp
sudo incus exec alm-ci -- touch /tmp/build.log && chmod 666 /tmp/build.log

# Check runner user
sudo incus exec alm-ci -- ls -la /opt/gbo/

# Fix ownership
sudo incus exec alm-ci -- chown -R gbuser:gbuser /opt/gbo/bin/
sudo incus exec alm-ci -- chown -R gbuser:gbuser /opt/gbo/work/
```

**CI Runner Down:**
```bash
sudo incus exec alm-ci -- pkill -9 forgejo
sleep 2
sudo incus exec alm-ci -- bash -c 'cd /opt/gbo/bin && nohup ./forgejo-runner daemon --config config.yaml >> /opt/gbo/logs/forgejo-runner.log 2>&1 &'
```

---

### MinIO (Drive) Operations with `mc`

**Setup:**
```bash
# Access drive container
sudo incus exec drive -- bash

# Set PATH
export PATH=/opt/gbo/bin:$PATH

# Verify mc works
mc --version
```

**Common Commands:**
```bash
# List all buckets
mc ls local/

# List bot bucket
mc ls local/<botname>.gbai/

# Read start.bas
mc cat local/<botname>.gbai/<botname>.gbdialog/start.bas

# Download file
mc cp local/<botname>.gbai/<botname>.gbdialog/config.csv /tmp/config.csv

# Upload file (triggers DriveMonitor)
mc cp /tmp/config.csv local/<botname>.gbai/<botname>.gbot/config.csv

# Force re-sync (change ETag)
mc cp local/<bot>.gbai/<bot>.gbot/config.csv local/<bot>.gbai/<bot>.gbot/config.csv

# Create new bucket
mc mb local/newbot.gbai

# Check MinIO health
mc admin info local
```

**If `local` alias missing:**
```bash
# Create alias
mc alias set local http://localhost:9000 <access-key> <secret-key>
```

---

### Forgejo ALM Database Operations

**Access ALM database (PROD-ALM):**
```bash
# On tables container
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM
```

**Common Queries:**
```sql
-- Check CI runs
SELECT id, status, commit_sha, created FROM action_run ORDER BY id DESC LIMIT 10;

-- Status codes: 0=pending, 1=success, 2=failure, 3=cancelled, 6=running

-- Check specific run jobs
SELECT id, status, name FROM action_run_job WHERE run_id = <ID>;

-- Reset stuck run
UPDATE action_task SET status = 0 WHERE id = <ID>;
UPDATE action_run_job SET status = 0 WHERE run_id = <RUN_ID>;
UPDATE action_run SET status = 0 WHERE id = <RUN_ID>;

-- Check runner token
SELECT * FROM action_runner_token;

-- List runners
SELECT * FROM action_runner;
```

**Check CI from host:**
```bash
export PGPASSWORD=<password>
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -c "SELECT id, status, created FROM action_run ORDER BY id DESC LIMIT 5;"
```

---

### Zitadel API v2 Operations

**Important:** Always use **v2 API** - v1 is deprecated and non-functional.

**Get PAT:**
```bash
PAT=$(sudo incus exec directory -- cat /opt/gbo/conf/directory/admin-pat.txt)
```

**Common Operations:**

**Create User (v2):**
```bash
curl -X POST "http://<directory-ip>:8080/v2/users/human" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PAT" \
  -H "Host: <directory-ip>" \
  -d '{
    "username": "newuser",
    "profile": {"givenName": "New", "familyName": "User"},
    "email": {"email": "user@example.com", "isVerified": true},
    "password": {"password": "<password>", "changeRequired": false}
  }'
```

**List Users (v2):**
```bash
curl -X POST "http://<directory-ip>:8080/v2/users" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PAT" \
  -H "Host: <directory-ip>" \
  -d '{"query": {"offset": 0, "limit": 100}}'
```

**Create Organization (v2):**
```bash
curl -X POST "http://<directory-ip>:8080/v2/organizations" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PAT" \
  -H "Host: <directory-ip>" \
  -d '{"name": "organization-name"}'
```

**Add Domain to Org (v2):**
```bash
curl -X POST "http://<directory-ip>:8080/v2/organizations/<org-id>/domains" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $PAT" \
  -H "Host: <directory-ip>" \
  -d '{"domainName": "example.com"}'
```

**⚠️ Critical:** Always include `-H "Host: <directory-ip>"` header or API returns 404.

---

### Common Errors & Quick Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `No IPv4 on container` | DHCP failed | Set static IP (see above) |
| `/tmp permission denied` | Wrong permissions | `chmod 1777 /tmp` |
| `Errors.Token.Invalid (AUTH-7fs1e)` | Zitadel PAT expired | Regenerate via console |
| `failed SASL auth` | Wrong DB password | Check Vault credentials |
| `GLIBC_2.39 not found` | Wrong build environment | Rebuild in system container |
| `connection refused` | Service down | `systemctl restart <service>` |
| `exec format error` | Architecture mismatch | Recompile for target arch |
| `address already in use` | Port conflict | `lsof -i :<port>` |
| `certificate verify failed` | Wrong CA cert | Copy from vault container |
| `DNS lookup failed` | No IPv4 connectivity | Check network config |

---

## Contact Escalation

If quick fixes don't work:

1. Capture logs: `sudo incus exec system -- tar czf /tmp/debug-$(date +%Y%m%d).tar.gz /opt/gbo/logs/`
2. Check AGENTS.md for development troubleshooting
3. Review recent commits for breaking changes
4. Consider snapshot rollback (last resort)
