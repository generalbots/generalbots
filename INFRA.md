# Infrastructure Operations Guide — Generic Across Incus Projects

NEVER INCLUDE CREDENTIALS OR COMPANY INFORMATION — THIS IS COMPANY AGNOSTIC.

## ENVIRONMENT CONTEXT

Agent must identify which environment it is operating on by checking the hostname or asking the user:

| Environment | Chat URL | System Domain | ALM Domain | Login Domain | Subnet |
|-------------|----------|---------------|------------|--------------|--------|
| PROD | chat.domain.com | system.domain.com | alm.domain.com | login.domain.com | 10.0.2.x |
| STAGE | chat.stage.domain.com | system.stage.domain.com | alm.stage.domain.com | login.stage.domain.com | 10.0.3.x |

URL pattern: chat.{stage.}domain.com/botname for bot access.

If edit conf/data make a backup first to /tmp with datetime suffix.

Always manage services with systemctl inside the system container. Never run binaries directly — they fail without .env loading. Correct: sudo incus exec system -- systemctl start|stop|restart|status botserver

---

## CRITICAL SAFETY RULES

- NEVER modify iptables without explicit confirmation
- NEVER touch production without asking first
- ALWAYS backup files to /tmp before editing
- NEVER push secrets (API keys, passwords, tokens) to git
- NEVER commit init.json (contains Vault unseal keys)
- NEVER deploy manually via scp/ssh — always use CI/CD
- ALWAYS push all submodules before main repo
- ALWAYS ask before pushing to ALM
- NEVER include real IPs in documentation — use 10.x.x.x

---

## INFRASTRUCTURE PATHS

- Base: /opt/gbo/ | Bin: /opt/gbo/bin | Conf: /opt/gbo/conf | Logs: /opt/gbo/logs
- Bots are stored in MinIO (drive), NOT in /opt/gbo/data

---

## CONTAINER ARCHITECTURE

| Container | Service | Port | Notes |
|-----------|---------|------|-------|
| system | BotServer + Valkey | 8080/6379 | Main API + cache |
| tables | PostgreSQL | 5432 | Primary database |
| vault | Vault | 8200 | Secrets |
| drive | MinIO | 9000/9100 | Object storage |
| directory | Zitadel | 9000 | Identity provider |
| llm | llama.cpp | 8081 | Local LLM |
| vectordb | Qdrant | 6333 | Vector database |
| alm | Forgejo | 4747 | Git (NOT 3000!) |
| alm-ci | Runner | - | CI/CD |
| proxy | Caddy | 80/443 | Reverse proxy |
| email | Stalwart | 993/465/587 | Mail |
| dns | CoreDNS | 53 | DNS |
| meet | LiveKit | 7880 | Video |

> Container deployment details: botbook/src/02-architecture-packages/containers.md
> Backup/recovery procedures: botbook/src/12-ecosystem-reference/backup-recovery.md

---

## NETWORK — NAT PORT FORWARDING

External ports DNAT to container IPs via iptables. Rules in /etc/iptables.rules.
Always use external interface (-i iface) to avoid loopback issues.

Port Map: 53=DNS 80/443=HTTP/HTTPS 5432=PostgreSQL 993=IMAPS 465=SMTPS 587=Submission 4747=Forgejo 9000=MinIO 8200=Vault

---

## CONTAINER OPERATIONS

### Daily Health Check

```bash
# Container status
sudo incus list

# Service health - all should show active
sudo incus exec system -- systemctl is-active botserver
sudo incus exec system -- systemctl is-active ui
sudo incus exec tables -- pgrep -f postgres > /dev/null && echo OK || echo DOWN
sudo incus exec drive -- pgrep -f minio > /dev/null && echo OK || echo DOWN
sudo incus exec vault -- curl -ksf https://localhost:8200/v1/sys/health | grep -q sealed.*false && echo "Vault OK" || echo "Vault SEALED"

# App health endpoint
curl -sf https://<system-domain>/api/health && echo OK || echo FAILED

# Recent errors
sudo incus exec system -- tail -10 /opt/gbo/logs/err.log | grep -i "error|panic|failed" | head -5
```

### Container Management

```bash
sudo incus list                                        # List all
sudo incus start|stop|restart <container>              # Lifecycle
sudo incus exec <container> -- bash                    # Shell
sudo incus log <container> --show-log                  # Logs
sudo incus snapshot create <container> pre-change-$(date +%Y%m%d%H%M%S)  # Backup
sudo incus snapshot restore <container> <name>         # Restore
```

