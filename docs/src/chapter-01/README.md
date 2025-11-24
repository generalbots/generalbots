# Chapter 01: Run and Talk - System Bootstrap and Initial Configuration

Welcome to General Bots - a comprehensive, self-hosted artificial intelligence platform designed for enterprise deployment and individual sovereignty. This chapter provides detailed technical guidance on system initialization, bootstrap processes, and fundamental operational concepts that form the foundation of your AI infrastructure.

## Executive Summary

General Bots represents a paradigm shift in conversational AI deployment, offering a fully autonomous, self-contained system that eliminates dependencies on external cloud services while maintaining enterprise-grade capabilities. The platform implements a zero-configuration bootstrap methodology that provisions all required infrastructure components automatically, enabling rapid deployment without specialized DevOps expertise.

### Key Architectural Principles

The system adheres to four fundamental design principles that govern its implementation:

1. **Infrastructure Autonomy**: Complete ownership of the technology stack, from persistent storage layers to natural language processing engines. All components operate within your controlled environment, ensuring data sovereignty and regulatory compliance.

2. **Deterministic Bootstrap**: The initialization sequence follows a predictable, idempotent process that guarantees consistent deployment outcomes across heterogeneous environments. Each bootstrap operation validates prerequisites, provisions resources, and verifies operational readiness through comprehensive health checks.

3. **Economic Efficiency**: The platform's architecture optimizes for total cost of ownership (TCO) by leveraging commodity hardware, open-source components, and efficient resource utilization patterns. Operational costs typically achieve 10x reduction compared to equivalent cloud-based solutions at scale.

4. **Privacy by Design**: Data residency, processing, and storage remain exclusively within your infrastructure perimeter. The system implements defense-in-depth security layers, including encryption at rest, transport layer security, and role-based access control (RBAC).

## System Requirements and Prerequisites

### Hardware Requirements

The platform requires the following minimum hardware specifications for production deployment:

| Component | Minimum Specification | Recommended Specification | Enterprise Specification |
|-----------|----------------------|---------------------------|-------------------------|
| CPU | 4 cores x86_64 | 8 cores x86_64 | 16+ cores x86_64 |
| Memory | 8 GB RAM | 16 GB RAM | 32+ GB RAM |
| Storage | 50 GB SSD | 200 GB NVMe SSD | 1+ TB NVMe RAID |
| Network | 100 Mbps | 1 Gbps | 10 Gbps |
| GPU | Not required | NVIDIA GTX 1060 | NVIDIA A100 |

### Operating System Compatibility

The bootstrap system supports the following operating system distributions:

- **Linux Distributions**: Ubuntu 20.04 LTS+, Debian 11+, RHEL 8+, CentOS Stream 8+, Fedora 35+, openSUSE Leap 15.3+
- **macOS**: macOS 12 Monterey or later (Intel and Apple Silicon architectures)
- **Windows**: Windows 10 Professional/Enterprise (version 2004+) with WSL2
- **Container Platforms**: Docker 20.10+, Kubernetes 1.21+, LXC/LXD 4.0+

### Network Configuration

The system requires specific network ports for inter-process communication and external access:

```
Port Allocation Table:
├── 8080  : HTTP Web Interface (configurable)
├── 8081  : LLM Inference Server (internal)
├── 5432  : PostgreSQL Database (internal)
├── 6379  : Cache Service (internal)
├── 9333  : Vector Database HTTP (internal)
├── 9334  : Vector Database gRPC (internal)
├── 9090  : Storage Service (internal)
└── 9091  : Storage Admin Interface (internal)
```

## The Bootstrap Process - Technical Deep Dive

### Phase 1: Environment Validation

The bootstrap initiator performs comprehensive environment validation before resource provisioning:

```bash
$ ./botserver
[Bootstrap] Detecting system architecture...
[Bootstrap] Platform: linux-x86_64
[Bootstrap] Available memory: 16384 MB
[Bootstrap] Available storage: 487 GB
[Bootstrap] Network interfaces: eth0 (1000 Mbps)
[Bootstrap] Validating kernel capabilities...
[Bootstrap] Checking file system permissions...
[Bootstrap] All prerequisites satisfied ✓
```

The validation phase examines:
- Processor architecture and instruction set extensions
- Available memory and swap configuration
- Storage capacity and file system types
- Network interface availability and bandwidth
- Kernel capabilities and security modules
- User permissions and system limits

