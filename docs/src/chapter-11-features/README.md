# Feature Reference

This chapter provides a comprehensive reference of all General Bots features, organized by capability area.

## Complete Feature Matrix

### Core Platform Features

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Conversational AI | `chat` | Production | `TALK`, `HEAR`, `SET CONTEXT` | Core |
| Multi-turn Dialogs | `basic` | Production | `HEAR AS`, `SWITCH CASE` | Core |
| Session Management | `session` | Production | `SET`, `GET` | Core |
| Bot Memory | `bot_memory` | Production | `SET BOT MEMORY`, `GET BOT MEMORY` | Core |
| User Directory | `directory` | Production | `SET USER`, `ADD MEMBER` | Core |
| Task Scheduling | `tasks` | Production | `SET SCHEDULE`, `ON` | Core |
| Automation Engine | `automation` | Production | `FOR EACH`, `WHILE`, `CALL` | Core |

### AI and LLM Features

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| LLM Integration | `llm` | Production | `LLM`, `SET CONTEXT` | Core |
| Knowledge Base | `vectordb` | Production | `USE KB`, `CLEAR KB` | Enterprise |
| Semantic Search | `vectordb` | Production | `USE WEBSITE`, `FIND` | Enterprise |
| Context Management | `basic` | Production | `SET CONTEXT`, `CLEAR TOOLS` | Core |
| Tool Calling | `basic` | Production | `USE TOOL`, `CLEAR TOOLS` | Core |
| Multi-Agent | `basic` | Production | `ADD BOT`, `DELEGATE TO`, `USE BOT` | Enterprise |

### Communication Channels

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Web Chat | `web` | Production | `TALK`, `HEAR` | Core |
| WhatsApp | `whatsapp` | Production | `SEND`, `SEND TEMPLATE` | Communications |
| Email | `email` | Production | `SEND MAIL`, `CREATE DRAFT` | Standard |
| SMS | `sms` | Production | `SEND SMS` | Communications |
| Microsoft Teams | `msteams` | Production | `SEND` | Communications |
| Instagram | `instagram` | Production | `POST TO INSTAGRAM` | Communications |
| Telegram | `telegram` | Planned | - | Communications |
| Voice | `multimodal` | Production | `PLAY`, `RECORD` | Standard |

### Productivity Suite

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Calendar | `calendar` | Production | `BOOK`, `CREATE EVENT` | Standard |
| Tasks | `tasks` | Production | `CREATE TASK`, `SET SCHEDULE` | Core |
| Drive Storage | `drive` | Production | `UPLOAD`, `DOWNLOAD`, `READ`, `WRITE` | Core |
| Email Client | `mail` | Production | `SEND MAIL`, `GET EMAILS` | Standard |
| Video Meetings | `meet` | Production | `CREATE MEETING` | Standard |
| Document Editor | `paper` | Production | - | Standard |
| Research | `research` | Production | - | Standard |

### Data Operations

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Database CRUD | `data_operations` | Production | `SAVE`, `FIND`, `UPDATE`, `DELETE` | Core |
| Data Import | `import_export` | Production | `IMPORT`, `EXPORT` | Core |
| Aggregations | `data_operations` | Production | `AGGREGATE`, `GROUP BY`, `PIVOT` | Core |
| Joins | `data_operations` | Production | `JOIN`, `MERGE` | Core |
| Filtering | `data_operations` | Production | `FILTER`, `FIND`, `FIRST`, `LAST` | Core |
| Table Definition | `table_definition` | Production | `TABLE` | Core |

### HTTP and API Operations

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| REST Calls | `http_operations` | Production | `GET`, `POST`, `PUT`, `PATCH`, `DELETE HTTP` | Core |
| GraphQL | `http_operations` | Production | `GRAPHQL` | Core |
| SOAP | `http_operations` | Production | `SOAP` | Core |
| Webhooks | `webhook` | Production | `WEBHOOK`, `ON WEBHOOK` | Core |
| Headers | `http_operations` | Production | `SET HEADER` | Core |

### File Operations

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Read Files | `file_operations` | Production | `READ` | Core |
| Write Files | `file_operations` | Production | `WRITE` | Core |
| Copy/Move | `file_operations` | Production | `COPY`, `MOVE` | Core |
| Compress | `file_operations` | Production | `COMPRESS`, `EXTRACT` | Core |
| PDF Generation | `file_operations` | Production | `GENERATE PDF`, `MERGE PDF` | Core |
| Upload/Download | `file_operations` | Production | `UPLOAD`, `DOWNLOAD` | Core |

### CRM Features

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Lead Management | `crm` | Production | `CREATE LEAD`, `QUALIFY LEAD` | Enterprise |
| Contact Management | `crm` | Production | `CREATE CONTACT`, `UPDATE CONTACT` | Enterprise |
| Opportunity Tracking | `crm` | Production | `CREATE OPPORTUNITY`, `CLOSE OPPORTUNITY` | Enterprise |
| Account Management | `crm` | Production | `CREATE ACCOUNT` | Enterprise |
| Activity Logging | `crm` | Production | `LOG ACTIVITY`, `LOG CALL` | Enterprise |
| Lead Scoring | `lead_scoring` | Production | `SCORE LEAD` | Enterprise |
| Pipeline Management | `crm` | Production | `MOVE TO STAGE` | Enterprise |

