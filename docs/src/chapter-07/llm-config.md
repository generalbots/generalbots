# LLM Configuration

Configure Large Language Model providers, models, and parameters for optimal bot performance.

## Overview

BotServer supports multiple LLM providers with flexible configuration options. Each bot can use different models and settings based on requirements for performance, cost, and capabilities.

## Supported Providers

### OpenAI

The most popular provider with GPT models:

```csv
llmProvider,openai
llmModel,gpt-4
llmApiKey,sk-...
llmEndpoint,https://api.openai.com/v1
```

Available models:
- `gpt-4` - Most capable, higher cost
- `gpt-4-turbo` - Faster, more affordable GPT-4
- `gpt-3.5-turbo` - Fast and cost-effective
- `gpt-3.5-turbo-16k` - Extended context window

### Anthropic (Claude)

Advanced models with strong reasoning:

```csv
llmProvider,anthropic
llmModel,claude-3-opus-20240229
llmApiKey,sk-ant-...
llmEndpoint,https://api.anthropic.com
```

Available models:
- `claude-3-opus` - Most capable Claude model
- `claude-3-sonnet` - Balanced performance
- `claude-3-haiku` - Fast and efficient
- `claude-2.1` - Previous generation

### Google (Gemini)

Google's multimodal AI models:

```csv
llmProvider,google
llmModel,gemini-pro
llmApiKey,AIza...
llmProject,my-project-id
```

Available models:
- `gemini-pro` - Text generation
- `gemini-pro-vision` - Multimodal (text + images)
- `gemini-ultra` - Most advanced (limited access)

### Local Models

Self-hosted open-source models:

```csv
llmProvider,local
llmModel,llama-3-70b
llmEndpoint,http://localhost:8000
```

Supported local models:
- Llama 3 (8B, 70B)
- Mistral (7B, 8x7B)
- Falcon (7B, 40B)
- Vicuna
- Alpaca

## Model Selection Guide

### By Use Case

| Use Case | Recommended Model | Reasoning |
|----------|------------------|-----------|
| Customer Support | gpt-3.5-turbo | Fast, cost-effective, good quality |
| Technical Documentation | gpt-4 | Accurate, detailed responses |
| Creative Writing | claude-3-opus | Strong creative capabilities |
| Code Generation | gpt-4 | Best code understanding |
| Multilingual | gemini-pro | Excellent language support |
| Privacy-Sensitive | Local Llama 3 | Data stays on-premise |
| High Volume | gpt-3.5-turbo | Lowest cost per token |

### By Requirements

**Need accuracy?** → GPT-4 or Claude-3-opus
**Need speed?** → GPT-3.5-turbo or Claude-3-haiku
**Need low cost?** → Local models or GPT-3.5
**Need privacy?** → Local models only
**Need vision?** → Gemini-pro-vision or GPT-4V

## Temperature Settings

Temperature controls response creativity and randomness:

```csv
llmTemperature,0.7
```

### Temperature Guide

| Value | Use Case | Behavior |
|-------|----------|----------|
| 0.0 | Factual Q&A | Deterministic, same output |
| 0.2 | Technical docs | Very focused, minimal variation |
| 0.5 | Customer service | Balanced consistency |
| 0.7 | General chat | Natural variation (default) |
| 0.9 | Creative tasks | High creativity |
| 1.0 | Brainstorming | Maximum randomness |

### Examples by Domain

**Legal/Medical**: 0.1-0.3 (high accuracy required)
**Education**: 0.3-0.5 (clear, consistent)
**Sales/Marketing**: 0.6-0.8 (engaging, varied)
**Entertainment**: 0.8-1.0 (creative, surprising)

## Token Management

### Context Window Sizes

| Model | Max Tokens | Recommended |
|-------|------------|-------------|
| GPT-3.5 | 4,096 | 3,000 |
| GPT-3.5-16k | 16,384 | 12,000 |
| GPT-4 | 8,192 | 6,000 |
| GPT-4-32k | 32,768 | 25,000 |
| Claude-3 | 200,000 | 150,000 |
| Gemini-Pro | 32,768 | 25,000 |

### Token Allocation

```csv
llmMaxTokens,2000
llmContextTokens,2000
llmResponseTokens,1000
```

Best practices:
- Reserve 25% for system prompt
- Allocate 50% for context/history
- Keep 25% for response

### Cost Optimization

Monitor and control token usage:

```csv
llmTokenLimit,1000000
llmCostLimit,100
llmRequestLimit,10000
```

Tips for reducing costs:
1. Use smaller models when possible
2. Implement response caching
3. Compress conversation history
4. Set appropriate max tokens
5. Use temperature 0 for consistent caching

## Advanced Parameters

### Sampling Parameters

Fine-tune response generation:

```csv
llmTopP,0.9
llmTopK,50
llmFrequencyPenalty,0.5
llmPresencePenalty,0.5
```

| Parameter | Effect | Range | Default |
|-----------|--------|-------|---------|
| `topP` | Nucleus sampling threshold | 0-1 | 1.0 |
| `topK` | Top tokens to consider | 1-100 | None |
| `frequencyPenalty` | Reduce repetition | -2 to 2 | 0 |
| `presencePenalty` | Encourage new topics | -2 to 2 | 0 |

