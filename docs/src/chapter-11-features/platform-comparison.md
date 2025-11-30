# Platform Capabilities

General Bots provides a unique combination of capabilities that differentiate it from other AI platforms. This document outlines what makes General Bots suitable for organizations seeking customizable, self-hosted AI automation.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Core Differentiators

### Self-Hosted & Open Source

General Bots runs entirely on your infrastructure. Your data never leaves your servers, and you have full access to the source code under AGPL licensing.

| Capability | General Bots |
|------------|-------------|
| Self-hosted deployment | ✅ Full control |
| Open source | ✅ AGPL licensed |
| Data sovereignty | ✅ Your infrastructure |
| Custom modifications | ✅ Full source access |
| Per-user licensing | ✅ None required |

### Customization & Extensibility

Build exactly what you need with BASIC scripting and instant API creation:

```basic
' Create a custom API endpoint
WEBHOOK "customer-lookup"

customer_id = params.id
USE KB "customer-data"

response = LLM "Get information about customer " + customer_id

WITH result = NEW OBJECT
    .customer_id = customer_id
    .data = response
    .generated_at = NOW()
END WITH
```

This creates a working API endpoint in seconds—no separate deployment, no infrastructure configuration.

**What you can build:**
- Custom webhooks and APIs
- Automated workflows with BASIC scripts
- Integrations with any external system
- Multi-channel chatbots
- Document processing pipelines
- Scheduled automation tasks

### Knowledge Base & RAG

Full control over your knowledge base with built-in retrieval-augmented generation:

```basic
' Load multiple knowledge sources
USE KB "company-policies"
USE KB "product-catalog"
USE KB "customer-faq"

SET CONTEXT "You are a helpful customer service agent."

answer = LLM user_question

' Save for training and analysis
INSERT "conversations", #{
    question: user_question,
    answer: answer,
    timestamp: NOW()
}
```

**Features:**
- Multiple vector collections
- Custom embedding models
- Semantic search
- Context compaction
- Semantic caching
- Full document indexing

### Multi-Channel Deployment

Deploy once, reach users everywhere:

```basic
' Same code works across all channels
TALK "How can I help you today?"
HEAR question
response = LLM question
TALK response
```

**Supported channels:**
- Web chat
- WhatsApp Business
- Teams
- Slack
- Telegram
- SMS
- Email
- Voice (LiveKit)

### Database & Integration

Direct database access and unlimited API integrations:

```basic
' Direct SQL access
customers = FIND "customers", "region = 'EMEA'"

' REST APIs
data = GET "https://api.example.com/data"
POST "https://api.crm.com/leads", lead_data

' GraphQL
result = GRAPHQL "https://api.github.com/graphql", query, vars
```

No connector marketplace, no per-integration fees—connect to anything with HTTP.

### AI Capabilities

Native AI integration without additional licensing:

| Feature | Implementation |
|---------|---------------|
| Chat assistance | `LLM` keyword |
| Document Q&A | `USE KB` + RAG |
| Image generation | `IMAGE` keyword |
| Video generation | `VIDEO` keyword |
| Speech-to-text | `HEAR AS AUDIO` |
| Text-to-speech | `AUDIO` keyword |
| Vision/OCR | `SEE` keyword |

Use any LLM provider (OpenAI, Anthropic, local models) or run entirely offline with local inference.

## Automation Power

BASIC scripting provides full programming capabilities:

```basic
SET SCHEDULE "every day at 9am"

' Daily report automation
sales = AGGREGATE "orders", "SUM", "total", "date = TODAY()"
count = AGGREGATE "orders", "COUNT", "id", "date = TODAY()"

SET CONTEXT "You are a business analyst."
summary = LLM "Sales: $" + sales + ", Orders: " + count

SEND MAIL TO "team@company.com" SUBJECT "Daily Report" BODY summary
```

**Automation features:**
- Scheduled tasks (cron syntax)
- Event-driven webhooks
- Database triggers
- Conditional logic
- Loops and iterations
- Error handling
- Multi-step workflows

## When General Bots Excels

General Bots is the right choice when you need:

**Custom chatbots** for customer service, internal support, or specialized domains where you control the knowledge base and conversation flow.

**Workflow automation** that goes beyond simple triggers—full programming logic with database access, API calls, and AI integration.

**Multi-channel deployment** where the same bot serves users on web, mobile messaging, and enterprise platforms.

**Data sovereignty** with self-hosted deployment keeping all data on your infrastructure.

**Cost control** without per-user licensing that scales with your organization.

**Integration flexibility** connecting to any system without marketplace limitations.

## Deployment Options

### Self-Hosted

Run General Bots on your own infrastructure:
- Single binary deployment
- Container support (LXC, Docker)
- Scales horizontally
- Full observability

### Quick Start

```bash
./botserver
```

Access at `http://localhost:8080` and start building.

## Summary

General Bots combines:

- **Self-hosting** for complete data control
- **BASIC scripting** for powerful automation
- **Multi-channel** for broad reach
- **Native AI** without extra licensing
- **Open source** for transparency and customization
- **No per-user fees** for predictable costs

For organizations that need more than a simple chatbot—those requiring custom integrations, complex workflows, and full control over their AI deployment—General Bots provides the foundation to build exactly what you need.

## See Also

- [Quick Start](../chapter-01/quick-start.md) - Get running in minutes
- [Keywords Reference](../chapter-06-gbdialog/keywords.md) - Full BASIC reference
- [REST API](../chapter-10-api/README.md) - API documentation
- [Projects](./projects.md) - Team collaboration features