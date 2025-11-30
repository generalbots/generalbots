# LLM Providers

General Bots supports multiple Large Language Model (LLM) providers, both cloud-based services and local deployments. This guide helps you choose the right provider for your use case.

## Overview

LLMs are the intelligence behind General Bots' conversational capabilities. You can configure:

- **Cloud Providers** - External APIs (OpenAI, Anthropic, Groq, etc.)
- **Local Models** - Self-hosted models via llama.cpp
- **Hybrid** - Use local for simple tasks, cloud for complex reasoning

## Cloud Providers

### OpenAI (GPT Series)

The most widely known LLM provider, offering GPT-4 and GPT-4o models.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| GPT-4o | 128K | General purpose, vision | Fast |
| GPT-4o-mini | 128K | Cost-effective tasks | Very Fast |
| GPT-4 Turbo | 128K | Complex reasoning | Medium |
| o1-preview | 128K | Advanced reasoning, math | Slow |
| o1-mini | 128K | Code, logic tasks | Medium |

**Configuration:**

```csv
llm-provider,openai
llm-api-key,sk-xxxxxxxxxxxxxxxxxxxxxxxx
llm-model,gpt-4o
```

**Strengths:**
- Excellent general knowledge
- Strong code generation
- Good instruction following
- Vision capabilities (GPT-4o)

**Considerations:**
- API costs can add up
- Data sent to external servers
- Rate limits apply

### Anthropic (Claude Series)

Known for safety, helpfulness, and large context windows.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Claude 3.5 Sonnet | 200K | Best balance of capability/speed | Fast |
| Claude 3.5 Haiku | 200K | Quick, everyday tasks | Very Fast |
| Claude 3 Opus | 200K | Most capable, complex tasks | Slow |

**Configuration:**

```csv
llm-provider,anthropic
llm-api-key,sk-ant-xxxxxxxxxxxxxxxx
llm-model,claude-3-5-sonnet-20241022
```

**Strengths:**
- Largest context window (200K tokens)
- Excellent at following complex instructions
- Strong coding abilities
- Better at refusing harmful requests

**Considerations:**
- Premium pricing
- No vision in all models
- Newer provider, smaller ecosystem

### Groq

Ultra-fast inference using custom LPU hardware. Offers open-source models at high speed.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Llama 3.3 70B | 128K | Complex reasoning | Very Fast |
| Llama 3.1 8B | 128K | Quick responses | Extremely Fast |
| Mixtral 8x7B | 32K | Balanced performance | Very Fast |
| Gemma 2 9B | 8K | Lightweight tasks | Extremely Fast |

**Configuration:**

```csv
llm-provider,groq
llm-api-key,gsk_xxxxxxxxxxxxxxxx
llm-model,llama-3.3-70b-versatile
```

**Strengths:**
- Fastest inference speeds (500+ tokens/sec)
- Competitive pricing
- Open-source models
- Great for real-time applications

**Considerations:**
- Limited model selection
- Rate limits on free tier
- Models may be less capable than GPT-4/Claude

### Google (Gemini Series)

Google's multimodal AI models with strong reasoning capabilities.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Gemini 1.5 Pro | 2M | Extremely long documents | Medium |
| Gemini 1.5 Flash | 1M | Fast multimodal | Fast |
| Gemini 2.0 Flash | 1M | Latest capabilities | Fast |

**Configuration:**

```csv
llm-provider,google
llm-api-key,AIzaxxxxxxxxxxxxxxxx
llm-model,gemini-1.5-pro
```

**Strengths:**
- Largest context window (2M tokens)
- Native multimodal (text, image, video, audio)
- Strong at structured data
- Good coding abilities

**Considerations:**
- Newer ecosystem
- Some features region-limited
- API changes more frequently

### Mistral AI

European AI company offering efficient, open-weight models.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Mistral Large | 128K | Complex tasks | Medium |
| Mistral Medium | 32K | Balanced performance | Fast |
| Mistral Small | 32K | Cost-effective | Very Fast |
| Codestral | 32K | Code generation | Fast |

