# BotServer Crates Reference

Quick reference for all crates in `botserver/crates/`. For full rules, see root `AGENTS.md`.

## Core Infrastructure

| Crate | Description |
|-------|-------------|
| `botcore` | Core shared types, DB pool, AppState |
| `botcorebot` | Bot CRUD, models, schema, error types |
| `botcoredirectory` | Identity provider (Zitadel) integration, SCIM, user/org provisioning |
| `botcoreoauth` | OAuth2 provider abstraction (Google, GitHub, etc.) |
| `botcorepkg` | Package manager — plugin install, container orchestration, ALM setup |
| `botcoresecrets` | Secrets manager — Vault integration, service configs, tenant secrets |
| `botcoresession` | Session management, anonymous sessions, session migration |

## Security

| Crate | Description |
|-------|-------------|
| `botsecurity` | Security facade crate (re-exports sub-crates) |
| `botsecurity-auth` | Authentication, authorization, JWT, session tokens |
| `botsecurity-core` | Guards, sanitizers, validators (SQL, command, error) |
| `botsecurity-crypto` | TLS, CA, encryption, certificate management |
| `botsecurity-protection` | Firewall, IDS, hardening automation |

## BASIC Language

| Crate | Description |
|-------|-------------|
| `botbasic_types` | Shared types and runtime trait for BASIC keyword crates |
| `botbasic_core` | Core keywords — arrays, math, strings, control flow, procedures, errors |
| `botbasic_compiler` | Compiler — syntax transforms, goto logic, save conversion, tool parsing |
| `botbasic_system` | System keywords — bot management, file ops, scheduling, security, app server |
| `botbasic_ai` | AI keywords — LLM, AI tools, MCP, sandbox, orchestration, web scraping |
| `botbasic_data` | Data keywords — database, tables, memory, KB, CRM, products |
| `botbasic_comms` | Communication keywords — messaging, email, SMS, social, chat, webhooks |

## Channels & Communication

| Crate | Description |
|-------|-------------|
| `botchannels` | Social media channel integrations |
| `botchannels-core` | Channel core: OAuth and media upload |
| `botchannelbindings` | Per-bot default phone, WhatsApp and Telegram identities + call logs |
| `botwhatsapp` | WhatsApp Business API integration |
| `bottelegram` | Telegram bot integration |
| `botinstagram` | Instagram integration (adapter, campaign, webhook) |
| `botmsteams` | Microsoft Teams integration |
| `botemail` | Email triage, unified inbox, threading, search, draft handling |
| `botsocial` | Social media management |
| `bothr` | Human resources module |
| `bothandoff` | Handoff/escalation management |

## LLM & AI

| Crate | Description |
|-------|-------------|
| `botllm` | LLM provider implementations and core types |
| `botmultimodal` | Multimodal AI client (image, video, audio, speech) |
| `botqdrant` | Qdrant vector database client (embeddings, search) |
| `botmodelsbridge` | Face/visual AI bridge — Azure, AWS Rekognition, OpenCV, InsightFace |
| `botvision` | Computer vision and face recognition models |
| `botnvidia` | NVIDIA GPU monitoring |
| `botresearch` | Web search, knowledge base exploration, deep research |
| `boteval` | LLM response evaluation framework (datasets, contracts, CI gate) |

## Business Modules

| Crate | Description |
|-------|-------------|
| `botpeople` | People management — employees, attendance, payroll, time clock |
| `botcontacts` | Contact/lead management |
| `botcrm` | *(via botpeople)* CRM contacts, tickets, leads |
| `botproducts` | Products, services, inventory, and pricing management |
| `botbilling` | Billing, invoicing, quotas, and subscription management |
| `bottickets` | Ticketing system, ITSM integration |
| `botattendant` | Contact center attendant queue, session, and agent management |
| `botattendance` | Attendance module — queue, SLA, webhooks, LLM assist |
| `bottimeclock` | Electronic time clock |
| `botcalendar` | Calendar with conflict resolution |
| `botplan` | Project planner (Gantt + Kanban) with real-time collaboration |
| `bottasks` | Task scheduling and auto-task engine |
| `botautomation` | NL scheduled adaptive agents, planner-executor-verifier engine |
| `botautotask` | Auto-task intent classification, execution, safety layer |
| `botlearn` | Learning Management System (LMS) |

## Commerce & Finance

