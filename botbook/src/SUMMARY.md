# Summary

[Executive Vision](./executive-vision.md)
[Roadmap 2024-2026](./ROADMAP.md)
[Introduction](./introduction.md)

---

# Part I: Getting Started

- [Chapter 1: Getting Started](./01-getting-started/README.md)
   - [Overview](./01-getting-started/overview.md)
   - [Quick Start](./01-getting-started/quick-start.md)
   - [Installation](./01-getting-started/installation.md)
   - [Configuring .local Domains](./01-getting-started/local-domains.md)
   - [First Conversation](./01-getting-started/first-conversation.md)
   - [Sessions and Channels](./01-getting-started/sessions.md)

---

# Part II: Core Architecture

- [Chapter 2: Architecture & Packages](./02-architecture-packages/README.md)
   - [Architecture Overview](./02-architecture-packages/architecture.md)
   - [Module Structure](./02-architecture-packages/crates.md)
   - [Service Layer](./02-architecture-packages/services.md)
   - [Building from Source](./02-architecture-packages/building.md)
   - [Cargo Tools Reference](./02-architecture-packages/cargo-tools.md)
   - [Container Deployment (LXC)](./02-architecture-packages/containers.md)
   - [Docker Deployment](./02-architecture-packages/docker-deployment.md)
   - [Kubernetes Deployment](./02-architecture-packages/kubernetes-deployment.md)
   - [Scaling and Load Balancing](./02-architecture-packages/scaling.md)
   - [Infrastructure Design](./02-architecture-packages/infrastructure.md)
   - [Observability](./02-architecture-packages/observability.md)
   - [Monitoring Setup](./02-architecture-packages/monitoring-setup.md)
   - [Autonomous Task AI](./02-architecture-packages/autonomous-tasks.md)
   - [Philosophy](./02-architecture-packages/philosophy.md)
   - [.gbai Package Format](./02-architecture-packages/gbai.md)
   - [.gbdialog Dialogs](./02-architecture-packages/gbdialog.md)
   - [.gbkb Knowledge Base](./02-architecture-packages/gbkb.md)
   - [.gbot Bot Configuration](./02-architecture-packages/gbot.md)
   - [.gbtheme UI Theming](./02-architecture-packages/gbtheme.md)
   - [.gbdrive File Storage](./02-architecture-packages/gbdrive.md)
   - [Templates Overview](./02-architecture-packages/templates.md)
   - [Template: BI](./02-architecture-packages/template-bi.md)
   - [Template: Web Crawler](./02-architecture-packages/template-crawler.md)
   - [Template: CRM](./02-architecture-packages/template-crm.md)
   - [Template: Marketing](./02-architecture-packages/template-marketing.md)

---

# Part III: Intelligence

- [Chapter 3: Knowledge & AI](./03-knowledge-ai/README.md)
   - [KB and Tools System](./03-knowledge-ai/kb-and-tools.md)
   - [Vector Collections](./03-knowledge-ai/vector-collections.md)
   - [Document Indexing](./03-knowledge-ai/indexing.md)
   - [Semantic Search](./03-knowledge-ai/semantic-search.md)
   - [Episodic Memory](./03-knowledge-ai/episodic-memory.md)
   - [Semantic Caching](./03-knowledge-ai/caching.md)
   - [AI and LLM Integration](./03-knowledge-ai/ai-llm.md)
   - [Hybrid RAG Search](./03-knowledge-ai/hybrid-search.md)
   - [Memory Management](./03-knowledge-ai/memory-management.md)
   - [Conversation Management](./03-knowledge-ai/conversation.md)
   - [Automation](./03-knowledge-ai/automation.md)
   - [Email Integration](./03-knowledge-ai/email.md)
   - [Transfer to Human](./03-knowledge-ai/transfer-to-human.md)
   - [LLM-Assisted Attendant](./03-knowledge-ai/attendant-llm-assist.md)

---

# Part IV: Programming

