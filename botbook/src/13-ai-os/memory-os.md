# Memory OS

Durable, editable per-user memory with branch-scoped sharing:

- Extraction: after assistant turns, durable facts and preferences are
  proposed by the model (sampled; explicit phrases such as "remember that"
  bypass sampling) and deduplicated on write (skip or supersede).
- Recall: relevant memories are injected as context before generation with a
  token budget; pinned memories rank first.
- Management: the **Memory** app lists, edits, pins, merges and deletes
  entries; export/import uses JSON with a dry-run report.
- Forgetting: explicit "forget …" turns delete only the rows the request
  names. Matching uses a conservative four-character stem (`writing` matches
  `write`, `tables` matches `table`) and ignores two-letter locale tokens such
  as `pt`/`br`. A subject-bearing request never deletes every row; only a pure
  "forget everything"/"esqueça tudo" does. When nothing matches, the assistant
  reports that instead of claiming success.

Legacy `REMEMBER`/`RECALL` keywords continue to operate unchanged.
