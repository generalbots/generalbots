# Configuration Parameters

Complete reference of all available parameters in `config.csv`.

## Server Parameters

### Web Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `server-host` | Server bind address | `0.0.0.0` | IP address |
| `server-port` | Server listen port | `8080` | Number (1-65535) |
| `sites-root` | Generated sites directory | `/tmp` | Path |

### MCP Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `mcp-server` | Enable MCP protocol server | `false` | Boolean |

## LLM Parameters

### Core LLM Settings
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-key` | API key for LLM service | `none` | String |
| `llm-url` | LLM service endpoint | `http://localhost:8081` | URL |
| `llm-model` | Model path or identifier | Required | Path/String |
| `llm-models` | Available model aliases for routing | `default` | Semicolon-separated |

### LLM Cache
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-cache` | Enable response caching | `false` | Boolean |
| `llm-cache-ttl` | Cache time-to-live | `3600` | Seconds |
| `llm-cache-semantic` | Semantic similarity cache | `true` | Boolean |
| `llm-cache-threshold` | Similarity threshold | `0.95` | Float (0-1) |

### Embedded LLM Server
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-server` | Run embedded server | `false` | Boolean |
| `llm-server-path` | Server binary path | `botserver-stack/bin/llm/build/bin` | Path |
| `llm-server-host` | Server bind address | `0.0.0.0` | IP address |
| `llm-server-port` | Server port | `8081` | Number |
| `llm-server-gpu-layers` | GPU offload layers | `0` | Number |
| `llm-server-n-moe` | MoE experts count | `0` | Number |
| `llm-server-ctx-size` | Context size | `4096` | Tokens |
| `llm-server-n-predict` | Max predictions | `1024` | Tokens |
| `llm-server-parallel` | Parallel requests | `6` | Number |
| `llm-server-cont-batching` | Continuous batching | `true` | Boolean |
| `llm-server-mlock` | Lock in memory | `false` | Boolean |
| `llm-server-no-mmap` | Disable mmap | `false` | Boolean |
| `llm-server-reasoning-format` | Reasoning output format for llama.cpp | `none` | String |

### Hardware-Specific LLM Tuning

#### For RTX 3090 (24GB VRAM)
You can run impressive models with proper configuration:
- **DeepSeek-R1-Distill-Qwen-7B**: Set `llm-server-gpu-layers` to 35-40
- **Qwen2.5-32B-Instruct (Q4_K_M)**: Fits with `llm-server-gpu-layers` to 40-45
- **DeepSeek-V3 (with MoE)**: Set `llm-server-n-moe` to 2-4 to run even 120B models! MoE only loads active experts
- **Optimization**: Use `llm-server-ctx-size` of 8192 for longer contexts

#### For RTX 4070/4070Ti (12-16GB VRAM)  
Mid-range cards work great with quantized models:
- **Qwen2.5-14B (Q4_K_M)**: Set `llm-server-gpu-layers` to 25-30
- **DeepSeek-R1-Distill-Llama-8B**: Fully fits with layers at 32
- **Tips**: Keep `llm-server-ctx-size` at 4096 to save VRAM

#### For CPU-Only (No GPU)
Modern CPUs can still run capable models:
- **DeepSeek-R1-Distill-Qwen-1.5B**: Fast on CPU, great for testing
- **Phi-3-mini (3.8B)**: Excellent CPU performance
- **Settings**: Set `llm-server-mlock` to `true` to prevent swapping
- **Parallel**: Increase `llm-server-parallel` to CPU cores -2

#### Recommended Models (GGUF Format)
- **Best Overall**: DeepSeek-R1-Distill series (1.5B to 70B)
- **Best Small**: Qwen2.5-3B-Instruct-Q5_K_M
- **Best Medium**: DeepSeek-R1-Distill-Qwen-14B-Q4_K_M  
- **Best Large**: DeepSeek-V3, Qwen2.5-32B, or GPT2-120B-GGUF (with MoE enabled)

**Pro Tip**: The `llm-server-n-moe` parameter is magic for large models - it enables Mixture of Experts, letting you run 120B+ models on consumer hardware by only loading the experts needed for each token!

#### Local vs Cloud: A Practical Note

General Bots excels at local deployment - you own your hardware, your data stays private, and there are no recurring costs. However, if you need cloud inference:

**Groq is the speed champion** - They use custom LPU (Language Processing Unit) chips instead of GPUs, delivering 10x faster inference than traditional cloud providers. Their hardware is purpose-built for transformers, avoiding the general-purpose overhead of NVIDIA GPUs.

