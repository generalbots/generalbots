# Runbook: restore PostgreSQL or MinIO from backup

Alert: data loss, corruption, a failed migration, or any doubt about tenant
isolation. Panel: **Analytics → Operations → Traffic** (turns stop) and
**Requests by endpoint class** (5xx spike).

## Trigger

- A migration failed halfway and left the schema inconsistent.
- Rows disappeared, or a tenant reports seeing another tenant's data.
- MinIO refuses writes or lost objects.

## Impact

Restoring PostgreSQL stops the product: every bot, session and queue read goes
through it. Restoring MinIO breaks Drive, knowledge bases and file delivery.
Plan the window with the incident commander; a restore is always SEV1.

## Diagnosis

```bash
# 1. Is the database reachable at all?
sudo incus exec tables -- pg_isready -U postgres

# 2. Which migration is the last applied one?
psql -h localhost -U postgres -d botserver -c \
  "SELECT version, run_on FROM __diesel_schema_migrations ORDER BY run_on DESC LIMIT 5;"

# 3. Is the damage data or schema? Compare a known table's row count with the
#    last backup manifest before doing anything destructive.
psql -h localhost -U postgres -d botserver -c "SELECT COUNT(*) FROM bots;"

# 4. Tenant isolation doubt: check that every bot row has an org.
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) AS bots_without_org FROM bots WHERE org_id IS NULL;"

# 5. MinIO health.
sudo incus exec drive -- mc admin info local
```

## Mitigation — PostgreSQL

```bash
# 1. Take a safety dump of the current (broken) state before restoring. It is
#    often the only forensic copy.
sudo incus exec tables -- pg_dump -U postgres -F c -f /tmp/pre-restore.dump botserver

# 2. Restore into a scratch database first, never straight into the live one.
sudo incus exec tables -- createdb -U postgres botserver_restore
sudo incus exec tables -- pg_restore -U postgres -d botserver_restore /opt/gbo/data/backup/botserver.dump

# 3. Verify the scratch copy: row counts, the last migration, and the
#    tenant-isolation query above.
psql -h localhost -U postgres -d botserver_restore -c \
  "SELECT version FROM __diesel_schema_migrations ORDER BY run_on DESC LIMIT 1;"

# 4. Only after the scratch copy verifies: stop the service, rename the live
#    database and promote the restore.
sudo incus exec bot -- systemctl stop botserver
psql -h localhost -U postgres -c "ALTER DATABASE botserver RENAME TO botserver_broken;"
psql -h localhost -U postgres -c "ALTER DATABASE botserver_restore RENAME TO botserver;"
sudo incus exec bot -- systemctl start botserver
```

## Mitigation — MinIO

```bash
# 1. Inspect the bucket that is failing (each workspace is a .gborg/.gbai tree).
sudo incus exec drive -- mc ls local/

# 2. Restore the affected prefix from the backup mirror.
sudo incus exec drive -- mc mirror --overwrite /opt/gbo/data/backup/drive/<bucket> local/<bucket>

# 3. Never restore a whole drive over a live one while bots are writing: stop
#    the affected bots first (see runbook-disable-tenant.md).
```

## Verification

```bash
# Tenant isolation must pass before the incident is closed.
cargo test -p bottest --features integration tenant_isolation

# A smoke chat proves the bot can read its own bucket.
#   Analytics → Operations → Traffic must show turns again within 5 minutes.
```

Expected: the isolation suite is green, a smoke chat answers, and the Traffic
panel shows turns and sessions again.

## Rollback

Keep `botserver_broken` until the postmortem closes. If the restore is wrong,
rename it back:

```bash
psql -h localhost -U postgres -c "ALTER DATABASE botserver RENAME TO botserver_failed_restore;"
psql -h localhost -U postgres -c "ALTER DATABASE botserver_broken RENAME TO botserver;"
```

## Escalation

Owner: `storage-oncall` for MinIO, `platform-oncall` for PostgreSQL. Escalate
to the incident commander immediately when tenant isolation is in doubt — that
is always SEV1, regardless of how few rows are involved.
