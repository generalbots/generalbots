# Caching

BotServer includes automatic caching to improve response times and reduce redundant processing.

## How Caching Works

Caching in BotServer is controlled by configuration parameters in `config.csv`. The system automatically caches LLM responses and manages conversation history.

## Configuration

From `default.gbai/default.gbot/config.csv`:

```csv
llm-cache,false              # Enable/disable LLM response caching
llm-cache-ttl,3600          # Cache time-to-live in seconds
llm-cache-semantic,true     # Use semantic similarity for cache matching
llm-cache-threshold,0.95    # Similarity threshold for cache hits
```

## Conversation History Management

The system manages conversation context through these parameters:

```csv
prompt-history,2    # Number of previous messages to include in context
prompt-compact,4    # Compact conversation after N exchanges
```

### What These Settings Do

- **prompt-history**: Keeps the last 2 exchanges in the conversation context
- **prompt-compact**: After 4 exchanges, older messages are summarized or removed to save tokens

## LLM Response Caching

When `llm-cache` is enabled:

1. User asks a question
2. System checks if a semantically similar question was asked before
3. If similarity > threshold (0.95), returns cached response
4. Otherwise, generates new response and caches it

## Example Usage

```basic
' Caching happens automatically when enabled
USE KB "policies"

' First user asks: "What's the vacation policy?"
' System generates response and caches it

' Second user asks: "Tell me about vacation days"
' System finds cached response (high semantic similarity)
' Returns instantly without calling LLM
```

## Cache Storage

The cache is stored in the cache component (Valkey) when available, providing:
- Fast in-memory access
- Persistence across restarts
- Shared cache across sessions

## Benefits

- **Faster responses** for common questions
- **Lower costs** by reducing LLM API calls
- **Consistent answers** for similar questions
- **Automatic management** with no code changes

## Best Practices

1. **Enable for FAQ bots** - High cache hit rate
2. **Adjust threshold** - Lower for more cache hits, higher for precision
3. **Set appropriate TTL** - Balance freshness vs performance
4. **Monitor cache hits** - Ensure it's providing value

## Performance Impact

With caching enabled:
- Common questions: <50ms response time
- Cache misses: Normal LLM response time
- Memory usage: Minimal (only stores text responses)

## Clearing Cache

Cache is automatically cleared when:
- TTL expires (after 3600 seconds by default)
- Bot configuration changes
- Knowledge base is updated
- System restarts (if not using persistent cache)

## Important Notes

- Caching is transparent to dialog scripts
- No special commands needed
- Works with all LLM providers
- Respects conversation context

Remember: Caching is configured in `config.csv`, not through BASIC commands!