### Analytics and Reporting

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Time-Series Metrics | `timeseries` | Production | `RECORD METRIC`, `QUERY METRICS` | Enterprise |
| Dashboard | `analytics` | Production | - | Enterprise |
| Custom Reports | `reporting` | Production | `GENERATE REPORT` | Enterprise |
| Usage Analytics | `analytics` | Production | `GET ANALYTICS` | Enterprise |
| Performance Monitoring | `monitoring` | Production | - | Core |

### Compliance and Security

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Audit Logging | `compliance` | Production | - | Enterprise |
| LGPD Compliance | `compliance` | Production | - | Enterprise |
| GDPR Compliance | `compliance` | Production | - | Enterprise |
| HIPAA Compliance | `compliance` | Production | - | Enterprise |
| Access Control | `security` | Production | `SET USER`, `ADD MEMBER` | Core |
| Encryption | `security` | Production | - | Core |
| Consent Management | `compliance` | Production | - | Enterprise |

### Social Media

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Instagram Posts | `social_media` | Production | `POST TO INSTAGRAM` | Communications |
| Facebook Posts | `social_media` | Planned | `POST TO FACEBOOK` | Communications |
| LinkedIn Posts | `social_media` | Planned | `POST TO LINKEDIN` | Communications |
| Twitter/X Posts | `social_media` | Planned | `POST TO TWITTER` | Communications |

### Array and Data Manipulation

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Array Operations | `arrays` | Production | `PUSH`, `POP`, `SHIFT`, `UNSHIFT` | Core |
| Sorting | `arrays` | Production | `SORT` | Core |
| Filtering | `arrays` | Production | `FILTER`, `UNIQUE`, `DISTINCT` | Core |
| Slicing | `arrays` | Production | `SLICE`, `CONTAINS` | Core |

### String Functions

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| String Manipulation | `string_functions` | Production | `LEN`, `LEFT`, `RIGHT`, `MID` | Core |
| Case Conversion | `string_functions` | Production | `UCASE`, `LCASE`, `TRIM` | Core |
| Search/Replace | `string_functions` | Production | `INSTR`, `REPLACE`, `SPLIT` | Core |

### Date and Time

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Date Functions | `datetime` | Production | `NOW`, `TODAY`, `FORMAT` | Core |
| Date Arithmetic | `datetime` | Production | `DATEADD`, `DATEDIFF` | Core |
| Date Parts | `datetime` | Production | `YEAR`, `MONTH`, `DAY`, `HOUR` | Core |

### Math Functions

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Basic Math | `math` | Production | `ABS`, `ROUND`, `FLOOR`, `CEILING` | Core |
| Aggregations | `math` | Production | `SUM`, `AVG`, `MIN`, `MAX`, `COUNT` | Core |
| Random | `math` | Production | `RANDOM` | Core |

### Validation

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Type Checking | `validation` | Production | `IS NUMERIC`, `IS NULL` | Core |
| Input Validation | `validation` | Production | `HEAR AS EMAIL`, `HEAR AS INTEGER` | Core |

### User Interface

| Feature | Module | Status | Keywords | Edition |
|---------|--------|--------|----------|---------|
| Suggestions | `add_suggestion` | Production | `ADD SUGGESTION`, `CLEAR SUGGESTIONS` | Core |
| QR Codes | `qrcode` | Production | `GENERATE QR` | Core |
| Forms | `on_form_submit` | Production | `ON FORM SUBMIT` | Core |
| Sites | `create_site` | Production | `CREATE SITE` | Core |

### Infrastructure Components

| Component | Technical Name | Port | Edition |
|-----------|---------------|------|---------|
| Database | PostgreSQL | 5432 | Core |
| Cache | Redis/Valkey | 6379 | Core |
| Vector Database | Qdrant | 6333 | Enterprise |
| Time-Series Database | InfluxDB | 8086 | Enterprise |
| Video Server | LiveKit | 7880 | Standard |
| Email Server | SMTP/IMAP | 25/993 | Standard |
| Object Storage | S3/MinIO | 9000 | Core |

## Edition Summary

| Edition | Target Use Case | Key Features |
|---------|-----------------|--------------|
| Minimal | Embedded, IoT | Basic chat only |
| Lightweight | Small teams | Chat, Drive, Tasks |
| Core | General business | Full productivity, Automation |
| Standard | Professional teams | Email, Calendar, Meet |
| Enterprise | Large organizations | Compliance, CRM, Analytics, Multi-channel |
| Full | Maximum capability | All features enabled |

## See Also

- [Feature Editions](./editions.md) - Detailed edition comparison
- [Core Features](./core-features.md) - Platform fundamentals
- [Conversation Management](./conversation.md) - Dialog flows
- [AI and LLM](./ai-llm.md) - AI integration
- [Knowledge Base](./knowledge-base.md) - RAG patterns
- [Automation](./automation.md) - Scheduled tasks
- [Email Integration](./email.md) - Email features
- [Storage and Data](./storage.md) - Data persistence
- [Multi-Channel Support](./channels.md) - Communication channels
- [Drive Monitor](./drive-monitor.md) - File monitoring
- [Platform Comparison](./platform-comparison.md) - vs other platforms