### Stop Sequences

Control when generation stops:

```csv
llmStopSequences,"Human:,Assistant:,###"
```

Common stop sequences:
- Conversation markers: `"Human:", "User:", "AI:"`
- Section dividers: `"###", "---", "==="`
- Custom tokens: `"[END]", "</response>"`

## System Prompts

Define bot personality and behavior:

```csv
llmSystemPrompt,"You are a helpful customer service agent for ACME Corp. Be friendly, professional, and concise. Always verify customer information before making changes."
```

### System Prompt Templates

**Professional Assistant**:
```
You are a professional assistant. Provide accurate, helpful responses.
Be concise but thorough. Maintain a formal, respectful tone.
```

**Technical Support**:
```
You are a technical support specialist. Help users troubleshoot issues.
Ask clarifying questions. Provide step-by-step solutions.
```

**Sales Representative**:
```
You are a friendly sales representative. Help customers find products.
Be enthusiastic but not pushy. Focus on customer needs.
```

## Model Fallbacks

Configure backup models for reliability:

```csv
llmProvider,openai
llmModel,gpt-4
llmFallbackModel,gpt-3.5-turbo
llmFallbackOnError,true
llmFallbackOnRateLimit,true
```

Fallback strategies:
1. **Error fallback**: Use backup on API errors
2. **Rate limit fallback**: Switch when rate limited
3. **Cost fallback**: Use cheaper model at limits
4. **Load balancing**: Distribute across models

## Response Caching

Improve performance and reduce costs:

```csv
llmCacheEnabled,true
llmCacheTTL,3600
llmCacheKey,message_hash
llmCacheOnlyDeterministic,true
```

Cache strategies:
- Cache identical queries (temperature=0)
- Cache by semantic similarity
- Cache common questions/FAQs
- Invalidate cache on knowledge updates

## Streaming Configuration

Enable real-time response streaming:

```csv
llmStreamEnabled,true
llmStreamChunkSize,10
llmStreamTimeout,30000
```

Benefits:
- Faster perceived response time
- Better user experience
- Allows interruption
- Progressive rendering

## Error Handling

Configure error behavior:

```csv
llmRetryAttempts,3
llmRetryDelay,1000
llmTimeoutSeconds,30
llmErrorMessage,"I'm having trouble processing that. Please try again."
```

Error types and handling:
- **Timeout**: Retry with shorter prompt
- **Rate limit**: Wait and retry or fallback
- **Invalid request**: Log and return error
- **Service unavailable**: Use fallback model

## Monitoring and Logging

Track LLM performance:

```csv
llmLogRequests,true
llmLogResponses,false
llmMetricsEnabled,true
llmLatencyAlert,5000
```

Key metrics to monitor:
- Response latency
- Token usage
- Cost per conversation
- Error rates
- Cache hit rates

## Multi-Model Strategies

Use different models for different tasks:

```csv
llmModelRouting,true
llmSimpleQueries,gpt-3.5-turbo
llmComplexQueries,gpt-4
llmCreativeTasks,claude-3-opus
```

Routing logic:
1. Analyze query complexity
2. Check user tier/permissions
3. Consider cost budget
4. Route to appropriate model

## Best Practices

### Development vs Production

**Development**:
```csv
llmModel,gpt-3.5-turbo
llmLogResponses,true
llmCacheEnabled,false
llmMockMode,true
```

**Production**:
```csv
llmModel,gpt-4
llmLogResponses,false
llmCacheEnabled,true
llmMockMode,false
```

### Cost Management

1. Start with smaller models
2. Use caching aggressively
3. Implement token limits
4. Monitor usage daily
5. Set up cost alerts
6. Use fallback models
7. Compress contexts

### Performance Optimization

1. Enable streaming for long responses
2. Use appropriate temperature
3. Set reasonable max tokens
4. Implement response caching
5. Use connection pooling
6. Consider edge deployment

### Security

1. Never commit API keys
2. Use environment variables
3. Rotate keys regularly
4. Implement rate limiting
5. Validate all inputs
6. Sanitize outputs
7. Log security events

## Troubleshooting

### Common Issues

**Slow responses**: Lower max tokens, enable streaming
**High costs**: Use cheaper models, enable caching
**Inconsistent output**: Lower temperature, add examples
**Rate limits**: Implement backoff, use multiple keys
**Timeouts**: Increase timeout, reduce prompt size

### Debugging

Enable debug mode:
```csv
llmDebugMode,true
llmVerboseLogging,true
llmTraceRequests,true
```

## Migration Guide

### Switching Providers

1. Update `llmProvider` and `llmModel`
2. Set appropriate API key
3. Adjust token limits for new model
4. Test with sample queries
5. Update system prompts if needed
6. Monitor for behavior changes

### Upgrading Models

1. Test new model in development
2. Compare outputs with current model
3. Adjust temperature/parameters
4. Update fallback configuration
5. Gradual rollout to production
6. Monitor metrics closely