# Runbook: rotate Vault secrets and provider tokens

Alert: a credential leaked, a provider token expired, or the rotation window
arrived. Panel: **Analytics → Operations → LLM cost & tokens** (failed calls)
and **Requests by endpoint class** (auth 5xx).

## Trigger

- A secret was committed, logged or shared outside Vault.
- An LLM provider, mail provider or channel token expired.
- Scheduled rotation (90 days for provider keys, 180 days for internal
  service credentials).

## Impact

Rotating a secret that the running process already cached has no visible
effect until the next restart or reconnect. Rotating a secret that is read per
request (Drive, LLM, mail) breaks the product the moment the old value is
revoked. Always update Vault first, then restart, then revoke.

## Diagnosis

```bash
# 1. Which secrets exist for this deployment?
vault kv list secret/gbo
vault kv get secret/gbo/llm
vault kv get secret/gbo/drive

# 2. Is the running process failing to reach a dependency?
sudo incus exec bot -- tail -200 /opt/gbo/logs/stdout.log | grep -iE "401|403|unauthorized|expired"

# 3. Which provider is failing? The cost panel splits failures by call, and
#    llm_metrics carries the error text.
psql -h localhost -U postgres -d botserver -c \
  "SELECT model, COUNT(*) AS failures, MAX(error) AS last_error
     FROM llm_metrics
    WHERE NOT success AND timestamp > NOW() - INTERVAL '1 hour'
    GROUP BY model ORDER BY failures DESC;"
```

## Mitigation

```bash
# 1. Write the new value to Vault FIRST (never to a file in the repository).
vault kv put secret/gbo/llm url=<chat-completions-url> model=<model> openai_key=<new-key>

# 2. Per-bot overrides live under secret/gbo/<org_id>/<branch_id>/<bot_id>.
#    Resolve the ids from the database, never from memory.
psql -h localhost -U postgres -d botserver -c \
  "SELECT id, org_id, branch_id, name FROM bots WHERE name = '<bot>';"

# 3. Restart the process that cached the secret. The LLM provider is built at
#    boot, so a Vault-only change is invisible until the restart.
sudo incus exec bot -- systemctl restart botserver

# 4. Revoke the old credential at the provider only after step 3 succeeded.
```

## Verification

```bash
# Health first, then a real completion.
sudo incus exec bot -- curl -s -o /dev/null -w '%{http_code}\n' http://localhost:5858/health

# Failed calls must stop growing.
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) FROM llm_metrics
    WHERE NOT success AND timestamp > NOW() - INTERVAL '10 minutes';"

# Drive: one listing proves the new key works.
sudo incus exec bot -- mc ls local/
```

Expected: `/health` returns `200`, no new failed LLM calls, and the cost panel
shows calls succeeding again. The SLO panel's `first_token_p95` returns inside
budget within one burn window.

## Rollback

Vault keeps versioned secrets: read the previous version instead of inventing a
value.

```bash
vault kv get -version=<previous> secret/gbo/llm
vault kv put secret/gbo/llm url=<old-url> model=<old-model> openai_key=<old-key>
sudo incus exec bot -- systemctl restart botserver
```

## Escalation

Owner: `platform-oncall`. Escalate to the incident commander when a leaked
credential grants access to tenant data: that is SEV1 and requires the tenant
notification process, not just a rotation.
