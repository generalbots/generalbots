# Summary

[Introduction](./introduction.md)

# Part I - Getting Started

- [Chapter 01: Run and Talk](./chapter-01/README.md)
  - [Overview](./chapter-01/overview.md)
  - [Quick Start](./chapter-01/quick-start.md)
  - [Installation](./chapter-01/installation.md)
  - [First Conversation](./chapter-01/first-conversation.md)
  - [Sessions and Channels](./chapter-01/sessions.md)

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
  - [Semantic Search](./chapter-03/semantic-search.md)
  - [Context Compaction](./chapter-03/context-compaction.md)
  - [Semantic Caching](./chapter-03/caching.md)

# Part IV - User Interface

- [Chapter 04: .gbui Interface Reference](./chapter-04-gbui/README.md)
  - [default.gbui - Full Desktop](./chapter-04-gbui/default-gbui.md)
  - [single.gbui - Simple Chat](./chapter-04-gbui/single-gbui.md)
  - [Console Mode](./chapter-04-gbui/console-mode.md)

# Part V - Themes and Styling

- [Chapter 05: gbtheme CSS Reference](./chapter-05-gbtheme/README.md)
  - [Theme Structure](./chapter-05-gbtheme/structure.md)
  - [CSS Customization](./chapter-05-gbtheme/css.md)

# Part VI - BASIC Dialogs

- [Chapter 06: gbdialog Reference](./chapter-06-gbdialog/README.md)
  - [Dialog Basics](./chapter-06-gbdialog/basics.md)
  - [Universal Messaging & Multi-Channel](./chapter-06-gbdialog/universal-messaging.md)
  - [Template Examples](./chapter-06-gbdialog/templates.md)
    - [start.bas](./chapter-06-gbdialog/template-start.md)
    - [generate-summary.bas](./chapter-06-gbdialog/template-summary.md)
    - [enrollment Tool Example](./chapter-06-gbdialog/template-enrollment.md)
  - [Keywords Reference](./chapter-06-gbdialog/keywords.md)
    - [TALK](./chapter-06-gbdialog/keyword-talk.md)
    - [HEAR](./chapter-06-gbdialog/keyword-hear.md)
    - [SET CONTEXT](./chapter-06-gbdialog/keyword-set-context.md)
    - [GET BOT MEMORY](./chapter-06-gbdialog/keyword-get-bot-memory.md)
    - [SET BOT MEMORY](./chapter-06-gbdialog/keyword-set-bot-memory.md)
    - [USE KB](./chapter-06-gbdialog/keyword-use-kb.md)
    - [CLEAR KB](./chapter-06-gbdialog/keyword-clear-kb.md)
    - [ADD WEBSITE](./chapter-06-gbdialog/keyword-add-website.md)
    - [USE TOOL](./chapter-06-gbdialog/keyword-use-tool.md)
    - [CLEAR TOOLS](./chapter-06-gbdialog/keyword-clear-tools.md)
    - [GET](./chapter-06-gbdialog/keyword-get.md)
    - [SET](./chapter-06-gbdialog/keyword-set.md)
    - [ON](./chapter-06-gbdialog/keyword-on.md)
    - [SET SCHEDULE](./chapter-06-gbdialog/keyword-set-schedule.md)
    - [CREATE SITE](./chapter-06-gbdialog/keyword-create-site.md)
    - [CREATE DRAFT](./chapter-06-gbdialog/keyword-create-draft.md)
    - [CREATE TASK](./chapter-06-gbdialog/keyword-create-task.md)
    - [PRINT](./chapter-06-gbdialog/keyword-print.md)
    - [WAIT](./chapter-06-gbdialog/keyword-wait.md)
    - [FORMAT](./chapter-06-gbdialog/keyword-format.md)
    - [FIRST](./chapter-06-gbdialog/keyword-first.md)
    - [LAST](./chapter-06-gbdialog/keyword-last.md)
    - [FOR EACH](./chapter-06-gbdialog/keyword-for-each.md)
    - [EXIT FOR](./chapter-06-gbdialog/keyword-exit-for.md)
    - [SEND MAIL](./chapter-06-gbdialog/keyword-send-mail.md)
    - [FIND](./chapter-06-gbdialog/keyword-find.md)

# Part VII - Extending BotServer

- [Chapter 07: gbapp Architecture Reference](./chapter-07-gbapp/README.md)
  - [Architecture Overview](./chapter-07-gbapp/architecture.md)
  - [Building from Source](./chapter-07-gbapp/building.md)
  - [Container Deployment (LXC)](./chapter-07-gbapp/containers.md)
  - [Philosophy](./chapter-07-gbapp/philosophy.md)
  - [Example gbapp](./chapter-07-gbapp/example-gbapp.md)
  - [Module Structure](./chapter-07-gbapp/crates.md)
  - [Service Layer](./chapter-07-gbapp/services.md)
  - [Creating Custom Keywords](./chapter-07-gbapp/custom-keywords.md)
  - [Adding Dependencies](./chapter-07-gbapp/dependencies.md)

