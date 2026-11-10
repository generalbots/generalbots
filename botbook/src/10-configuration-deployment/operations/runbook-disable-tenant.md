# Runbook: drain or disable one tenant, bot, app or channel

Alert: `ws_handshake` burn or a single tenant misbehaving. Panel:
**Analytics → Operations → Traffic** (sessions per window) and **Requests by
endpoint class**.

## Trigger

- One tenant or bot is abusing the platform (runaway loops, quota burn).
- A channel is broken and must stop accepting messages.
- An app must be switched off without taking the whole product down.

## Impact

Draining stops new work for the target only; the rest of the platform keeps
serving. Disabling a bot hides it from its channels. Neither operation deletes
data — both are reversible.

## Diagnosis

```bash
# 1. Which bot is burning? Sort by LLM calls in the window.
psql -h localhost -U postgres -d botserver -c \
  "SELECT b.name, COUNT(*) AS calls, SUM(m.estimated_cost) AS cost
     FROM llm_metrics m JOIN bots b ON b.id = m.bot_id
    WHERE m.timestamp > NOW() - INTERVAL '1 hour'
    GROUP BY b.name ORDER BY cost DESC LIMIT 10;"

# 2. Which sessions are still open for that bot?
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) FROM user_sessions
    WHERE bot_id = (SELECT id FROM bots WHERE name = '<bot>')
      AND updated_at > NOW() - INTERVAL '30 minutes';"

# 3. Which channels are bound to it?
psql -h localhost -U postgres -d botserver -c \
  "SELECT channel_type, is_active FROM bot_channels
    WHERE bot_id = (SELECT id FROM bots WHERE name = '<bot>');"
```

## Mitigation — disable a bot

```bash
# Deactivate the bot: drive monitors stop materializing new work for it and the
# channels stop routing to it.
psql -h localhost -U postgres -d botserver -c \
  "UPDATE bots SET is_active = false WHERE name = '<bot>';"
```

## Mitigation — kill switch for one channel

```bash
# Turn off a single channel binding (channel_type is the numeric id stored on
# the row; read it from the query above before writing).
psql -h localhost -U postgres -d botserver -c \
  "UPDATE bot_channels SET is_active = false
    WHERE bot_id = (SELECT id FROM bots WHERE name = '<bot>')
      AND channel_type = <channel_type>;"
```

## Mitigation — drain a tenant

```bash
# 1. Stop new sessions for the tenant by deactivating its bots.
psql -h localhost -U postgres -d botserver -c \
  "UPDATE bots SET is_active = false
    WHERE org_id = (SELECT org_id FROM organizations WHERE slug = '<tenant>');"

# 2. Let the open sessions finish (or close them explicitly, which the user
#    sees as a disconnect — warn the tenant first).
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) FROM user_sessions us JOIN bots b ON b.id = us.bot_id
    WHERE b.org_id = (SELECT org_id FROM organizations WHERE slug = '<tenant>')
      AND us.updated_at > NOW() - INTERVAL '30 minutes';"
```

## Verification

```bash
# The bot must disappear from the active fleet.
psql -h localhost -U postgres -d botserver -c \
  "SELECT name, is_active FROM bots WHERE name = '<bot>';"
```

Then confirm in the dashboard: **Requests by endpoint class** shows the
tenant's traffic share falling to zero in the next window, and the tenant is
absent from **Cost per tenant**.

Expected: `is_active = false`, the tenant is absent from the cost panel's
tenant table, and the SLO panel stops burning on the tenant's traffic.

## Rollback

```bash
psql -h localhost -U postgres -d botserver -c \
  "UPDATE bots SET is_active = true WHERE name = '<bot>';"
```

Re-enabling a channel is the same update with `is_active = true`. A disabled
bot is re-materialized by the drive monitor on its next pass.

## Escalation

Owner: `channels-oncall` for channel kill switches, `platform-oncall` for
tenant drains. Escalate to the incident commander when a tenant must be
disabled for a data-protection reason.
