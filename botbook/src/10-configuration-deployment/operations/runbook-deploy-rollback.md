# Runbook: deploy and rollback

Alert: `error_rate` burn (see [operations README](./README.md)) or a release
that must ship. Panel: **Analytics → Operations → Requests by endpoint class**.

## Trigger

- A new version must reach production (binary, UI assets or desktop payload).
- Or the running version is failing and must be reverted.

## Impact

Deploys interrupt the process that restarts; in-flight WebSocket sessions
reconnect. Rollbacks restore the previous binary but never touch the database,
so a migration that already ran stays applied — check the migration section
before rolling back a release that included one.

## Diagnosis

```bash
# What is running, and since when? The deploy markers are recorded at boot.
# Analytics → Operations → Traffic lists them; the same rows are in SQL:
psql -h localhost -U postgres -d botserver -c \
  "SELECT version, commit_hash, build_date, deployed_at
     FROM ops_deploy_markers ORDER BY deployed_at DESC LIMIT 5;"

# Which build is the binary?
#   GET /api/product returns the version, commit and build date.

# Are the request counters moving (i.e. is the server serving)?
psql -h localhost -U postgres -d botserver -c \
  "SELECT endpoint_class, SUM(request_count) AS requests,
          SUM(CASE WHEN status_class = '5xx' THEN request_count ELSE 0 END) AS errors
     FROM ops_request_metrics
    WHERE bucket > NOW() - INTERVAL '15 minutes'
    GROUP BY endpoint_class ORDER BY requests DESC;"
```

## Mitigation — deploy

```bash
# 1. Push to the ALM remote; CI builds and deploys. Never copy binaries by hand.
git push alm main
git push origin main

# 2. Watch the run until it finishes.
#    ALM listens on port 4747 (not 3000).
curl -s "http://<ALM_HOST>:4747/api/v1/repos/GeneralBots/BotServer/actions/runs?per_page=1"

# 3. Confirm the new build: /api/product must report the new version, and the
#    Traffic panel must show a new deploy marker at the deploy time.
```

## Mitigation — rollback

```bash
# 1. Stop the service BEFORE replacing the binary ("text file busy" otherwise).
sudo incus exec bot -- systemctl stop botserver

# 2. Push the previous known-good binary and start it.
sudo incus file push /opt/gbo/bin/botserver.previous bot/opt/gbo/bin/botserver
sudo incus exec bot -- bash -c 'chmod +x /opt/gbo/bin/botserver && systemctl start botserver'

# 3. UI-only rollback: push the previous ui/ directory. Static files are read
#    per request, so no restart is needed.
sudo incus file push -r /opt/gbo/bin/ui.previous bot/opt/gbo/bin/ui
```

A migration is **not** rolled back by the binary rollback. If the release
included a migration, follow [restore PostgreSQL/MinIO](./runbook-restore-backup.md)
before assuming the old binary will boot.

## Verification

```bash
# Health, then the SLO panel.
sudo incus exec bot -- curl -s -o /dev/null -w '%{http_code}\n' http://localhost:5858/health

# Error rate must fall back inside the budget within one burn window.
psql -h localhost -U postgres -d botserver -c \
  "SELECT SUM(request_count) AS requests,
          SUM(CASE WHEN status_class = '5xx' THEN request_count ELSE 0 END) AS errors
     FROM ops_request_metrics WHERE bucket > NOW() - INTERVAL '10 minutes';"
```

Expected: `/health` returns `200`; the SLO panel shows the `error_rate`
objective back to `within budget`; a new deploy marker appears in Traffic.

## Rollback of the rollback

Re-deploy the version that was rolled back only after the root cause is known
and the failing condition is fixed. Two rollbacks in a row without a diagnosis
is an escalation trigger.

## Escalation

Owner: `platform-oncall` (see `ops/slo.yaml`). Escalate to the incident
commander if the service does not answer `/health` within 15 minutes of the
rollback, or if the rollback requires restoring data.
