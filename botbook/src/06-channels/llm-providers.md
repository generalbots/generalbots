# LLM Providers

General Bots supports multiple Large Language Model (LLM) providers, both cloud-based services and local deployments. This guide helps you choose the right provider for your use case.

## Overview

LLMs are the intelligence behind General Bots' conversational capabilities. You can configure:

- **Cloud Providers** — External APIs (OpenAI, Anthropic, Google, etc.)
- **Local Models** — Self-hosted models via llama.cpp
- **Hybrid** — Use local for simple tasks, cloud for complex reasoning

## Cloud Providers

### OpenAI (GPT Series)

The most widely known LLM provider, offering the GPT-5 flagship model.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| GPT-5 | 1M | All-in-one advanced reasoning | Medium |
| GPT-oss 120B | 128K | Open-weight, agent workflows | Medium |
| GPT-oss 20B | 128K | Cost-effective open-weight | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,openai
llm-model,gpt-5
```

**Strengths:**
- Most advanced all-in-one model
- Excellent general knowledge
- Strong code generation
- Good instruction following

**Considerations:**
- API costs can add up
- Data sent to external servers
- Rate limits apply

### Anthropic (Claude Series)

Known for safety, helpfulness, and extended thinking capabilities.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Claude Opus 4.5 | 200K | Most capable, complex reasoning | Slow |
| Claude Sonnet 4.5 | 200K | Best balance of capability/speed | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,anthropic
llm-model,claude-sonnet-4.5
```

**Strengths:**
- Extended thinking mode for multi-step tasks
- Excellent at following complex instructions
- Strong coding abilities
- Better at refusing harmful requests

**Considerations:**
- Premium pricing
- Newer provider, smaller ecosystem

### Google (Gemini & Vertex AI)

Google's multimodal AI models with strong reasoning capabilities. General Bots natively supports both the public AI Studio API and Enterprise Vertex AI.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Gemini 1.5 Pro | 2M | Complex reasoning, benchmarks | Medium |
| Gemini 1.5 Flash | 1M | Fast multimodal tasks | Fast |

**Configuration for AI Studio (Public API):**

```csv
name,value
llm-provider,google
llm-model,gemini-1.5-pro
llm-url,https://generativelanguage.googleapis.com
llm-key,AIza...
```

**Configuration for Vertex AI (Enterprise):**

```csv
name,value
llm-provider,vertex
llm-model,gemini-1.5-pro
llm-url,https://us-central1-aiplatform.googleapis.com
llm-key,~/.vertex.json
```
*Note: The bots will handle the Google OAuth2 JWT authentication internally if you provide the path or the raw JSON to a Service Account.*

**Strengths:**
- Largest context window (2M tokens)
- Native multimodal (text, image, video, audio)
- Vertex AI support enables enterprise VPC/IAM integration

**Considerations:**
- Different endpoints for public vs enterprise deployments

### xAI (Grok Series)

Integration with real-time data from X platform.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Grok 4 | 128K | Real-time research, analysis | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,xai
llm-model,grok-4
```

**Strengths:**
- Real-time data access from X
- Strong research and analysis
- Good for trend analysis

**Considerations:**
- Newer provider
- X platform integration focus

### Groq

Ultra-fast inference using custom LPU hardware. Offers open-source models at high speed.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Llama 4 Scout | 10M | Long context, multimodal | Very Fast |
| Llama 4 Maverick | 1M | Complex tasks | Very Fast |
| Qwen3 | 128K | Efficient MoE architecture | Extremely Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,groq
llm-model,llama-4-scout
```

**Strengths:**
- Fastest inference speeds (500+ tokens/sec)
- Competitive pricing
- Open-source models
- Great for real-time applications

**Considerations:**
- Rate limits on free tier
- Models may be less capable than GPT-5/Claude

### Mistral AI

European AI company offering efficient, open-weight models.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Mixtral-8x22B | 64K | Multi-language, coding | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,mistral
llm-model,mixtral-8x22b
```

**Strengths:**
- European data sovereignty (GDPR)
- Excellent code generation
- Open-weight models available
- Competitive pricing
- Proficient in multiple languages

**Considerations:**
- Smaller context than competitors
- Less brand recognition

### DeepSeek

Known for efficient, capable models with exceptional reasoning.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| DeepSeek-V3.1 | 128K | General purpose, optimized cost | Fast |
| DeepSeek-R3 | 128K | Reasoning, math, science | Medium |

**Configuration (config.csv):**

```csv
name,value
llm-provider,deepseek
llm-model,deepseek-r3
llm-server-url,https://api.deepseek.com
```

**Strengths:**
- Extremely cost-effective
- Strong reasoning (R1 model)
- Rivals proprietary leaders in performance
- Open-weight versions available (MIT/Apache 2.0)

**Considerations:**
- Data processed in China
- Newer provider

### Amazon Bedrock

AWS managed service for foundation models, supporting Claude, Llama, Titan, and others.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Claude 3.5 Sonnet | 200K | High capability tasks | Fast |
| Llama 3.1 70B | 128K | Open-weight performance | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,bedrock
llm-model,anthropic.claude-3-5-sonnet-20240620-v1:0
llm-url,https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-5-sonnet-20240620-v1:0/invoke
llm-key,YOUR_BEDROCK_API_KEY
```

