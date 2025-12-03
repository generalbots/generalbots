# Caching

BotServer includes automatic caching to improve response times and reduce redundant processing, including semantic caching for LLM responses using an in-memory cache component.

<img src="../assets/chapter-03/caching-architecture.svg" alt="Caching Architecture" style="max-height: 400px; width: 100%; object-fit: contain;">

## Features

The caching system provides exact match caching for identical prompts and semantic similarity matching to find and reuse responses for semantically similar prompts. Configurable TTL settings control how long cached responses remain valid. Caching can be enabled or disabled on a per-bot basis through configuration. Embedding-based similarity uses local embedding models for semantic matching, and comprehensive statistics and monitoring track cache hits, misses, and performance metrics.

## How Caching Works

Caching in BotServer is controlled by configuration parameters in `config.csv`. The system automatically caches LLM responses and manages conversation history.

When enabled, the semantic cache operates through a straightforward process. When a user asks a question, the system checks if a semantically similar question was asked before. If the similarity exceeds the threshold (typically 0.95), it returns the cached response. Otherwise, it generates a new response and caches it for future queries.

## Configuration

### Basic Cache Settings

From `default.gbai/default.gbot/config.csv`:

```csv
llm-cache,false              # Enable/disable LLM response caching
llm-cache-ttl,3600          # Cache time-to-live in seconds
llm-cache-semantic,true     # Use semantic similarity for cache matching
llm-cache-threshold,0.95    # Similarity threshold for cache hits
```

### Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `llm-cache` | boolean | false | Enable/disable LLM response caching |
| `llm-cache-ttl` | integer | 3600 | Time-to-live for cached entries (in seconds) |
| `llm-cache-semantic` | boolean | true | Enable semantic similarity matching |
| `llm-cache-threshold` | float | 0.95 | Similarity threshold for semantic matches (0.0-1.0) |

### Embedding Service Configuration

For semantic similarity matching, ensure your embedding service is configured:

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

## Conversation History Management

The system manages conversation context through these parameters:

```csv
episodic-memory-history,2    # Number of previous messages to include in context
episodic-memory-threshold,4    # Compact conversation after N exchanges
```

The `episodic-memory-history` setting keeps the last 2 exchanges in the conversation context, providing continuity without excessive token usage. The `episodic-memory-threshold` setting triggers summarization or removal of older messages after 4 exchanges to save tokens while preserving essential context.

## Cache Storage

### Architecture

The caching system uses a multi-level approach for optimal performance, combining fast in-memory access with configurable persistence options.

### Cache Key Structure

The cache uses a multi-level key structure where exact matches use a hash of the exact prompt while semantic matches store embedding vectors with a semantic index for similarity comparison.

### Cache Component Features

The cache component provides fast in-memory storage with sub-millisecond response times. Automatic expiration handles TTL-based cache invalidation without manual intervention. Distributed caching enables sharing the cache across multiple bot instances for consistent performance. Persistence options offer optional disk persistence for cache durability across restarts.

## Example Usage

### Basic Caching

```basic
' Caching happens automatically when enabled
USE KB "policies"

' First user asks: "What's the vacation policy?"
' System generates response and caches it

' Second user asks: "Tell me about vacation rules"
' System finds semantic match (>0.95 similarity) and returns cached response
```

### Tool Response Caching

```basic
' Tool responses can also be cached
USE TOOL "weather-api"

' First request: "What's the weather in NYC?"
' Makes API call, caches response for 1 hour

' Second request within TTL: "NYC weather?"
' Returns cached response without API call
```

## Cache Management

The cache operates automatically based on your configuration settings. Cache entries are managed through TTL expiration and memory policies without requiring manual intervention.

## Best Practices

### When to Enable Caching

Enable caching for FAQ bots with repetitive questions, knowledge base queries where the same information is requested frequently, API-heavy integrations where external calls are expensive, and high-traffic bots where response latency impacts user experience.

Disable caching for real-time data queries where freshness is critical, personalized responses that should vary per user, time-sensitive information that changes frequently, and development or testing environments where you need to see actual responses.

### Tuning Cache Parameters

TTL settings should match your data freshness requirements. Use short TTL values around 300 seconds for news, weather, and stock prices. Medium TTL values around 3600 seconds work well for general knowledge and FAQs. Long TTL values around 86400 seconds suit static documentation and policies.

Similarity threshold affects matching precision. High thresholds of 0.95 or above provide strict matching with fewer false positives. Medium thresholds between 0.85 and 0.95 balance coverage and accuracy. Low thresholds below 0.85 enable broad matching but risk returning incorrect responses.

### Memory Management

The cache component automatically manages memory through LRU (Least Recently Used) eviction policies that remove the oldest accessed entries first. Configurable memory limits prevent unbounded growth. Automatic key expiration cleans up entries that have exceeded their TTL.

## Performance Impact

Typical performance improvements with caching enabled:

| Metric | Without Cache | With Cache | Improvement |
|--------|--------------|------------|-------------|
| Response Time | 2-5s | 50-200ms | 10-100x faster |
| API Calls | Every request | First request only | 90%+ reduction |
| Token Usage | Full context | Cached response | 95%+ reduction |
| Cost | $0.02/request | $0.001/request | 95% cost saving |

## Troubleshooting

### Cache Not Working

If caching isn't working as expected, verify that the cache service is running and accessible. Confirm caching is enabled in your config with `llm-cache,true`. Check that the TTL hasn't expired for entries you expect to be cached. Review the similarity threshold to ensure it isn't set too high for your use case.

### Clear Cache

Cache is managed automatically through TTL expiration and eviction policies. To clear the cache manually, restart the cache component or use the admin API endpoint `/api/admin/cache/clear`.

## Summary

The semantic caching system in BotServer provides intelligent response caching that reduces response latency by 10-100x and cuts API costs by 90% or more. Response quality is maintained through semantic matching that understands query intent rather than requiring exact matches. The system scales automatically with the cache component to handle increasing load. Configure caching based on your bot's needs, monitor performance metrics, and tune parameters for optimal results.