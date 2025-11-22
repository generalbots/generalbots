# Summary

[Introduction](./introduction.md)

# Part I - Getting Started

- [Chapter 01: Run and Talk](./chapter-01/README.md)
  - [Overview](./chapter-01/overview.md)
  - [Quick Start](./chapter-01/quick-start.md)
  - [Installation](./chapter-01/installation.md)
  - [First Conversation](./chapter-01/first-conversation.md)
  - [Understanding Sessions](./chapter-01/sessions.md)

# Part II - Package System

- [Chapter 02: About Packages](./chapter-02/README.md)
  - [.gbai Architecture](./chapter-02/gbai.md)
  - [.gbdialog Dialogs](./chapter-02/gbdialog.md)
  - [.gbkb Knowledge Base](./chapter-02/gbkb.md)
  - [.gbot Bot Configuration](./chapter-02/gbot.md)
  - [.gbtheme UI Theming](./chapter-02/gbtheme.md)
  - [.gbdrive File Storage](./chapter-02/gbdrive.md)
  - [Bot Templates](./chapter-02/templates.md)

# Part III - Knowledge Base

- [Chapter 03: gbkb Reference](./chapter-03/README.md)
  - [KB and Tools System](./chapter-03/kb-and-tools.md)
  - [Vector Collections](./chapter-03/vector-collections.md)
  - [Document Indexing](./chapter-03/indexing.md)
  - [Qdrant Integration](./chapter-03/qdrant.md)
  - [Semantic Search](./chapter-03/semantic-search.md)
  - [Context Compaction](./chapter-03/context-compaction.md)
  - [Semantic Caching](./chapter-03/caching.md)
  - [Semantic Cache with Valkey](./chapter-03/semantic-cache.md)

# Part IV - Themes and UI

- [Chapter 04: gbtheme Reference](./chapter-04/README.md)
  - [Theme Structure](./chapter-04/structure.md)
  - [Web Interface](./chapter-04/web-interface.md)
  - [CSS Customization](./chapter-04/css.md)
  - [HTML Templates](./chapter-04/html.md)

# Part V - BASIC Dialogs

- [Chapter 05: gbdialog Reference](./chapter-05/README.md)
  - [Dialog Basics](./chapter-05/basics.md)
  - [Universal Messaging & Multi-Channel](./chapter-05/universal-messaging.md)
  - [Template Examples](./chapter-05/templates.md)
    - [start.bas](./chapter-05/template-start.md)
    - [auth.bas](./chapter-05/template-auth.md)
    - [generate-summary.bas](./chapter-05/template-summary.md)
    - [enrollment Tool Example](./chapter-05/template-enrollment.md)
  - [Keyword Reference](./chapter-05/keywords.md)
    - [TALK](./chapter-05/keyword-talk.md)
    - [HEAR](./chapter-05/keyword-hear.md)
    - [SET_USER](./chapter-05/keyword-set-user.md)
    - [SET_CONTEXT](./chapter-05/keyword-set-context.md)
    - [LLM](./chapter-05/keyword-llm.md)
    - [GET_BOT_MEMORY](./chapter-05/keyword-get-bot-memory.md)
    - [SET_BOT_MEMORY](./chapter-05/keyword-set-bot-memory.md)
    - [USE_KB](./chapter-05/keyword-use-kb.md)
    - [CLEAR_KB](./chapter-05/keyword-clear-kb.md)
    - [ADD_WEBSITE](./chapter-05/keyword-add-website.md)
    - [USE_TOOL](./chapter-05/keyword-use-tool.md)
    - [CLEAR_TOOLS](./chapter-05/keyword-clear-tools.md)
    - [GET](./chapter-05/keyword-get.md)
    - [FIND](./chapter-05/keyword-find.md)
    - [SET](./chapter-05/keyword-set.md)
    - [ON](./chapter-05/keyword-on.md)
    - [SET_SCHEDULE](./chapter-05/keyword-set-schedule.md)
    - [CREATE_SITE](./chapter-05/keyword-create-site.md)
    - [CREATE_DRAFT](./chapter-05/keyword-create-draft.md)
    - [CREATE_TASK](./chapter-05/keyword-create-task.md)
    - [PRINT](./chapter-05/keyword-print.md)
    - [WAIT](./chapter-05/keyword-wait.md)
    - [FORMAT](./chapter-05/keyword-format.md)
    - [FIRST](./chapter-05/keyword-first.md)
    - [LAST](./chapter-05/keyword-last.md)
    - [FOR EACH](./chapter-05/keyword-for-each.md)
    - [EXIT FOR](./chapter-05/keyword-exit-for.md)
    - [ADD_MEMBER](./chapter-05/keyword-add-member.md)
    - [ADD_SUGGESTION](./chapter-05/keyword-add-suggestion.md)
    - [CLEAR_SUGGESTIONS](./chapter-05/keyword-clear-suggestions.md)
    - [BOOK](./chapter-05/keyword-book.md)
    - [REMEMBER](./chapter-05/keyword-remember.md)
    - [SAVE_FROM_UNSTRUCTURED](./chapter-05/keyword-save-from-unstructured.md)
    - [SEND_MAIL](./chapter-05/keyword-send-mail.md)
    - [WEATHER](./chapter-05/keyword-weather.md)

# Part VI - Extending BotServer

