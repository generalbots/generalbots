# LLM Configuration

Configuration for Language Model integration in BotServer, supporting both local GGUF models and external API services.

## Local Model Configuration

BotServer is designed to work with local GGUF models by default:

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,../../../../data/llm/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

### Model Path

The `llm-model` parameter accepts:
- **Relative paths**: `../../../../data/llm/model.gguf`
- **Absolute paths**: `/opt/models/model.gguf`
- **Model names**: When using external APIs like `gpt-4`

### Supported Model Formats

- **GGUF**: Quantized models for CPU/GPU inference
- **Q3_K_M, Q4_K_M, Q5_K_M**: Different quantization levels
- **F16, F32**: Full precision models

## LLM Server Configuration

### Running Embedded Server

BotServer can run its own LLM server:

```csv
llm-server,true
llm-server-path,botserver-stack/bin/llm/build/bin
llm-server-host,0.0.0.0
llm-server-port,8081
```

### Server Performance Parameters

```csv
llm-server-gpu-layers,0
llm-server-ctx-size,4096
llm-server-n-predict,1024
llm-server-parallel,6
llm-server-cont-batching,true
```

| Parameter | Description | Impact |
|-----------|-------------|---------|
| `llm-server-gpu-layers` | Layers to offload to GPU | 0 = CPU only, higher = more GPU |
| `llm-server-ctx-size` | Context window size | More context = more memory |
| `llm-server-n-predict` | Max tokens to generate | Limits response length |
| `llm-server-parallel` | Concurrent requests | Higher = more throughput |
| `llm-server-cont-batching` | Continuous batching | Improves multi-user performance |

### Memory Management

```csv
llm-server-mlock,false
llm-server-no-mmap,false
```

- **mlock**: Locks model in RAM (prevents swapping)
- **no-mmap**: Disables memory mapping (uses more RAM)

## Cache Configuration

### Basic Cache Settings

```csv
llm-cache,false
llm-cache-ttl,3600
```

Caching reduces repeated LLM calls for identical inputs.

### Semantic Cache

```csv
llm-cache-semantic,true
llm-cache-threshold,0.95
```

Semantic caching matches similar (not just identical) queries:
- **threshold**: 0.95 = 95% similarity required
- Lower threshold = more cache hits but less accuracy

## External API Configuration

### Groq and OpenAI-Compatible APIs

For cloud inference, Groq offers the fastest performance:

```csv
llm-key,gsk-your-groq-api-key
llm-url,https://api.groq.com/openai/v1
llm-model,mixtral-8x7b-32768
```

### Local API Servers

```csv
llm-key,none
llm-url,http://localhost:8081
llm-model,local-model-name
```

## Configuration Examples

### Minimal Local Setup
```csv
name,value
llm-url,http://localhost:8081
llm-model,../../../../data/llm/model.gguf
```

### High-Performance Local
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
```csv
name,value
llm-server-ctx-size,2048
llm-server-n-predict,512
llm-server-parallel,2
llm-cache,false
llm-server-mlock,false
```

### External API
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
- Decrease `llm-server-ctx-size`
- Decrease `llm-server-n-predict`
- Enable `llm-cache`
- Enable `llm-cache-semantic`

### For Quality
- Increase `llm-server-ctx-size`
- Increase `llm-server-n-predict`
- Use higher quantization (Q5_K_M or F16)
- Disable semantic cache or increase threshold

### For Multiple Users
- Enable `llm-server-cont-batching`
- Increase `llm-server-parallel`
- Enable caching
- Consider GPU offloading

## Model Selection Guidelines

### Small Models (1-3B parameters)
- Fast responses
- Low memory usage
- Good for simple tasks
- Example: `DeepSeek-R1-Distill-Qwen-1.5B`

### Medium Models (7-13B parameters)
- Balanced performance
- Moderate memory usage
- Good general purpose
- Example: `Llama-2-7B`, `Mistral-7B`

### Large Models (30B+ parameters)
- Best quality
- High memory requirements
- Complex reasoning
- Example: `Llama-2-70B`, `Mixtral-8x7B`

## Troubleshooting

### Model Won't Load
- Check file path exists
- Verify sufficient RAM
- Ensure compatible GGUF version

### Slow Responses
- Reduce context size
- Enable caching
- Use GPU offloading
- Choose smaller model

### Out of Memory
- Reduce `llm-server-ctx-size`
- Reduce `llm-server-parallel`
- Use more quantized model (Q3 instead of Q5)
- Disable `llm-server-mlock`

### Connection Refused
- Verify `llm-server` is true
- Check port not in use
- Ensure firewall allows connection

## Best Practices

1. **Start Small**: Begin with small models and scale up
2. **Use Caching**: Enable for production deployments
3. **Monitor Memory**: Watch RAM usage during operation
4. **Test Thoroughly**: Verify responses before production
5. **Document Models**: Keep notes on model performance
6. **Version Control**: Track config.csv changes