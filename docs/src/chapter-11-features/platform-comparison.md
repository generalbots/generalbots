# Platform Comparison: General Bots vs Notion AI vs Perplexity

This document compares General Bots with popular AI-powered platforms to highlight unique capabilities and help users understand when to choose each solution.

## Executive Summary

| Capability | General Bots | Notion AI | Perplexity |
|------------|-------------|-----------|------------|
| **Primary Focus** | Customizable AI automation platform | Document collaboration with AI | AI-powered search engine |
| **Self-hosted** | ✅ Yes | ❌ No | ❌ No |
| **Open Source** | ✅ AGPL | ❌ Proprietary | ❌ Proprietary |
| **Custom APIs/Webhooks** | ✅ Full control | ❌ No | ❌ No |
| **Database Integration** | ✅ Full SQL | ⚠️ Limited (Notion DBs) | ❌ No |
| **Custom LLM Backend** | ✅ Any provider | ❌ OpenAI only | ❌ Proprietary |
| **BASIC Programming** | ✅ Native | ❌ No | ❌ No |
| **Multi-channel** | ✅ WhatsApp, Teams, Web, etc. | ❌ Web only | ❌ Web only |
| **File Storage** | ✅ .gbdrive | ⚠️ Notion pages | ❌ No |
| **Email Integration** | ✅ Send/receive | ❌ No | ❌ No |
| **Pricing** | Free (self-hosted) | $10/user/month | $20/month |

## Detailed Comparison

### 1. Customization & Extensibility

#### General Bots

General Bots is designed for maximum customization:

```basic
' Create a custom API endpoint in one line
WEBHOOK "customer-lookup"

customer_id = params.id
USE KB "customer-data"

' AI-powered response
response = LLM "Get all information about customer " + customer_id

WITH result = NEW OBJECT
    .customer_id = customer_id
    .data = response
    .generated_at = NOW()
END WITH
```

**Capabilities:**
- Create unlimited custom webhooks/APIs
- Write automation in BASIC syntax
- Integrate with any external system
- Self-host with full control
- Modify source code (AGPL license)

#### Notion AI

Limited to built-in features:
- Summarize pages
- Generate content
- Translate text
- Brainstorm ideas
- No custom API creation
- No webhook support
- No programmatic automation

#### Perplexity

Search-focused only:
- Ask questions, get answers
- No customization
- No API creation
- No automation capabilities

### 2. Knowledge Base & RAG

#### General Bots

Full control over your knowledge base:

```basic
' Load multiple knowledge sources
USE KB "company-policies"
USE KB "product-catalog"
USE KB "customer-faq"

' Set custom context
SET CONTEXT "You are a helpful customer service agent for Acme Corp."

' Query with RAG
answer = LLM user_question

' Save conversation for training
WITH conversation = NEW OBJECT
    .question = user_question
    .answer = answer
    .timestamp = NOW()
    .user_id = user.id
END WITH
INSERT "conversations", conversation
```

**Features:**
- Multiple vector collections
- Custom embedding models
- Semantic search
- Context compaction
- Semantic caching
- Full document indexing

#### Notion AI

- Only searches within Notion workspace
- No external document upload for AI
- Limited to Notion page structure
- No custom embeddings

#### Perplexity

- Searches the web in real-time
- No custom document upload (free tier)
- Pro tier allows file upload
- No persistent knowledge base

### 3. Automation & Workflows

#### General Bots

Complete automation platform:

```basic
' Scheduled automation
SET SCHEDULE "daily-report", "0 9 * * *"

' Fetch data from multiple sources
sales = GET "https://api.crm.com/sales/today"
inventory = FIND "inventory", "stock < 10"
support_tickets = GET "https://api.zendesk.com/tickets/open"

' AI-generated summary
SET CONTEXT "You are a business analyst. Create an executive summary."
summary = LLM "Summarize: Sales: " + sales + ", Low stock: " + inventory + ", Open tickets: " + support_tickets

' Send report
SEND MAIL "executives@company.com", "Daily Business Report", summary

' Post to Slack
WITH slack_msg = NEW OBJECT
    .text = summary
    .channel = "#daily-reports"
END WITH
POST "https://hooks.slack.com/services/xxx", slack_msg
```

**Automation types:**
- Scheduled tasks (cron)
- Webhooks (event-driven)
- Database triggers (ON keyword)
- Multi-channel messaging
- Email workflows
- File processing pipelines

#### Notion AI