### Service Management (inside container)

```bash
sudo incus exec <container> -- pgrep -a <process>      # Check running
sudo incus exec <container> -- systemctl restart <svc>  # Restart
sudo incus exec <container> -- ss -tlnp                # Ports
```

> Full container docs: botbook/src/02-architecture-packages/containers.md

---

## VAULT SECURITY ARCHITECTURE

Vault is the single source of truth for all secrets. Botserver reads VAULT_ADDR and VAULT_TOKEN from /opt/gbo/bin/.env at startup.

### Global Vault Paths

| Path | Contents |
|------|----------|
| gbo/tables | PostgreSQL credentials |
| gbo/drive | MinIO access key and secret |
| gbo/cache | Valkey password |
| gbo/llm | LLM URL and API keys |
| gbo/directory | Zitadel config |
| gbo/email | SMTP credentials |
| gbo/vectordb | Qdrant config |
| gbo/jwt | JWT signing secret |
| gbo/encryption | Master encryption key |

Organization-scoped: gbo/orgs/{org_id}/bots/{bot_id}
Tenant infrastructure: gbo/tenants/{tenant_id}/infrastructure

### Credential Resolution Order

org+bot level → default bot path → global path → env vars (dev only)

### Vault Operations

```bash
# Health check
sudo incus exec vault -- curl -ksf https://localhost:8200/v1/sys/health

# Unseal (3 of 5 keys from init.json)
sudo incus exec vault -- vault operator unseal $KEY1
sudo incus exec vault -- vault operator unseal $KEY2
sudo incus exec vault -- vault operator unseal $KEY3

# Read secret
sudo incus exec vault -- vault kv get secret/gbo/tables

# Generate new token
sudo incus exec vault -- vault token create -policy="botserver" -ttl="8760h" -format=json
```

### Vault Troubleshooting

- Cannot connect: check systemd, token not expired (vault token lookup), CA cert path, network to vault container
- Secrets missing: vault kv get — if NOT FOUND, add with vault kv put
- Sealed after restart: unseal with 3 keys from init.json
- TLS errors: confirm /opt/gbo/conf/system/certificates/ca/ca.crt exists, copy from vault container if missing
- init.json at /opt/gbo/bin/botserver-stack/conf/vault/vault-conf/ — root token + 5 unseal keys. NEVER commit.

---

## DNS MANAGEMENT

### Critical Rules

1. Update serial number in SOA record (format: YYYYMMDDNN)
2. Run sync-zones.sh to propagate to secondary nameservers
3. Anonymize IPs and credentials in all documentation

### Workflow

```bash
# 1. Edit zone file
sudo incus exec dns -- nano /opt/gbo/data/<domain>.zone

# 2. Update serial
sudo incus exec dns -- sed -i 's/YYYYMMDD01/YYYYMMDD02/' /opt/gbo/data/<domain>.zone

# 3. Reload CoreDNS
sudo incus exec dns -- pkill -HUP coredns

# 4. Sync to secondary NS
sudo /opt/gbo/bin/sync-zones.sh

# 5. Verify
dig @9.9.9.9 <domain> A +short
```

### Adding HTTPS Subdomain

Order: DNS record → wait propagation → add Caddy config → Caddy auto-obtains Let's Encrypt cert

```bash
# After DNS propagated, add Caddy config
sudo sh -c 'cat >> /opt/gbo/conf/config << CADDYEOF

<subdomain>.<domain> { import tls_config; reverse_proxy http://<container-ip>:<port> { header_up Host {host}; header_up X-Real-IP {remote}; header_up X-Forwarded-Proto https } }
CADDYEOF'
sudo incus exec proxy -- systemctl restart proxy
```

> DNS/Proxy details: botbook/src/02-architecture-packages/containers.md

---

## CI/CD — FORGEJO ALM

ALM port is 4747 (NOT 3000!). Runner token in action_runner_token table.
Runner: gbuser uid 1000, workspace /opt/gbo/data/, SSH key /home/gbuser/.ssh/id_ed25519

### CI Status Codes

0=pending, 1=success, 2=failure, 3=cancelled, 6=running

### CI Queries (PROD-ALM database)