# Part VIII - Bot Configuration

- [Chapter 08: gbot Reference](./chapter-08-config/README.md)
  - [config.csv Format](./chapter-08-config/config-csv.md)
  - [Bot Parameters](./chapter-08-config/parameters.md)
  - [LLM Configuration](./chapter-08-config/llm-config.md)
  - [Context Configuration](./chapter-08-config/context-config.md)
  - [Drive Integration](./chapter-08-config/minio.md)

# Part IX - Tools and Integration

- [Chapter 09: API and Tooling](./chapter-09-api/README.md)
  - [Tool Definition](./chapter-09-api/tool-definition.md)
  - [PARAM Declaration](./chapter-09-api/param-declaration.md)
  - [Tool Compilation](./chapter-09-api/compilation.md)
  - [MCP Format](./chapter-09-api/mcp-format.md)
  - [Tool Format](./chapter-09-api/openai-format.md)
  - [GET Keyword Integration](./chapter-09-api/get-integration.md)
  - [External APIs](./chapter-09-api/external-apis.md)
  - [NVIDIA GPU Setup for LXC](./chapter-09-api/nvidia-gpu-setup.md)


# Part X - Feature Deep Dive

- [Chapter 10: Feature Reference](./chapter-10-features/README.md)
  - [Core Features](./chapter-10-features/core-features.md)
  - [Conversation Management](./chapter-10-features/conversation.md)
  - [AI and LLM](./chapter-10-features/ai-llm.md)
  - [Knowledge Base](./chapter-10-features/knowledge-base.md)
  - [Automation](./chapter-10-features/automation.md)
  - [Email Integration](./chapter-10-features/email.md)
  - [UI Automation](./chapter-10-features/ui-automation.md)
  - [Storage and Data](./chapter-10-features/storage.md)
  - [Multi-Channel Support](./chapter-10-features/channels.md)

# Part XI - Community

- [Chapter 11: Contributing](./chapter-11-community/README.md)
  - [Development Setup](./chapter-11-community/setup.md)
  - [Testing Guide](./chapter-11-community/testing.md)
  - [Documentation](./chapter-11-community/documentation.md)
  - [Pull Requests](./chapter-11-community/pull-requests.md)
  - [Community Guidelines](./chapter-11-community/community.md)
  - [IDE Extensions](./chapter-11-community/ide-extensions.md)

# Part XII - Authentication and Security

- [Chapter 12: Authentication](./chapter-12-auth/README.md)
  - [User Authentication](./chapter-12-auth/user-auth.md)
  - [Password Security](./chapter-12-auth/password-security.md)
  - [API Endpoints](./chapter-12-auth/api-endpoints.md)
  - [Bot Authentication](./chapter-12-auth/bot-auth.md)
  - [Security Features](./chapter-12-auth/security-features.md)
  - [Security Policy](./chapter-12-auth/security-policy.md)
  - [Compliance Requirements](./chapter-12-auth/compliance-requirements.md)

# Part XIII - REST API Reference

- [Chapter 13: REST API Reference](./chapter-13-api/README.md)
  - [Files API](./chapter-13-api/files-api.md)
  - [Document Processing API](./chapter-13-api/document-processing.md)
  - [Users API](./chapter-13-api/users-api.md)
  - [User Security API](./chapter-13-api/user-security.md)
  - [Groups API](./chapter-13-api/groups-api.md)
  - [Group Membership API](./chapter-13-api/group-membership.md)
  - [Conversations API](./chapter-13-api/conversations-api.md)
  - [Calls API](./chapter-13-api/calls-api.md)
  - [Whiteboard API](./chapter-13-api/whiteboard-api.md)
  - [Email API](./chapter-13-api/email-api.md)
  - [Notifications API](./chapter-13-api/notifications-api.md)
  - [Calendar API](./chapter-13-api/calendar-api.md)
  - [Tasks API](./chapter-13-api/tasks-api.md)
  - [Storage API](./chapter-13-api/storage-api.md)
  - [Backup API](./chapter-13-api/backup-api.md)
  - [Analytics API](./chapter-13-api/analytics-api.md)
  - [Reports API](./chapter-13-api/reports-api.md)
  - [Admin API](./chapter-13-api/admin-api.md)
  - [Monitoring API](./chapter-13-api/monitoring-api.md)
  - [AI API](./chapter-13-api/ai-api.md)
  - [ML API](./chapter-13-api/ml-api.md)
  - [Security API](./chapter-13-api/security-api.md)
  - [Compliance API](./chapter-13-api/compliance-api.md)
  - [Example Integrations](./chapter-13-api/examples.md)

# Appendices

- [Appendix I: Database Model](./appendix-i/README.md)
  - [Schema Overview](./appendix-i/schema.md)
  - [Tables](./appendix-i/tables.md)
  - [Relationships](./appendix-i/relationships.md)

[Glossary](./glossary.md)