# BASIC vs Automation Tools: A Practical Comparison 🟡 BETA

> **Understanding how General Bots BASIC compares to other automation platforms**

## Overview

General Bots BASIC provides a conversational-first approach to automation. This chapter compares BASIC with popular automation tools to help you understand when each approach works best.

---

## Comparison Matrix

| Feature | Zapier | n8n | Make | Power Automate | **BASIC** |
|---------|--------|-----|------|----------------|-----------|
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ |
| Scheduling | ✅ | ✅ | ✅ | ✅ | ✅ `SET SCHEDULE` |
| HTTP/REST | ✅ | ✅ | ✅ | ✅ | ✅ |
| GraphQL | ❌ | ✅ | ✅ | ❌ | ✅ |
| SOAP | ❌ | ❌ | ✅ | ✅ | ✅ |
| Database Native | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Conversations** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **WhatsApp Native** | Plugin | Plugin | Plugin | Plugin | ✅ Built-in |
| **Telegram Native** | Plugin | Plugin | Plugin | ❌ | ✅ Built-in |
| **Multi-Channel** | Limited | Limited | Limited | Limited | ✅ Native |
| **LLM Integration** | Plugin | Plugin | Plugin | GPT-5 | ✅ Any model |
| **Self-Hosted** | ❌ | ✅ | ❌ | ❌ | ✅ |
| **Open Source** | ❌ | ✅ | ❌ | ❌ | ✅ AGPL |

---

## Key Differences

### Conversation-First Design

Traditional automation tools focus on backend workflows. BASIC adds interactive conversations:

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I'll help you file an expense report. What's the amount?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>$150</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>What category? (travel/meals/supplies)</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>meals</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Upload the receipt photo</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>📎 receipt.jpg</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Expense submitted! Reference: EXP-2025-0142</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

### Multi-Channel Native

The same bot works across all channels without modification:

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Your order has shipped! 📦</p>
      <p>Tracking: 7891234567890</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

This message reaches users on WhatsApp, Telegram, Web, or any configured channel—same content, adapted formatting.

### LLM Model Freedom

BASIC supports any LLM provider:

- OpenAI (GPT-5, o3)
- Anthropic (Claude Sonnet 4.5, Opus 4.5)
- Local models (Llama, Mistral via llama.cpp)
- Groq, DeepSeek, and others
- Any OpenAI-compatible API

Configure in `config.csv`:

```csv
name,value
llm-url,http://localhost:8081
llm-model,model.gguf
```

---

## When to Use Each Tool

### Choose BASIC When You Need

- **Interactive workflows** - Users participate in the process
- **Multi-channel presence** - Same bot on WhatsApp, Telegram, Web
- **AI-powered conversations** - Natural language understanding
- **Self-hosted deployment** - Full data control
- **Open source flexibility** - Modify and extend as needed

### Choose Traditional Automation When You Need

- **Backend-only workflows** - No user interaction required
- **Visual workflow builders** - Prefer drag-and-drop interfaces
- **Existing integrations** - Specific pre-built connectors
- **Team familiarity** - Team already knows the tool

---

## Migration Examples

### From Zapier

Zapier workflow: Form submission → Slack notification → CRM entry → Welcome email

BASIC equivalent:

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>New lead captured! 🎉</p>
      <p>👤 John Smith</p>
      <p>📧 john@example.com</p>
      <p>🏢 Acme Corp</p>
      <p>Added to CRM and notified the team.</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

### From n8n

n8n workflow: Monitor website → Alert on error → Create ticket

BASIC equivalent runs on schedule and notifies immediately:

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔴 Website Alert</p>
      <p>mysite.com returned status 503</p>
      <p>Ticket #IT-2025-0891 created</p>
      <p>DevOps team notified</p>
      <div class="wa-time">03:42</div>
    </div>
  </div>
</div>

---

## Complete Office Suite

BASIC provides built-in capabilities for common office tasks:

| Capability | BASIC Keyword |
|------------|---------------|
| Send email | `SEND MAIL` |
| Create draft | `CREATE DRAFT` |
| Schedule meetings | `BOOK` |
| Manage files | `UPLOAD`, `DOWNLOAD`, `LIST` |
| Create tasks | `CREATE TASK` |
| Video meetings | `CREATE MEETING` |

### Example: Daily Report Automation

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 Daily Sales Report - May 15, 2025</p>
      <p>━━━━━━━━━━━━━━━━━━━━━</p>
      <p>💰 Revenue: $15,340</p>
      <p>📦 Orders: 47</p>
      <p>📈 +12% vs yesterday</p>
      <p>━━━━━━━━━━━━━━━━━━━━━</p>
      <p>Report sent to executives@company.com</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
</div>

---

## Getting Started

### Quick Start

1. Download and run botserver
2. Edit your bot's `.bas` files
3. Configure settings in `config.csv`
4. Deploy to any channel

### Resources

- [Keywords Reference](./keywords.md) - Complete keyword documentation
- [SET SCHEDULE](./keyword-set-schedule.md) - Automate with schedules
- [WEBHOOK](./keyword-webhook.md) - Event-driven automation
- [Templates](./templates.md) - Ready-to-use examples

