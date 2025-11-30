# Feature Editions

General Bots offers flexible feature configurations to match different deployment needs. Features can be enabled at compile time using Cargo feature flags or selected through pre-configured edition bundles.

## Edition Overview

| Edition | Target Use Case | Key Features |
|---------|-----------------|--------------|
| **Minimal** | Embedded, IoT, testing | Basic chat only |
| **Lightweight** | Small teams, startups | Chat + Drive + Tasks |
| **Core** | General business use | Full productivity suite |
| **Standard** | Professional teams | + Email + Calendar + Meet |
| **Enterprise** | Large organizations | + Compliance + Multi-channel + GPU |
| **Full** | Maximum capability | All features enabled |

---

## Minimal Edition

**Use Case:** Embedded systems, IoT devices, testing environments

**Cargo Feature:** `minimal`

```bash
cargo build --features minimal
```

### Included Features
- ✅ UI Server (web interface)
- ✅ Basic chat functionality

### Not Included
- ❌ Console TUI
- ❌ File storage
- ❌ Task management
- ❌ Email
- ❌ LLM integration
- ❌ Vector search

**Typical Deployment:** Raspberry Pi, edge devices, containerized microservices

---

## Lightweight Edition

**Use Case:** Small teams, startups, personal use

**Cargo Feature:** `lightweight`

```bash
cargo build --features lightweight
```

### Included Features
- ✅ UI Server
- ✅ Chat
- ✅ Drive (file storage)
- ✅ Tasks
- ✅ Redis caching

### Not Included
- ❌ Email integration
- ❌ Calendar
- ❌ Video meetings
- ❌ Compliance tools
- ❌ Multi-channel messaging

**Typical Deployment:** Small office server, developer workstation

---

## Core Edition (Default)

**Use Case:** General business operations, mid-size teams

**Cargo Feature:** `default` (or no feature flag)

```bash
cargo build
# or explicitly:
cargo build --features default
```

### Included Features
- ✅ UI Server
- ✅ Console TUI
- ✅ Chat
- ✅ Automation (Rhai scripting)
- ✅ Tasks (with cron scheduling)
- ✅ Drive
- ✅ LLM integration
- ✅ Redis caching
- ✅ Progress bars
- ✅ Directory services

### Not Included
- ❌ Email (IMAP/SMTP)
- ❌ Calendar management
- ❌ Video meetings
- ❌ Vector database
- ❌ Compliance monitoring
- ❌ Multi-channel (WhatsApp, Teams, etc.)
- ❌ NVIDIA GPU support
- ❌ Desktop application

**Typical Deployment:** On-premise server, cloud VM, container

---

## Standard Edition

**Use Case:** Professional teams requiring full productivity features

**Cargo Feature:** `productivity`

```bash
cargo build --features productivity
```

### Included Features
All Core features plus:
- ✅ Email integration (IMAP/SMTP)
- ✅ Calendar management
- ✅ Video meetings (LiveKit)
- ✅ Mail client interface
- ✅ Redis caching

### Additional Dependencies
- `imap` - Email receiving
- `lettre` - Email sending
- `mailparse` - Email parsing
- `livekit` - Video conferencing

**Typical Deployment:** Business office, remote teams

---

## Enterprise Edition

**Use Case:** Large organizations with compliance and integration requirements

**Cargo Feature:** `enterprise`

```bash
cargo build --features enterprise
```

### Included Features
All Standard features plus:
- ✅ Compliance monitoring (LGPD/GDPR/HIPAA/SOC2)
- ✅ Attendance tracking
- ✅ Directory services (LDAP/AD compatible)
- ✅ Vector database (Qdrant)
- ✅ Advanced monitoring (sysinfo)
- ✅ LLM integration

### Compliance Features
| Framework | Status | Implementation |
|-----------|--------|----------------|
| LGPD | ✅ | Data subject rights dialogs |
| GDPR | ✅ | Consent management, data portability |
| HIPAA | ✅ | PHI handling, audit trails |
| SOC 2 | ✅ | Access controls, logging |
| ISO 27001 | ✅ | Asset management, risk assessment |
| PCI DSS | ✅ | Payment data protection |

**Typical Deployment:** Enterprise data center, regulated industries

---

## Communications Edition

**Use Case:** Organizations needing multi-channel customer engagement

**Cargo Feature:** `communications`

```bash
cargo build --features communications
```

### Included Features
- ✅ Email (IMAP/SMTP)
- ✅ WhatsApp Business
- ✅ Instagram messaging
- ✅ Microsoft Teams
- ✅ Chat
- ✅ Redis caching