**Configuration:**

```csv
llm-provider,mistral
llm-api-key,xxxxxxxxxxxxxxxx
llm-model,mistral-large-latest
```

**Strengths:**
- European data sovereignty (GDPR)
- Excellent code generation (Codestral)
- Open-weight models available
- Competitive pricing

**Considerations:**
- Smaller context than competitors
- Less brand recognition
- Fewer fine-tuning options

### DeepSeek

Chinese AI company known for efficient, capable models.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| DeepSeek-V3 | 128K | General purpose | Fast |
| DeepSeek-R1 | 128K | Reasoning, math | Medium |
| DeepSeek-Coder | 128K | Programming | Fast |

**Configuration:**

```csv
llm-provider,deepseek
llm-api-key,sk-xxxxxxxxxxxxxxxx
llm-model,deepseek-chat
llm-server-url,https://api.deepseek.com
```

**Strengths:**
- Extremely cost-effective
- Strong reasoning (R1 model)
- Excellent code generation
- Open-weight versions available

**Considerations:**
- Data processed in China
- Newer provider
- May have content restrictions

## Local Models

Run models on your own hardware for privacy, cost control, and offline operation.

### Setting Up Local LLM

General Bots uses **llama.cpp** server for local inference:

```csv
llm-provider,local
llm-server-url,https://localhost:8081
llm-model,DeepSeek-R1-Distill-Qwen-1.5B
```

### Recommended Local Models

#### For High-End GPU (24GB+ VRAM)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| GPT-OSS 120B Q4 | 70GB | 48GB+ | Excellent |
| Llama 3.1 70B Q4 | 40GB | 48GB+ | Excellent |
| DeepSeek-R1 32B Q4 | 20GB | 24GB | Very Good |
| Qwen 2.5 72B Q4 | 42GB | 48GB+ | Excellent |

#### For Mid-Range GPU (12-16GB VRAM)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| GPT-OSS 20B F16 | 40GB | 16GB | Very Good |
| Llama 3.1 8B Q8 | 9GB | 12GB | Good |
| DeepSeek-R1-Distill 14B Q4 | 8GB | 12GB | Good |
| Mistral Nemo 12B Q4 | 7GB | 10GB | Good |

#### For Small GPU or CPU (8GB VRAM or less)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| DeepSeek-R1-Distill 1.5B Q4 | 1GB | 4GB | Basic |
| Phi-3 Mini 3.8B Q4 | 2.5GB | 6GB | Acceptable |
| Gemma 2 2B Q8 | 3GB | 6GB | Acceptable |
| Qwen 2.5 3B Q4 | 2GB | 4GB | Basic |

### Model Download URLs

Add models to `installer.rs` data_download_list:

```rust
// GPT-OSS 20B - Recommended for small GPU
"https://huggingface.co/unsloth/gpt-oss-20b-GGUF/resolve/main/gpt-oss-20b-F16.gguf"

// DeepSeek R1 Distill - For CPU or minimal GPU
"https://huggingface.co/unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf"

// Llama 3.1 8B - Good balance
"https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf"
```

### Embedding Models

For vector search, you need an embedding model:

```csv
embedding-provider,local
embedding-server-url,https://localhost:8082
embedding-model,bge-small-en-v1.5
```

Recommended embedding models:

| Model | Dimensions | Size | Quality |
|-------|------------|------|---------|
| bge-small-en-v1.5 | 384 | 130MB | Good |
| bge-base-en-v1.5 | 768 | 440MB | Better |
| bge-large-en-v1.5 | 1024 | 1.3GB | Best |
| nomic-embed-text | 768 | 550MB | Good |

## Hybrid Configuration

Use different models for different tasks:

```csv
# Primary model for complex conversations
llm-provider,anthropic
llm-model,claude-3-5-sonnet-20241022

# Fast model for simple tasks
llm-fast-provider,groq
llm-fast-model,llama-3.1-8b-instant

# Local fallback for offline operation
llm-fallback-provider,local
llm-fallback-model,DeepSeek-R1-Distill-Qwen-1.5B

# Embeddings always local
embedding-provider,local
embedding-model,bge-small-en-v1.5
```

