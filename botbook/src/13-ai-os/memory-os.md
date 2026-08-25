# Memory OS

Durable, editable per-user memory with branch-scoped sharing:

- Extraction: after assistant turns, durable facts and preferences are
  proposed by the model (sampled; explicit phrases such as "remember that"
  bypass sampling) and deduplicated on write (skip or supersede).
- Recall: relevant memories are injected as context before generation with a
  token budget; pinned memories rank first.
- Management: the **Memory** app lists, edits, pins, merges and deletes
  entries; export/import uses JSON with a dry-run report.

Legacy `REMEMBER`/`RECALL` keywords continue to operate unchanged.
