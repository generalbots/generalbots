# Runbook: disk full, failed migration or stuck queue

Alert: `api_p95` burn (the database slows down when the disk is full) or a queue
panel that keeps growing. Panel: **Analytics → Operations → Queue depth** and
**Requests by endpoint class**.

## Trigger

- The disk that holds PostgreSQL, MinIO or the logs is above 90 %.
- A migration failed and the process is retrying on every boot.
- Queue depth grows while running stays flat (nothing is draining).
- Dead letters accumulate in the queue panel.

## Impact

A full disk stops writes first (new messages, new sessions), then reads. A
stuck queue looks like slow answers to users because each turn waits behind
work that never finishes.

## Diagnosis

```bash
# 1. Where is the space going?
df -h /
sudo incus exec tables -- df -h /var/lib/postgresql
sudo incus exec drive -- df -h /opt/gbo/data

# 2. Largest directories on the host.
sudo du -sh /opt/gbo/data/* /opt/gbo/logs/* 2>/dev/null | sort -h | tail -10

# 3. Queue state: depth, running and dead letters per queue.
#    The same numbers the dashboard shows:
psql -h localhost -U postgres -d botserver -c \
  "SELECT status, COUNT(*) FROM agent_runs GROUP BY status ORDER BY 2 DESC;"
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) FROM handoff_queue WHERE status = 'queued';"

# 4. A failed migration leaves the process retrying at boot.
sudo incus exec bot -- tail -200 /opt/gbo/logs/stdout.log | grep -iE "migration|diesel"

# 5. Which table is growing fastest?
psql -h localhost -U postgres -d botserver -c \
  "SELECT relname, pg_size_pretty(pg_total_relation_size(relid)) AS size
     FROM pg_catalog.pg_statio_user_tables
    ORDER BY pg_total_relation_size(relid) DESC LIMIT 10;"
```

## Mitigation — disk

```bash
# 1. Rotate logs first: it is the cheapest space and never loses product data.
sudo find /opt/gbo/logs -name "*.log.*" -mtime +7 -delete
sudo find /opt/gbo/logs -name "stdout.log" -size +500M -exec truncate -s 200M {} \;

# 2. Prune the operational counters. The middleware keeps 30 days; lowering the
#    retention is safe and does not touch product data.
psql -h localhost -U postgres -d botserver -c \
  "DELETE FROM ops_request_metrics WHERE bucket < NOW() - INTERVAL '7 days';"

# 3. Vacuum the tables that grew.
psql -h localhost -U postgres -d botserver -c "VACUUM (ANALYZE) message_history;"

# 4. Only then consider moving MinIO data to a larger volume.
```

## Mitigation — failed migration

```bash
# 1. Read the exact failing statement; never re-run blindly.
sudo incus exec bot -- grep -A 20 "migration" /opt/gbo/logs/stdout.log | tail -40

# 2. Take a dump before touching the schema.
sudo incus exec tables -- pg_dump -U postgres -F c -f /tmp/pre-migration.dump botserver

# 3. If the migration is partially applied, follow
#    restore PostgreSQL/MinIO before rolling the binary back.
```

## Mitigation — stuck queue

```bash
# 1. Find the oldest queued run and its age.
psql -h localhost -U postgres -d botserver -c \
  "SELECT id, status, created_at, started_at, error
     FROM agent_runs WHERE status IN ('queued','running')
    ORDER BY created_at ASC LIMIT 10;"

# 2. Runs stuck in 'running' past their deadline are dead: mark them failed so
#    the scheduler can retry or the operator can see the failure.
psql -h localhost -U postgres -d botserver -c \
  "UPDATE agent_runs SET status = 'failed', finished_at = NOW(),
          error = COALESCE(error, 'reaped by runbook-disk-and-queue')
    WHERE status = 'running' AND started_at < NOW() - INTERVAL '2 hours';"

# 3. Restart the worker that drains the queue if depth still does not fall.
sudo incus exec bot -- systemctl restart botserver
```

## Verification

```bash
df -h /
psql -h localhost -U postgres -d botserver -c \
  "SELECT status, COUNT(*) FROM agent_runs GROUP BY status;"
```

Expected: disk below 80 %, `queued` depth falling between two consecutive
reads, and the Queue depth panel showing a wait p95 that stops growing. A smoke
chat must answer within the SLO.

## Rollback

Nothing here is destructive except the reaper update and the counter prune, and
both are additive (status change, retention delete). A migration rollback is a
binary rollback plus a database restore — follow
[deploy and rollback](./runbook-deploy-rollback.md) and
[restore PostgreSQL/MinIO](./runbook-restore-backup.md).

## Escalation

Owner: `platform-oncall`. Escalate to `storage-oncall` when the volume that is
full holds MinIO, and to the incident commander when the queue cannot be
drained without dropping work.
