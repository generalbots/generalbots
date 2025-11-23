# LLM Configuration

Configure Large Language Model providers for bot conversations. BotServer prioritizes local models for privacy and cost-effectiveness.

## Overview

BotServer supports both local models (GGUF format) and cloud APIs. The default configuration uses local models running on your hardware.

## Local Models (Default)

### Configuration

From `default.gbai/default.gbot/config.csv`:

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

### LLM Server Settings

```csv
llm-server,false
llm-server-path,botserver-stack/bin/llm/build/bin
llm-server-host,0.0.0.0
llm-server-port,8081
llm-server-gpu-layers,0
llm-server-ctx-size,4096
llm-server-n-predict,1024
llm-server-parallel,6
llm-server-cont-batching,true
```

### Supported Local Models

- **DeepSeek-R1-Distill-Qwen** - Efficient reasoning model
- **Llama-3** - Open source, high quality
- **Mistral** - Fast and capable
- **Phi-3** - Microsoft's small but powerful model
- **Qwen** - Multilingual support

### GPU Acceleration

```csv
llm-server-gpu-layers,33  # Number of layers to offload to GPU
```

Set to 0 for CPU-only operation.

## Embeddings Configuration

For semantic search and vector operations:

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

## Caching Configuration

Reduce latency and costs with intelligent caching:

```csv
llm-cache,true
llm-cache-ttl,3600
llm-cache-semantic,true
llm-cache-threshold,0.95
```

## Cloud Providers (Optional)

### External API Configuration

For cloud LLM services, configure:

```csv
llm-key,your-api-key
llm-url,https://api.provider.com/v1
llm-model,model-name
```

### Provider Examples

| Provider | URL | Model Examples |
|----------|-----|----------------|
| Local | http://localhost:8081 | GGUF models |
| API Compatible | Various | Various models |
| Custom | Your endpoint | Your models |

## Performance Tuning

### Context Size

```csv
llm-server-ctx-size,4096  # Maximum context window
prompt-compact,4           # Compact after N exchanges
```

### Parallel Processing

```csv
llm-server-parallel,6           # Concurrent requests
llm-server-cont-batching,true  # Continuous batching
```

### Memory Settings

```csv
llm-server-mlock,false    # Lock model in memory
llm-server-no-mmap,false  # Disable memory mapping
```

## Model Selection Guide

| Use Case | Recommended Model | Configuration |
|----------|------------------|---------------|
| General chat | DeepSeek-R1-Distill | Default config |
| Code assistance | Qwen-Coder | Increase context |
| Multilingual | Qwen-Multilingual | Add language params |
| Fast responses | Phi-3-mini | Reduce predict tokens |
| High accuracy | Llama-3-70B | Increase GPU layers |

## Monitoring

Check LLM server status:

```bash
curl http://localhost:8081/health
```

View model information:

```bash
curl http://localhost:8081/v1/models
```

## Troubleshooting

### Model Not Loading

1. Check file path is correct
2. Verify GGUF format
3. Ensure sufficient memory
4. Check GPU drivers (if using GPU)

### Slow Responses

1. Reduce context size
2. Enable GPU acceleration
3. Use smaller model
4. Enable caching

### High Memory Usage

1. Use quantized models (Q4, Q5)
2. Reduce batch size
3. Enable memory mapping
4. Lower context size

## Best Practices

1. **Start with local models** - Better privacy and no API costs
2. **Use appropriate model size** - Balance quality vs speed
3. **Enable caching** - Reduce redundant computations
4. **Monitor resources** - Watch CPU/GPU/memory usage
5. **Test different models** - Find the best fit for your use case