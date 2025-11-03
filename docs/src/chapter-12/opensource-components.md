# Open Source Components in GeneralBots Installer

This article lists all open-source components integrated into the GeneralBots system through the `PackageManager` installer.  
Each component is registered automatically and downloaded from verified open-source repositories.

---

## Core Infrastructure

### PostgreSQL (Tables)
- **Source:** [theseus-rs/postgresql-binaries](https://github.com/theseus-rs/postgresql-binaries)
- **Purpose:** Provides relational database storage for bot data and user sessions.
- **License:** PostgreSQL License (Open Source)

### Valkey (Cache)
- **Source:** [valkey.io](https://valkey.io)
- **Purpose:** In-memory caching system compatible with Redis.
- **License:** BSD 3-Clause

### MinIO (Drive)
- **Source:** [min.io](https://min.io)
- **Purpose:** Object storage compatible with Amazon S3.
- **License:** AGPLv3

### Qdrant (Vector Database)
- **Source:** [qdrant/qdrant](https://github.com/qdrant/qdrant)
- **Purpose:** Vector similarity search engine for embeddings and AI indexing.
- **License:** Apache 2.0

---

## AI and LLM Components

### LLaMA.cpp (LLM Server)
- **Source:** [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp)
- **Purpose:** Runs local LLM inference for chat and embedding models.
- **License:** MIT

### DeepSeek & BGE Models
- **Source:** [HuggingFace](https://huggingface.co)
- **Purpose:** Provides open models for reasoning and embeddings.
- **License:** Apache 2.0 / MIT (depending on model)

---

## Communication and Networking

### Stalwart Mail Server
- **Source:** [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart)
- **Purpose:** Full-featured mail server supporting SMTP, IMAP, and POP3.
- **License:** AGPLv3

### Caddy (Proxy)
- **Source:** [caddyserver/caddy](https://github.com/caddyserver/caddy)
- **Purpose:** Reverse proxy and web server with automatic HTTPS.
- **License:** Apache 2.0

### CoreDNS (DNS)
- **Source:** [coredns/coredns](https://github.com/coredns/coredns)
- **Purpose:** DNS server for internal and external name resolution.
- **License:** Apache 2.0

---

## Identity and Collaboration

### Zitadel (Directory)
- **Source:** [zitadel/zitadel](https://github.com/zitadel/zitadel)
- **Purpose:** Identity and access management system.
- **License:** Apache 2.0

### Forgejo (ALM)
- **Source:** [codeberg.org/forgejo/forgejo](https://codeberg.org/forgejo/forgejo)
- **Purpose:** Git-based project management and CI/CD platform.
- **License:** AGPLv3

### Forgejo Runner (ALM-CI)
- **Source:** [forgejo/runner](https://code.forgejo.org/forgejo/runner)
- **Purpose:** Continuous integration runner for Forgejo.
- **License:** AGPLv3

---

## Productivity Tools

### Roundcube (Webmail)
- **Source:** [roundcube/roundcubemail](https://github.com/roundcube/roundcubemail)
- **Purpose:** Web-based email client.
- **License:** GPLv3

### LiveKit (Meeting)
- **Source:** [livekit/livekit](https://github.com/livekit/livekit)
- **Purpose:** Real-time video conferencing and media server.
- **License:** Apache 2.0

### NocoDB (Table Editor)
- **Source:** [nocodb/nocodb](https://github.com/nocodb/nocodb)
- **Purpose:** Open-source Airtable alternative for database visualization.
- **License:** GPLv3

### LibreOffice Online (Doc Editor)
- **Source:** [Collabora Online](https://github.com/CollaboraOnline/online)
- **Purpose:** Collaborative document editing via `coolwsd`.
- **License:** MPL 2.0

---

## System and Development Utilities

### XFCE + XRDP (Desktop)
- **Source:** [xfce.org](https://xfce.org), [xrdp.org](https://xrdp.org)
- **Purpose:** Lightweight remote desktop environment.
- **License:** GPLv2

### DevTools
- **Includes:** Git, Curl, Xclip
- **Purpose:** Developer utilities for automation and scripting.
- **License:** GPL / MIT / BSD

### Host (LXD)
- **Source:** [linuxcontainers/lxd](https://github.com/lxc/lxd)
- **Purpose:** Container and virtualization management.
- **License:** Apache 2.0

---

## Summary

All components integrated into GeneralBots are open-source, ensuring transparency, security, and extensibility.  
They form a cohesive ecosystem supporting AI, automation, storage, and collaboration.
