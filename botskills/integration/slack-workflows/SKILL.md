---
name: slack-workflows
description: |
  Post, thread and triage Slack messages through the API. Use for Slack automation.
version: 1.0.0
source: in-repo
---

# slack-workflows

## Workflow

1. Use chat.postMessage with channel id (not name) and thread_ts for replies.
2. Format with Block Kit for structure; plain text fallback always included.
3. Respect rate limits (429 with Retry-After) and back off.
4. For triage, react/assign through the documented endpoints, not message editing.
5. Keep tokens server-side; never in client code.
6. Confirm delivery from the API response, not optimistically.

## Quality checks

- Messages render when blocks are unsupported (fallback text).
- Rate limits are handled without dropping messages.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
