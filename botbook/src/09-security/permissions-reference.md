# Complete Permissions Reference

This document provides a comprehensive reference of all permissions available in General Bots Suite. These permissions are designed to provide enterprise-grade access control comparable to Microsoft 365 and Google Workspace.

## Permission Naming Convention

All permissions follow the pattern: `resource.action`

- **resource**: The application or feature being accessed
- **action**: The specific operation being performed

Example: `mail.send` = Send emails in the Mail application

## Quick Reference by Application

| Application | Read | Write | Delete | Share | Admin |
|-------------|------|-------|--------|-------|-------|
| Mail | `mail.read` | `mail.send` | `mail.delete` | `mail.delegate` | `mail.admin` |
| Calendar | `calendar.read` | `calendar.write` | - | `calendar.share` | `calendar.rooms_admin` |
| Drive | `drive.read` | `drive.write` | `drive.delete` | `drive.share` | `drive.admin` |
| Docs | `docs.read` | `docs.write` | - | `docs.share` | `docs.templates_manage` |
| Sheet | `sheet.read` | `sheet.write` | - | `sheet.share` | - |
| Slides | `slides.read` | `slides.write` | - | `slides.share` | - |
| Meet | `meet.join` | `meet.create` | - | - | `meet.admin` |
| Chat | `chat.read` | `chat.write` | `chat.delete` | - | `chat.admin` |
| Tasks | `tasks.read` | `tasks.write` | `tasks.delete` | - | `tasks.projects_manage` |

---

## Administration Permissions

### Organization Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `org.read` | View Organization | View organization settings and information |
| `org.write` | Manage Organization | Modify organization settings |
| `org.delete` | Delete Organization | Delete organization data |
| `org.billing` | Manage Billing | Access billing and subscription management |

### User Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `users.read` | View Users | View user profiles and directory |
| `users.create` | Create Users | Create new user accounts |
| `users.write` | Edit Users | Modify user profiles and settings |
| `users.delete` | Delete Users | Delete user accounts |
| `users.password_reset` | Reset Passwords | Reset user passwords |
| `users.mfa_manage` | Manage MFA | Enable/disable multi-factor authentication |
| `users.impersonate` | Impersonate Users | Sign in as another user for troubleshooting |
| `users.export` | Export Users | Export user data and directory |
| `users.import` | Import Users | Bulk import users from CSV/LDAP |

### Group Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `groups.read` | View Groups | View groups and memberships |
| `groups.create` | Create Groups | Create new groups |
| `groups.write` | Edit Groups | Modify group settings and membership |
| `groups.delete` | Delete Groups | Delete groups |
| `groups.manage_members` | Manage Members | Add/remove group members |
| `groups.manage_owners` | Manage Owners | Assign group owners |

### Role Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `roles.read` | View Roles | View role definitions |
| `roles.create` | Create Roles | Create custom roles |
| `roles.write` | Edit Roles | Modify role permissions |
| `roles.delete` | Delete Roles | Delete custom roles |
| `roles.assign` | Assign Roles | Assign roles to users and groups |

### DNS & Domain Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `dns.read` | View DNS | View DNS records and domain settings |
| `dns.write` | Manage DNS | Add/modify DNS records |
| `domains.verify` | Verify Domains | Verify domain ownership |

---

## Compliance Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `audit.read` | View Audit Logs | Access audit and activity logs |
| `audit.export` | Export Audit Logs | Export audit data for compliance |
| `compliance.read` | View Compliance | View compliance dashboard and reports |
| `compliance.write` | Manage Compliance | Configure compliance policies |
| `dlp.read` | View DLP Policies | View data loss prevention rules |
| `dlp.write` | Manage DLP | Create and modify DLP policies |
| `retention.read` | View Retention | View data retention policies |
| `retention.write` | Manage Retention | Configure retention policies |
| `ediscovery.access` | eDiscovery Access | Access eDiscovery tools and holds |

---

## Security Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `security.read` | View Security | View security dashboard and alerts |
| `security.write` | Manage Security | Configure security settings |
| `threats.read` | View Threats | View threat detection and incidents |
| `threats.respond` | Respond to Threats | Take action on security incidents |
| `secrets.read` | View Secrets | View API keys and secrets |
| `secrets.write` | Manage Secrets | Create and rotate secrets |

---

## Mail Permissions