### Channel Support
| Channel | Protocol | Status |
|---------|----------|--------|
| WhatsApp | Cloud API | ✅ |
| Instagram | Graph API | ✅ |
| MS Teams | Bot Framework | ✅ |
| Telegram | Bot API | Planned |
| Slack | Web API | Planned |
| SMS | Twilio | Planned |

**Typical Deployment:** Customer service center, marketing teams

---

## Full Edition

**Use Case:** Maximum capability, all features enabled

**Cargo Feature:** `full`

```bash
cargo build --features full
```

### All Features Enabled
- ✅ UI Server + Desktop application
- ✅ Console TUI
- ✅ Vector database (Qdrant)
- ✅ LLM integration
- ✅ NVIDIA GPU acceleration
- ✅ All communication channels
- ✅ Full productivity suite
- ✅ Compliance & attendance
- ✅ Directory services
- ✅ Web automation
- ✅ Redis caching
- ✅ System monitoring
- ✅ Automation (Rhai)
- ✅ gRPC support
- ✅ Progress bars

### Hardware Recommendations
| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 32 GB |
| Storage | 100 GB SSD | 500 GB NVMe |
| GPU | Optional | NVIDIA RTX 3060+ |
| Network | 100 Mbps | 1 Gbps |

**Typical Deployment:** Enterprise AI platform, research environments

---

## Feature Matrix

| Feature | Minimal | Light | Core | Standard | Enterprise | Full |
|---------|---------|-------|------|----------|------------|------|
| UI Server | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Chat | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Console TUI | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Drive | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tasks | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Automation | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| LLM | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Email | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Calendar | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Meet | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |
| Vector DB | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Compliance | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Multi-channel | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Desktop | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| GPU | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

---

## Custom Feature Combinations

You can combine individual features for custom builds:

```bash
# Chat + Email + Vector search
cargo build --features "chat,email,vectordb"

# Productivity + Compliance
cargo build --features "productivity,compliance"

# Everything except desktop
cargo build --features "full" --no-default-features
```

### Available Feature Flags

```toml
[features]
# UI Features
desktop = ["dep:tauri", ...]
ui-server = []
console = ["dep:crossterm", "dep:ratatui", "monitoring"]

# Core Integrations
vectordb = ["dep:qdrant-client"]
llm = []
nvidia = []

# Communication Channels
email = ["dep:imap", "dep:lettre", ...]
whatsapp = []
instagram = []
msteams = []

# Productivity Features
chat = []
drive = ["dep:aws-config", "dep:aws-sdk-s3", ...]
tasks = ["dep:cron"]
calendar = []
meet = ["dep:livekit"]
mail = ["email"]

# Enterprise Features
compliance = ["dep:csv"]
attendance = []
directory = []
weba = []

# Infrastructure
redis-cache = ["dep:redis"]
monitoring = ["dep:sysinfo"]
automation = ["dep:rhai"]
grpc = ["dep:tonic"]
progress-bars = ["dep:indicatif"]
```

---

## Deployment Recommendations

### By Organization Size

| Size | Employees | Recommended Edition |
|------|-----------|---------------------|
| Solo | 1 | Lightweight |
| Startup | 2-10 | Core |
| SMB | 11-50 | Standard |
| Mid-market | 51-200 | Enterprise |
| Enterprise | 200+ | Full |

### By Industry

| Industry | Recommended Edition | Key Features |
|----------|---------------------|--------------|
| Healthcare | Enterprise | HIPAA compliance |
| Finance | Enterprise | SOC 2, PCI DSS |
| Education | Standard | Calendar, Meet |
| Retail | Communications | Multi-channel |
| Legal | Enterprise | Document management, compliance |
| Manufacturing | Core | Automation, tasks |
| Tech/SaaS | Full | All capabilities |

---

## Upgrading Editions

Editions can be changed by rebuilding with different feature flags:

```bash
# From Core to Enterprise
cargo build --release --features enterprise

# From Standard to Full
cargo build --release --features full
```

**Note:** Some features may require additional infrastructure components:
- `vectordb` → Requires Qdrant service
- `meet` → Requires LiveKit server
- `redis-cache` → Requires Redis/Valkey
- `nvidia` → Requires NVIDIA GPU + CUDA

---

## See Also

- [Cargo.toml Feature Definitions](../chapter-07-gbapp/dependencies.md)
- [Installation Guide](../chapter-01/installation.md)
- [Architecture Overview](../chapter-07-gbapp/architecture.md)
- [Compliance Requirements](../chapter-12-auth/compliance-requirements.md)