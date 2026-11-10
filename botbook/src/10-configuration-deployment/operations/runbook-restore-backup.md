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

## Restore drill (rehearse the restore)

`scripts/restore_drill.sh` is the rehearsed version of the scratch-restore above:
it takes a real dump, restores it into a throwaway database and asserts the copy
before anyone is asked to trust it. Run it monthly, before a release, and after
any migration that touches a tenant-scoped table.

```bash
# 1. Offline proof that the assertions themselves work — no server needed.
scripts/restore_drill.sh --self-test

# 2. Dump the source, restore into gb_restore_drill_<pid>, assert, drop it.
scripts/restore_drill.sh --dsn "$DATABASE_URL" --report /tmp/gb-restore-drill.md

# 3. Rehearse a dump you already have (no dump is taken; <dump>.manifest.json
#    is used for the row-count comparison when it sits next to the file).
scripts/restore_drill.sh --dsn "$DATABASE_URL" --dump /opt/gbo/data/backup/botserver.dump

# 4. Assert on a database that was restored by hand, without another restore.
scripts/restore_drill.sh --dsn "$RESTORED_URL" --verify-only

# 5. Keep the throwaway database and the dump for forensics.
scripts/restore_drill.sh --dsn "$DATABASE_URL" --keep
```

The drill asserts, on the restored copy:

- the core tables (`organizations`, `branches`, `bots`, `users`) exist and
  `__diesel_schema_migrations` is non-empty — a restore that lost the schema or
  the migration history fails;
- every table has exactly the row count recorded in the source manifest — a
  short or missing table names itself in the failure;
- every generated tenant probe returns zero rows: for each table carrying
  `org_id`, `branch_id`, `bot_id` or `tenant_id`, rows that point at another
  organization, branch or bot are counted, and one non-zero probe fails the
  drill.

Exit codes: `0` pass, `1` mismatch (the restored copy is not trustworthy), `2`
a tool or input is missing. The Markdown report written by `--report` is the
evidence to attach to the incident or the release.

The same drill runs unattended in CI: the `data-integrity` job in
`.github/workflows/ci.yml` loads a deterministic dataset, runs the Rust
integrity suite, then runs `scripts/restore_drill.sh` against that database.

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
