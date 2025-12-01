# AI and LLM

BotServer integrates with Large Language Models to provide intelligent conversational capabilities and natural language understanding. This integration forms the core of what makes General Bots conversations feel natural and contextually aware.


## Overview

The LLM integration in BotServer enables sophisticated conversational experiences. Natural language conversations flow smoothly without rigid command structures. Responses are context-aware, drawing on conversation history and loaded knowledge bases. The system automatically discovers and invokes tools when they would help answer user questions. Document understanding allows bots to read and reason about uploaded files. Text generation and summarization capabilities support content creation and information distillation.


## LLM Providers

### OpenAI

OpenAI serves as the primary LLM provider with support for multiple model tiers. GPT-3.5 Turbo provides fast, cost-effective responses for straightforward conversations. GPT-4 delivers more nuanced understanding for complex queries. GPT-4 Turbo offers an optimal balance of capability and speed. Custom fine-tuned models can be used when you have specialized requirements.

Configuration requires setting your API key and selecting a model:

```
OPENAI_API_KEY=your-api-key
LLM_MODEL=gpt-4
```

### Local Models

For privacy-sensitive deployments or cost control, BotServer supports self-hosted models. Llama.cpp compatible servers provide open-source model hosting. Custom inference endpoints allow integration with any API-compatible service. Privacy-preserving deployments keep all data on-premises without external API calls.

Configuration for local models specifies the provider type and endpoint:

```
LLM_PROVIDER=local
LLM_ENDPOINT=http://localhost:8081
```


## The LLM Keyword

The LLM keyword provides direct access to language model capabilities within BASIC scripts. Usage patterns differ between background processing and interactive conversations.

### Background Processing

For scheduled tasks and background jobs that do not interact directly with users, the LLM keyword generates content that can be stored for later use.

```basic
' For background/scheduled tasks only - not for interactive conversations
summary = LLM "Explain quantum computing in simple terms"
SET BOT MEMORY "quantum_explanation", summary
```

### Document Summarization

Scheduled tasks can process documents and generate summaries available to all users.

```basic
' Scheduled task to generate summaries for all users
document = GET "knowledge/policy.pdf"
summary = LLM "Summarize this document: " + document
SET BOT MEMORY "policy_summary", summary
```

### Context-Aware Conversations

For interactive conversations, use SET CONTEXT to provide information that the System AI incorporates automatically when responding. This approach lets the AI generate natural responses rather than scripted outputs.

```basic
' For interactive conversations - use SET CONTEXT, not LLM
TALK "What's your question?"
question = HEAR
context = GET BOT MEMORY "knowledge"
SET CONTEXT "background", context
TALK "Based on our knowledge base, here's what I can tell you..."
' System AI automatically uses the context when responding
```


## LLM Provider Implementation

The provider architecture lives in the `src/llm/` directory with a modular design. The `mod.rs` file defines the provider trait and factory for instantiating providers. The `openai.rs` file implements the OpenAI provider with all API operations. The `local.rs` file provides support for local model servers.

### Provider Trait

All LLM providers implement a common trait ensuring consistent behavior. The generate method produces text completions from prompts. The generate_stream method returns tokens incrementally for real-time display. The get_embedding method creates vector representations for semantic search. The count_tokens method estimates token usage before making API calls.


## Context Management

### Context Window

Managing the limited context window requires careful attention to what information reaches the model. Automatic truncation removes older content when approaching limits. Context compaction summarizes extensive histories into shorter representations. Relevance filtering prioritizes information most likely to help with the current query. History summarization condenses long conversations into essential points.

### Context Sources

The context provided to the LLM comes from multiple sources that combine to create informed responses. Conversation history provides recent messages for continuity. Knowledge base chunks supply relevant document excerpts. Bot memory contributes persistent context that applies across conversations. Tool definitions tell the model what functions it can invoke. User profile information enables personalization based on known preferences.


## Prompt Engineering

### System Prompts

System prompts establish the bot's personality and capabilities. These are typically configured in bot memory and loaded into context at the start of conversations.

```basic
system_prompt = GET BOT MEMORY "system_prompt"
SET CONTEXT "system", system_prompt
```

### Dynamic Prompts

Building prompts programmatically allows context to reflect current conditions. Variables set in context become available to the System AI for generating responses.

```basic
' For interactive conversations - use SET CONTEXT
SET CONTEXT "user_name", user_name
SET CONTEXT "current_date", NOW()
' System AI automatically incorporates this context
```