### Phase 2: Component Provisioning

The system provisions infrastructure components in a carefully orchestrated sequence that respects inter-service dependencies:

#### Database Layer Initialization

PostgreSQL deployment includes:
- Binary distribution download and verification
- Database cluster initialization with optimal configuration
- Schema creation and migration execution
- Index generation and statistics initialization
- Connection pooling configuration
- Backup and recovery setup

```
[PostgreSQL] Downloading PostgreSQL 16.2...
[PostgreSQL] Verifying checksum: SHA256:a7b9c5d...✓
[PostgreSQL] Initializing database cluster...
[PostgreSQL] Creating system catalogs...
[PostgreSQL] Configuring shared buffers: 4096 MB
[PostgreSQL] Configuring effective cache: 12288 MB
[PostgreSQL] Creating connection pool (size: 100)
[PostgreSQL] Applying schema migrations...
[PostgreSQL] Database ready on port 5432 ✓
```

#### Caching Layer Deployment

The high-performance caching subsystem provides:
- In-memory data structure storage
- Session state persistence
- Distributed locking mechanisms
- Pub/sub messaging channels
- Time-series data support

```
[Cache] Installing Valkey 7.2.5...
[Cache] Configuring memory limit: 2048 MB
[Cache] Setting eviction policy: allkeys-lru
[Cache] Enabling persistence: AOF+RDB
[Cache] Configuring replication: disabled
[Cache] Cache service ready on port 6379 ✓
```

#### Object Storage System

The distributed object storage layer implements:
- Content-addressable storage (CAS)
- Automatic replication and erasure coding
- Multi-tier storage with hot/cold data management
- S3-compatible API interface
- Inline deduplication and compression

```
[Storage] Deploying SeaweedFS 3.59...
[Storage] Initializing master server...
[Storage] Starting volume servers (count: 3)...
[Storage] Configuring replication: 001 (no replication)
[Storage] Setting up S3 gateway...
[Storage] Creating default buckets...
[Storage] Object storage ready on port 9090 ✓
```

#### Vector Database Engine

The semantic search infrastructure provides:
- High-dimensional vector indexing
- Approximate nearest neighbor search
- Dynamic index updates
- Filtered search capabilities
- Horizontal scaling support

```
[Vectors] Installing Qdrant 1.7.4...
[Vectors] Configuring storage: ./botserver-stack/qdrant
[Vectors] Setting vector dimensions: 384
[Vectors] Creating default collection...
[Vectors] Building HNSW index (M=16, ef=100)...
[Vectors] Vector database ready on port 9333 ✓
```

### Phase 3: AI Model Deployment

The platform provisions machine learning models for various cognitive tasks:

#### Embedding Model Configuration

```
[Models] Downloading embedding model...
[Models] Model: all-MiniLM-L6-v2 (384 dimensions)
[Models] Format: ONNX (optimized for CPU)
[Models] Quantization: INT8 (4x size reduction)
[Models] Loading model into memory...
[Models] Warming up inference engine...
[Models] Embeddings ready (latency: 12ms) ✓
```

#### Language Model Setup

For local inference capability (optional):

```
[LLM] Configuring language model server...
[LLM] Model path: ./models/llama-7b-q4.gguf
[LLM] Context size: 4096 tokens
[LLM] Batch size: 512 tokens
[LLM] Thread count: 8
[LLM] GPU acceleration: disabled
[LLM] Starting inference server on port 8081...
[LLM] Model loaded (3.9 GB memory) ✓
```

### Phase 4: Application Initialization

The final bootstrap phase configures the application runtime:

```
[App] Creating default bot package...
[App] Loading dialog templates...
[App] Compiling BASIC interpreter...
[App] Registering system keywords...
[App] Initializing REST API endpoints...
[App] Starting WebSocket server...
[App] Launching web interface on :8080...
[App] System operational ✓

Bootstrap completed in 4m 32s
Web interface: http://localhost:8080
```

## Creating Your First Bot - Implementation Guide

### Package Structure and Organization

The bot package system implements a convention-over-configuration approach with hierarchical resource organization:

```
templates/
└── my-bot.gbai/                    # Package root (mandatory .gbai extension)
    ├── my-bot.gbdialog/            # Dialog scripts container
    │   ├── start.bas               # Entry point (required)
    │   ├── handlers/               # Event handlers
    │   │   ├── welcome.bas         # User onboarding
    │   │   └── error.bas           # Error handling
    │   └── tools/                  # Custom tools
    │       ├── scheduler.bas       # Appointment booking
    │       └── calculator.bas      # Calculations
    ├── my-bot.gbkb/                # Knowledge base
    │   ├── policies/               # Document collection
    │   │   ├── hr-manual.pdf       # Human resources
    │   │   └── it-security.pdf     # IT policies
    │   └── products/               # Product information
    │       ├── catalog.pdf         # Product catalog
    │       └── pricing.xlsx        # Pricing matrix
    ├── my-bot.gbot/                # Configuration
    │   └── config.csv              # Bot parameters
    ├── my-bot.gbtheme/             # Visual customization
    │   └── default.css             # Style definitions
    └── my-bot.gbdrive/             # File storage
        └── templates/              # Document templates
```

### Dialog Script Implementation

BASIC dialog scripts orchestrate conversational flows with minimal complexity:

```basic
' start.bas - Primary conversation entry point
' This script executes when users initiate conversation

' Initialize conversation context
SET CONTEXT "greeting_shown" = FALSE

' Load knowledge resources
USE KB "policies"      ' HR and IT policy documents
USE KB "products"      ' Product catalog and pricing

' Enable tool extensions
USE TOOL "scheduler"   ' Appointment booking capability
USE TOOL "calculator"  ' Mathematical computations

' Conversation state machine
main_loop:
    IF GET CONTEXT "greeting_shown" = FALSE THEN
        TALK "Welcome! I'm your intelligent assistant."
        TALK "I have access to company policies and product information."
        TALK "I can also schedule appointments and perform calculations."
        SET CONTEXT "greeting_shown" = TRUE
    END IF
    
    TALK "How may I assist you today?"
    
    ' Capture user input with intent classification
    user_input = HEAR
    
    ' The system AI automatically processes the input using:
    ' - Loaded knowledge bases for information retrieval
    ' - Enabled tools for action execution
    ' - Context history for coherent responses
    
    ' Continue conversation loop
    GOTO main_loop
```

### Knowledge Base Integration

Document collections are automatically indexed for semantic retrieval:

#### Document Processing Pipeline

1. **Ingestion**: Documents are parsed from supported formats (PDF, DOCX, XLSX, TXT, MD)
2. **Chunking**: Content is segmented into semantic units (typically 256-512 tokens)
3. **Embedding**: Each chunk is converted to vector representation using the embedding model
4. **Indexing**: Vectors are inserted into the vector database with metadata
5. **Retrieval**: Semantic search identifies relevant chunks based on query similarity

#### Supported Document Formats

| Format | Extensions | Processing Method | Metadata Extraction |
|--------|------------|------------------|-------------------|
| PDF | .pdf | Apache PDFBox | Title, Author, Creation Date |
| Word | .docx, .doc | Apache POI | Properties, Styles |
| Excel | .xlsx, .xls | Apache POI | Sheets, Named Ranges |
| Text | .txt, .md | Direct | File Properties |
| HTML | .html, .htm | JSoup | Meta Tags, Structure |
| CSV | .csv | Native Parser | Headers, Schema |

### Tool Development

Tools extend bot capabilities with parameterized functions:

```basic
' scheduler.bas - Meeting scheduling tool
PARAM person AS STRING DESCRIPTION "Person to meet"
PARAM date AS DATE DESCRIPTION "Meeting date"
PARAM time AS TIME DESCRIPTION "Meeting time"
PARAM duration AS INTEGER DEFAULT 30 DESCRIPTION "Duration in minutes"
PARAM location AS STRING DEFAULT "Conference Room" DESCRIPTION "Meeting location"

DESCRIPTION "Schedule a meeting with specified parameters"

' Validate scheduling constraints
IF date < TODAY() THEN
    TALK "Cannot schedule meetings in the past."
    EXIT
END IF

IF time < "08:00" OR time > "18:00" THEN
    TALK "Meetings must be scheduled during business hours (8 AM - 6 PM)."
    EXIT
END IF

' Check availability (simplified example)
conflicts = FIND "meetings", "date=" + date + " AND time=" + time
IF conflicts.COUNT > 0 THEN
    TALK "Time slot unavailable. Suggesting alternatives..."
    ' AI generates alternative suggestions
    EXIT
END IF

' Create meeting record
meeting_id = GENERATE_ID()
SAVE "meetings", meeting_id, person, date, time, duration, location

' Send confirmation
TALK "Meeting scheduled successfully!"
TALK "Details: " + person + " on " + FORMAT(date, "MMMM dd, yyyy")
TALK "Time: " + FORMAT(time, "h:mm a") + " for " + duration + " minutes"
TALK "Location: " + location

' Optional: Send calendar invitation
IF GET BOT MEMORY "send_invites" = TRUE THEN
    SEND MAIL person, "Meeting Invitation", "You're invited to a meeting..."
END IF
```