## Model Selection Guide

### By Use Case

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Customer support | Claude 3.5 Sonnet | Best at following guidelines |
| Code generation | DeepSeek-Coder, GPT-4o | Specialized for code |
| Document analysis | Gemini 1.5 Pro | 2M context window |
| Real-time chat | Groq Llama 3.1 8B | Fastest responses |
| Privacy-sensitive | Local DeepSeek-R1 | No external data transfer |
| Cost-sensitive | DeepSeek-V3, Local | Lowest cost per token |
| Complex reasoning | Claude 3 Opus, o1 | Best reasoning ability |

### By Budget

| Budget | Recommended Setup |
|--------|-------------------|
| Free | Local models only |
| Low ($10-50/mo) | Groq + Local fallback |
| Medium ($50-200/mo) | GPT-4o-mini + Claude Haiku |
| High ($200+/mo) | GPT-4o + Claude Sonnet |
| Enterprise | Private deployment + premium APIs |

## Configuration Reference

### Environment Variables

```bash
# Primary LLM
LLM_PROVIDER=openai
LLM_API_KEY=sk-xxx
LLM_MODEL=gpt-4o
LLM_SERVER_URL=https://api.openai.com

# Local LLM Server
LLM_LOCAL_URL=https://localhost:8081
LLM_LOCAL_MODEL=DeepSeek-R1-Distill-Qwen-1.5B

# Embedding
EMBEDDING_PROVIDER=local
EMBEDDING_URL=https://localhost:8082
EMBEDDING_MODEL=bge-small-en-v1.5
```

### config.csv Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `llm-provider` | Provider name | `openai`, `anthropic`, `local` |
| `llm-api-key` | API key for cloud providers | `sk-xxx` |
| `llm-model` | Model identifier | `gpt-4o` |
| `llm-server-url` | API endpoint | `https://api.openai.com` |
| `llm-server-ctx-size` | Context window size | `128000` |
| `llm-temperature` | Response randomness (0-2) | `0.7` |
| `llm-max-tokens` | Maximum response length | `4096` |
| `llm-cache-enabled` | Enable semantic caching | `true` |
| `llm-cache-ttl` | Cache time-to-live (seconds) | `3600` |

## Security Considerations

### Cloud Providers

- API keys should be stored in environment variables or secrets manager
- Consider data residency requirements (EU: Mistral, US: OpenAI)
- Review provider data retention policies
- Use separate keys for production/development

### Local Models

- All data stays on your infrastructure
- No internet required after model download
- Full control over model versions
- Consider GPU security for sensitive deployments

## Performance Optimization

### Caching

Enable semantic caching to reduce API calls:

```csv
llm-cache-enabled,true
llm-cache-ttl,3600
llm-cache-similarity-threshold,0.92
```

### Batching

For bulk operations, use batch APIs when available:

```csv
llm-batch-enabled,true
llm-batch-size,10
```

### Context Management

Optimize context window usage:

```csv
llm-context-compaction,true
llm-max-history-turns,10
llm-summarize-long-contexts,true
```

## Troubleshooting

### Common Issues

**API Key Invalid**
- Verify key is correct and not expired
- Check if key has required permissions
- Ensure billing is active

**Model Not Found**
- Check model name spelling
- Verify model is available in your region
- Some models require waitlist access

**Rate Limits**
- Implement exponential backoff
- Use caching to reduce calls
- Consider upgrading API tier

**Local Model Slow**
- Check GPU memory usage
- Reduce context size
- Use quantized models (Q4 instead of F16)

### Logging

Enable LLM logging for debugging:

```csv
llm-log-requests,true
llm-log-responses,false
llm-log-timing,true
```

## Next Steps

- [LLM Configuration](../chapter-08-config/llm-config.md) - Detailed configuration guide
- [Semantic Caching](../chapter-03/caching.md) - Cache configuration
- [NVIDIA GPU Setup](../chapter-09-api/nvidia-gpu-setup.md) - GPU configuration for local models