- [Chapter 4: BASIC Scripting](./04-basic-scripting/README.md)
   - [BASIC Basics](./04-basic-scripting/basics.md)
   - [Execution Modes: RUNTIME vs WORKFLOW](./04-basic-scripting/execution-modes.md)
   - [API Possibilities](./04-basic-scripting/api-possibilities.md)
   - [Universal Messaging](./04-basic-scripting/universal-messaging.md)
   - [BASIC vs n8n/Zapier/Make](./04-basic-scripting/basic-vs-automation-tools.md)
   - [Template Variables](./04-basic-scripting/template-variables.md)
   - [TALK](./04-basic-scripting/keyword-talk.md)
   - [HEAR](./04-basic-scripting/keyword-hear.md)
   - [SET CONTEXT](./04-basic-scripting/keyword-set-context.md)
   - [GET BOT MEMORY](./04-basic-scripting/keyword-get-bot-memory.md)
   - [SET BOT MEMORY](./04-basic-scripting/keyword-set-bot-memory.md)
   - [GET USER MEMORY](./04-basic-scripting/keyword-get-user-memory.md)
   - [SET USER MEMORY](./04-basic-scripting/keyword-set-user-memory.md)
   - [REMEMBER / RECALL](./04-basic-scripting/keyword-remember.md)
   - [BOOK / BOOK_MEETING](./04-basic-scripting/keyword-book.md)
   - [WEATHER / FORECAST](./04-basic-scripting/keyword-weather.md)
   - [ADD BOT](./04-basic-scripting/keyword-add-bot.md)
   - [USE MODEL](./04-basic-scripting/keyword-use-model.md)
   - [DELEGATE TO BOT](./04-basic-scripting/keyword-delegate-to-bot.md)
   - [RUN CODE](./04-basic-scripting/keyword-run-code.md)
   - [USE KB](./04-basic-scripting/keyword-use-kb.md)
   - [THINK KB](./04-basic-scripting/keyword-think-kb.md)
   - [GET](./04-basic-scripting/keyword-get.md)
   - [SET](./04-basic-scripting/keyword-set.md)
   - [ON](./04-basic-scripting/keyword-on.md)
   - [SET SCHEDULE](./04-basic-scripting/keyword-set-schedule.md)
   - [CREATE TASK](./04-basic-scripting/keyword-create-task.md)
   - [FOR EACH](./04-basic-scripting/keyword-for-each.md)
   - [SWITCH](./04-basic-scripting/keyword-switch.md)
   - [SAVE](./04-basic-scripting/keyword-save.md)
   - [INSERT](./04-basic-scripting/keyword-insert.md)
   - [UPDATE](./04-basic-scripting/keyword-update.md)
   - [DELETE](./04-basic-scripting/keyword-delete.md)
   - [FIND](./04-basic-scripting/keyword-find.md)
   - [FILTER](./04-basic-scripting/keyword-filter.md)
   - [MAP](./04-basic-scripting/keyword-map.md)
   - [AGGREGATE](./04-basic-scripting/keyword-aggregate.md)
   - [POST](./04-basic-scripting/keyword-post.md)
   - [GRAPHQL](./04-basic-scripting/keyword-graphql.md)
   - [WEBHOOK](./04-basic-scripting/keyword-webhook.md)
   - [PLAY](./04-basic-scripting/keyword-play.md)
   - [SEND MAIL](./04-basic-scripting/keyword-send-mail.md)
   - [SEND SMS](./04-basic-scripting/keyword-sms.md)
   - [READ](./04-basic-scripting/keyword-read.md)
   - [WRITE](./04-basic-scripting/keyword-write.md)
   - [UPLOAD](./04-basic-scripting/keyword-upload.md)
   - [DOWNLOAD](./04-basic-scripting/keyword-download.md)
   - [GENERATE PDF](./04-basic-scripting/keyword-generate-pdf.md)
   - [start.bas](./04-basic-scripting/templates/start.md)
   - [default.bas](./04-basic-scripting/templates/default.md)
   - [auth.bas](./04-basic-scripting/templates/auth.md)
   - [enrollment.bas](./04-basic-scripting/templates/enrollment.md)
   - [sales-pipeline.bas](./04-basic-scripting/templates/sales-pipeline.md)

---

# Part V: Orchestration

- [Chapter 5: Multi-Agent Orchestration](./05-multi-agent/README.md)
   - [Task Workflow](./05-multi-agent/workflow.md)
   - [App Generation](./05-multi-agent/app-generation.md)
   - [Data Model](./05-multi-agent/data-model.md)
   - [Designer](./05-multi-agent/designer.md)
   - [Agent Workspaces](./05-multi-agent/agent-workspaces.md)

---

# Part VI: Connectivity