*Equivalent to Outlook / Gmail*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `mail.read` | Read Mail | Read own mailbox and messages |
| `mail.send` | Send Mail | Send emails |
| `mail.delete` | Delete Mail | Delete emails |
| `mail.organize` | Organize Mail | Create folders, apply labels, set rules |
| `mail.delegate` | Mail Delegation | Grant mailbox access to others |
| `mail.shared_read` | Read Shared Mailbox | Access shared mailboxes |
| `mail.shared_send` | Send from Shared | Send as shared mailbox |
| `mail.admin` | Mail Admin | Administer mail settings globally |
| `mail.rules_global` | Global Mail Rules | Create organization-wide mail rules |
| `mail.signatures_global` | Global Signatures | Manage organization email signatures |
| `mail.distribution_lists` | Distribution Lists | Manage distribution lists |
| `mail.encryption` | Mail Encryption | Send encrypted messages |
| `mail.archive` | Mail Archive | Access mail archive |

---

## Calendar Permissions

*Equivalent to Outlook Calendar / Google Calendar*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `calendar.read` | View Calendar | View own calendar and events |
| `calendar.write` | Manage Calendar | Create, edit, delete events |
| `calendar.share` | Share Calendar | Share calendar with others |
| `calendar.delegate` | Calendar Delegation | Allow others to manage calendar |
| `calendar.free_busy` | View Free/Busy | View availability of others |
| `calendar.rooms` | Book Rooms | Reserve meeting rooms and resources |
| `calendar.rooms_admin` | Manage Rooms | Administer room resources |
| `calendar.shared_read` | Read Shared Calendars | View shared team calendars |
| `calendar.shared_write` | Edit Shared Calendars | Modify shared team calendars |

---

## Drive Permissions

*Equivalent to OneDrive / SharePoint / Google Drive*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `drive.read` | View Files | View own files and folders |
| `drive.write` | Upload Files | Upload and create files |
| `drive.delete` | Delete Files | Delete own files |
| `drive.share` | Share Files | Share files with others |
| `drive.share_external` | External Sharing | Share files externally |
| `drive.download` | Download Files | Download files locally |
| `drive.sync` | Sync Files | Use desktop sync client |
| `drive.version_history` | Version History | View and restore file versions |
| `drive.shared_read` | Read Shared Drives | Access team shared drives |
| `drive.shared_write` | Write Shared Drives | Modify files in shared drives |
| `drive.shared_admin` | Manage Shared Drives | Administer shared drive settings |
| `drive.trash` | Manage Trash | View and restore deleted items |
| `drive.quota` | View Storage Quota | View storage usage |
| `drive.admin` | Drive Admin | Full administrative access to all drives |

---

## Docs Permissions

*Equivalent to Word Online / Google Docs*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `docs.read` | View Documents | View documents |
| `docs.write` | Edit Documents | Create and edit documents |
| `docs.comment` | Comment on Documents | Add comments and suggestions |
| `docs.share` | Share Documents | Share documents with others |
| `docs.export` | Export Documents | Export to PDF, Word, etc. |
| `docs.templates` | Use Templates | Access document templates |
| `docs.templates_manage` | Manage Templates | Create organization templates |
| `docs.collaborate` | Real-time Collaboration | Co-author documents in real-time |

---

## Sheet Permissions

*Equivalent to Excel Online / Google Sheets*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `sheet.read` | View Spreadsheets | View spreadsheets |
| `sheet.write` | Edit Spreadsheets | Create and edit spreadsheets |
| `sheet.share` | Share Spreadsheets | Share spreadsheets with others |
| `sheet.export` | Export Spreadsheets | Export to Excel, CSV, etc. |
| `sheet.import` | Import Data | Import data from external sources |
| `sheet.macros` | Run Macros | Execute spreadsheet macros |
| `sheet.connections` | Data Connections | Create database connections |
| `sheet.pivot` | Pivot Tables | Create pivot tables and charts |

---

## Slides Permissions

*Equivalent to PowerPoint Online / Google Slides*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `slides.read` | View Presentations | View presentations |
| `slides.write` | Edit Presentations | Create and edit presentations |
| `slides.share` | Share Presentations | Share presentations with others |
| `slides.present` | Present Live | Start live presentations |
| `slides.export` | Export Presentations | Export to PDF, PowerPoint |
| `slides.templates` | Slide Templates | Access presentation templates |

---

## Meet Permissions

*Equivalent to Teams / Zoom / Google Meet*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `meet.join` | Join Meetings | Join video meetings |
| `meet.create` | Create Meetings | Schedule and create meetings |
| `meet.host` | Host Meetings | Full host controls in meetings |
| `meet.record` | Record Meetings | Record meeting sessions |
| `meet.transcript` | Meeting Transcripts | Access meeting transcriptions |
| `meet.screen_share` | Screen Share | Share screen in meetings |
| `meet.breakout` | Breakout Rooms | Create and manage breakout rooms |
| `meet.webinar` | Host Webinars | Host large webinar events |
| `meet.admin` | Meet Admin | Administer meeting settings globally |
| `meet.external` | External Meetings | Meet with external participants |