- [Chapter 06: Rust Architecture Reference](./chapter-06/README.md)
  - [Architecture Overview](./chapter-06/architecture.md)
  - [Building from Source](./chapter-06/building.md)
  - [Container Deployment (LXC)](./chapter-06/containers.md)
  - [SMB Deployment Guide](./chapter-06/smb-deployment.md)
  - [Module Structure](./chapter-06/crates.md)
  - [Service Layer](./chapter-06/services.md)
  - [Creating Custom Keywords](./chapter-06/custom-keywords.md)
  - [Adding Dependencies](./chapter-06/dependencies.md)
  - [Prompt Manager](./chapter-06/prompt-manager.md)

# Part VII - Bot Configuration

- [Chapter 07: gbot Reference](./chapter-07/README.md)
  - [config.csv Format](./chapter-07/config-csv.md)
  - [Bot Parameters](./chapter-07/parameters.md)
  - [Answer Modes](./chapter-07/answer-modes.md)
  - [LLM Configuration](./chapter-07/llm-config.md)
  - [Context Configuration](./chapter-07/context-config.md)
  - [MinIO Drive Integration](./chapter-07/minio.md)

# Part VIII - Tools and Integration

- [Chapter 08: Tooling](./chapter-08/README.md)
  - [Tool Definition](./chapter-08/tool-definition.md)
  - [PARAM Declaration](./chapter-08/param-declaration.md)
  - [Tool Compilation](./chapter-08/compilation.md)
  - [MCP Format](./chapter-08/mcp-format.md)
  - [OpenAI Tool Format](./chapter-08/openai-format.md)
  - [GET Keyword Integration](./chapter-08/get-integration.md)
  - [External APIs](./chapter-08/external-apis.md)

# Part IX - Feature Reference

- [Chapter 09: Feature Matrix](./chapter-09/README.md)
  - [Core Features](./chapter-09/core-features.md)
  - [Conversation Management](./chapter-09/conversation.md)
  - [AI and LLM](./chapter-09/ai-llm.md)
  - [Knowledge Base](./chapter-09/knowledge-base.md)
  - [Automation](./chapter-09/automation.md)
  - [Email Integration](./chapter-09/email.md)
  - [Web Automation](./chapter-09/web-automation.md)
  - [Storage and Data](./chapter-09/storage.md)
  - [Multi-Channel Support](./chapter-09/channels.md)

# Part X - Community

- [Chapter 10: Contributing](./chapter-10/README.md)
  - [Development Setup](./chapter-10/setup.md)
  - [Contributing Guidelines](./chapter-10/contributing-guidelines.md)
  - [Code of Conduct](./chapter-10/code-of-conduct.md)
  - [Código de Conduta (Português)](./chapter-10/code-of-conduct-pt-br.md)
  - [Code Standards](./chapter-10/standards.md)
  - [Testing](./chapter-10/testing.md)
  - [Pull Requests](./chapter-10/pull-requests.md)
  - [Documentation](./chapter-10/documentation.md)

# Part XI - Authentication and Security

- [Chapter 11: Authentication](./chapter-11/README.md)
  - [User Authentication](./chapter-11/user-auth.md)
  - [Password Security](./chapter-11/password-security.md)
  - [API Endpoints](./chapter-11/api-endpoints.md)
  - [Bot Authentication](./chapter-11/bot-auth.md)
  - [Security Features](./chapter-11/security-features.md)
  - [Security Policy](./chapter-11/security-policy.md)
  - [Compliance Requirements](./chapter-11/compliance-requirements.md)

# Part XII - REST API Reference

- [Chapter 12: REST API Reference](./chapter-12/README.md)
  - [Files API](./chapter-12/files-api.md)
  - [Document Processing API](./chapter-12/document-processing.md)
  - [Users API](./chapter-12/users-api.md)
  - [User Security API](./chapter-12/user-security.md)
  - [Groups API](./chapter-12/groups-api.md)
  - [Group Membership API](./chapter-12/group-membership.md)
  - [Conversations API](./chapter-12/conversations-api.md)
  - [Calls API](./chapter-12/calls-api.md)
  - [Whiteboard API](./chapter-12/whiteboard-api.md)
  - [Email API](./chapter-12/email-api.md)
  - [Notifications API](./chapter-12/notifications-api.md)
  - [Calendar API](./chapter-12/calendar-api.md)
  - [Tasks API](./chapter-12/tasks-api.md)
  - [Storage API](./chapter-12/storage-api.md)
  - [Backup API](./chapter-12/backup-api.md)
  - [Analytics API](./chapter-12/analytics-api.md)
  - [Reports API](./chapter-12/reports-api.md)
  - [Admin API](./chapter-12/admin-api.md)
  - [Monitoring API](./chapter-12/monitoring-api.md)
  - [AI API](./chapter-12/ai-api.md)
  - [ML API](./chapter-12/ml-api.md)
  - [Security API](./chapter-12/security-api.md)
  - [Compliance API](./chapter-12/compliance-api.md)
  - [Example Integrations](./chapter-12/examples.md)

# Appendices

- [Appendix I: Database Model](./appendix-i/README.md)
  - [Schema Overview](./appendix-i/schema.md)
  - [Tables](./appendix-i/tables.md)
  - [Relationships](./appendix-i/relationships.md)

- [Appendix II: Project Status](./appendix-ii/README.md)
  - [Build Status](./appendix-ii/build-status.md)
  - [Production Status](./appendix-ii/production-status.md)
  - [Integration Status](./appendix-ii/integration-status.md)

[Glossary](./glossary.md)
