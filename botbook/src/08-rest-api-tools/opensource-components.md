# Open Source Components in GeneralBots Installer

This article lists all open-source components integrated into the GeneralBots system through the `PackageManager` installer. Each component is registered automatically and downloaded from verified open-source repositories, ensuring transparency, security, and extensibility throughout the platform.

---

## Core Infrastructure

The foundation of GeneralBots relies on several battle-tested open-source infrastructure components that handle data persistence, caching, and search capabilities.

### PostgreSQL (Tables)

PostgreSQL provides the relational database storage layer for bot data and user sessions. The system uses binaries from [theseus-rs/postgresql-binaries](https://github.com/theseus-rs/postgresql-binaries) and operates under the PostgreSQL License, which is fully open source.

### Valkey (Cache)

Valkey serves as the in-memory caching system, providing Redis-compatible functionality for high-performance data access. Available from [valkey.io](https://valkey.io), it operates under the BSD 3-Clause license, making it suitable for both commercial and open-source deployments.

### Drive (S3-Compatible Storage)

For file management and object storage, GeneralBots integrates MinIO from [min.io](https://min.io). This S3-compatible storage solution enables seamless file operations and is licensed under AGPLv3.

### Qdrant (Vector Database)

The vector similarity search engine Qdrant, available from [qdrant/qdrant](https://github.com/qdrant/qdrant), powers embeddings and AI indexing capabilities. This Apache 2.0 licensed component enables semantic search and AI-driven document retrieval.

---

## AI and LLM Components

GeneralBots incorporates cutting-edge AI components that enable local inference and intelligent processing without requiring external API dependencies.

### LLaMA.cpp (LLM Server)

Local LLM inference for both chat and embedding models is handled by LLaMA.cpp from [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp). This MIT-licensed component enables the system to run language models directly on the host machine, providing privacy and reduced latency.

### DeepSeek & BGE Models

Open models for reasoning and embeddings are sourced from [HuggingFace](https://huggingface.co). These models provide state-of-the-art capabilities for natural language understanding and are available under Apache 2.0 or MIT licenses depending on the specific model selected.

---

## Communication and Networking

The platform includes a comprehensive suite of networking components that handle email, web traffic, and DNS resolution.

### Stalwart Mail Server

For email functionality, GeneralBots integrates the Stalwart mail server from [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart). This full-featured mail server supports SMTP, IMAP, and POP3 protocols, operating under the AGPLv3 license.

### Caddy (Proxy)

Caddy from [caddyserver/caddy](https://github.com/caddyserver/caddy) serves as the reverse proxy and web server, providing automatic HTTPS certificate management. This Apache 2.0 licensed component simplifies secure web deployment.

### CoreDNS (DNS)

Internal and external name resolution is handled by CoreDNS from [coredns/coredns](https://github.com/coredns/coredns). This flexible DNS server operates under the Apache 2.0 license and integrates seamlessly with the rest of the infrastructure.

---

## Identity and Collaboration

Managing user identity and enabling team collaboration requires robust tooling, which GeneralBots provides through these integrated components.

### Zitadel (Directory)

Identity and access management is powered by Zitadel from [zitadel/zitadel](https://github.com/zitadel/zitadel). This Apache 2.0 licensed system provides comprehensive user management, authentication, and authorization capabilities.

### Forgejo (ALM)

Git-based project management and CI/CD capabilities come from Forgejo, available at [codeberg.org/forgejo/forgejo](https://codeberg.org/forgejo/forgejo). This AGPLv3 licensed platform enables teams to manage code and automate deployments.

### Forgejo Runner (ALM-CI)

Continuous integration pipelines are executed by the Forgejo Runner from [forgejo/runner](https://code.forgejo.org/forgejo/runner). This AGPLv3 licensed component handles build and deployment automation tasks.

---

## Productivity Tools

GeneralBots includes a suite of productivity applications that enable users to communicate, collaborate, and create content.

### Roundcube (Webmail)

Web-based email access is provided by Roundcube from [roundcube/roundcubemail](https://github.com/roundcube/roundcubemail). This GPLv3 licensed client offers a familiar interface for managing email through the browser.

### LiveKit (Meeting)

Real-time video conferencing and media capabilities are powered by LiveKit from [livekit/livekit](https://github.com/livekit/livekit). This Apache 2.0 licensed component enables high-quality video meetings and streaming.

### NocoDB (Table Editor)

For database visualization and management, GeneralBots integrates NocoDB from [nocodb/nocodb](https://github.com/nocodb/nocodb). This GPLv3 licensed tool provides an Airtable-like interface for working with structured data.

### LibreOffice Online (Doc Editor)

Collaborative document editing is enabled through Collabora Online from [CollaboraOnline/online](https://github.com/CollaboraOnline/online). The `coolwsd` service provides browser-based document editing under the MPL 2.0 license.

---

## System and Development Utilities

Supporting the core platform are various system utilities that enable remote access, development workflows, and container management.

### XFCE + XRDP (Desktop)

A lightweight remote desktop environment is provided through XFCE from [xfce.org](https://xfce.org) combined with XRDP from [xrdp.org](https://xrdp.org). These GPLv2 licensed components enable graphical remote access to the system.

### DevTools

Essential developer utilities including Git, Curl, and Xclip are bundled with the platform. These tools, available under GPL, MIT, and BSD licenses respectively, support automation and scripting workflows.

### Host (LXD)

Container and virtualization management is handled by LXD from [linuxcontainers/lxd](https://github.com/lxc/lxd). This Apache 2.0 licensed component enables isolated environments for bot deployment and testing.

---

## Summary

Every component integrated into GeneralBots is fully open-source, ensuring that users have complete transparency into the system's operation. This commitment to open-source software provides security through community review, extensibility through standard interfaces, and freedom from vendor lock-in. Together, these components form a cohesive ecosystem that supports AI automation, secure communication, persistent storage, and seamless collaboration.