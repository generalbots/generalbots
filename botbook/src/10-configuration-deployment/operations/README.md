# Operations: on-call, SLOs and runbooks 🟡 BETA

This chapter is the operating manual for the platform: who is on call, which
service levels are promised, how alerts reach a human and what to do when one
fires. The numbers live in `ops/slo.yaml`; the dashboards that read them live
in the Analytics app (**Analytics → Operations** tab).

## Where the operational view lives

| Panel | Endpoint | Answers |
|-------|----------|---------|
| SLO burn & error budget | `GET /api/ui/analytics/ops/slo` | Are we inside the error budget for API latency, availability, first token, answer latency and WebSocket handshakes? |
| Traffic | `GET /api/ui/analytics/ops/traffic?window=<min>` | Turns per bucket, sessions, WebSocket upgrades, active bots/tenants, deploy markers |
| Requests by endpoint class | `GET /api/ui/analytics/ops/endpoints?window=<min>` | Request rate, 5xx rate, p95 and traffic share per endpoint class |
| Queue depth | `GET /api/ui/analytics/ops/queue?window=<min>` | Depth, running, dead letters and wait p95 per queue |
| LLM cost & tokens | `GET /api/ui/analytics/ops/cost?window=<min>` | Tokens, spend, cost per conversation, spend per tenant, daily quota burn |

Every panel is empty-but-honest when its source has no rows: the UI prints
"no data" instead of a fabricated series. The request counters come from
`ops_request_metrics` (written by the server's ops middleware every 30 s), LLM
numbers from `llm_metrics`, queues from `agent_runs`, `subagent_runs`,
`handoff_queue` and `email_offline_queue`, deploy markers from
`ops_deploy_markers`.

Monitoring (host CPU, memory, disk, process health) stays in the Monitoring
app. This chapter does not duplicate it: Analytics owns service levels and
business-facing operations, Monitoring owns the host.

## Ownership

| Scope | Owner | Backup |
|-------|-------|--------|
| Platform (API, WebSocket, queues) | `platform-oncall` | `platform-backup` |
| LLM providers and model routing | `ai-oncall` | `platform-oncall` |
| Drive / storage (MinIO) | `storage-oncall` | `platform-oncall` |
| Chat and WhatsApp channels | `channels-oncall` | `platform-oncall` |

The owner of a scope is the person allowed to decide a rollback for it. The
rotation is recorded in `ops/slo.yaml` (`owner:`), so the dashboard header and
the alert routing always agree.

## Severity and response times

| Severity | Meaning | First response | Update cadence |
|----------|---------|----------------|----------------|
| SEV1 | Users cannot chat, or data loss is possible | 15 min, 24/7 | every 30 min |
| SEV2 | A dependency is degraded; answers are slow or a channel is down | 1 h in business hours, 4 h otherwise | every 2 h |
| SEV3 | Budget burn without user impact, single tenant issue | next business day | daily |

Severity follows the burn rate shown in the SLO panel:

| Burn window | Threshold | Severity |
|-------------|-----------|----------|
| 1 h | ≥ 14.4× budget | SEV1 — page |
| 6 h | ≥ 6× budget | SEV2 — page in business hours |
| 3 d | ≥ 1× budget | SEV3 — ticket |

A "no data" SLO panel is itself an alert: the dashboard reads the same database
the product uses, so an empty panel usually means the database, the middleware
or the migration is broken. Treat it as SEV2 until proven otherwise.

## Who can deploy at 2 a.m.

- The on-call engineer may **roll back** any deployment without approval.
- The on-call engineer may **deploy forward** only with the approval of the
  scope owner or, when unreachable for 15 minutes, of the incident commander.
- Nobody deploys a schema migration at 2 a.m. unless the migration is the fix
  for the incident and the backup restore path has been verified.

## Alert routing

1. Burn-rate alerts go to the on-call channel of the scope owner, with the
   runbook link and the dashboard panel anchor attached.
2. Each alert names exactly one runbook; the mapping lives in `ops/slo.yaml`
   (`objective.<id>.runbook`).
3. Acknowledging an alert without opening its runbook is not an
   acknowledgement.
4. Escalation: on-call → backup → scope owner → incident commander. If the
   first responder has not acknowledged in 15 minutes, the backup is paged.

## Runbooks

| Runbook | Use when |
|---------|----------|
| [Deploy and rollback](./runbook-deploy-rollback.md) | A release must ship or be reverted (binary, UI, desktop payload) |
| [Restore PostgreSQL/MinIO](./runbook-restore-backup.md) | Data loss, corruption, failed migration, tenant isolation doubt |
| [Rotate Vault secrets](./runbook-rotate-secrets.md) | Credential leak, provider token expiry, rotation window |
| [Drain or disable a tenant](./runbook-disable-tenant.md) | One tenant/bot must be stopped, or an app/channel needs a kill switch |
| [LLM provider outage](./runbook-llm-outage.md) | Provider errors, first-token/answer SLO burn, quota exhaustion |
| [Disk full / stuck queue](./runbook-disk-and-queue.md) | Disk pressure, failed migration, queue depth growing without draining |

## Postmortem template

Copy this into the issue tracker when an incident closes. The postmortem is not
closed until every action item is an issue with an owner and a date.

```markdown
## Summary
One paragraph: what broke, for whom, for how long.

## Impact
- Users affected:
- Tenants/bots affected:
- Channels affected:
- SLO budget consumed (from Analytics → Operations → SLO):
- Severity:

## Timeline (UTC)
- HH:MM first signal (which panel/alert)
- HH:MM acknowledgement
- HH:MM mitigation started
- HH:MM recovery confirmed
- HH:MM incident closed

## Root cause
The mechanism, not the symptom. Link the change that introduced it.

## Detection gap
Why did the SLO panel, the alert or a user notice it first?

## What worked / what did not
Runbook steps that were wrong, missing or misleading.

## Action items
| Action | Owner | Issue | Due |
|--------|-------|-------|-----|
```

## Game day record

A drill is only useful when its timings and gaps are written down. Record one
row per scenario exercised:

| Scenario | Expected | Observed | Gap found | Issue |
|----------|----------|----------|-----------|-------|
| LLM provider killed for 5 min | Users get a clear fallback message | | | |
| PostgreSQL stopped for 60 s | Reconnect or fail gracefully | | | |
| Backup restored into a scratch DB | Tenant-isolation suite passes, smoke chat works | | | |
| One deploy rolled back | Previous version serves traffic | | | |

## See also

- [Secrets Management](../secrets-management.md)
- [System Limits](../system-limits.md)
- [Drive](../drive.md)
- [LLM Configuration](../llm-config.md)