## Streaming Responses

### WebSocket Streaming

Real-time token streaming creates a responsive user experience. As the LLM generates tokens, each token is sent immediately via WebSocket to the connected client. The UI updates progressively as tokens arrive, showing the response as it forms. The complete response is assembled on the client side once generation finishes.

### Stream Control

Several controls manage the streaming process. Users can start and stop generation as needed. Long responses can be cancelled if they are not useful. Timeout protection prevents indefinitely hanging connections. Error recovery handles network interruptions gracefully by resuming or restarting generation.


## Embeddings

### Vector Generation

Creating embeddings transforms text into vectors for semantic search. The embedding process converts natural language into high-dimensional numerical representations that capture meaning.

### Embedding Models

Different embedding models serve different needs. OpenAI's text-embedding-ada-002 provides high-quality embeddings through their API. Local deployments can use sentence transformers for on-premises embedding generation. Custom models can be configured when you have specialized embedding requirements.


## Token Management

### Token Counting

Estimating token usage before making API calls helps with cost control and context management. Token counting uses the same tokenizer as the target model to produce accurate estimates.

### Token Limits

Several factors constrain token usage. Each model has specific limits on total tokens per request. Context window constraints determine how much history and knowledge base content fits. Rate limiting prevents exceeding API quotas. Cost management tracks token usage against budgets.


## Error Handling

### Common Errors

Several error conditions occur frequently when working with LLMs. Invalid API keys prevent authentication with the provider. Rate limit exceeded errors indicate too many requests in a time window. Context too long errors mean the prompt exceeds the model's maximum. Model unavailable errors happen during provider outages. Network timeouts occur when connections take too long.

### Fallback Strategies

Robust error handling employs multiple fallback strategies. Retry with exponential backoff handles transient failures. Switching to a backup model maintains service when the primary is unavailable. Reducing context size can resolve context length errors. Caching responses reduces API calls and provides fallback content. Returning graceful errors keeps users informed when recovery is not possible.


## Performance Optimization

### Caching

Response caching dramatically improves performance for repeated queries. Semantic caching identifies similar questions and returns cached responses without API calls. Cache invalidation strategies ensure responses remain fresh as knowledge bases update. Cache warming pre-generates responses for common questions during off-peak times.

### Batching

Batching multiple requests improves throughput and reduces per-request overhead. Embedding generation particularly benefits from batching when processing many documents. Rate limit management becomes simpler with controlled batch submission.

### Connection Pooling

Connection pooling to LLM providers reduces latency from connection establishment. Keep-alive connections persist across requests. Pool sizing balances resource usage against responsiveness.


## Model Selection

Choosing the right model involves balancing several factors. Capability requirements determine the minimum model sophistication needed. Response latency requirements favor faster models for interactive use. Cost constraints may push toward more economical model tiers. Privacy requirements might mandate local models over cloud APIs.

### Model Comparison

GPT-3.5 Turbo offers the fastest responses at the lowest cost, suitable for straightforward questions. GPT-4 provides superior reasoning for complex queries at higher cost and latency. Local models like Llama variants offer privacy and cost predictability with varying capability levels. Specialized models may excel at particular domains like code or medical content.


## Integration with Tools

LLMs in BotServer work closely with the tool system. The model receives tool definitions describing available functions. When a user request would benefit from tool use, the model generates a tool call. BotServer executes the tool and returns results to the model. The model incorporates tool results into its final response.

This integration enables bots to take actions beyond conversation, such as querying databases, sending emails, or calling external APIs, all orchestrated naturally through conversation.


## Best Practices

Effective LLM usage follows several guidelines. Keep system prompts focused and specific rather than trying to cover every scenario. Use SET CONTEXT for interactive conversations rather than generating responses directly with LLM calls. Load relevant knowledge bases before conversations to improve response quality. Monitor token usage to manage costs. Test responses across different query types to ensure consistent quality.


## Debugging and Monitoring

Debugging LLM interactions requires visibility into prompts and responses. Enable verbose logging during development to see full API exchanges. Monitor response quality metrics over time. Track token usage and costs per conversation. Review conversation logs to identify improvement opportunities.


## See Also

The Context Configuration chapter explains context window management in detail. The LLM Configuration chapter covers all configuration options. The Tool Definition chapter describes creating tools the LLM can invoke. The Knowledge Base chapter explains how documents integrate with LLM context.