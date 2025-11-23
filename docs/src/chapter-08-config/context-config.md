# Context Configuration

Configure how BotServer manages conversation context, memory, and state across user interactions.

## Overview

Context configuration determines how the bot maintains conversation state, manages memory, processes historical interactions, and provides relevant responses based on accumulated knowledge. Proper context configuration is crucial for coherent, contextually-aware conversations.

## Context Providers

### Database Context Provider

Default provider using PostgreSQL for context storage:

```csv
contextProvider,database
contextConnectionPool,10
contextQueryTimeout,5000
```

Features:
- Persistent context storage
- Fast retrieval with indexes
- Supports complex queries
- Scales with database
- ACID compliance

### Memory Context Provider

In-memory context for maximum performance:

```csv
contextProvider,memory
contextMaxMemoryMB,512
contextEvictionPolicy,lru
```

Features:
- Ultra-fast access
- No persistence overhead
- Ideal for stateless bots
- Limited by RAM
- Lost on restart

### Hybrid Context Provider

Combines database and memory for optimal performance:

```csv
contextProvider,hybrid
contextCacheTTL,3600
contextCacheSize,1000
```

Features:
- Memory cache with DB backing
- Best of both approaches
- Automatic synchronization
- Configurable cache policies
- Failover support

## Context Window Management

### Window Size Configuration

Control how much history is maintained:

```csv
contextWindowSize,10
contextMaxTokens,4000
contextPruningStrategy,sliding
```

| Parameter | Description | Default | Range |
|-----------|-------------|---------|-------|
| `contextWindowSize` | Number of messages to keep | 10 | 1-100 |
| `contextMaxTokens` | Maximum context tokens | 4000 | 500-32000 |
| `contextPruningStrategy` | How to prune context | sliding | sliding, summary, selective |

### Pruning Strategies

**Sliding Window**: Keep last N messages
```csv
contextPruningStrategy,sliding
contextWindowSize,10
```

**Summarization**: Compress older messages
```csv
contextPruningStrategy,summary
contextSummaryThreshold,20
contextSummaryRatio,0.3
```

**Selective**: Keep important messages only
```csv
contextPruningStrategy,selective
contextImportanceThreshold,0.7
contextKeepSystemMessages,true
```

## Memory Types

### Short-Term Memory

Temporary context for current session:

```csv
shortTermMemoryEnabled,true
shortTermMemoryDuration,3600
shortTermMemorySize,100
```

Stores:
- Current conversation
- Temporary variables
- Session state
- Recent interactions

### Long-Term Memory

Persistent memory across sessions:

```csv
longTermMemoryEnabled,true
longTermMemoryRetention,365
longTermMemoryCompression,true
```

Stores:
- User preferences
- Historical interactions
- Learned patterns
- Important facts

### Working Memory

Active context for processing:

```csv
workingMemorySize,5
workingMemoryRefresh,message
workingMemoryScope,conversation
```

Contains:
- Current topic
- Active variables
- Immediate context
- Processing state

## Context Enhancement

### Semantic Context

Add semantically relevant information:

```csv
semanticContextEnabled,true
semanticSearchTopK,5
semanticThreshold,0.75
```

Features:
- Retrieves related past conversations
- Adds relevant knowledge base entries
- Includes similar resolved issues
- Enhances response relevance

### Entity Tracking

Track entities throughout conversation:

```csv
entityTrackingEnabled,true
entityTypes,"person,product,location,date"
entityResolution,true
```

Tracks:
- Named entities
- References (pronouns)
- Relationships
- Entity state changes

### Topic Detection

Identify and track conversation topics:

```csv
topicDetectionEnabled,true
topicChangeThreshold,0.6
topicHistorySize,5
```

Benefits:
- Context switching awareness
- Relevant response generation
- Topic-based memory retrieval
- Conversation flow management

## State Management

### Session State

Maintain state within sessions:

```csv
sessionStateEnabled,true
sessionTimeout,1800
sessionStorage,cache
```

Stores:
- User authentication
- Current dialog position
- Variable values
- Temporary flags

### Global State

Shared state across sessions:

```csv
globalStateEnabled,true
globalStateNamespace,bot
globalStateSyncInterval,60
```

Contains:
- System-wide settings
- Shared resources
- Global counters
- Feature flags

### User State

Per-user persistent state:

