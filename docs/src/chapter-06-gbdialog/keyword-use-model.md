# USE MODEL

Dynamically switches the LLM model used for AI operations within a script. Enables model routing based on task requirements, cost optimization, or performance needs.

## Syntax

```basic
USE MODEL "modelname"
```

```basic
USE MODEL "auto"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `modelname` | String | Name of the model to use, or "auto" for automatic routing |

## Description

`USE MODEL` allows scripts to dynamically select which language model to use for subsequent AI operations. This is essential for:

- **Cost optimization** - Use smaller/cheaper models for simple tasks
- **Quality control** - Use powerful models for complex reasoning
- **Speed optimization** - Use fast models for real-time responses
- **Specialized tasks** - Use code-specific models for programming

When set to `"auto"`, the system automatically routes queries to the most appropriate model based on task complexity, latency requirements, and cost considerations.

## Examples

### Basic Model Selection

```basic
' Use a fast model for simple queries
USE MODEL "fast"
response = LLM "What time is it in New York?"
TALK response

' Switch to quality model for complex analysis
USE MODEL "quality"
analysis = LLM "Analyze the market trends for Q4 and provide recommendations"
TALK analysis
```

### Automatic Model Routing

```basic
' Let the system choose the best model
USE MODEL "auto"

' Simple query -> routes to fast model
greeting = LLM "Say hello"

' Complex query -> routes to quality model  
report = LLM "Generate a detailed financial analysis with projections"
```

### Code Generation

```basic
' Use code-specialized model
USE MODEL "code"

code = LLM "Write a Python function to calculate fibonacci numbers"
TALK code
```

### Cost-Aware Processing

```basic
' Process bulk items with cheap model
USE MODEL "fast"
FOR EACH item IN items
    summary = LLM "Summarize in one sentence: " + item.text
    item.summary = summary
NEXT item

' Final review with quality model
USE MODEL "quality"
review = LLM "Review these summaries for accuracy: " + summaries
```

### Model Fallback Pattern

```basic
' Try preferred model first
USE MODEL "gpt-4"
ON ERROR GOTO fallback
response = LLM prompt
GOTO done

fallback:
' Fall back to local model if API fails
USE MODEL "local"
response = LLM prompt

done:
TALK response
```

## Model Routing Strategies

The system supports several routing strategies configured in `config.csv`:

| Strategy | Description |
|----------|-------------|
| `manual` | Explicit model selection only |
| `auto` | Automatic routing based on query analysis |
| `load-balanced` | Distribute across models for throughput |
| `fallback` | Try models in order until one succeeds |

## Built-in Model Aliases

| Alias | Description | Use Case |
|-------|-------------|----------|
| `fast` | Optimized for speed | Simple queries, real-time chat |
| `quality` | Optimized for accuracy | Complex reasoning, analysis |
| `code` | Code-specialized model | Programming tasks |
| `local` | Local GGUF model | Offline/private operation |
| `auto` | System-selected | Let routing decide |

## Config.csv Options

```csv
name,value
model-routing-strategy,auto
model-default,fast
model-fast,DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
model-quality,gpt-4
model-code,codellama-7b.gguf
model-fallback-enabled,true
model-fallback-order,quality,fast,local
```

| Option | Default | Description |
|--------|---------|-------------|
| `model-routing-strategy` | `auto` | Routing strategy to use |
| `model-default` | `fast` | Default model when not specified |
| `model-fast` | (configured) | Model for fast/simple tasks |
| `model-quality` | (configured) | Model for quality/complex tasks |
| `model-code` | (configured) | Model for code generation |
| `model-fallback-enabled` | `true` | Enable automatic fallback |
| `model-fallback-order` | `quality,fast,local` | Order to try on failure |

## Auto-Routing Criteria

When `USE MODEL "auto"` is active, the system considers:

1. **Query complexity** - Token count, reasoning required
2. **Task type** - Code, analysis, chat, translation
3. **Latency requirements** - Real-time vs batch
4. **Cost budget** - Per-query and daily limits
5. **Model availability** - Health checks, rate limits

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`LLM`](./keyword-llm.md) | Query the language model |
| [`SET CONTEXT`](./keyword-set-context.md) | Add context for LLM |
| [`BEGIN SYSTEM PROMPT`](./prompt-blocks.md) | Define AI persona |

## Performance Considerations

- Model switching has minimal overhead
- Auto-routing adds ~10ms for classification
- Consider batching similar queries under one model
- Local models avoid network latency

## Best Practices

1. **Start with auto** - Let the system optimize, then tune
2. **Batch by model** - Group similar tasks to reduce switching
3. **Monitor costs** - Track per-model usage in analytics
4. **Test fallbacks** - Ensure graceful degradation
5. **Profile your queries** - Understand which need quality vs speed

## See Also

- [LLM Configuration](../chapter-08-config/llm-config.md) - Model setup
- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Model routing in multi-agent systems
- [Cost Tracking](../chapter-11-features/observability.md#cost-tracking) - Monitor model costs