```bash
# List recent runs
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -c \
  "SELECT id, title, status, to_timestamp(created) AS created_at FROM action_run ORDER BY id DESC LIMIT 10;"
# Failed run jobs
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -c \
  "SELECT id, name, status, task_id FROM action_run_job WHERE run_id = <RUN_ID>;"
# Step status
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -c \
  "SELECT name, status, log_index, log_length FROM action_task_step WHERE task_id = <TASK_ID> ORDER BY index;"
# Read build log (zstd-compressed)
sudo incus file pull alm/opt/gbo/data/data/actions_log/<LOG_FILENAME> /tmp/ci-log.log.zst
zstd -d /tmp/ci-log.log.zst -o /tmp/ci-log.log && cat /tmp/ci-log.log
```

### CI Runner Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Runner not connecting | Wrong ALM port | Use port 4747 |
| /tmp permission denied | Wrong permissions | chmod 1777 /tmp on alm-ci |
| Runner down | Process crashed | pkill -9 forgejo; restart daemon |
| Build stuck at status 6 | DB race condition | Reset status in action_task/action_run |
| GLIBC mismatch | Wrong build env | Rebuild inside system container (Debian 12) |

### Reset Stuck CI Run

```sql
UPDATE action_task SET status = 0 WHERE id = <ID>;
UPDATE action_run_job SET status = 0 WHERE run_id = <RUN_ID>;
UPDATE action_run SET status = 0 WHERE id = <RUN_ID>;
```

### Verify Deployment

```bash
sudo incus exec system -- stat -c '%y' /opt/gbo/bin/botserver
sudo incus exec system -- systemctl status botserver --no-pager
curl -sf https://<system-domain>/api/health && echo OK || echo FAILED
```

Build timing: 2-5 min cold, 30-60s incremental, ~5s deploy

> CI/CD details: botbook/src/12-ecosystem-reference/ci-cd.md

---

## MINIO (DRIVE) OPERATIONS

All bot files live in MinIO buckets. Use mc CLI at /opt/gbo/bin/mc from drive container.

### Bucket Structure Per Bot

{bot}.gbai/{bot}.gbdialog/  — BASIC scripts
{bot}.gbai/{bot}.gbot/      — config.csv
{bot}.gbai/{bot}.gbkb/      — knowledge base

### Common mc Commands

```bash
# All mc commands need PATH set
sudo incus exec drive -- bash -c 'export PATH=/opt/gbo/bin:$PATH && mc <command>'

mc ls local/                                    # List all buckets
mc ls local/<bot>.gbai/                         # List bot bucket
mc cat local/<bot>.gbai/<bot>.gbdialog/start.bas  # Read file
mc cp local/<bot>.gbai/<bot>.gbdialog/file /tmp/  # Download
mc cp /tmp/file local/<bot>.gbai/<bot>.gbot/config.csv  # Upload (triggers DriveMonitor)
mc stat local/<bot>.gbai/<bot>.gbot/config.csv  # Show ETag/metadata
mc mb local/newbot.gbai                         # Create bucket
mc admin info local                             # Health check

# Force re-sync (change ETag without content change)
mc cp local/<bot>.gbai/<bot>.gbot/config.csv local/<bot>.gbai/<bot>.gbot/config.csv
```

### Upload config.csv workflow: download via mc cat → edit locally → push via mc cp → wait 15s → verify in logs

---

## DRIVEMONITOR & BOT CONFIGURATION

DriveMonitor watches MinIO buckets and syncs changes to local filesystem and database every 10 seconds.

Monitors: .gbdialog/ (BASIC scripts, downloads+recompiles), .gbot/ (config.csv, syncs to bot_configuration table), .gbkb/ (KB docs, downloads+indexes for vector search)

### Database Tables

- bot_configuration: bot_id, config_key, config_value, config_type, is_encrypted, updated_at
- gbot_config_sync: bot_id, config_file_path, last_sync_at, file_hash, sync_count

### Config CSV Format

