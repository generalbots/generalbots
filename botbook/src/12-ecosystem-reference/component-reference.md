# Component Reference

This reference provides detailed information about each component in the botserver stack, including current versions, alternatives, and configuration options.

---

## Core Components

### Vault (Secrets Management)

| Property | Value |
|----------|-------|
| **Service** | HashiCorp Vault |
| **Current Version** | 1.15.4 |
| **Default Port** | 8200 |
| **Binary Path** | `botserver-stack/bin/vault/vault` |
| **Config Path** | `botserver-stack/conf/vault/` |
| **Data Path** | `botserver-stack/data/vault/` |
| **Log File** | `botserver-stack/logs/vault.log` |

**Download URL:**
```
https://releases.hashicorp.com/vault/1.15.4/vault_1.15.4_linux_amd64.zip
```

**Purpose:**
- Stores all service credentials (database, drive, cache)
- Manages encryption keys
- Provides secrets rotation
- Issues short-lived tokens

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [OpenBao](https://openbao.org/) | MPL-2.0 | Fork of Vault, fully open source |
| [Infisical](https://infisical.com/) | MIT | Modern secrets management |
| [SOPS](https://github.com/getsops/sops) | MPL-2.0 | File-based encryption |
| [Doppler](https://doppler.com/) | Proprietary | Cloud-based alternative |

---

### PostgreSQL (Tables/Database)

| Property | Value |
|----------|-------|
| **Service** | PostgreSQL |
| **Current Version** | 17.2.0 |
| **Default Port** | 5432 |
| **Binary Path** | `botserver-stack/bin/tables/` |
| **Config Path** | `botserver-stack/conf/tables/` |
| **Data Path** | `botserver-stack/data/tables/` |
| **Log File** | `botserver-stack/logs/postgres.log` |

**Download URL:**
```
https://github.com/theseus-rs/postgresql-binaries/releases/download/17.2.0/postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz
```

**Purpose:**
- Primary relational database
- Stores bot configurations, users, conversations
- Supports full-text search
- Handles transactions and ACID compliance

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [CockroachDB](https://www.cockroachlabs.com/) | BSL/CCL | Distributed SQL, PostgreSQL-compatible |
| [YugabyteDB](https://www.yugabyte.com/) | Apache-2.0 | Distributed PostgreSQL |
| [Neon](https://neon.tech/) | Apache-2.0 | Serverless PostgreSQL |
| [Supabase](https://supabase.com/) | Apache-2.0 | PostgreSQL with extras |

---

### Zitadel (Directory/Identity)

| Property | Value |
|----------|-------|
| **Service** | Zitadel |
| **Current Version** | 2.70.4 |
| **Default Port** | 8080 |
| **Binary Path** | `botserver-stack/bin/directory/zitadel` |
| **Config Path** | `botserver-stack/conf/directory/` |
| **Data Path** | Uses PostgreSQL |
| **Log File** | `botserver-stack/logs/zitadel.log` |

**Download URL:**
```
https://github.com/zitadel/zitadel/releases/download/v2.70.4/zitadel-linux-amd64.tar.gz
```

**Purpose:**
- User authentication and authorization
- OAuth2/OIDC provider
- Single Sign-On (SSO)
- Multi-factor authentication
- Service credential provisioning

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Keycloak](https://www.keycloak.org/) | Apache-2.0 | Java-based, feature-rich |
| [Authentik](https://goauthentik.io/) | Custom OSS | Python-based, modern UI |
| [Authelia](https://www.authelia.com/) | Apache-2.0 | Lightweight, Nginx integration |
| [Ory](https://www.ory.sh/) | Apache-2.0 | Modular identity infrastructure |
| [Casdoor](https://casdoor.org/) | Apache-2.0 | Go-based, UI-focused |

---

### MinIO (Drive/Object Storage)

| Property | Value |
|----------|-------|
| **Service** | MinIO |
| **Current Version** | Latest |
| **Default Ports** | 9000 (API), 9001 (Console) |
| **Binary Path** | `botserver-stack/bin/drive/minio` |
| **Config Path** | `botserver-stack/conf/drive/` |
| **Data Path** | `botserver-stack/data/drive/` |
| **Log File** | `botserver-stack/logs/minio.log` |

**Download URL:**
```
https://dl.min.io/server/minio/release/linux-amd64/minio
```

**Purpose:**
- S3-compatible object storage
- Stores bot packages (.gbai, .gbkb, etc.)
- File uploads and downloads
- Static asset hosting

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [SeaweedFS](https://github.com/seaweedfs/seaweedfs) | Apache-2.0 | Distributed, fast |
| [Garage](https://garagehq.deuxfleurs.fr/) | AGPL-3.0 | Lightweight, geo-distributed |
| [Ceph](https://ceph.io/) | LGPL-2.1 | Enterprise-grade, complex |
| [LakeFS](https://lakefs.io/) | Apache-2.0 | Git-like versioning for data |

---

### Valkey (Cache)

| Property | Value |
|----------|-------|
| **Service** | Valkey |
| **Current Version** | 8.0.2 |
| **Default Port** | 6379 |
| **Binary Path** | `botserver-stack/bin/cache/valkey-server` |
| **Config Path** | `botserver-stack/conf/cache/` |
| **Data Path** | `botserver-stack/data/cache/` |
| **Log File** | `botserver-stack/logs/valkey.log` |

**Download URL:**
```
https://github.com/valkey-io/valkey/archive/refs/tags/8.0.2.tar.gz
```

**Note:** Valkey requires compilation from source. Build dependencies: `gcc`, `make`

**Purpose:**
- In-memory caching
- Session storage
- Rate limiting
- Pub/Sub messaging
- Queue management

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [KeyDB](https://docs.keydb.dev/) | BSD-3 | Multi-threaded Redis fork |
| [Dragonfly](https://www.dragonflydb.io/) | BSL | High-performance, Redis-compatible |
| [Garnet](https://github.com/microsoft/garnet) | MIT | Microsoft's cache store |
| [Skytable](https://skytable.io/) | AGPL-3.0 | Modern NoSQL |

---

### llama.cpp (LLM Server)

| Property | Value |
|----------|-------|
| **Service** | llama.cpp |
| **Current Version** | b7345 |
| **Default Ports** | 8081 (LLM), 8082 (Embedding) |
| **Binary Path** | `botserver-stack/bin/llm/llama-server` |
| **Config Path** | `botserver-stack/conf/llm/` |
| **Data Path** | `botserver-stack/data/llm/` (models) |
| **Log File** | `botserver-stack/logs/llm.log` |

**Download URLs by Platform:**

| Platform | URL |
|----------|-----|
| Linux x64 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-x64.zip` |
| Linux x64 Vulkan | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-ubuntu-vulkan-x64.zip` |
| macOS ARM64 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-macos-arm64.zip` |
| macOS x64 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-macos-x64.zip` |
| Windows x64 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-win-cpu-x64.zip` |
| Windows CUDA 12 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-win-cuda-12.4-x64.zip` |
| Windows CUDA 13 | `https://github.com/ggml-org/llama.cpp/releases/download/b7345/llama-b7345-bin-win-cuda-13.1-x64.zip` |

**SHA256 Checksums:**
```
llama-b7345-bin-ubuntu-x64.zip:        91b066ecc53c20693a2d39703c12bc7a69c804b0768fee064d47df702f616e52
llama-b7345-bin-ubuntu-vulkan-x64.zip: 03f0b3acbead2ddc23267073a8f8e0207937c849d3704c46c61cf167c1001442
llama-b7345-bin-macos-arm64.zip:       72ae9b4a4605aa1223d7aabaa5326c66c268b12d13a449fcc06f61099cd02a52
llama-b7345-bin-macos-x64.zip:         bec6b805cf7533f66b38f29305429f521dcb2be6b25dbce73a18df448ec55cc5
llama-b7345-bin-win-cpu-x64.zip:       ea449082c8e808a289d9a1e8331f90a0379ead4dd288a1b9a2d2c0a7151836cd
llama-b7345-bin-win-cuda-12.4-x64.zip: 7a82aba2662fa7d4477a7a40894de002854bae1ab8b0039888577c9a2ca24cae
llama-b7345-bin-win-cuda-13.1-x64.zip: 06ea715cefb07e9862394e6d1ffa066f4c33add536b1f1aa058723f86ae05572
```

**Purpose:**
- Local LLM inference
- Text embeddings for semantic search
- OpenAI-compatible API
- Supports GGUF model format

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Ollama](https://ollama.ai/) | MIT | User-friendly, model management |
| [vLLM](https://github.com/vllm-project/vllm) | Apache-2.0 | High throughput, production-grade |
| [Text Generation Inference](https://github.com/huggingface/text-generation-inference) | Apache-2.0 | HuggingFace's solution |
| [LocalAI](https://localai.io/) | MIT | Drop-in OpenAI replacement |
| [LM Studio](https://lmstudio.ai/) | Proprietary | Desktop GUI application |

---

## Supporting Components

### Stalwart (Email Server)

| Property | Value |
|----------|-------|
| **Service** | Stalwart Mail Server |
| **Current Version** | 0.10.7 |
| **Default Ports** | 25 (SMTP), 993 (IMAPS), 587 (Submission) |
| **Binary Path** | `botserver-stack/bin/email/stalwart-mail` |
| **Config Path** | `botserver-stack/conf/email/` |
| **Data Path** | `botserver-stack/data/email/` |
| **Log File** | `botserver-stack/logs/stalwart.log` |

**Download URL:**
```
https://github.com/stalwartlabs/mail-server/releases/download/v0.10.7/stalwart-mail-x86_64-linux.tar.gz
```

**Purpose:**
- Full email server (SMTP, IMAP, JMAP)
- Email sending and receiving
- Spam filtering
- DKIM/SPF/DMARC support

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Maddy](https://maddy.email/) | GPL-3.0 | Composable mail server |
| [Mail-in-a-Box](https://mailinabox.email/) | CC0 | All-in-one solution |
| [Postal](https://postalserver.io/) | MIT | Sending-focused |
| [Haraka](https://haraka.github.io/) | MIT | Node.js SMTP |

---

### Caddy (Proxy)

| Property | Value |
|----------|-------|
| **Service** | Caddy |
| **Current Version** | 2.9.1 |
| **Default Ports** | 443 (HTTPS), 80 (HTTP) |
| **Binary Path** | `botserver-stack/bin/proxy/caddy` |
| **Config Path** | `botserver-stack/conf/proxy/Caddyfile` |
| **Data Path** | `botserver-stack/data/proxy/` |
| **Log File** | `botserver-stack/logs/caddy.log` |

**Download URL:**
```
https://github.com/caddyserver/caddy/releases/download/v2.9.1/caddy_2.9.1_linux_amd64.tar.gz
```

**Purpose:**
- Automatic HTTPS with Let's Encrypt
- Reverse proxy for all services
- Load balancing
- HTTP/2 and HTTP/3 support

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Nginx](https://nginx.org/) | BSD-2 | Industry standard |
| [Traefik](https://traefik.io/) | MIT | Cloud-native, auto-discovery |
| [HAProxy](https://www.haproxy.org/) | GPL-2.0 | High performance |
| [Envoy](https://www.envoyproxy.io/) | Apache-2.0 | Service mesh ready |

---

### CoreDNS (DNS)

| Property | Value |
|----------|-------|
| **Service** | CoreDNS |
| **Current Version** | 1.11.1 |
| **Default Port** | 53 |
| **Binary Path** | `botserver-stack/bin/dns/coredns` |
| **Config Path** | `botserver-stack/conf/dns/Corefile` |
| **Log File** | `botserver-stack/logs/coredns.log` |

**Download URL:**
```
https://github.com/coredns/coredns/releases/download/v1.11.1/coredns_1.11.1_linux_amd64.tgz
```

**Purpose:**
- Local DNS resolution
- Service discovery (*.botserver.local)
- DNS-based load balancing

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [PowerDNS](https://www.powerdns.com/) | GPL-2.0 | Feature-rich, authoritative |
| [Unbound](https://nlnetlabs.nl/projects/unbound/) | BSD | Validating resolver |
| [dnsmasq](https://thekelleys.org.uk/dnsmasq/doc.html) | GPL-2.0 | Lightweight |

---

### Forgejo (ALM/Git)

| Property | Value |
|----------|-------|
| **Service** | Forgejo |
| **Current Version** | 10.0.2 |
| **Default Port** | 3000 |
| **Binary Path** | `botserver-stack/bin/alm/forgejo` |
| **Config Path** | `botserver-stack/conf/alm/` |
| **Data Path** | `botserver-stack/data/alm/` |
| **Log File** | `botserver-stack/logs/forgejo.log` |

**Download URL:**
```
https://codeberg.org/forgejo/forgejo/releases/download/v10.0.2/forgejo-10.0.2-linux-amd64
```

**Purpose:**
- Git repository hosting
- Issue tracking
- CI/CD pipelines
- Code review

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Gitea](https://gitea.io/) | MIT | Original project |
| [GitLab](https://gitlab.com/) | MIT (CE) | Full DevOps platform |
| [Gogs](https://gogs.io/) | MIT | Lightweight |
| [OneDev](https://onedev.io/) | MIT | Built-in CI/CD |

---

### LiveKit (Meeting/Video)

| Property | Value |
|----------|-------|
| **Service** | LiveKit |
| **Current Version** | 2.8.2 |
| **Default Ports** | 7880 (HTTP), 7881 (RTC) |
| **Binary Path** | `botserver-stack/bin/meeting/livekit-server` |
| **Config Path** | `botserver-stack/conf/meeting/` |
| **Log File** | `botserver-stack/logs/livekit.log` |

**Download URL:**
```
https://github.com/livekit/livekit/releases/download/v2.8.2/livekit_2.8.2_linux_amd64.tar.gz
```

**Purpose:**
- Real-time video/audio communication
- WebRTC infrastructure
- Screen sharing
- Recording

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Jitsi](https://jitsi.org/) | Apache-2.0 | Full-featured, established |
| [BigBlueButton](https://bigbluebutton.org/) | LGPL-3.0 | Education-focused |
| [Janus](https://janus.conf.meetecho.com/) | GPL-3.0 | WebRTC gateway |
| [mediasoup](https://mediasoup.org/) | ISC | Node.js SFU |

---

## Optional Components

### Qdrant (Vector Database)

| Property | Value |
|----------|-------|
| **Service** | Qdrant |
| **Current Version** | Latest |
| **Default Ports** | 6333 (HTTP), 6334 (gRPC) |
| **Binary Path** | `botserver-stack/bin/vector_db/qdrant` |

**Download URL:**
```
https://github.com/qdrant/qdrant/releases/latest/download/qdrant-x86_64-unknown-linux-gnu.tar.gz
```

**Purpose:**
- Vector similarity search
- Knowledge base embeddings
- Semantic search

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [Milvus](https://milvus.io/) | Apache-2.0 | Distributed, scalable |
| [Weaviate](https://weaviate.io/) | BSD-3 | GraphQL API |
| [Chroma](https://www.trychroma.com/) | Apache-2.0 | Simple, embedded |
| [pgvector](https://github.com/pgvector/pgvector) | PostgreSQL | PostgreSQL extension |

---

### InfluxDB (Time Series)

| Property | Value |
|----------|-------|
| **Service** | InfluxDB |
| **Current Version** | 2.7.5 |
| **Default Port** | 8086 |
| **Binary Path** | `botserver-stack/bin/timeseries_db/influxd` |

**Download URL:**
```
https://download.influxdata.com/influxdb/releases/influxdb2-2.7.5-linux-amd64.tar.gz
```

**Purpose:**
- Metrics storage
- Time-series analytics
- Monitoring dashboards

**Alternatives:**
| Alternative | License | Notes |
|-------------|---------|-------|
| [TimescaleDB](https://www.timescale.com/) | Apache-2.0 | PostgreSQL extension |
| [VictoriaMetrics](https://victoriametrics.com/) | Apache-2.0 | Prometheus-compatible |
| [QuestDB](https://questdb.io/) | Apache-2.0 | High-performance SQL |
| [Prometheus](https://prometheus.io/) | Apache-2.0 | Monitoring-focused |

---

## Default LLM Models

### DeepSeek R1 Distill Qwen 1.5B

| Property | Value |
|----------|-------|
| **Filename** | `DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf` |
| **Size** | ~1.1 GB |
| **RAM Required** | 4 GB |
| **Use Case** | Default conversational model |

**Download URL:**
```
https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
```

### BGE Small EN v1.5

| Property | Value |
|----------|-------|
| **Filename** | `bge-small-en-v1.5-f32.gguf` |
| **Size** | ~130 MB |
| **RAM Required** | 512 MB |
| **Use Case** | Text embeddings for semantic search |

**Download URL:**
```
https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-f32.gguf
```

---

## Configuration Files Reference

| File | Purpose |
|------|---------|
| `3rdparty.toml` | Component download URLs and checksums |
| `config/llm_releases.json` | Platform-specific LLM builds |
| `botserver-stack/conf/*/` | Per-component configuration |
| `.env` | Environment variables (generated) |

---

## See Also

- [Updating Components](./updating-components.md) - How to update
- [Security Auditing](./security-auditing.md) - Vulnerability scanning
- [Troubleshooting](./troubleshooting.md) - Common issues