# Episodic Memory

Episodic memory automatically manages conversation history to stay within LLM token limits while preserving important information through intelligent summarization. This system handles context compaction transparently, ensuring conversations remain coherent without manual intervention.

## Overview

Large Language Models have fixed context windows (e.g., 8K, 32K, 128K tokens). Long conversations can exceed these limits, causing truncation or errors. Episodic memory solves this by:

1. Monitoring conversation length
2. Summarizing older exchanges when thresholds are reached
3. Keeping recent messages in full detail
4. Storing summaries as "episodic memory" for continuity

## Configuration

Episodic memory is controlled by parameters in `config.csv`:

```csv
name,value
episodic-memory-enabled,true
episodic-memory-threshold,4
episodic-memory-history,2
episodic-memory-model,fast
episodic-memory-max-episodes,100
episodic-memory-retention-days,365
episodic-memory-auto-summarize,true
```

## Parameter Reference

| Parameter | Default | Type | Description |
|-----------|---------|------|-------------|
| `episodic-memory-enabled` | `true` | Boolean | Enable/disable episodic memory system |
| `episodic-memory-threshold` | `4` | Integer | Number of exchanges before compaction triggers |
| `episodic-memory-history` | `2` | Integer | Recent exchanges to keep in full detail |
| `episodic-memory-model` | `fast` | String | Model for generating summaries (`fast`, `quality`, or model name) |
| `episodic-memory-max-episodes` | `100` | Integer | Maximum episode summaries per user |
| `episodic-memory-retention-days` | `365` | Integer | Days to retain episode summaries |
| `episodic-memory-auto-summarize` | `true` | Boolean | Automatically summarize when threshold reached |

## How It Works

### Context Compaction Process

1. **Monitor**: System tracks message count since last summary
2. **Trigger**: When count reaches `episodic-memory-threshold`, compaction starts
3. **Summarize**: Older messages are summarized using the configured LLM
4. **Preserve**: Last `episodic-memory-history` exchanges remain in full
5. **Store**: Summary saved with role "episodic" for future context

### Example Timeline

With defaults (`episodic-memory-threshold=4`, `episodic-memory-history=2`):

| Exchange | Action | Context State |
|----------|--------|---------------|
| 1-2 | Normal | Messages 1-2 in full |
| 3-4 | Normal | Messages 1-4 in full |
| 5 | **Compaction** | Summary of 1-2 + Messages 3-5 in full |
| 6-7 | Normal | Summary + Messages 3-7 in full |
| 8 | **Compaction** | Summary of 1-5 + Messages 6-8 in full |

### Automatic Behavior

The system automatically:

1. Tracks conversation length
2. Triggers compaction when exchanges exceed `episodic-memory-threshold`
3. Summarizes older messages using the configured LLM
4. Keeps only the last `episodic-memory-history` exchanges in full
5. Stores the summary as an "episodic memory" for future context

The scheduler runs every 60 seconds, checking all active sessions and processing those that exceed the threshold.

## Tuning Guidelines

### High-Context Conversations

For complex discussions requiring more history:

```csv
name,value
episodic-memory-history,5
episodic-memory-threshold,10
```

### Token-Constrained Environments

For smaller context windows or cost optimization:

```csv
name,value
episodic-memory-history,1
episodic-memory-threshold,2
```

### Disable Compaction

Set threshold to 0 to disable automatic compaction:

```csv
name,value
episodic-memory-threshold,0
```

### Extended Retention

For long-term memory across sessions:

```csv
name,value
episodic-memory-max-episodes,500
episodic-memory-retention-days,730
```

## Use Case Recommendations

| Use Case | History | Threshold | Rationale |
|----------|---------|-----------|-----------|
| FAQ Bot | 1 | 2 | Questions are independent |
| Customer Support | 2 | 4 | Some context needed |
| Technical Discussion | 4 | 8 | Complex topics require history |
| Therapy/Coaching | 5 | 10 | Continuity is critical |
| Long-term Assistant | 3 | 6 | Balance memory and context |

## Token Savings

Compaction significantly reduces token usage:

| Scenario | Without Compaction | With Compaction | Savings |
|----------|-------------------|-----------------|---------|
| 10 exchanges | ~5,000 tokens | ~2,000 tokens | 60% |
| 20 exchanges | ~10,000 tokens | ~3,000 tokens | 70% |
| 50 exchanges | ~25,000 tokens | ~5,000 tokens | 80% |

*Actual savings depend on message length and summary quality.*

## Summary Storage

Summaries are stored with special role identifiers:

- Role `episodic` or `compact` marks summary messages
- Summaries include key points from compacted exchanges
- Original messages are not deleted, just excluded from active context
- Episodes are searchable for context retrieval across sessions

## Benefits

- **Automatic management** - No manual intervention needed
- **Token efficiency** - Stay within model context limits
- **Context preservation** - Important information kept via summaries
- **Relevant context** - Recent exchanges kept in full detail
- **Cost savings** - Fewer tokens = lower API costs
- **Long-term memory** - Episode storage enables recall across sessions

## Interaction with Caching

Episodic memory works alongside semantic caching:

- **Caching**: Reuses responses for similar queries (see [Semantic Caching](./caching.md))
- **Episodic Memory**: Manages conversation length over time

Both features reduce costs and improve performance independently.

## Best Practices

1. **Start with defaults** - Work well for most use cases
2. **Monitor token usage** - Adjust if hitting context limits
3. **Consider conversation type** - Support vs complex discussion
4. **Test different values** - Find optimal balance for your users
5. **Set retention appropriately** - Balance memory vs privacy requirements

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Context too long | Threshold too high | Lower `episodic-memory-threshold` |
| Lost context | History too low | Increase `episodic-memory-history` |
| Summaries missing info | Model limitations | Use `quality` instead of `fast` |
| No compaction occurring | Threshold is 0 or disabled | Set positive threshold, enable feature |
| Old episodes not deleted | Retention too long | Lower `episodic-memory-retention-days` |

## See Also

- [Semantic Caching](./caching.md) - Response caching system
- [Configuration Parameters](../10-configuration-deployment/parameters.md) - Full parameter reference
- [LLM Configuration](../10-configuration-deployment/llm-config.md) - Model settings