- [Chapter 6: Channels & Connectivity](./06-channels/README.md)
   - [Channel Integrations](./06-channels/channels.md)
   - [Service Catalog](./06-channels/catalog.md)
   - [WhatsApp Quick Start](./06-channels/whatsapp-quick-start.md)
   - [WhatsApp Webhooks](./06-channels/whatsapp-webhooks.md)
   - [Teams Channel](./06-channels/teams-channel.md)
   - [SMS Providers](./06-channels/sms-providers.md)
   - [Attendance Queue](./06-channels/attendance-queue.md)
   - [LLM Providers](./06-channels/llm-providers.md)
   - [Storage Services](./06-channels/storage.md)
   - [Directory Services](./06-channels/directory.md)

---

# Part VII: User Interface

- [Chapter 7: User Interface](./07-user-interface/README.md)
   - [Suite User Manual](./07-user-interface/suite-manual.md)
   - [Admin vs User Views](./07-user-interface/admin-user-views.md)
   - [UI Structure](./07-user-interface/ui-structure.md)
   - [single.gbui - Simple Chat](./07-user-interface/single-gbui.md)
   - [Console Mode](./07-user-interface/console-mode.md)
   - [Monitoring Dashboard](./07-user-interface/monitoring.md)
   - [HTMX Architecture](./07-user-interface/htmx-architecture.md)
   - [Suite - Full Desktop](./07-user-interface/apps/suite.md)
   - [Vibe - AI Dev Environment](./07-user-interface/apps/vibe.md)
   - [Chat - AI Assistant](./07-user-interface/apps/chat.md)
   - [Drive - File Management](./07-user-interface/apps/drive.md)
   - [Tasks](./07-user-interface/apps/tasks.md)
   - [Mail - Email Client](./07-user-interface/apps/mail.md)
   - [Calendar - Scheduling](./07-user-interface/apps/calendar.md)
   - [Meet - Video Calls](./07-user-interface/apps/meet.md)
   - [CRM - Sales Pipeline](./07-user-interface/apps/crm.md)
   - [Billing - Invoices](./07-user-interface/apps/billing.md)
   - [Tickets - Support Cases](./07-user-interface/apps/tickets.md)
   - [Analytics - Dashboards](./07-user-interface/apps/analytics.md)
   - [Designer - Visual Builder](./07-user-interface/apps/designer.md)
   - [Create Your First Bot](./07-user-interface/how-to/create-first-bot.md)
   - [Write Your First Dialog](./07-user-interface/how-to/write-first-dialog.md)
   - [Add Documents to Knowledge Base](./07-user-interface/how-to/add-kb-documents.md)
   - [Connect WhatsApp](./07-user-interface/how-to/connect-whatsapp.md)
   - [Theme Structure](./07-user-interface/structure.md)
   - [CSS Customization](./07-user-interface/css.md)

---

# Part VIII: Integration

- [Chapter 8: REST API & Tools](./08-rest-api-tools/README.md)
   - [Files API](./08-rest-api-tools/files-api.md)
   - [Users API](./08-rest-api-tools/users-api.md)
   - [Groups API](./08-rest-api-tools/groups-api.md)
   - [Conversations API](./08-rest-api-tools/conversations-api.md)
   - [Email API](./08-rest-api-tools/email-api.md)
   - [Calendar API](./08-rest-api-tools/calendar-api.md)
   - [Tasks API](./08-rest-api-tools/tasks-api.md)
   - [Storage API](./08-rest-api-tools/storage-api.md)
   - [Analytics API](./08-rest-api-tools/analytics-api.md)
   - [Admin API](./08-rest-api-tools/admin-api.md)
   - [AI API](./08-rest-api-tools/ai-api.md)
   - [Tool Definition](./08-rest-api-tools/tool-definition.md)
   - [PARAM Declaration](./08-rest-api-tools/param-declaration.md)
   - [Tool Compilation](./08-rest-api-tools/compilation.md)
   - [MCP Format](./08-rest-api-tools/mcp-format.md)
   - [Tool Format](./08-rest-api-tools/openai-format.md)
   - [External APIs](./08-rest-api-tools/external-apis.md)
   - [LLM REST Server](./08-rest-api-tools/llm-rest-server.md)

---

# Part IX: Security