---

## Chat Permissions

*Equivalent to Teams Chat / Slack / Google Chat*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `chat.read` | Read Messages | Read chat messages |
| `chat.write` | Send Messages | Send chat messages |
| `chat.delete` | Delete Messages | Delete own messages |
| `chat.edit` | Edit Messages | Edit sent messages |
| `chat.files` | Share Files in Chat | Share files in conversations |
| `chat.channels_create` | Create Channels | Create chat channels |
| `chat.channels_manage` | Manage Channels | Manage channel settings |
| `chat.external` | External Chat | Chat with external users |
| `chat.reactions` | Reactions | Add reactions to messages |
| `chat.threads` | Thread Replies | Reply in threads |
| `chat.mentions` | Mentions | Mention users and groups |
| `chat.admin` | Chat Admin | Administer chat settings globally |

---

## Tasks Permissions

*Equivalent to Planner / Asana / Google Tasks*

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `tasks.read` | View Tasks | View own and assigned tasks |
| `tasks.write` | Manage Tasks | Create and edit tasks |
| `tasks.delete` | Delete Tasks | Delete tasks |
| `tasks.assign` | Assign Tasks | Assign tasks to others |
| `tasks.projects_create` | Create Projects | Create task projects/boards |
| `tasks.projects_manage` | Manage Projects | Administer project settings |
| `tasks.time_track` | Time Tracking | Log time against tasks |
| `tasks.reports` | Task Reports | View task analytics and reports |
| `tasks.automation` | Task Automation | Create task automation rules |

---

## Bot & AI Permissions

### Bot Management

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `bots.read` | View Bots | View bot configurations |
| `bots.create` | Create Bots | Create new bots |
| `bots.write` | Edit Bots | Modify bot settings |
| `bots.delete` | Delete Bots | Delete bots |
| `bots.publish` | Publish Bots | Publish bots to channels |
| `bots.channels` | Manage Channels | Configure bot communication channels |

### AI Assistant

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `ai.chat` | AI Chat | Use AI chat assistant |
| `ai.summarize` | AI Summarize | Use AI to summarize content |
| `ai.compose` | AI Compose | Use AI to draft content |
| `ai.translate` | AI Translate | Use AI translation |
| `ai.analyze` | AI Analyze | Use AI for data analysis |
| `ai.advanced` | Advanced AI | Access advanced AI features |

### Knowledge Base

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `kb.read` | View Knowledge Base | Access knowledge base documents |
| `kb.write` | Edit Knowledge Base | Add/edit knowledge base content |
| `kb.admin` | KB Admin | Administer knowledge base settings |

### Conversations

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `conversations.read` | View Conversations | View bot conversations |
| `conversations.write` | Manage Conversations | Intervene in conversations |
| `conversations.transfer` | Transfer Conversations | Transfer to human agent |
| `conversations.history` | Conversation History | Access conversation history |
| `attendant.access` | Attendant Access | Access human attendant queue |
| `attendant.respond` | Attendant Respond | Respond to queued conversations |

---

## Analytics & Reporting Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `analytics.read` | View Analytics | View usage analytics and dashboards |
| `analytics.export` | Export Analytics | Export analytics data |
| `analytics.custom` | Custom Reports | Create custom reports and dashboards |
| `analytics.realtime` | Real-time Analytics | Access real-time analytics |
| `reports.read` | View Reports | Access standard reports |
| `reports.schedule` | Schedule Reports | Schedule automated report delivery |

---

## Monitoring & System Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `monitoring.read` | View Monitoring | View system health and metrics |
| `monitoring.alerts` | Manage Alerts | Configure monitoring alerts |
| `logs.read` | View Logs | Access system and application logs |
| `logs.export` | Export Logs | Export log data |
| `services.read` | View Services | View service status |
| `services.manage` | Manage Services | Start/stop/restart services |
| `resources.read` | View Resources | View resource usage |

---

## Paper & Research Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `paper.read` | View Papers | View AI-generated papers and notes |
| `paper.write` | Create Papers | Create and edit AI-assisted documents |
| `paper.publish` | Publish Papers | Publish papers to knowledge base |
| `research.read` | View Research | Access AI research results |
| `research.create` | Create Research | Start AI research queries |
| `research.deep` | Deep Research | Access deep research features |
| `quicknote.access` | Quick Notes | Access quick note feature |

---