## Configuration Management

### Environment Variables

The system uses environment variables for deployment-specific configuration:

```bash
# Network Configuration
HTTP_PORT=8080                    # Web interface port
HTTP_HOST=0.0.0.0                # Binding address
BASE_URL=https://bot.company.com  # Public URL

# Resource Limits
MAX_MEMORY=8192                   # Maximum memory (MB)
MAX_CONNECTIONS=1000              # Connection pool size
MAX_WORKERS=16                    # Worker thread count

# Storage Paths
DATA_DIR=/var/lib/botserver      # Data directory
TEMP_DIR=/tmp/botserver           # Temporary files
LOG_DIR=/var/log/botserver        # Log files

# Feature Flags
ENABLE_GPU=false                  # GPU acceleration
ENABLE_ANALYTICS=true             # Usage analytics
ENABLE_BACKUP=true                # Automatic backups
```

### Configuration File (config.csv)

Bot-specific parameters are defined in CSV format:

```csv
name,value,description,category
# LLM Configuration
llm-provider,local,LLM service provider,llm
llm-model,./models/llama2-7b.gguf,Model file path,llm
llm-context,4096,Context window size,llm
llm-temperature,0.7,Sampling temperature,llm
llm-max-tokens,2048,Maximum response tokens,llm

# Context Management
context-compaction,auto,Compaction strategy,context
context-max-messages,50,Message history limit,context
context-summary-threshold,1000,Summary trigger tokens,context

# Knowledge Base
kb-chunk-size,512,Document chunk size,knowledge
kb-chunk-overlap,50,Chunk overlap tokens,knowledge
kb-relevance-threshold,0.7,Minimum similarity score,knowledge
kb-max-results,5,Maximum search results,knowledge

# Session Management
session-timeout,3600,Session timeout (seconds),session
session-persistence,true,Persist sessions to disk,session
session-encryption,true,Encrypt session data,session
```

## Operational Procedures

### System Monitoring

Monitor system health through built-in endpoints:

```bash
# Check component status
$ ./botserver status
Component       Status    CPU    Memory    Uptime
─────────────────────────────────────────────────
PostgreSQL      Running   2.1%   487 MB    4h 32m
Cache           Running   0.8%   156 MB    4h 32m
Storage         Running   1.4%   234 MB    4h 32m
Vectors         Running   3.2%   892 MB    4h 32m
LLM Server      Running   45.6%  3.9 GB    4h 31m
Web Server      Running   5.3%   312 MB    4h 31m

# View metrics
$ ./botserver metrics
Metric                    Value       Change (1h)
──────────────────────────────────────────────────
Total Conversations       1,247       +82
Active Sessions          34          +5
Messages Processed       15,823      +1,247
Knowledge Queries        3,421       +234
Tool Invocations        892         +67
Average Response Time    342ms       -12ms
Cache Hit Rate          87.3%       +2.1%
Vector Search Latency   23ms        -3ms
```

### Backup and Recovery

The system implements comprehensive backup strategies:

#### Automated Backups

```bash
# Configure automated backups
$ ./botserver backup configure
Backup Schedule: Daily at 02:00 UTC
Retention Policy: 30 days
Destination: ./backups/
Compression: Enabled (zstd)
Encryption: Enabled (AES-256)

# Manual backup
$ ./botserver backup create
Creating backup snapshot...
[Database] Dumping PostgreSQL... 234 MB
[Vectors] Exporting collections... 567 MB
[Storage] Copying objects... 1.2 GB
[Config] Saving configuration... 12 KB
Compressing backup... 689 MB (42% ratio)
Backup created: backup-20240315-143022.tar.zst
```

#### Disaster Recovery

