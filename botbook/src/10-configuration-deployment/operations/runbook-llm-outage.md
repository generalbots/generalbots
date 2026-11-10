# Runbook: LLM provider outage

Alert: `first_token_p95` or `answer_p95` burn, or `error_rate` burn caused by
LLM calls. Panel: **Analytics → Operations → LLM cost & tokens** (failed
calls) and **SLO burn**.

## Trigger

- The provider returns 5xx, times out, or rejects the key.
- First-token latency crosses the threshold long enough to burn the budget.
- A quota is exhausted and calls start failing.

## Impact

Chat answers stall or fail. Tool calls that do not need the model keep working,
and so do channels that use scripted replies, so the product degrades instead
of stopping. Users must see an explanation and a retry action — never a blank
answer.

## Diagnosis

```bash
# 1. What is failing, and with which error?
psql -h localhost -U postgres -d botserver -c \
  "SELECT model, COUNT(*) AS failures, MAX(error) AS last_error
     FROM llm_metrics
    WHERE NOT success AND timestamp > NOW() - INTERVAL '30 minutes'
    GROUP BY model ORDER BY failures DESC;"

# 2. Is it latency or errors? p95 of both signals in the last hour.
psql -h localhost -U postgres -d botserver -c \
  "SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY ttft_ms)    AS ttft_p95,
          percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms) AS answer_p95,
          COUNT(*) FILTER (WHERE NOT success) AS failures,
          COUNT(*) AS calls
     FROM llm_metrics WHERE timestamp > NOW() - INTERVAL '1 hour';"

# 3. Is the provider reachable at all?
curl -s -o /dev/null -w '%{http_code}\n' <llm-url>

# 4. Is the local model server up (self-hosted deployments)?
sudo incus exec llm -- pgrep -a llama-server || sudo incus exec llm -- systemctl status llm
```

## Mitigation

1. **Switch provider or model.** Model routing supports a fallback chain; the
   configured chain is documented in [Model Routing](../../13-ai-os/model-routing.md).
   Point the bot (or the global fallback in `secret/gbo/llm`) at a healthy
   model, then restart the process so the provider is rebuilt.

```bash
vault kv put secret/gbo/llm url=<healthy-chat-completions-url> model=<healthy-model> openai_key=<key>
sudo incus exec bot -- systemctl restart botserver
```

2. **Stop the burn while the provider is down.** A degraded answer is better
   than a hung request: reduce the per-request timeout and let the fallback
   message run. The user-facing text must name the outage and offer a retry.

3. **Protect the budget.** If a single bot or tenant is consuming the quota,
   drain it with [drain or disable a tenant](./runbook-disable-tenant.md)
   before rotating providers, so the fallback capacity serves everyone else.

4. **Never mock in production.** A mock provider is for staging drills only;
   in production, degrade with an explicit message and keep the real error in
   `llm_metrics` so the postmortem has evidence.

## Verification

```bash
# Failed calls must stop accumulating.
psql -h localhost -U postgres -d botserver -c \
  "SELECT COUNT(*) FROM llm_metrics
    WHERE NOT success AND timestamp > NOW() - INTERVAL '5 minutes';"

# And first-token latency must return under the threshold.
psql -h localhost -U postgres -d botserver -c \
  "SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY ttft_ms)
     FROM llm_metrics WHERE timestamp > NOW() - INTERVAL '10 minutes';"
```

Expected: no new failures, `ttft_p95` under the 2 s threshold in
`ops/slo.yaml`, and the SLO panel back to `within budget` within one burn
window. Then run one real chat turn in the browser: the answer must stream
normally, not fall back.

## Rollback

Restore the previous provider entry (Vault keeps versions) and restart. If the
fallback is what is failing, remove it so traffic returns to the primary.

## Escalation

Owner: `ai-oncall`. Escalate to `platform-oncall` when the outage also affects
the local model server or the vector database, and to the incident commander
when the outage lasts more than 30 minutes (SEV1 by duration).