- No automation capabilities
- Manual AI invocation only
- No scheduled tasks
- No webhooks
- No external integrations

#### Perplexity

- No automation
- Interactive queries only
- No scheduling
- No integrations

### 4. Multi-Channel Communication

#### General Bots

Deploy to any channel:

```basic
' Same BASIC code works on all channels
TALK "Hello! How can I help you today?"

HEAR question

USE KB "support-docs"
answer = LLM question

TALK answer

' Channel-specific features
IF channel = "whatsapp" THEN
    ' Send WhatsApp-specific media
    TALK IMAGE "product-photo.jpg"
ELSE IF channel = "teams" THEN
    ' Teams adaptive card
    TALK CARD "product-details"
END IF
```

**Supported channels:**
- WhatsApp Business
- Microsoft Teams
- Slack
- Telegram
- Web chat
- SMS
- Email
- Voice (coming soon)

#### Notion AI

- Web interface only
- No mobile app AI
- No chat deployments
- No messaging integrations

#### Perplexity

- Web interface
- Mobile apps
- No third-party deployments
- No messaging integrations

### 5. Data Privacy & Control

#### General Bots

**Full data sovereignty:**
- Self-hosted on your infrastructure
- Data never leaves your servers
- Choose your own LLM provider
- Audit logs and compliance
- GDPR/HIPAA ready

```basic
' All data stays on your servers
SAVE "customer_data", customer_id, sensitive_info

' Use local LLM if needed
SET CONTEXT "Use local Llama model"
response = LLM query
```

#### Notion AI

- Data stored on Notion servers (US/EU)
- Uses OpenAI (data sent to OpenAI)
- Limited compliance features
- SOC 2 Type 2 certified

#### Perplexity

- Data stored on Perplexity servers
- Search queries may be logged
- Limited privacy controls
- No self-hosting option

### 6. Integration Capabilities

#### General Bots

Native HTTP/API support:

```basic
' REST APIs
customers = GET "https://api.salesforce.com/customers"
POST "https://api.hubspot.com/contacts", contact_data
PUT "https://api.stripe.com/customers/123", update_data
DELETE "https://api.service.com/items/456"

' GraphQL
query = "query { user(id: 123) { name email } }"
result = GRAPHQL "https://api.github.com/graphql", query, vars

' SOAP (legacy systems)
result = SOAP "https://legacy.corp.com/service.wsdl", "GetCustomer", params

' Database
data = FIND "products", "category='electronics'"
INSERT "orders", order_data
UPDATE "inventory", "sku=ABC123", stock_update
```

**Integrations:**
- Any REST API
- GraphQL endpoints
- SOAP services
- SQL databases
- S3-compatible storage
- Email (SMTP/IMAP)
- Calendar (CalDAV)
- Any webhook-capable service

#### Notion AI

- Notion API only
- Limited integrations via Notion
- No direct external API calls
- No database connections

#### Perplexity

- No API integrations
- Search only
- No external data sources

### 7. Document Processing

#### General Bots

Full document pipeline:

```basic
' Upload and process documents
HEAR document AS FILE
url = UPLOAD document, "uploads/"

' Extract text from various formats
content = GET "documents/report.pdf"

' AI processing
SET CONTEXT "Extract key metrics from this report"
metrics = LLM content

' Generate new documents
WITH invoice_data = NEW OBJECT
    .customer = customer_name
    .items = order_items
    .total = order_total
END WITH
pdf = GENERATE PDF "templates/invoice.html", invoice_data, "invoices/inv-001.pdf"

' Merge multiple PDFs
merged = MERGE PDF ["cover.pdf", "report.pdf", "appendix.pdf"], "final-report.pdf"

' Compress and send
archive = COMPRESS ["report.pdf", "data.xlsx"], "delivery.zip"
SEND MAIL customer_email, "Your Report", "Please find attached.", archive
```

**Supported formats:**
- PDF (read, generate, merge)
- Office documents (Word, Excel, PowerPoint)
- Images (OCR support)
- CSV/JSON data
- Archives (ZIP, TAR)

#### Notion AI

- Notion pages only
- Export to PDF/Markdown
- No document generation
- No PDF processing

#### Perplexity

- Can read uploaded PDFs (Pro)
- No document generation
- No processing capabilities

### 8. Cost Comparison

#### General Bots (Self-hosted)

| Component | Cost |
|-----------|------|
| Software | Free (AGPL) |
| Infrastructure | $20-100/month (your servers) |
| LLM API | Pay per use (OpenAI, etc.) |
| **Total** | **$20-200/month** (unlimited users) |