## Integrations Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `sources.read` | View Sources | View configured data sources |
| `sources.create` | Create Sources | Add new data sources |
| `sources.write` | Edit Sources | Modify data source configurations |
| `sources.delete` | Delete Sources | Remove data sources |
| `webhooks.read` | View Webhooks | View webhook configurations |
| `webhooks.write` | Manage Webhooks | Create and edit webhooks |
| `api.access` | API Access | Access REST API endpoints |
| `api.keys` | API Key Management | Create and manage API keys |
| `integrations.read` | View Integrations | View third-party integrations |
| `integrations.write` | Manage Integrations | Configure third-party integrations |
| `mcp.access` | MCP Access | Access Model Context Protocol tools |

---

## Automation Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `autotask.read` | View AutoTasks | View automated task definitions |
| `autotask.create` | Create AutoTasks | Create new automated tasks |
| `autotask.write` | Edit AutoTasks | Modify automated task settings |
| `autotask.delete` | Delete AutoTasks | Remove automated tasks |
| `autotask.execute` | Execute AutoTasks | Run automated tasks manually |
| `autotask.schedule` | Schedule AutoTasks | Schedule task automation |
| `workflows.read` | View Workflows | View workflow definitions |
| `workflows.write` | Manage Workflows | Create and edit workflows |
| `intents.read` | View Intents | View AI intent definitions |
| `intents.write` | Manage Intents | Create and edit intents |

---

## Designer Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `designer.access` | Access Designer | Open visual designer tool |
| `designer.create` | Create Designs | Create new UI designs |
| `designer.edit` | Edit Designs | Modify existing designs |
| `designer.publish` | Publish Designs | Publish designs to production |
| `designer.templates` | Design Templates | Access and create design templates |

---

## Settings Permissions

| Permission | Display Name | Description |
|------------|--------------|-------------|
| `settings.personal` | Personal Settings | Manage own user settings |
| `settings.organization` | Organization Settings | Manage organization settings |
| `settings.security` | Security Settings | Manage security configuration |
| `settings.notifications` | Notification Settings | Manage notification preferences |
| `settings.appearance` | Appearance Settings | Customize appearance and themes |
| `settings.language` | Language Settings | Set language and locale |
| `settings.backup` | Backup Settings | Configure backup and export |

---

## Role-Permission Matrix

### Global Administrator
**Has ALL permissions** - Full system control

### Billing Administrator
- `org.read`, `org.billing`
- `users.read`
- `reports.read`, `analytics.read`

### Compliance Administrator
- `org.read`, `users.read`, `groups.read`
- `audit.read`, `audit.export`
- `compliance.read`, `compliance.write`
- `dlp.read`, `dlp.write`
- `retention.read`, `retention.write`
- `ediscovery.access`
- `analytics.read`, `reports.read`
- `logs.read`, `logs.export`

### Security Administrator
- `org.read`, `users.read`, `users.mfa_manage`, `groups.read`
- `security.read`, `security.write`
- `threats.read`, `threats.respond`
- `secrets.read`, `secrets.write`
- `audit.read`, `logs.read`
- `monitoring.read`, `monitoring.alerts`

### User Administrator
- `users.read`, `users.create`, `users.write`, `users.delete`
- `users.password_reset`, `users.mfa_manage`
- `users.export`, `users.import`
- `groups.read`, `groups.create`, `groups.write`, `groups.manage_members`
- `roles.read`, `roles.assign`
- `audit.read`

### Standard User
Full access to:
- Mail (read, send, delete, organize)
- Calendar (read, write, share, rooms)
- Drive (read, write, delete, share, sync)
- Docs, Sheet, Slides (read, write, collaborate)
- Meet (join, create, host, screen share)
- Chat (read, write, edit, reactions)
- Tasks (read, write, assign)
- AI (chat, summarize, compose, translate)
- Personal settings

### Guest User
Limited access to:
- Mail (read, send only)
- Calendar (read, free/busy only)
- Drive (read, download shared only)
- Docs (read, comment only)
- Meet (join, screen share)
- Chat (read, write, reactions)
- Tasks (read only)
- Personal settings

### Viewer
Read-only access to:
- Mail, Calendar, Drive, Docs, Sheet, Slides
- Meet (join only), Chat (read only)
- Tasks (read only), Analytics
- Personal settings

---

## See Also

- [RBAC Overview](./rbac-overview.md) - Understanding role-based access control
- [User Authentication](./user-auth.md) - Authentication mechanisms
- [Security Policy](./security-policy.md) - Security best practices
- [API Endpoints](./api-endpoints.md) - API documentation with permission requirements