**Strengths:**
- Native AWS integration
- Enterprise-grade security
- Multiple model families in one API

### Azure OpenAI

Enterprise-grade deployment of OpenAI models hosted on Microsoft Azure.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| GPT-4o | 128K | Advanced multimodal | Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,azureclaude
llm-model,gpt-4o
llm-url,https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT/chat/completions?api-version=2024-02-15-preview
llm-key,YOUR_AZURE_API_KEY
```

**Strengths:**
- High enterprise compliance (HIPAA, SOC2)
- Azure VNet integration
- Guaranteed provisioned throughput available

### Cerebras

Ultra-fast inference powered by Wafer-Scale Engine hardware, specifically tuned for open-source models like Llama.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| Llama 3.1 70B | 8K | High-speed general tasks | Extremely Fast |

**Configuration (config.csv):**

```csv
name,value
llm-provider,cerebras
llm-model,llama3.1-8b
llm-url,https://api.cerebras.ai/v1/chat/completions
llm-key,YOUR_CEREBRAS_API_KEY
```

**Strengths:**
- Highest tokens-per-second available
- Excellent for real-time agent loops

### Zhipu AI (GLM)

High-capability bilingual models (English/Chinese) directly competing with state-of-the-art global models.

| Model | Context | Best For | Speed |
|-------|---------|----------|-------|
| GLM-4 | 128K | General purpose | Medium |
| GLM-4-Long | 1M | Long document analysis | Medium |

**Configuration (config.csv):**

```csv
name,value
llm-provider,glm
llm-model,glm-4
llm-url,https://open.bigmodel.cn/api/paas/v4/chat/completions
llm-key,YOUR_ZHIPU_API_KEY
```

**Strengths:**
- Excellent bilingual performance
- Large context windows (up to 1M)

## Local Models

Run models on your own hardware for privacy, cost control, and offline operation.

### Setting Up Local LLM

General Bots uses **llama.cpp** server for local inference:

```csv
name,value
llm-provider,local
llm-server-url,http://localhost:8081
llm-model,DeepSeek-R3-Distill-Qwen-1.5B
```

### Recommended Local Models

#### For High-End GPU (24GB+ VRAM)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| Llama 4 Scout 17B Q8 | 18GB | 24GB | Excellent |
| Qwen3 72B Q4 | 42GB | 48GB+ | Excellent |
| DeepSeek-R3 32B Q4 | 20GB | 24GB | Very Good |

#### For Mid-Range GPU (12-16GB VRAM)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| Qwen3 14B Q8 | 15GB | 16GB | Very Good |
| GPT-oss 20B Q4 | 12GB | 16GB | Very Good |
| DeepSeek-R3-Distill 14B Q4 | 8GB | 12GB | Good |
| Gemma 3 27B Q4 | 16GB | 16GB | Good |

#### For Small GPU or CPU (8GB VRAM or less)

| Model | Size | VRAM | Quality |
|-------|------|------|---------|
| DeepSeek-R3-Distill 1.5B Q4 | 1GB | 4GB | Basic |
| Gemma 2 9B Q4 | 5GB | 8GB | Acceptable |
| Gemma 3 27B Q2 | 10GB | 8GB | Acceptable |

### Model Download URLs

Add models to `installer.rs` data_download_list:

```rust
// Qwen3 14B - Recommended for mid-range GPU
"https://huggingface.co/Qwen/Qwen3-14B-GGUF/resolve/main/qwen3-14b-q4_k_m.gguf"

// DeepSeek R1 Distill - For CPU or minimal GPU
"https://huggingface.co/unsloth/DeepSeek-R3-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R3-Distill-Qwen-1.5B-Q4_K_M.gguf"

// GPT-oss 20B - Good balance for agents
"https://huggingface.co/openai/gpt-oss-20b-GGUF/resolve/main/gpt-oss-20b-q4_k_m.gguf"