#### Notion AI

| Plan | Cost |
|------|------|
| Free | Limited AI features |
| Plus | $10/user/month |
| Business | $18/user/month |
| **10 users** | **$100-180/month** |

#### Perplexity

| Plan | Cost |
|------|------|
| Free | Limited queries |
| Pro | $20/month |
| Team | $20/user/month |
| **10 users** | **$200/month** |

### 9. Use Case Recommendations

#### Choose General Bots when you need:

- ✅ Custom chatbots for customer service
- ✅ Internal automation workflows
- ✅ Multi-channel deployment (WhatsApp, Teams, etc.)
- ✅ Integration with existing systems
- ✅ Custom APIs without traditional development
- ✅ Data privacy and self-hosting
- ✅ Complex business logic in simple BASIC
- ✅ Document processing pipelines
- ✅ Scheduled tasks and webhooks

#### Choose Notion AI when you need:

- ✅ Document collaboration with AI assist
- ✅ Team knowledge management
- ✅ Content writing assistance
- ✅ Simple Q&A within documents
- ✅ Project management with AI

#### Choose Perplexity when you need:

- ✅ Research and fact-checking
- ✅ Real-time web search with AI
- ✅ Quick answers with citations
- ✅ Exploring topics

## Feature Matrix

| Feature | General Bots | Notion AI | Perplexity |
|---------|-------------|-----------|------------|
| **AI Chat** | ✅ | ✅ | ✅ |
| **Custom Knowledge Base** | ✅ | ⚠️ | ⚠️ |
| **API Creation** | ✅ | ❌ | ❌ |
| **Webhooks** | ✅ | ❌ | ❌ |
| **Scheduled Tasks** | ✅ | ❌ | ❌ |
| **Database** | ✅ | ⚠️ | ❌ |
| **File Storage** | ✅ | ⚠️ | ❌ |
| **Email** | ✅ | ❌ | ❌ |
| **WhatsApp** | ✅ | ❌ | ❌ |
| **Teams** | ✅ | ❌ | ❌ |
| **Slack** | ✅ | ⚠️ | ❌ |
| **PDF Generation** | ✅ | ❌ | ❌ |
| **Custom LLM** | ✅ | ❌ | ❌ |
| **Self-hosted** | ✅ | ❌ | ❌ |
| **Open Source** | ✅ | ❌ | ❌ |
| **Real-time Web Search** | ⚠️ | ❌ | ✅ |
| **Document Collaboration** | ⚠️ | ✅ | ❌ |

## Migration Path

### From Notion AI to General Bots

1. Export Notion pages as Markdown
2. Upload to General Bots knowledge base
3. Create BASIC scripts for automation
4. Deploy to additional channels

```basic
' Import Notion export
files = LIST "notion-export/"
FOR EACH file IN files
    content = GET file
    USE KB "imported-docs"
    ' Documents automatically indexed
NEXT file

TALK "Notion content imported and ready for AI queries!"
```

### From Perplexity to General Bots

1. Identify common queries
2. Build knowledge base from trusted sources
3. Create custom Q&A endpoint

```basic
' Replace Perplexity with custom research endpoint
WEBHOOK "research"

query = body.query
USE KB "trusted-sources"
USE WEBSITE "https://docs.company.com"

SET CONTEXT "Provide accurate, cited answers based on the knowledge base."
answer = LLM query

WITH result = NEW OBJECT
    .query = query
    .answer = answer
    .sources = "Internal knowledge base"
END WITH
```

## Conclusion

**General Bots** stands apart as the only platform that combines:

1. **Full customization** - Write BASIC code for any automation
2. **Self-hosting** - Complete data control
3. **Multi-channel** - Deploy anywhere
4. **Open source** - Modify and extend freely
5. **Cost-effective** - No per-user pricing

While Notion AI excels at document collaboration and Perplexity at web search, General Bots is the choice for organizations that need **customizable, self-hosted AI automation** with **enterprise-grade capabilities**.

## See Also

- [Quick Start](../chapter-01/quick-start.md) - Get started in 5 minutes
- [WEBHOOK](../chapter-06-gbdialog/keyword-webhook.md) - Create instant APIs
- [Keywords Reference](../chapter-06-gbdialog/keywords.md) - Full BASIC reference
- [Architecture](../chapter-07-gbapp/architecture.md) - Technical deep dive