```bash
# Restore from backup
$ ./botserver restore backup-20240315-143022.tar.zst
Extracting backup archive...
Stopping services...
[Database] Restoring PostgreSQL...
[Vectors] Importing collections...
[Storage] Restoring objects...
[Config] Applying configuration...
Starting services...
Restoration completed successfully
```

### Performance Optimization

#### Database Tuning

```sql
-- Analyze query performance
EXPLAIN ANALYZE
SELECT * FROM conversations
WHERE bot_id = $1 AND created_at > $2
ORDER BY created_at DESC
LIMIT 100;

-- Create optimized indexes
CREATE INDEX idx_conversations_bot_created 
ON conversations(bot_id, created_at DESC);

-- Update statistics
ANALYZE conversations;
```

#### Cache Optimization

```bash
# Monitor cache performance
$ ./botserver cache stats
Keyspace hits: 127,834
Keyspace misses: 14,234
Hit ratio: 89.97%
Evicted keys: 3,421
Memory used: 1.4 GB / 2.0 GB
```

#### Vector Index Tuning

```python
# Optimize HNSW parameters
{
    "index_type": "hnsw",
    "parameters": {
        "m": 32,              # Connectivity parameter
        "ef_construction": 200, # Construction time quality
        "ef_runtime": 100,     # Search time quality
        "metric": "cosine"    # Distance metric
    }
}
```

## Container Deployment Architecture

### LXC Container Isolation

The platform supports Linux Container (LXC) deployment for enhanced isolation:

```bash
$ ./botserver --container
[LXC] Creating container network...
[LXC] Network: 10.10.10.0/24
[LXC] Creating containers...
├── botserver-postgres (10.10.10.2)
├── botserver-cache (10.10.10.3)
├── botserver-storage (10.10.10.4)
├── botserver-vectors (10.10.10.5)
└── botserver-app (10.10.10.6)
[LXC] Configuring inter-container communication...
[LXC] Setting resource limits...
[LXC] Starting containers...
[LXC] All containers operational ✓
```

### Container Resource Management

```yaml
# Container resource limits
postgres:
  cpu: 2
  memory: 4GB
  storage: 100GB
  
cache:
  cpu: 1
  memory: 2GB
  storage: 10GB
  
storage:
  cpu: 2
  memory: 2GB
  storage: 500GB
  
vectors:
  cpu: 2
  memory: 4GB
  storage: 50GB
  
app:
  cpu: 4
  memory: 8GB
  storage: 20GB
```

## Security Considerations

### Network Security

The platform implements defense-in-depth network security:

1. **Transport Layer Security**: All inter-service communication uses TLS 1.3
2. **Network Segmentation**: Services operate in isolated network namespaces
3. **Firewall Rules**: Automatic iptables/nftables configuration
4. **Rate Limiting**: Connection and request throttling
5. **DDoS Protection**: SYN flood mitigation and connection limits

### Data Protection

Comprehensive data protection mechanisms include:

1. **Encryption at Rest**: AES-256 for database and storage
2. **Key Management**: Hardware security module (HSM) integration
3. **Access Control**: Role-based permissions with audit logging
4. **Data Anonymization**: PII detection and masking
5. **Compliance**: GDPR, HIPAA, SOC 2 support

## Troubleshooting Guide

### Common Issues and Resolutions

#### Port Conflicts

```bash
# Error: Address already in use
$ lsof -i :8080
COMMAND  PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
nginx   1234 www    6u  IPv4  12345      0t0  TCP *:8080

# Resolution
$ HTTP_PORT=3000 ./botserver
# Or stop conflicting service
$ sudo systemctl stop nginx
```

#### Memory Limitations

```bash
# Error: Cannot allocate memory
# Check system limits
$ ulimit -a
$ sysctl vm.overcommit_memory

# Increase limits
$ ulimit -n 65536  # File descriptors
$ ulimit -u 32768  # Processes
$ sudo sysctl vm.overcommit_memory=1
```

#### Database Connection Issues

```bash
# Error: Connection refused
# Check PostgreSQL status
$ ./botserver status postgres
PostgreSQL: Stopped

# View logs
$ tail -f botserver-stack/postgres/logs/postgresql.log

# Restart service
$ ./botserver restart postgres
```

### Diagnostic Commands