// Gemma 3 27B - For quality local inference
"https://huggingface.co/google/gemma-3-27b-it-GGUF/resolve/main/gemma-3-27b-it-q4_k_m.gguf"
```

### Embedding Models

For vector search, you need an embedding model:

```csv
name,value
embedding-provider,local
embedding-server-url,http://localhost:8082
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
name,value
llm-provider,anthropic
llm-model,claude-sonnet-4.5
llm-fast-provider,groq
llm-fast-model,llama-3.3-70b
llm-fallback-provider,local
llm-fallback-model,DeepSeek-R3-Distill-Qwen-1.5B
embedding-provider,local
embedding-model,bge-small-en-v1.5
```

## Model Selection Guide

### By Use Case

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Customer support | Claude Sonnet 4.5 | Best at following guidelines |
| Code generation | DeepSeek-R3, Claude Sonnet 4.5 | Specialized for code |
| Document analysis | Gemini Pro | 2M context window |
| Real-time chat | Groq Llama 3.3 | Fastest responses |
| Privacy-sensitive | Local DeepSeek-R3 | No external data transfer |
| Cost-sensitive | DeepSeek, Local models | Lowest cost per token |
| Complex reasoning | Claude Opus, Gemini Pro | Best reasoning ability |
| Real-time research | Grok | Live data access |
| Long context | Gemini Pro, Claude | Largest context windows |

### By Budget

| Budget | Recommended Setup |
|--------|-------------------|
| Free | Local models only |
| Low ($10-50/mo) | Groq + Local fallback |
| Medium ($50-200/mo) | DeepSeek-V3.1 + Claude Sonnet 4.5 |
| High ($200+/mo) | GPT-5 + Claude Opus 4.5 |
| Enterprise | Private deployment + premium APIs |

## Configuration Reference

### config.csv Parameters

All LLM configuration belongs in `config.csv`, not environment variables:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `llm-provider` | Provider name | `openai`, `anthropic`, `local` |
| `llm-model` | Model identifier | `gpt-5` |
| `llm-server-url` | API endpoint (local only) | `http://localhost:8081` |
| `llm-server-ctx-size` | Context window size | `128000` |
| `llm-temperature` | Response randomness (0-2) | `0.7` |
| `llm-max-tokens` | Maximum response length | `4096` |
| `llm-cache-enabled` | Enable semantic caching | `true` |
| `llm-cache-ttl` | Cache time-to-live (seconds) | `3600` |

### API Keys

API keys are stored in **Vault**, not in config files or environment variables:

```bash
# Store API key in Vault
vault kv put gbo/llm/openai api_key="sk-..."
vault kv put gbo/llm/anthropic api_key="sk-ant-..."
vault kv put gbo/llm/google api_key="AIza..."
```

Reference in config.csv:

```csv
name,value
llm-provider,openai
llm-model,gpt-5
llm-api-key,vault:gbo/llm/openai/api_key
```

## Security Considerations

### Cloud Providers

- API keys stored in Vault, never in config files
- Consider data residency requirements (EU: Mistral)
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
name,value
llm-cache-enabled,true
llm-cache-ttl,3600
llm-cache-similarity-threshold,0.92
```

### Batching

For bulk operations, use batch APIs when available:

```csv
name,value
llm-batch-enabled,true
llm-batch-size,10
```

### Context Management

Optimize context window usage with episodic memory:

```csv
name,value
episodic-memory-enabled,true
episodic-memory-threshold,4
episodic-memory-history,2
episodic-memory-auto-summarize,true
```

See [Episodic Memory](../03-knowledge-ai/episodic-memory.md) for details.

## Troubleshooting

### Common Issues

**API Key Invalid**
- Verify key is stored correctly in Vault
- Check if key has required permissions
- Ensure billing is active on provider account

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
name,value
llm-log-requests,true
llm-log-responses,false
llm-log-timing,true
```

## 2025 Model Comparison

| Model | Creator | Type | Strengths |
|-------|---------|------|-----------|
| GPT-5 | OpenAI | Proprietary | Most advanced all-in-one |
| Claude Opus/Sonnet 4.5 | Anthropic | Proprietary | Extended thinking, complex reasoning |
| Gemini 1.5/3 Pro | Google | Proprietary | Benchmarks, reasoning, 2M context |
| Grok 4 | xAI | Proprietary | Real-time X data |
| Claude / Llama | Amazon Bedrock | Managed API | Enterprise AWS integration |
| GPT-4o / GPT-5 | Azure OpenAI | Managed API | Enterprise compliance, Azure VNet |
| Llama / Open Models | Cerebras | Hardware Cloud | Extreme inference speed |
| GLM-4 | Zhipu AI | Proprietary | English/Chinese bilingual, up to 1M context |
| DeepSeek-V3.1/R1 | DeepSeek | Open (MIT/Apache) | Cost-optimized, reasoning |
| Llama 4 | Meta | Open-weight | 10M context, multimodal |
| Qwen3 | Alibaba | Open (Apache) | Efficient MoE |
| Mixtral-8x22B | Mistral | Open (Apache) | Multi-language, coding |
| GPT-oss | OpenAI | Open (Apache) | Agent workflows |
| Gemma 2/3 | Google | Open-weight | Lightweight, efficient |

## Next Steps

- [config.csv Reference](../10-configuration-deployment/config-csv.md) — Complete configuration guide
- [Secrets Management](../10-configuration-deployment/secrets-management.md) — Vault integration
- [Semantic Caching](../03-knowledge-ai/caching.md) — Cache configuration
- [NVIDIA GPU Setup](../appendix-external-services/nvidia.md) — GPU configuration for local models