| Crate | Description |
|-------|-------------|
| `boterp` | ERP module |
| `botgl` | General ledger — types, handlers, reports |
| `botbanking` | Banking reconciliation and delivery-platform integration |
| `botbrazil` | Brazilian tax compliance (NFe, NFSe, CTe, MDFe, SPED, EFD, ICMS) |
| `bottax` | Tax calculator |
| `botretail` | Retail management — products, stock, POS, NFCe |
| `botpos` | Point of Sale |
| `botinventory` | Inventory management |
| `botsales` | Sales — leads, quotes, pipeline |
| `botmarketing` | Marketing campaigns, email, WhatsApp, IP routing, warmup |
| `botfraud` | Fraud detection engine — rules, scoring, actions |
| `botcompliance` | Compliance — access review, audit, backup verification, code scanner |
| `botconsent` | Per-app agent consent system — grants, prompts, audit |

## Data & Documents

| Crate | Description |
|-------|-------------|
| `botdatabase` | Database query routes and storage |
| `botdrive` | Drive file repository, S3/MinIO, vectordb, streaming |
| `botdocs` | Document processing, collaboration, conversion |
| `botsheet` | Spreadsheet engine (collaboration, formulas, PDF export) |
| `botsheet-core` | Sheet core — types, formulas, state |
| `botslides` | Presentation engine (OOXML/PPTX, collaboration, UI) |
| `botpaper` | Document/paper management with LLM integration |
| `botkb` | Face recognition and computer vision models |
| `botsearch` | Full-text search service |
| `botsources` | Knowledge connectors — permissioned indexing, redaction, search over external sources |
| `botconnectors` | Enterprise connectors — chat, mail, drive sources |

## UI & Frontend

| Crate | Description |
|-------|-------------|
| `botapi` | API routes + terminal interface |
| `botuifragments` | HTMX UI fragment routes for bot apps |
| `botdesigner` | Visual designer — BAS analyzer, canvas API, workflow canvas |
| `botdesktop` | Desktop session proxy — WebSocket-to-TCP relay |
| `botdashboards` | Dashboard CRUD, storage, rendering |
| `botcanvas` | Canvas board (Axum routes, Diesel models) |
| `botweba` | Web application builder |
| `botslides` | Presentation builder (UI + OOXML) |

## Cloud & SaaS

| Crate | Description |
|-------|-------------|
| `botcloud` | SaaS — public checkout, subscriber dashboard, signup, cloud API |
| `botworkspaces` | Workspace management (CRUD, events, real-time broadcast) |
| `botmarketplace` | Skills Marketplace — catalog, publish/install into bot Drive |
| `bottemplates` | Bot template management |

## DevOps & Infrastructure

| Crate | Description |
|-------|-------------|
| `botdeployment` | Deployment infrastructure for VibeCode platform |
| `botvibe` | VibeCode — agent loop, Incus VMs, projects, publish, domains |
| `botgit` | Git integration routes |
| `botbrowser` | Browser automation |
| `botbrowserpolicy` | Agentic browsing control plane — domain policy, budgets, memory |
| `botmonitoring` | Metrics collection, alerting, distributed tracing |
| `botmaintenance` | System maintenance and cleanup |
| `botproviders` | Cloud provider adapters — RunPod, Vultr, Vast, Contabo |
| `bottimeseries` | Time-series metrics service (InfluxDB-compatible) |

## Specialized

| Crate | Description |
|-------|-------------|
| `botagent` | Always-On Agent Mode — per-chat Incus VMs, snapshots, org API keys |
| `botbiometry` | Biometric identity verification, KYC, digital signature (Zitadel) |
| `botkyc` | KYC module routes |
| `botlegal` | Legal compliance, account deletion |
| `botintegrations` | Integration framework — connections, actions, automations, OAuth |
| `botmemory` | Durable per-user/branch memory — extraction, recall, import/export |
| `botplayer` | Media player |
| `botvideo` | Video processing — engine, analytics, MCP tools, WebSocket |
| `botminutes` | Meeting minutes management |
| `botmeet` | LiveKit video conferencing integration |
| `botsampledata` | Idempotent demo data seeding |
| `botsettings` | Settings, RBAC, audit log, OAuth, webhooks, billing settings |
| `botcontacts` | Contact CRUD and change triggers |

## Count

**100+ crates** total. The root `AGENTS.md` covers universal rules (no `unwrap`, no `panic!`, deploy workflow, security directives).
