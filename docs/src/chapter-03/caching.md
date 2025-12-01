# Caching

BotServer includes automatic caching to improve response times and reduce redundant processing, including semantic caching for LLM responses using an in-memory cache component.

## Features

- **Exact Match Caching**: Cache responses for identical prompts
- **Semantic Similarity Matching**: Find and reuse responses for semantically similar prompts
- **Configurable TTL**: Control how long cached responses remain valid
- **Per-Bot Configuration**: Enable/disable caching on a per-bot basis
- **Embedding-Based Similarity**: Use local embedding models for semantic matching
- **Statistics & Monitoring**: Track cache hits, misses, and performance metrics

## How Caching Works

Caching in BotServer is controlled by configuration parameters in `config.csv`. The system automatically caches LLM responses and manages conversation history.

When enabled, the semantic cache:
1. User asks a question
2. System checks if a semantically similar question was asked before
3. If similarity > threshold (0.95), returns cached response
4. Otherwise, generates new response and caches it

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
prompt-history,2    # Number of previous messages to include in context
prompt-compact,4    # Compact conversation after N exchanges
```

### What These Settings Do

- **prompt-history**: Keeps the last 2 exchanges in the conversation context
- **prompt-compact**: After 4 exchanges, older messages are summarized or removed to save tokens

## Cache Storage

### Architecture

The caching system uses a multi-level approach for optimal performance.

### Cache Key Structure

The cache uses a multi-level key structure:
- **Exact match**: Hash of the exact prompt
- **Semantic match**: Embedding vector stored with semantic index

### Cache Component Features

The cache component provides:
- **Fast in-memory storage**: Sub-millisecond response times
- **Automatic expiration**: TTL-based cache invalidation
- **Distributed caching**: Share cache across multiple bot instances
- **Persistence options**: Optional disk persistence for cache durability

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

The cache operates automatically based on your configuration settings. Cache entries are managed through TTL expiration and memory policies.

## Best Practices

### When to Enable Caching

Enable caching for:
- ✅ FAQ bots with repetitive questions
- ✅ Knowledge base queries
- ✅ API-heavy integrations
- ✅ High-traffic bots

Disable caching for:
- ❌ Real-time data queries
- ❌ Personalized responses
- ❌ Time-sensitive information
- ❌ Development/testing

### Tuning Cache Parameters

**TTL Settings**:
- Short (300s): News, weather, stock prices
- Medium (3600s): General knowledge, FAQs
- Long (86400s): Static documentation, policies

**Similarity Threshold**:
- High (0.95+): Strict matching, fewer false positives
- Medium (0.85-0.95): Balance between coverage and accuracy
- Low (<0.85): Broad matching, risk of incorrect responses

### Memory Management

The cache component automatically manages memory through:
- **Eviction policies**: LRU (Least Recently Used) by default
- **Max memory limits**: Configurable memory settings
- **Key expiration**: Automatic cleanup of expired entries

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

Check:
1. Cache service is running
2. Cache enabled in config: `llm-cache,true`
3. TTL not expired
4. Similarity threshold not too high

### Clear Cache

Cache is managed automatically. To clear cache manually, restart the cache component or use the admin API endpoint `/api/admin/cache/clear`.

## Summary

The semantic caching system in BotServer provides intelligent response caching that:
- Reduces response latency by 10-100x
- Cuts API costs by 90%+
- Maintains response quality through semantic matching
- Scales automatically with the cache component

Configure caching based on your bot's needs, monitor performance metrics, and tune parameters for optimal results.