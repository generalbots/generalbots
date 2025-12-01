# LLM Configuration

Configuration for Language Model integration in BotServer, supporting both local GGUF models and external API services.

## Local Model Configuration

BotServer is designed to work with local GGUF models by default. The minimal configuration requires only a few settings in your `config.csv`:

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

### Model Path

The `llm-model` parameter accepts relative paths like `../../../../data/llm/model.gguf`, absolute paths like `/opt/models/model.gguf`, or model names when using external APIs like `gpt-4`.

### Supported Model Formats

BotServer supports GGUF quantized models for CPU and GPU inference. Quantization levels include Q3_K_M, Q4_K_M, and Q5_K_M for reduced memory usage with acceptable quality trade-offs, while F16 and F32 provide full precision for maximum quality.

## LLM Server Configuration

### Running Embedded Server

BotServer can run its own LLM server for local inference:

```csv
llm-server,true
llm-server-path,botserver-stack/bin/llm/build/bin
llm-server-host,0.0.0.0
llm-server-port,8081
```

### Server Performance Parameters

Fine-tune server performance based on your hardware capabilities:

```csv
llm-server-gpu-layers,0
llm-server-ctx-size,4096
llm-server-n-predict,1024
llm-server-parallel,6
llm-server-cont-batching,true
```

| Parameter | Description | Impact |
|-----------|-------------|--------|
| `llm-server-gpu-layers` | Layers to offload to GPU | 0 = CPU only, higher = more GPU |
| `llm-server-ctx-size` | Context window size | More context = more memory |
| `llm-server-n-predict` | Max tokens to generate | Limits response length |
| `llm-server-parallel` | Concurrent requests | Higher = more throughput |
| `llm-server-cont-batching` | Continuous batching | Improves multi-user performance |

### Memory Management

Memory settings control how the model interacts with system RAM:

```csv
llm-server-mlock,false
llm-server-no-mmap,false
```

The `mlock` option locks the model in RAM to prevent swapping, which improves performance but requires sufficient memory. The `no-mmap` option disables memory mapping and loads the entire model into RAM, using more memory but potentially improving access patterns.

## Cache Configuration

### Basic Cache Settings

Caching reduces repeated LLM calls for identical inputs, significantly improving response times and reducing API costs:

```csv
llm-cache,false
llm-cache-ttl,3600
```

### Semantic Cache

Semantic caching matches similar queries, not just identical ones, providing cache hits even when users phrase questions differently:

```csv
llm-cache-semantic,true
llm-cache-threshold,0.95
```

The threshold parameter controls how similar queries must be to trigger a cache hit. A value of 0.95 requires 95% similarity. Lower thresholds produce more cache hits but may return less accurate cached responses.

## External API Configuration

### Groq and OpenAI-Compatible APIs

For cloud inference, Groq offers the fastest performance among major providers:

```csv
llm-key,gsk-your-groq-api-key
llm-url,https://api.groq.com/openai/v1
llm-model,mixtral-8x7b-32768
```

### Local API Servers

When running your own inference server or using another local service:

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,local-model-name
```

## Configuration Examples

### Minimal Local Setup

The simplest configuration for getting started with local models:

```csv
name,value
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### High-Performance Local

Optimized for maximum throughput on capable hardware:

```csv
name,value
llm-server,true
llm-server-gpu-layers,32
llm-server-ctx-size,8192
llm-server-parallel,8
llm-server-cont-batching,true
llm-cache,true
llm-cache-semantic,true
```

### Low-Resource Setup

Configured for systems with limited RAM or CPU:

```csv
name,value
llm-server-ctx-size,2048
llm-server-n-predict,512
llm-server-parallel,2
llm-cache,false
llm-server-mlock,false
```

### External API

Using a cloud provider for inference:

```csv
name,value
llm-key,sk-...
llm-url,https://api.anthropic.com
llm-model,claude-3
llm-cache,true
llm-cache-ttl,7200
```

## Performance Tuning

### For Responsiveness

When response speed is the priority, decrease `llm-server-ctx-size` and `llm-server-n-predict` to reduce processing time. Enable both `llm-cache` and `llm-cache-semantic` to serve repeated queries instantly.

### For Quality

When output quality matters most, increase `llm-server-ctx-size` and `llm-server-n-predict` to give the model more context and generation headroom. Use higher quantization models like Q5_K_M or F16 for better accuracy. Either disable semantic cache entirely or raise the threshold to avoid returning imprecise cached responses.

### For Multiple Users

Supporting concurrent users requires enabling `llm-server-cont-batching` and increasing `llm-server-parallel` to handle multiple requests simultaneously. Enable caching to reduce redundant inference calls. If available, GPU offloading significantly improves throughput under load.

## Model Selection Guidelines

### Small Models (1-3B parameters)

Small models like DeepSeek-R1-Distill-Qwen-1.5B deliver fast responses with low memory usage. They work well for simple tasks, quick interactions, and resource-constrained environments.

### Medium Models (7-13B parameters)

Medium-sized models such as Llama-2-7B and Mistral-7B provide balanced performance suitable for general-purpose applications. They require moderate memory but handle a wide range of tasks competently.

### Large Models (30B+ parameters)

Large models like Llama-2-70B and Mixtral-8x7B offer the best quality for complex reasoning tasks. They require substantial memory and compute resources but excel at nuanced understanding and generation.

## Troubleshooting

### Model Won't Load

If the model fails to load, first verify the file path exists and is accessible. Check that your system has sufficient RAM for the model size. Ensure the GGUF file version is compatible with your llama.cpp build.

### Slow Responses

Slow generation typically indicates resource constraints. Reduce context size, enable caching to avoid redundant inference, use GPU offloading if hardware permits, or switch to a smaller quantized model.

### Out of Memory

Memory errors require reducing resource consumption. Lower `llm-server-ctx-size` and `llm-server-parallel` values. Switch to more aggressively quantized models (Q3 instead of Q5). Disable `llm-server-mlock` to allow the OS to manage memory more flexibly.

### Connection Refused

Connection errors usually indicate server configuration issues. Verify `llm-server` is set to true if expecting BotServer to run the server. Check that the configured port is not already in use by another process. Ensure firewall rules allow connections on the specified port.

## Best Practices

Start with smaller models and scale up only as needed, since larger models consume more resources without always providing proportionally better results. Enable caching for any production deployment to reduce costs and improve response times. Monitor RAM usage during operation to catch memory pressure before it causes problems. Test model responses thoroughly before deploying to production to ensure quality meets requirements. Document which models you're using and their performance characteristics. Track changes to your `config.csv` in version control to maintain a history of configuration adjustments.