---
name: webhooks
description: |
  Receive webhooks safely: HMAC verification, idempotency, retries and dead letters. Use when
  integrating event callbacks.
version: 1.0.0
source: in-repo
---

# webhooks

## Workflow

1. Verify the signature over the raw body before parsing; reject on mismatch.
2. Store the event id and make handling idempotent.
3. Acknowledge fast (2xx) and process asynchronously.
4. Handle out-of-order delivery with timestamps or sequence numbers.
5. Retry your own downstream calls with backoff; dead-letter permanent failures.
6. Log the event id and outcome, never the secret.

## Quality checks

- A replayed event does not duplicate the effect.
- An invalid signature is rejected before any side effect.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
