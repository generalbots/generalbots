# CI/CD Integration

General Bots uses Forgejo (ALM) as Git server with Forgejo Runner for CI/CD. The runner lives in a separate container (alm-ci) and builds are triggered by pushing to the ALM repository.

---

## Architecture

| Component | Container | Port | Purpose |
|-----------|-----------|------|---------|
| Forgejo (ALM) | alm | 4747 | Git server, workflow definitions |
| Forgejo Runner | alm-ci | - | CI/CD executor |
| PostgreSQL | tables | 5432 | CI run database (PROD-ALM) |
| BotServer (deploy target) | system | 8080 | Receives built binary |

**Deploy flow:** Push to ALM → Runner picks up job → cargo build → tar+gzip binary → SSH to system container → extract to /opt/gbo/bin/botserver → restart via systemctl

---

## Status Codes

| Code | Status |
|------|--------|
| 0 | pending |
| 1 | success |
| 2 | failure |
| 3 | cancelled |
| 6 | running |

---

## Database Queries

All queries run against the `PROD-ALM` database:

```bash
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM
```

### List Recent Runs

```sql
SELECT id, title, workflow_id, status,
       to_timestamp(created) AS created_at
FROM action_run
ORDER BY id DESC LIMIT 10;
```

### Get Jobs for a Run

```sql
SELECT id, name, status, task_id
FROM action_run_job
WHERE run_id = <RUN_ID>;
```

### Get Step-Level Status

```sql
SELECT name, status, log_index, log_length
FROM action_task_step
WHERE task_id = <TASK_ID>
ORDER BY index;
```

### Check Runner Token

```sql
SELECT * FROM action_runner_token;
```

### List Registered Runners

```sql
SELECT * FROM action_runner;
```

### Reset a Stuck Run (status 6)

```sql
UPDATE action_task SET status = 0 WHERE id = <ID>;
UPDATE action_run_job SET status = 0 WHERE run_id = <RUN_ID>;
UPDATE action_run SET status = 0 WHERE id = <RUN_ID>;
```

---

## Reading Build Logs

Build logs are stored as zstd-compressed files in the alm container. The database tracks the filename.

### Step-by-Step

```bash
# 1. Get log filename from database
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM \
  -c "SELECT log_filename FROM action_task WHERE id = <TASK_ID>;"

# 2. Pull compressed log from alm container
sudo incus file pull alm/opt/gbo/data/data/actions_log/<LOG_FILENAME> /tmp/ci-log.log.zst

# 3. Decompress and read
zstd -d /tmp/ci-log.log.zst -o /tmp/ci-log.log
cat /tmp/ci-log.log
```

### One-Liner: Read Latest Failed Run

```bash
TASK_ID=$(sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -t -c \
  "SELECT at.id FROM action_task at JOIN action_run_job arj ON at.job_id = arj.id \
   JOIN action_run ar ON arj.run_id = ar.id \
   WHERE ar.status = 2 ORDER BY at.id DESC LIMIT 1;" | tr -d ' ')
LOG_FILE=$(sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM -t -c \
  "SELECT log_filename FROM action_task WHERE id = $TASK_ID;" | tr -d ' ')
sudo incus file pull "alm/opt/gbo/data/data/actions_log/$LOG_FILE" /tmp/ci-log.log.zst
zstd -d /tmp/ci-log.log.zst -o /tmp/ci-log.log 2>/dev/null && cat /tmp/ci-log.log
```

---

## Real-Time Monitoring

```bash
# Tail runner logs (live but ephemeral)
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log

# Watch for new runs
sudo incus exec tables -- psql -h localhost -U postgres -d PROD-ALM \
  -c "SELECT id, title, workflow_id, status FROM action_run ORDER BY id DESC LIMIT 5;"

# Check runner logs for build activity
sudo incus exec alm-ci -- tail -f /opt/gbo/logs/forgejo-runner.log | grep -E "Clone|Build|Deploy|Success|Failure"
```

---

## Build Timing

| Phase | Duration |
|-------|----------|
| Rust compilation (cold) | 2-5 minutes |
| Rust compilation (incremental) | 30-60 seconds |
| First build (dependencies) | Downloads ~200 crates |
| Deploy step | ~5 seconds |
| Total CI time | 2-6 minutes depending on cache |

---

## Verify Deployment

```bash
# Check binary timestamp
sudo incus exec system -- stat -c '%y' /opt/gbo/bin/botserver

# Check running version
sudo incus exec system -- /opt/gbo/bin/botserver --version

# Check systemd status
sudo incus exec system -- systemctl status botserver --no-pager

# Health endpoint
curl -sf https://<system-domain>/api/health && echo "OK" || echo "FAILED"
```

---

## Runner Configuration

- **Binary:** /opt/gbo/bin/forgejo-runner
- **Config:** /opt/gbo/bin/config.yaml
- **Systemd:** /etc/systemd/system/alm-ci-runner.service
- **User:** gbuser (uid 1000)
- **Workspace:** /opt/gbo/data/
- **SSH deploy key:** /home/gbuser/.ssh/id_ed25519
- **sccache:** /usr/local/bin/sccache (via RUSTC_WRAPPER=sccache)
- **Cargo cache:** /home/gbuser/.cargo/
- **Rustup:** /home/gbuser/.rustup/

### Register New Runner

```bash
forgejo-runner register \
  --instance http://<alm-ip>:4747 \
  --token <TOKEN> \
  --name gbo \
  --labels ubuntu-latest:docker://node:20-bookworm \
  --no-interactive
```

> Token from action_runner_token table in PROD-ALM database.

### Restart Runner

```bash
sudo incus exec alm-ci -- pkill -9 forgejo
sleep 2
sudo incus exec alm-ci -- bash -c 'cd /opt/gbo/bin && nohup ./forgejo-runner daemon --config config.yaml >> /opt/gbo/logs/forgejo-runner.log 2>&1 &'
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Runner not connecting | Wrong ALM port (3000 vs 4747) | Use port 4747 in runner registration |
| `registration file not found` | Missing/wrong .runner file | Delete .runner and re-register |
| `unsupported protocol scheme` | Wrong .runner JSON format | Delete .runner and re-register |
| `connection refused` to ALM | iptables or ALM down | Check `ss -tlnp \| grep 4747` |
| CI not picking up jobs | Runner not registered or labels mismatch | Check runner labels match workflow runs-on |
| `/tmp permission denied` | Wrong permissions on alm-ci | `chmod 1777 /tmp` on alm-ci |
| Build stuck at status 6 | DB race condition | Reset status in action_task/action_run tables |
| GLIBC mismatch | Built in wrong environment | Rebuild inside system container (Debian 12, glibc 2.36) |
| Binary not updating | CI did not rebuild | Push trivial change to force rebuild |
| Migrations not running | Binary not updated | Check stat timestamp, push code change |

---

## Deploy Workflow

```bash
# 1. Push submodules first
cd botserver && git push alm main && git push origin main
cd ../botui && git push alm main && git push origin main
cd ../botlib && git push alm main && git push origin main

# 2. Push main repo
cd .. && git add botserver botui botlib
git commit -m "Update submodules: <description>"
git push alm main && git push origin main

# 3. Wait for CI (~2-6 min)
# Monitor via runner logs or database queries

# 4. Verify deployment
sudo incus exec system -- stat -c '%y' /opt/gbo/bin/botserver
sudo incus exec system -- systemctl status botserver --no-pager
curl -sf https://<system-domain>/api/health
```
