# Context Compaction

When a conversation grows long, the bot’s context window can exceed the LLM’s token limit. **Context compaction** reduces the stored history while preserving essential information.

## Strategies

1. **Summarization** – Periodically run `TALK FORMAT` with a summarization prompt and replace older messages with the summary.
2. **Memory Pruning** – Use `SET_BOT_MEMORY` to store only key facts (e.g., user name, preferences) and discard raw chat logs.
3. **Chunk Rotation** – Keep a sliding window of the most recent *N* messages (configurable via `context_window` in `.gbot/config.csv`).

## Implementation Example

```basic
' After 10 exchanges, summarize
IF MESSAGE_COUNT >= 10 THEN
  TALK "Summarizing recent conversation..."
  SET_BOT_MEMORY "summary" FORMAT(RECENT_MESSAGES, "summarize")
  CLEAR_MESSAGES   ' removes raw messages
ENDIF
```

## Configuration

- `context_window` (in `.gbot/config.csv`) defines how many recent messages are kept automatically.
- `memory_enabled` toggles whether the bot uses persistent memory.

## Benefits

- Keeps token usage within limits.
- Improves response relevance by focusing on recent context.
- Allows long‑term facts to persist without bloating the prompt.

## Caveats

- Over‑aggressive pruning may lose important details.
- Summaries should be concise (max 200 tokens) to avoid re‑inflating the context.
