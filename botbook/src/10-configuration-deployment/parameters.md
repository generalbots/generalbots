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
- **DeepSeek-R3-Distill-Qwen-7B**: Set `llm-server-gpu-layers` to 35-40
- **Qwen2.5-32B-Instruct (Q4_K_M)**: Fits with `llm-server-gpu-layers` to 40-45
- **DeepSeek-V3 (with MoE)**: Set `llm-server-n-moe` to 2-4 to run even 120B models! MoE only loads active experts
- **Optimization**: Use `llm-server-ctx-size` of 8192 for longer contexts

#### For RTX 4070/4070Ti (12-16GB VRAM)  
Mid-range cards work great with quantized models:
- **Qwen2.5-14B (Q4_K_M)**: Set `llm-server-gpu-layers` to 25-30
- **DeepSeek-R3-Distill-Llama-8B**: Fully fits with layers at 32
- **Tips**: Keep `llm-server-ctx-size` at 4096 to save VRAM

#### For CPU-Only (No GPU)
Modern CPUs can still run capable models:
- **DeepSeek-R3-Distill-Qwen-1.5B**: Fast on CPU, great for testing
- **Phi-3-mini (3.8B)**: Excellent CPU performance
- **Settings**: Set `llm-server-mlock` to `true` to prevent swapping
- **Parallel**: Increase `llm-server-parallel` to CPU cores -2

#### Recommended Models (GGUF Format)
- **Best Overall**: DeepSeek-R3-Distill series (1.5B to 70B)
- **Best Small**: Qwen2.5-3B-Instruct-Q5_K_M
- **Best Medium**: DeepSeek-R3-Distill-Qwen-14B-Q4_K_M  
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

## Email Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `email-from` | Sender address | Required for email | Email |
| `email-server` | SMTP hostname | Required for email | Hostname |
| `email-port` | SMTP port | `587` | Number |
| `email-user` | SMTP username | Required for email | String |
| `email-pass` | SMTP password | Required for email | String |
| `email-read-pixel` | Enable read tracking pixel in HTML emails | `false` | Boolean |

### Email Read Tracking

When `email-read-pixel` is enabled, a 1x1 transparent tracking pixel is automatically injected into HTML emails sent via the API. This allows you to:

- Track when emails are opened
- See how many times an email was opened
- Get the approximate location (IP) and device (user agent) of the reader