```csv
userStateEnabled,true
userStateFields,"preferences,history,profile"
userStateEncrypted,true
```

Includes:
- User preferences
- Interaction history
- Personalization data
- Custom attributes

## Context Injection

### System Context

Always-present system information:

```csv
systemContextEnabled,true
systemContextTemplate,"You are {bot_name}. Current time: {time}. User: {user_name}"
```

### Dynamic Context

Conditionally injected context:

```csv
dynamicContextEnabled,true
dynamicContextRules,"business_hours,user_tier,location"
```

Examples:
- Business hours notice
- User tier benefits
- Location-based info
- Seasonal messages

### Tool Context

Context for active tools:

```csv
toolContextEnabled,true
toolContextAutoLoad,true
toolContextFormat,structured
```

## Performance Optimization

### Context Caching

Cache frequently accessed context:

```csv
contextCacheEnabled,true
contextCacheProvider,cache
contextCacheTTL,300
contextCacheMaxSize,1000
```

Benefits:
- Reduced database queries
- Faster response times
- Lower latency
- Resource efficiency

### Lazy Loading

Load context on demand:

```csv
lazyLoadingEnabled,true
lazyLoadThreshold,0.8
preloadCommonContext,true
```

### Context Compression

Compress stored context:

```csv
contextCompressionEnabled,true
contextCompressionLevel,6
contextCompressionThreshold,1000
```

## Multi-Turn Conversations

### Conversation Threading

Track conversation threads:

```csv
threadingEnabled,true
threadTimeout,300
threadMaxDepth,10
```

### Context Carryover

Maintain context across interactions:

```csv
contextCarryoverEnabled,true
carryoverFields,"topic,intent,entities"
carryoverDuration,600
```

### Dialog State

Track dialog flow state:

```csv
dialogStateEnabled,true
dialogStateProvider,memory
dialogStateTimeout,1800
```

## Context Rules

### Inclusion Rules

Define what to include in context:

```csv
contextIncludeUserMessages,true
contextIncludeSystemMessages,true
contextIncludeErrors,false
contextIncludeDebug,false
```

### Exclusion Rules

Define what to exclude:

```csv
contextExcludePII,true
contextExcludePasswords,true
contextExcludeSensitive,true
```

### Transformation Rules

Transform context before use:

```csv
contextMaskPII,true
contextNormalizeText,true
contextTranslate,false
```

## Monitoring

### Context Metrics

Track context performance:

```csv
contextMetricsEnabled,true
contextMetricsInterval,60
contextMetricsRetention,7
```

Key metrics:
- Context size
- Retrieval time
- Cache hit rate
- Memory usage
- Pruning frequency

### Debug Logging

Debug context operations:

```csv
contextDebugEnabled,false
contextDebugLevel,info
contextDebugOutput,file
```

## Best Practices

1. **Right-size context window**: Balance completeness vs performance
2. **Use appropriate provider**: Database for persistence, memory for speed
3. **Enable caching**: Significantly improves performance
4. **Prune strategically**: Keep relevant, remove redundant
5. **Monitor metrics**: Track and optimize based on usage
6. **Secure sensitive data**: Encrypt and mask PII
7. **Test context switching**: Ensure smooth topic transitions
8. **Document configuration**: Explain choices and trade-offs

## Common Configurations

### High-Performance Chat

```csv
contextProvider,memory
contextWindowSize,5
contextCacheEnabled,true
lazyLoadingEnabled,true
```

### Long Conversations

```csv
contextProvider,database
contextWindowSize,50
contextPruningStrategy,summary
contextCompressionEnabled,true
```

### Privacy-Focused

```csv
contextProvider,memory
contextExcludePII,true
contextMaskSensitive,true
contextEncryption,true
```

### Multi-User Support

```csv
contextProvider,hybrid
userStateEnabled,true
sessionIsolation,strict
contextNamespacing,true
```

## Troubleshooting

### Context Loss
- Check session timeout settings
- Verify database connectivity
- Review memory limits
- Check pruning settings

### Slow Context Retrieval
- Enable caching
- Optimize queries
- Reduce window size
- Use lazy loading

### Memory Issues
- Reduce context window
- Enable compression
- Increase pruning frequency
- Switch to database provider

### Inconsistent Responses
- Check context carryover
- Verify entity tracking
- Review pruning strategy
- Test context injection