This isn't about market competition - it's about architecture. NVIDIA GPUs are designed for many tasks, while Groq's chips do one thing incredibly well: transformer inference. If speed matters and you're using cloud, Groq is currently the fastest option available.

For local deployment, stick with General Bots and the configurations above. For cloud bursts or when you need extreme speed, consider Groq's API with these settings:
```csv
llm-url,https://api.groq.com/openai/v1
llm-key,your-groq-api-key
llm-model,mixtral-8x7b-32768
```

## Embedding Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `embedding-url` | Embedding service endpoint | `http://localhost:8082` | URL |
| `embedding-model` | Embedding model path | Required for KB | Path |

## Prompt Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `episodic-memory-threshold` | Context compaction level | `4` | Number |
| `episodic-memory-history` | Messages in history | Not set | Number |

## Email Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `email-from` | Sender address | Required for email | Email |
| `email-server` | SMTP hostname | Required for email | Hostname |
| `email-port` | SMTP port | `587` | Number |
| `email-user` | SMTP username | Required for email | String |
| `email-pass` | SMTP password | Required for email | String |

## Theme Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `theme-color1` | Primary color | Not set | Hex color |
| `theme-color2` | Secondary color | Not set | Hex color |
| `theme-logo` | Logo URL | Not set | URL |
| `theme-title` | Bot display title | Not set | String |

## Custom Database Parameters

These parameters configure external database connections for use with BASIC keywords like MariaDB/MySQL connections.

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `custom-server` | Database server hostname | `localhost` | Hostname |
| `custom-port` | Database port | `3306` | Number |
| `custom-database` | Database name | Not set | String |
| `custom-username` | Database user | Not set | String |
| `custom-password` | Database password | Not set | String |

### Example: MariaDB Connection
```csv
custom-server,db.example.com
custom-port,3306
custom-database,myapp
custom-username,botuser
custom-password,secretpass
```

## Multi-Agent Parameters

### Agent-to-Agent (A2A) Communication
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `a2a-enabled` | Enable agent-to-agent communication | `true` | Boolean |
| `a2a-timeout` | Default delegation timeout | `30` | Seconds |
| `a2a-max-hops` | Maximum delegation chain depth | `5` | Number |
| `a2a-retry-count` | Retry attempts on failure | `3` | Number |
| `a2a-queue-size` | Maximum pending messages | `100` | Number |
| `a2a-protocol-version` | A2A protocol version | `1.0` | String |
| `a2a-persist-messages` | Persist A2A messages to database | `false` | Boolean |

### Bot Reflection
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `bot-reflection-enabled` | Enable bot self-analysis | `true` | Boolean |
| `bot-reflection-interval` | Messages between reflections | `10` | Number |
| `bot-reflection-prompt` | Custom reflection prompt | (none) | String |
| `bot-reflection-types` | Reflection types to perform | `ConversationQuality` | Semicolon-separated |
| `bot-improvement-auto-apply` | Auto-apply suggested improvements | `false` | Boolean |
| `bot-improvement-threshold` | Score threshold for improvements (0-10) | `6.0` | Float |

#### Reflection Types
Available values for `bot-reflection-types`:
- `ConversationQuality` - Analyze conversation quality and user satisfaction
- `ResponseAccuracy` - Analyze response accuracy and relevance
- `ToolUsage` - Analyze tool usage effectiveness
- `KnowledgeRetrieval` - Analyze knowledge retrieval performance
- `Performance` - Analyze overall bot performance

Example:
```csv
bot-reflection-enabled,true
bot-reflection-interval,10
bot-reflection-types,ConversationQuality;ResponseAccuracy;ToolUsage
bot-improvement-auto-apply,false
bot-improvement-threshold,7.0
```

## Memory Parameters

### User Memory (Cross-Bot)
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `user-memory-enabled` | Enable user-level memory | `true` | Boolean |
| `user-memory-max-keys` | Maximum keys per user | `1000` | Number |
| `user-memory-default-ttl` | Default time-to-live (0=no expiry) | `0` | Seconds |

### Episodic Memory
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `episodic-memory-enabled` | Enable conversation summaries | `true` | Boolean |
| `episodic-summary-model` | Model for summarization | `fast` | String |
| `episodic-max-episodes` | Maximum episodes per user | `100` | Number |
| `episodic-retention-days` | Days to retain episodes | `365` | Number |
| `episodic-auto-summarize` | Enable automatic summarization | `true` | Boolean |

## Model Routing Parameters

