# AI and LLM

BotServer integrates with Large Language Models (LLMs) to provide intelligent conversational capabilities and natural language understanding.

## Overview

The LLM integration in BotServer enables:
- Natural language conversations
- Context-aware responses
- Tool discovery and invocation
- Document understanding
- Text generation and summarization

## LLM Providers

### OpenAI

Primary LLM provider with support for:
- GPT-3.5 Turbo
- GPT-4
- GPT-4 Turbo
- Custom fine-tuned models

Configuration:
```
OPENAI_API_KEY=your-api-key
LLM_MODEL=gpt-4
```

### Local Models

Support for self-hosted models:
- Llama.cpp compatible servers
- Custom inference endpoints
- Privacy-preserving deployments

Configuration:
```
LLM_PROVIDER=local
LLM_ENDPOINT=http://localhost:8081
```

## The LLM Keyword

### Basic Usage

### Basic Generation (Background Processing)
```basic
# For background/scheduled tasks only - not for interactive conversations
let summary = LLM "Explain quantum computing in simple terms"
SET_BOT_MEMORY "quantum_explanation", summary  # Store for all users
```

### With Context

### Document Summarization (Background Processing)
```basic
# Scheduled task to generate summaries for all users
let document = GET "knowledge/policy.pdf"
let summary = LLM "Summarize this document: " + document
SET_BOT_MEMORY "policy_summary", summary  # Available to all users
```

### Question Answering

### Context-Aware Conversations (Interactive)
```basic
# For interactive conversations - use SET CONTEXT, not LLM
TALK "What's your question?"
let question = HEAR
let context = GET_BOT_MEMORY("knowledge")
SET CONTEXT "background", context
TALK "Based on our knowledge base, here's what I can tell you..."
# System AI automatically uses the context when responding
```

## LLM Provider Implementation

Located in `src/llm/`:
- `mod.rs` - Provider trait and factory
- `openai.rs` - OpenAI implementation
- `local.rs` - Local model support

### Provider Trait

All LLM providers implement:
- `generate()` - Text generation
- `generate_stream()` - Streaming responses
- `get_embedding()` - Vector embeddings
- `count_tokens()` - Token counting

## Context Management

### Context Window

Managing limited context size:
- Automatic truncation
- Context compaction
- Relevance filtering
- History summarization

### Context Sources

1. **Conversation History** - Recent messages
2. **Knowledge Base** - Relevant documents
3. **Bot Memory** - Persistent context
4. **Tool Definitions** - Available functions
5. **User Profile** - Personalization

## Prompt Engineering

### System Prompts

Configured in bot memory:
```basic
let system_prompt = GET_BOT_MEMORY("system_prompt")
SET_CONTEXT "system" AS system_prompt
```

### Dynamic Prompts

Building prompts programmatically:
```basic
# For interactive conversations - use SET CONTEXT
SET CONTEXT "user_name", user_name
SET CONTEXT "current_date", NOW()
# System AI automatically incorporates this context
```

## Streaming Responses

### WebSocket Streaming

Real-time token streaming:
1. LLM generates tokens
2. Tokens sent via WebSocket
3. UI updates progressively
4. Complete response assembled

### Stream Control

- Start/stop generation
- Cancel long responses
- Timeout protection
- Error recovery

## Embeddings

### Vector Generation

Creating embeddings for semantic search:
```rust
let embedding = llm_provider.get_embedding(text).await?;
```

### Embedding Models

- OpenAI: text-embedding-ada-002
- Local: Sentence transformers
- Custom: Configurable models

## Token Management

### Token Counting

Estimating usage before calls:
```rust
let token_count = llm_provider.count_tokens(prompt)?;
```

### Token Limits

- Model-specific limits
- Context window constraints
- Rate limiting
- Cost management

## Error Handling

### Common Errors

- API key invalid
- Rate limit exceeded
- Context too long
- Model unavailable
- Network timeout

### Fallback Strategies

1. Retry with backoff
2. Switch to backup model
3. Reduce context size
4. Cache responses
5. Return graceful error

## Performance Optimization

### Caching

- Response caching
- Embedding caching
- Token count caching
- Semantic cache

### Batching

- Group similar requests
- Batch embeddings
- Parallel processing
- Queue management

## Cost Management

### Usage Tracking

- Tokens per request
- Cost per conversation
- Daily/monthly limits
- Per-user quotas

### Optimization Strategies

1. Use smaller models when possible
2. Cache frequent queries
3. Compress context
4. Limit conversation length
5. Implement quotas

## Security Considerations

### API Key Management

- Never hardcode keys
- Use environment variables
- Rotate keys regularly
- Monitor usage

### Content Filtering

- Input validation
- Output sanitization
- PII detection
- Inappropriate content blocking

## Monitoring

### Metrics

- Response time
- Token usage
- Error rate
- Cache hit rate
- Model performance

### Logging

- Request/response pairs
- Error details
- Performance metrics
- Usage statistics

## Best Practices

1. **Choose Appropriate Models**: Balance cost and capability
2. **Optimize Prompts**: Clear, concise instructions
3. **Manage Context**: Keep relevant information only
4. **Handle Errors**: Graceful degradation
5. **Monitor Usage**: Track costs and performance
6. **Cache Wisely**: Reduce redundant calls
7. **Stream When Possible**: Better user experience

## Summary

The LLM integration is central to BotServer's intelligence, providing natural language understanding and generation capabilities. Through careful prompt engineering, context management, and provider abstraction, bots can deliver sophisticated conversational experiences while managing costs and performance.