**API Endpoints for tracking:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/email/tracking/pixel/{tracking_id}` | GET | Serves the tracking pixel (called by email client) |
| `/api/email/tracking/status/{tracking_id}` | GET | Get read status for a specific email |
| `/api/email/tracking/list` | GET | List all sent emails with tracking status |
| `/api/email/tracking/stats` | GET | Get overall tracking statistics |

**Example configuration:**
```csv
email-read-pixel,true
server-url,https://yourdomain.com
```

**Note:** The `server-url` parameter is used to generate the tracking pixel URL. Make sure it's accessible from the recipient's email client.

**Privacy considerations:** Email tracking should be used responsibly. Consider disclosing tracking in your email footer for transparency.

## Theme Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `theme-color1` | Primary color | Not set | Hex color |
| `theme-color2` | Secondary color | Not set | Hex color |
| `theme-logo` | Logo URL | Not set | URL |
| `theme-title` | Bot display title | Not set | String |
| `bot-name` | Bot display name | Not set | String |
| `welcome-message` | Initial greeting message | Not set | String |

## Custom Database Parameters

These parameters configure external database connections for use with BASIC keywords like MariaDB/MySQL connections.

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `custom-server` | Database server hostname | `localhost` | Hostname |
| `custom-port` | Database port | `5432` | Number |
| `custom-database` | Database name | Not set | String |
| `custom-username` | Database user | Not set | String |
| `custom-password` | Database password | Not set | String |

## Website Crawling Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `website-expires` | Cache expiration for crawled content | `1d` | Duration |
| `website-max-depth` | Maximum crawl depth | `3` | Number |
| `website-max-pages` | Maximum pages to crawl | `100` | Number |

## Image Generator Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `image-generator-model` | Diffusion model path | Not set | Path |
| `image-generator-steps` | Inference steps | `4` | Number |
| `image-generator-width` | Output width | `512` | Pixels |
| `image-generator-height` | Output height | `512` | Pixels |
| `image-generator-gpu-layers` | GPU offload layers | `20` | Number |
| `image-generator-batch-size` | Batch size | `1` | Number |

## Video Generator Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `video-generator-model` | Video model path | Not set | Path |
| `video-generator-frames` | Frames to generate | `24` | Number |
| `video-generator-fps` | Frames per second | `8` | Number |
| `video-generator-width` | Output width | `320` | Pixels |
| `video-generator-height` | Output height | `576` | Pixels |
| `video-generator-gpu-layers` | GPU offload layers | `15` | Number |
| `video-generator-batch-size` | Batch size | `1` | Number |

## BotModels Service Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `botmodels-enabled` | Enable BotModels service | `true` | Boolean |
| `botmodels-host` | BotModels bind address | `0.0.0.0` | IP address |
| `botmodels-port` | BotModels port | `8085` | Number |

## Generator Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `default-generator` | Default content generator | `all` | String |

## Teams Channel Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `teams-app-id` | Microsoft Teams App ID | Not set | String |
| `teams-app-password` | Microsoft Teams App Password | Not set | String |
| `teams-tenant-id` | Microsoft Teams Tenant ID | Not set | String |
| `teams-bot-id` | Microsoft Teams Bot ID | Not set | String |

## SMS Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `sms-provider` | SMS provider (`twilio`, `aws`, `vonage`, `messagebird`, `custom`) | Not set | String |
| `sms-fallback-provider` | Fallback provider if primary fails | Not set | String |

### Twilio Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `twilio-account-sid` | Twilio Account SID | Not set | String |
| `twilio-auth-token` | Twilio Auth Token | Not set | String |
| `twilio-phone-number` | Twilio phone number (E.164 format) | Not set | String |
| `twilio-messaging-service-sid` | Messaging Service SID for routing | Not set | String |
| `twilio-status-callback` | Webhook URL for delivery status | Not set | URL |

### AWS SNS Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `aws-access-key-id` | AWS Access Key ID | Not set | String |
| `aws-secret-access-key` | AWS Secret Access Key | Not set | String |
| `aws-region` | AWS Region (e.g., `us-east-1`) | Not set | String |
| `aws-sns-sender-id` | Sender ID (alphanumeric) | Not set | String |
| `aws-sns-message-type` | `Promotional` or `Transactional` | `Transactional` | String |

### Vonage (Nexmo) Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `vonage-api-key` | Vonage API Key | Not set | String |
| `vonage-api-secret` | Vonage API Secret | Not set | String |
| `vonage-from` | Sender number or alphanumeric ID | Not set | String |
| `vonage-callback-url` | Delivery receipt webhook | Not set | URL |

### MessageBird Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `messagebird-access-key` | MessageBird Access Key | Not set | String |
| `messagebird-originator` | Sender number or name | Not set | String |
| `messagebird-report-url` | Status report webhook | Not set | URL |

### Custom Provider Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `sms-custom-url` | API endpoint URL | Not set | URL |
| `sms-custom-method` | HTTP method (`POST`, `GET`) | `POST` | String |
| `sms-custom-auth-header` | Authorization header value | Not set | String |
| `sms-custom-body-template` | JSON body with `{{to}}`, `{{message}}` placeholders | Not set | String |
| `sms-custom-from` | Sender number for custom provider | Not set | String |

### Example: Twilio Configuration
```csv
sms-provider,twilio
twilio-account-sid,ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-auth-token,your_auth_token
twilio-phone-number,+15551234567
```

### Example: AWS SNS Configuration
```csv
sms-provider,aws
aws-access-key-id,AKIAIOSFODNN7EXAMPLE
aws-secret-access-key,wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
aws-region,us-east-1
aws-sns-message-type,Transactional
```

See [SMS Provider Configuration](./sms-providers.md) for detailed setup instructions.

## WhatsApp Parameters

| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `whatsapp-api-key` | Access token from Meta Business | Not set | String |
| `whatsapp-phone-number-id` | Phone number ID from WhatsApp Business | Not set | String |
| `whatsapp-verify-token` | Token for webhook verification | Not set | String |
| `whatsapp-business-account-id` | WhatsApp Business Account ID | Not set | String |
| `whatsapp-api-version` | Graph API version | `v17.0` | String |

### Example: WhatsApp Configuration
```csv
whatsapp-api-key,EAABs...your_access_token
whatsapp-phone-number-id,123456789012345
whatsapp-verify-token,my-secret-verify-token
whatsapp-business-account-id,987654321098765
```

See [WhatsApp Channel Configuration](./whatsapp-channel.md) for detailed setup instructions.

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

### Episodic Memory (Context Compaction)
| Parameter | Description | Default | Type |
|-----------|-------------|---------|------|
| `episodic-memory-enabled` | Enable episodic memory system | `true` | Boolean |
| `episodic-memory-threshold` | Exchanges before compaction triggers | `4` | Number |
| `episodic-memory-history` | Recent exchanges to keep in full | `2` | Number |
| `episodic-memory-model` | Model for summarization | `fast` | String |
| `episodic-memory-max-episodes` | Maximum episodes per user | `100` | Number |
| `episodic-memory-retention-days` | Days to retain episodes | `365` | Number |
| `episodic-memory-auto-summarize` | Enable automatic summarization | `true` | Boolean |

Episodic memory automatically manages conversation context to stay within LLM token limits. When conversation exchanges exceed `episodic-memory-threshold`, older messages are summarized and only the last `episodic-memory-history` exchanges are kept in full. See [Chapter 03 - Episodic Memory](../03-knowledge-ai/episodic-memory.md) for details.

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