- [Chapter 9: Security](./09-security/README.md)
   - [Initial Setup & Bootstrap](./09-security/initial-setup.md)
   - [User Authentication](./09-security/user-auth.md)
   - [Password Security](./09-security/password-security.md)
   - [API Endpoints](./09-security/api-endpoints.md)
   - [Bot Authentication](./09-security/bot-auth.md)
   - [Security Features](./09-security/security-features.md)
   - [Security Policy](./09-security/security-policy.md)
   - [Compliance Requirements](./09-security/compliance-requirements.md)
   - [RBAC Overview](./09-security/rbac-overview.md)
   - [Permissions Matrix](./09-security/permissions-matrix.md)
   - [RBAC Configuration Guide](./09-security/rbac-configuration.md)
   - [Organization Multi-Tenancy](./09-security/organizations.md)
   - [Knowledge Base Permissions](./09-security/kb-permissions.md)
   - [SOC 2 Compliance](./09-security/soc2-compliance.md)
   - [Security Matrix Reference](./09-security/security-matrix.md)
   - [Endpoint Security Checklist](./09-security/endpoint-checklist.md)

---

# Part X: Deployment

- [Chapter 10: Configuration & Deployment](./10-configuration-deployment/README.md)
   - [config.csv Format](./10-configuration-deployment/config-csv.md)
   - [Bot Parameters](./10-configuration-deployment/parameters.md)
   - [LLM Configuration](./10-configuration-deployment/llm-config.md)
   - [Context Configuration](./10-configuration-deployment/context-config.md)
   - [Drive Integration](./10-configuration-deployment/drive.md)
   - [Multimodal Configuration](./10-configuration-deployment/multimodal.md)
   - [Secrets Management](./10-configuration-deployment/secrets-management.md)
   - [System Limits](./10-configuration-deployment/system-limits.md)
   - [MinIO Storage](./10-configuration-deployment/minio.md)

---

# Part XI: Hardware & Scale

- [Chapter 11: Hardware & Scaling](./11-hardware-scaling/README.md)
   - [Buying Guide for Beginners](./11-hardware-scaling/buying-guide.md)
   - [Mobile (Android & HarmonyOS)](./11-hardware-scaling/mobile.md)
   - [Supported Hardware (SBCs)](./11-hardware-scaling/hardware.md)
   - [Desktop & Server Hardware](./11-hardware-scaling/desktop-hardware.md)
   - [Local LLM with llama.cpp](./11-hardware-scaling/local-llm.md)
   - [Sharding Architecture](./11-hardware-scaling/sharding.md)
   - [Database Optimization](./11-hardware-scaling/database-optimization.md)

---

# Part XII: Ecosystem

- [Chapter 12: Ecosystem & Reference](./12-ecosystem-reference/README.md)
   - [Migration Overview](./12-ecosystem-reference/overview.md)
   - [Platform Comparison Matrix](./12-ecosystem-reference/comparison-matrix.md)
   - [Knowledge Base Migration](./12-ecosystem-reference/kb-migration.md)
   - [Cloud Productivity Migration](./12-ecosystem-reference/google-workspace.md)
   - [Enterprise Platform Migration](./12-ecosystem-reference/microsoft-365.md)
   - [n8n Migration](./12-ecosystem-reference/n8n.md)
   - [Notion Migration](./12-ecosystem-reference/notion.md)
   - [Zapier and Make Migration](./12-ecosystem-reference/zapier-make.md)
   - [CLI Reference](./12-ecosystem-reference/cli-reference.md)
   - [Updating Components](./12-ecosystem-reference/updating-components.md)
   - [Component Reference](./12-ecosystem-reference/component-reference.md)
   - [Security Auditing](./12-ecosystem-reference/security-auditing.md)
   - [Backup and Recovery](./12-ecosystem-reference/backup-recovery.md)
   - [Troubleshooting](./12-ecosystem-reference/troubleshooting.md)
   - [Testing Strategy](./12-ecosystem-reference/architecture.md)
   - [End-to-End Testing](./12-ecosystem-reference/e2e-testing.md)
   - [Performance Testing](./12-ecosystem-reference/performance.md)
   - [CI/CD Integration](./12-ecosystem-reference/ci-cd.md)
   - [Contributing Overview](./12-ecosystem-reference/setup.md)
   - [Local Development](./12-ecosystem-reference/local-development.md)
   - [Pull Requests](./12-ecosystem-reference/pull-requests.md)
   - [Community Guidelines](./12-ecosystem-reference/community.md)
   - [Schema Overview](./12-ecosystem-reference/schema.md)
   - [Tables](./12-ecosystem-reference/tables.md)
   - [Relationships](./12-ecosystem-reference/relationships.md)
   - [Glossary](./glossary.md)
   - [Contact](./contact/README.md)
   - [Features](./features.md)
   - [Attendance](./attendance.md)
