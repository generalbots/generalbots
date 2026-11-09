---
name: gmail-workflows
description: |
  Search, label, draft and send Gmail with correct MIME and scopes. Use for mailbox automation.
version: 1.0.0
source: in-repo
---

# gmail-workflows

## Workflow

1. Search with Gmail query syntax (from:, newer_than:, has:attachment) before fetching bodies.
2. Use threads.get to keep context; reply within the thread.
3. Compose MIME with text and HTML alternatives; attachments base64 with correct type.
4. Draft first when the user must review; send only on explicit instruction.
5. Label with user labels (never system labels unless asked).
6. Respect quotas; batch small requests.

## Quality checks

- Threading is preserved on replies.
- No send without explicit user confirmation when drafting was requested.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