```bash
# System diagnostics
$ ./botserver diagnose
Running system diagnostics...
✓ CPU architecture compatible
✓ Memory sufficient (12.4 GB available)
✓ Storage adequate (234 GB free)
✓ Network connectivity verified
✓ DNS resolution working
✓ Time synchronization accurate
⚠ Swap disabled (recommended: 8 GB)
✓ Kernel parameters optimal
✓ File descriptors adequate (65536)

# Component health checks
$ ./botserver health
Service         Endpoint              Status   Latency
────────────────────────────────────────────────────
PostgreSQL      localhost:5432        Healthy  2ms
Cache           localhost:6379        Healthy  <1ms
Storage         localhost:9090        Healthy  5ms
Vectors         localhost:9333        Healthy  3ms
LLM             localhost:8081        Healthy  45ms
Application     localhost:8080        Healthy  12ms
```

## Performance Metrics and Benchmarks

### System Performance Characteristics

| Operation | Latency (p50) | Latency (p99) | Throughput | Resource Usage |
|-----------|---------------|---------------|------------|----------------|
| Message Processing | 42ms | 156ms | 2,400/sec | CPU: 15%, RAM: 1.2GB |
| Knowledge Query | 78ms | 234ms | 450/sec | CPU: 25%, RAM: 2.1GB |
| Tool Invocation | 156ms | 512ms | 180/sec | CPU: 35%, RAM: 1.8GB |
| Document Indexing | 2.3s/doc | 5.6s/doc | 25 docs/min | CPU: 45%, RAM: 3.2GB |
| Session Creation | 12ms | 34ms | 5,000/sec | CPU: 8%, RAM: 0.5GB |

### Scalability Characteristics

The platform demonstrates linear scalability characteristics:

- **Horizontal Scaling**: Add nodes for increased capacity
- **Vertical Scaling**: Utilize additional CPU/RAM resources
- **Load Distribution**: Automatic request balancing
- **Session Affinity**: Sticky sessions for stateful operations
- **Cache Coherence**: Distributed cache synchronization

## Integration Capabilities

### API Integration Patterns

The platform supports multiple integration patterns:

#### REST API Integration
```bash
# Create conversation
POST /api/conversations
{
  "bot_id": "my-bot",
  "user_id": "user-123",
  "message": "Hello, I need help"
}

# Response
{
  "conversation_id": "conv-789",
  "session_id": "sess-456",
  "response": "Hello! I'm here to help. What do you need assistance with?",
  "suggested_actions": ["Product Information", "Technical Support", "Billing"]
}
```

#### WebSocket Real-time Communication
```javascript
// Client connection
const ws = new WebSocket('ws://localhost:8080/ws');

ws.on('message', (data) => {
  const message = JSON.parse(data);
  console.log('Bot response:', message.text);
});

ws.send(JSON.stringify({
  type: 'message',
  text: 'What are your business hours?'
}));
```

#### Webhook Event Notifications
```yaml
webhooks:
  - url: https://api.company.com/bot-events
    events:
      - conversation.started
      - conversation.ended
      - tool.invoked
      - error.occurred
    headers:
      Authorization: Bearer ${WEBHOOK_SECRET}
```

## Advanced Configuration

### Multi-Tenant Deployment

Configure isolated bot instances for multiple tenants:

```csv
name,value,category
tenant-mode,enabled,system
tenant-isolation,strict,system
tenant-database-prefix,tenant_,system
tenant-storage-bucket,tenant-{id},system
tenant-resource-limits,true,system
tenant-max-conversations,1000,limits
tenant-max-storage,10GB,limits
tenant-max-compute,4CPU,limits
```

### High Availability Configuration

Implement redundancy for production deployments:

```yaml
cluster:
  mode: active-passive
  nodes:
    - primary: bot1.company.com
    - standby: bot2.company.com
  
  replication:
    type: streaming
    lag_threshold: 100ms
  
  failover:
    automatic: true
    timeout: 30s
    
  load_balancing:
    algorithm: least_connections
    health_check: /health
    interval: 10s
```

## Compliance and Governance

### Audit Logging

Comprehensive audit trail for all system operations:

```json
{
  "timestamp": "2024-03-15T14:30:22.123Z",
  "event_type": "conversation.message",
  "actor": {
    "type": "user",
    "id": "user-123",
    "ip": "192.168.1.100"
  },
  "resource": {
    "type": "conversation",
    "id": "conv-789",
    "bot_id": "my-bot"
  },
  "action": "create",
  "result": "success",
  "metadata": {
    "message_length": 45,
    "tools_used": ["scheduler"],
    "knowledge_bases": ["policies