These parameters configure multi-model routing for different task types. Requires multiple llama.cpp server instances.

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `llm-models` | Available model aliases | `default` | Semicolon-separated |
| `model-routing-strategy` | Routing strategy (manual/auto/load-balanced/fallback) | `auto` | String |
| `model-default` | Default model alias | `default` | String |
| `model-fast` | Model for fast/simple tasks | (configured) | Path/String |
| `model-quality` | Model for quality/complex tasks | (configured) | Path/String |
| `model-code` | Model for code generation | (configured) | Path/String |
| `model-fallback-enabled` | Enable automatic fallback | `true` | Boolean |
| `model-fallback-order` | Order to try on failure | `quality,fast,local` | Comma-separated |

### Multi-Model Example
```csv
llm-models,default;fast;quality;code
llm-url,http://localhost:8081
model-routing-strategy,auto
model-default,fast
model-fallback-enabled,true
model-fallback-order,quality,fast
```

## Hybrid RAG Search Parameters

General Bots uses hybrid search combining **dense (embedding)** and **sparse (BM25 keyword)** search for optimal retrieval. The BM25 implementation is powered by [Tantivy](https://github.com/quickwit-oss/tantivy), a full-text search engine library similar to Apache Lucene.

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `rag-hybrid-enabled` | Enable hybrid dense+sparse search | `true` | Boolean |
| `rag-dense-weight` | Weight for semantic results | `0.7` | Float (0-1) |
| `rag-sparse-weight` | Weight for keyword results | `0.3` | Float (0-1) |
| `rag-reranker-enabled` | Enable LLM reranking | `false` | Boolean |
| `rag-reranker-model` | Model for reranking | `cross-encoder/ms-marco-MiniLM-L-6-v2` | String |
| `rag-reranker-top-n` | Candidates for reranking | `20` | Number |
| `rag-max-results` | Maximum results to return | `10` | Number |
| `rag-min-score` | Minimum relevance score threshold | `0.0` | Float (0-1) |
| `rag-rrf-k` | RRF smoothing constant | `60` | Number |
| `rag-cache-enabled` | Enable search result caching | `true` | Boolean |
| `rag-cache-ttl` | Cache time-to-live | `3600` | Seconds |

### BM25 Sparse Search (Tantivy)

BM25 is a keyword-based ranking algorithm that excels at finding exact term matches. It's powered by Tantivy when the `vectordb` feature is enabled.

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `bm25-enabled` | **Enable/disable BM25 sparse search** | `true` | Boolean |
| `bm25-k1` | Term frequency saturation (0.5-3.0 typical) | `1.2` | Float |
| `bm25-b` | Document length normalization (0.0-1.0) | `0.75` | Float |
| `bm25-stemming` | Apply word stemming (running→run) | `true` | Boolean |
| `bm25-stopwords` | Filter common words (the, a, is) | `true` | Boolean |

### Switching Search Modes

**Hybrid Search (Default - Best for most use cases)**
```csv
bm25-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
```
Uses both semantic understanding AND keyword matching. Best for general queries.

**Dense Only (Semantic Search)**
```csv
bm25-enabled,false
rag-dense-weight,1.0
rag-sparse-weight,0.0
```
Uses only embedding-based search. Faster, good for conceptual/semantic queries where exact words don't matter.

**Sparse Only (Keyword Search)**
```csv
bm25-enabled,true
rag-dense-weight,0.0
rag-sparse-weight,1.0
```
Uses only BM25 keyword matching. Good for exact term searches, technical documentation, or when embeddings aren't available.

### BM25 Parameter Tuning

The `k1` and `b` parameters control BM25 behavior:

- **`bm25-k1`** (Term Saturation): Controls how much additional term occurrences contribute to the score
  - Lower values (0.5-1.0): Diminishing returns for repeated terms
  - Higher values (1.5-2.0): More weight to documents with many term occurrences
  - Default `1.2` works well for most content

- **`bm25-b`** (Length Normalization): Controls document length penalty
  - `0.0`: No length penalty (long documents scored equally)
  - `1.0`: Full length normalization (strongly penalizes long documents)
  - Default `0.75` balances length fairness

**Tuning for specific content:**
```csv
# For short documents (tweets, titles)
bm25-b,0.3

# For long documents (articles, manuals)
bm25-b,0.9

# For code search (exact matches important)
bm25-k1,1.5
bm25-stemming,false
```

## Code Sandbox Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `sandbox-enabled` | Enable code sandbox | `true` | Boolean |
| `sandbox-runtime` | Isolation backend (lxc/docker/firecracker/process) | `lxc` | String |
| `sandbox-timeout` | Maximum execution time | `30` | Seconds |
| `sandbox-memory-mb` | Memory limit in megabytes | `256` | MB |
| `sandbox-cpu-percent` | CPU usage limit | `50` | Percent |
| `sandbox-network` | Allow network access | `false` | Boolean |
| `sandbox-python-packages` | Pre-installed Python packages | (none) | Comma-separated |
| `sandbox-allowed-paths` | Accessible filesystem paths | `/data,/tmp` | Comma-separated |

### Example: Python Sandbox
```csv
sandbox-enabled,true
sandbox-runtime,lxc
sandbox-timeout,60
sandbox-memory-mb,512
sandbox-cpu-percent,75
sandbox-network,false
sandbox-python-packages,numpy,pandas,requests,matplotlib
sandbox-allowed-paths,/data,/tmp,/uploads
```

## SSE Streaming Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `sse-enabled` | Enable Server-Sent Events | `true` | Boolean |
| `sse-heartbeat` | Heartbeat interval | `30` | Seconds |
| `sse-max-connections` | Maximum concurrent connections | `1000` | Number |

## Parameter Types

### Boolean
Values: `true` or `false` (case-sensitive)

### Number
Integer values, must be within valid ranges:
- Ports: 1-65535
- Tokens: Positive integers
- Percentages: 0-100

### Float
Decimal values:
- Thresholds: 0.0 to 1.0
- Weights: 0.0 to 1.0

### Path
File system paths:
- Relative: `../../../../data/model.gguf`
- Absolute: `/opt/models/model.gguf`

### URL
Valid URLs:
- HTTP: `http://localhost:8081`
- HTTPS: `https://api.example.com`

### String
Any text value (no quotes needed in CSV)

### Email
Valid email format: `user@domain.com`

### Hex Color
HTML color codes: `#RRGGBB` format

### Semicolon-separated
Multiple values separated by semicolons: `value1;value2;value3`

### Comma-separated
Multiple values separated by commas: `value1,value2,value3`

## Required vs Optional

### Always Required
- None - all parameters have defaults or are optional

### Required for Features
- **LLM**: `llm-model` must be set
- **Email**: `email-from`, `email-server`, `email-user`
- **Embeddings**: `embedding-model` for knowledge base
- **Custom DB**: `custom-database` if using external database

## Configuration Precedence

1. **Built-in defaults** (hardcoded)
2. **config.csv values** (override defaults)
3. **Environment variables** (if implemented, override config)

## Special Values

- `none` - Explicitly no value (for `llm-key`)
- Empty string - Unset/use default
- `false` - Feature disabled
- `true` - Feature enabled

## Performance Tuning

### For Local Models
```csv
llm-server-ctx-size,8192
llm-server-n-predict,2048
llm-server-parallel,4
llm-cache,true
llm-cache-ttl,7200
```

### For Production
```csv
llm-server-cont-batching,true
llm-cache-semantic,true
llm-cache-threshold,0.90
llm-server-parallel,8
sse-max-connections,5000
```

### For Low Memory
```csv
llm-server-ctx-size,2048
llm-server-n-predict,512
llm-server-mlock,false
llm-server-no-mmap,false
llm-cache,false
sandbox-memory-mb,128
```

### For Multi-Agent Systems
```csv
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-retry-count,3
a2a-persist-messages,true
bot-reflection-enabled,true
bot-reflection-interval,10
user-memory-enabled,true
```

### For Hybrid RAG
```csv
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
rag-max-results,10
rag-min-score,0.3
rag-cache-enabled,true
bm25-enabled,true
bm25-k1,1.2
bm25-b,0.75
```

### For Dense-Only Search (Faster)
```csv
bm25-enabled,false
rag-dense-weight,1.0
rag-sparse-weight,0.0
rag-max-results,10
```

### For Code Execution
```csv
sandbox-enabled,true
sandbox-runtime,lxc
sandbox-timeout,30
sandbox-memory-mb,512
sandbox-network,false
sandbox-python-packages,numpy,pandas,requests
```

## Validation Rules

1. **Paths**: Model files must exist
2. **URLs**: Must be valid format
3. **Ports**: Must be 1-65535
4. **Emails**: Must contain @ and domain
5. **Colors**: Must be valid hex format
6. **Booleans**: Exactly `true` or `false`
7. **Weights**: Must sum to 1.0 (e.g., `rag-dense-weight` + `rag-sparse-weight`)