No header, each line: key,value (e.g. llm-provider,groq or theme-color1,#cc0000)

### Check Config Status

```bash
sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c \
  "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = (SELECT id FROM bots WHERE name = '<botname>') ORDER BY config_key;"
```

### Debug DriveMonitor

```bash
sudo incus exec system -- tail -f /opt/gbo/logs/out.log | grep -E "DRIVE_MONITOR|check_gbot|config"
```

Empty gbot_config_sync = DriveMonitor not synced yet. If no log entries after 30s, restart botserver. Force re-sync: mc cp file over itself to change ETag.

---

## DIRECTORY MANAGEMENT (ZITADEL)

### Access

- Internal: http://<directory-ip>:9000
- External: https://<login-domain>
- Console: https://<login-domain>/ui/console
- Always use v2 API (v1 is deprecated)
- Must include -H "Host: <directory-ip>" header or API returns 404

### Get Admin PAT

```bash
PAT=$(sudo incus exec directory -- cat /opt/gbo/conf/directory/admin-pat.txt)
```

### User Operations (v2) — always include Host header

Create user: POST /v2/users/human with username, profile, email, password JSON
List users: POST /v2/users with query offset/limit JSON
Update password: POST /v2/users/{id}/password with newPassword JSON
Create org: POST /v2/organizations with name JSON
Add domain: POST /v2/organizations/{org-id}/domains with domainName JSON

All require: -H "Authorization: Bearer $PAT" -H "Host: <directory-ip>"

> Directory auth details: botbook/src/09-security/

---


## ALERT RESPONSE PLAYBOOK

### No IPv4 → set static IP (sudo incus config device set <c> eth0 ipv4.address <ip>; write /etc/network/interfaces; restart)
### Vault Sealed → unseal with 3 of 5 keys from init.json
### Botserver Down → systemctl restart; check ldd for missing libs
### Email No Internet → fix DNS (nameserver 8.8.8.8); or fix IPv6-only (see No IPv4)
### CI Build Failed → see CI/CD section for log retrieval and stuck run reset

---

## BASIC COMPILATION

Compilation in BasicCompiler (DriveMonitor) → .ast in work/{bot}.gbai/{bot}.gbdialog/. Runtime loads .ast only via ScriptService::run(). No .bas fallback at runtime. Suggestion dedup: Redis SADD, key suggestions:{bot_id}:{session_id}, read SMEMBERS.

---

## LOGGING

```bash
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "error|panic|failed"  # Errors
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "<botname>"            # Bot activity
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "drive|config"         # DriveMonitor
sudo incus exec system -- tail -f /opt/gbo/logs/err.log | grep -i "model|llm"            # LLM calls
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log                       # CI runner
```

> Full troubleshooting: botbook/src/12-ecosystem-reference/troubleshooting.md

---

## PROGRAM ACCESS

| Program | Container | Path | Notes |
|---------|-----------|------|-------|
| botserver | system | /opt/gbo/bin/botserver | systemctl only |
| botui | system | /opt/gbo/bin/botui | systemctl only |
| mc | drive | /opt/gbo/bin/mc | PATH=/opt/gbo/bin:$PATH |
| psql | tables | /usr/bin/psql | psql -h localhost -U postgres -d botserver |
| vault | vault | /opt/gbo/bin/vault | Needs VAULT_ADDR, VAULT_TOKEN, VAULT_CACERT |

### Quick psql

```bash
# Bot config
sudo incus exec tables -- psql -h localhost -U postgres -d botserver -c \
  "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = (SELECT id FROM bots WHERE name = '<botname>') ORDER BY config_key;"
# ALM CI runs
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -c \
  "SELECT id, status, created FROM action_run ORDER BY id DESC LIMIT 5;"
```

---

## COMMON ERRORS

| Error | Cause | Fix |
|-------|-------|-----|
| No IPv4 | DHCP failed | Set static IP |
| /tmp permission denied | Wrong perms | chmod 1777 /tmp |
| Token.Invalid | PAT expired | Regenerate in Zitadel console |
| failed SASL auth | Wrong DB password | Check Vault gbo/tables |
| GLIBC not found | Wrong build env | Rebuild in system container (Debian 12) |
| connection refused | Service down | systemctl restart |
| exec format error | Arch mismatch | Recompile for target |
| address in use | Port conflict | lsof -i :port |
| cert verify failed | Wrong CA | Copy from vault container |
| DNS lookup failed | No IPv4 | Check network config |
| botui cant reach server | Wrong URL | BOTSERVER_URL=http://localhost:5858 |
| Suggestions missing | .bas error | Check logs, clear /opt/gbo/work/ AST cache |
| IPv6 DNS timeouts | AAAA no IPv6 | RES_OPTIONS=inet4, IPV6=no |
| Dev paths in logs | Missing .env | DATA_DIR=/opt/gbo/work/ WORK_DIR=/opt/gbo/work/ |

---

## ESCALATION

1. Capture logs: sudo incus exec system -- tar czf /tmp/debug-$(date +%Y%m%d).tar.gz /opt/gbo/logs/
2. Check AGENTS.md for dev troubleshooting
3. Review recent commits for breaking changes
4. Snapshot rollback (last resort)
