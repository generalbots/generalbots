# Botpress Migration Guide

Migrating chatbots from Botpress to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Botpress is an open-source chatbot platform with visual flow builder and NLU. General Bots provides a simpler approach using LLM-based understanding and BASIC scripting, with integrated productivity features and native multi-channel support.

## Why Migrate

| Aspect | Botpress | General Bots |
|--------|----------|--------------|
| NLU Approach | Intent training required | LLM-based (no training) |
| Flow Building | Visual + code | BASIC scripts |
| Self-hosting | Available | Available |
| AI Integration | Via hooks | Native LLM keywords |
| Knowledge Base | Limited | Full RAG system |
| Productivity Suite | Not included | Email, calendar, files, tasks |
| Multi-channel | Via connectors | Native support |
| Learning Curve | Moderate | Simple BASIC |

## Concept Mapping

| Botpress Concept | General Bots Equivalent |
|------------------|------------------------|
| Flows | BASIC scripts |
| Nodes | BASIC statements |
| Intents | LLM understanding |
| Entities | `HEAR AS <type>` |
| Slots | Variables |
| Actions | BASIC keywords |
| Hooks | `ON` triggers |
| Content Types | TALK variations |
| Knowledge Base | `.gbkb` folders |
| Channels | Native multi-channel |

## Flow Migration

### Botpress Flow Structure

```yaml
# Botpress flow (simplified)
nodes:
  - id: entry
    type: standard
    next: ask_name
  - id: ask_name
    type: say_something
    content: "What's your name?"
    next: capture_name
  - id: capture_name
    type: listen
    slot: name
    next: greet
  - id: greet
    type: say_something
    content: "Hello {{name}}!"
```

### General Bots Equivalent

```basic
' Simple and readable
TALK "What's your name?"
HEAR name AS NAME
TALK "Hello " + name + "!"
```

## Migration Examples

### Simple Welcome Flow

**Botpress:**
- Entry node → Say "Welcome" → Listen for intent → Route to sub-flow

**General Bots:**

```basic
USE KB "help-docs"

SET CONTEXT "You are a friendly assistant for Acme Corp."

TALK "Welcome! How can I help you today?"
HEAR question
answer = LLM question
TALK answer
```

### Lead Capture Flow

**Botpress:**
```
Entry → Ask Name → Capture Slot → Ask Email → Capture Slot → 
Ask Company → Capture Slot → Save to CRM → Thank You
```

**General Bots:**

```basic
' lead-capture.bas
TALK "I'd love to learn more about you!"

TALK "What's your name?"
HEAR name AS NAME

TALK "And your work email?"
HEAR email AS EMAIL

TALK "What company are you with?"
HEAR company

' Save directly - no external action needed
INSERT "leads", #{
    name: name,
    email: email,
    company: company,
    source: "chatbot",
    created_at: NOW()
}

' Score the lead
score = SCORE LEAD #{name: name, email: email, company: company}

IF score.status = "hot" THEN
    SEND MAIL TO "sales@company.com" SUBJECT "Hot Lead" BODY "New lead: " + name + " from " + company
END IF

TALK "Thanks, " + name + "! Someone from our team will be in touch soon."
```

### FAQ Bot with Fallback

**Botpress:**
- NLU intent matching
- Knowledge base query
- Fallback to human

**General Bots:**

```basic
USE KB "faq"
USE KB "product-docs"

SET CONTEXT "Answer customer questions helpfully. If you cannot answer confidently, offer to connect with a human."

TALK "What can I help you with?"
HEAR question

answer = LLM question

' Check if confident answer
IF CONTAINS(LOWER(answer), "i don't") OR CONTAINS(LOWER(answer), "not sure") THEN
    TALK "I'm not certain about that. Would you like to speak with someone?"
    HEAR wants_human AS BOOLEAN
    IF wants_human THEN
        CREATE TASK "Customer inquiry: " + question
        SEND MAIL TO "support@company.com" SUBJECT "Chat Handoff" BODY question
        TALK "I've notified our team. Someone will reach out shortly."
    END IF
ELSE
    TALK answer
END IF
```

### Multi-Step Booking Flow

**Botpress:**
```
Select Service → Choose Date → Choose Time → Confirm → Book
(Multiple nodes with slot filling)
```

**General Bots:**

```basic
TALK "Let's book your appointment."

TALK "What service do you need?"
HEAR service AS "Consultation", "Checkup", "Follow-up", "Emergency"

TALK "What date works for you?"
HEAR appt_date AS DATE

TALK "What time?"
HEAR appt_time AS HOUR

' Check availability
available = GET "https://calendar.api/available?date=" + appt_date + "&time=" + appt_time

IF available THEN
    BOOK service AT appt_date + " " + appt_time
    TALK "Your " + service + " is confirmed for " + FORMAT(appt_date, "MMMM d") + " at " + appt_time
ELSE
    TALK "That slot isn't available. Would " + available.next + " work instead?"
END IF
```

## NLU Migration

### Botpress Intents

```yaml
# Botpress intent definition
intents:
  - name: order_status
    utterances:
      - where is my order
      - track my order
      - order status
      - what happened to my order
```

### General Bots Approach

No intent definition needed. The LLM understands naturally:

```basic
USE KB "order-help"
SET CONTEXT "Help customers with their orders."

TALK "How can I help with your order?"
HEAR question

' LLM understands "where is my order", "track order", etc.
' without explicit training
answer = LLM question
```

### Entity Extraction

**Botpress:**
```yaml
entities:
  - name: order_number
    type: pattern
    pattern: "ORD-[0-9]{6}"
```

**General Bots:**

```basic
TALK "What's your order number?"
HEAR order_number

' Or with validation pattern
IF NOT MATCH(order_number, "ORD-[0-9]{6}") THEN
    TALK "Please enter a valid order number (e.g., ORD-123456)"
    HEAR order_number
END IF
```

## Actions Migration

### Botpress Custom Actions

```javascript
// Botpress action
const checkOrderStatus = async (orderId) => {
  const response = await axios.get(`/api/orders/${orderId}`);
  return response.data.status;
};
```

### General Bots

```basic
' Direct API call - no separate action file
order = GET "https://api.company.com/orders/" + order_id
TALK "Your order status is: " + order.status
```

## Hooks Migration

### Botpress Hooks

```javascript
// before_incoming_middleware hook
bp.events.on('before_incoming_middleware', async (event) => {
  // Custom logic
});
```

### General Bots Triggers

```basic
' Event-driven triggers
ON "message:received"
    ' Log all messages
    INSERT "message_log", #{
        content: params.content,
        user: params.user_id,
        timestamp: NOW()
    }
END ON

ON "session:started"
    ' Track new sessions
    INSERT "sessions", #{
        id: params.session_id,
        started: NOW()
    }
END ON
```

## Content Types

### Botpress Content

```javascript
// Botpress content types
{
  type: 'builtin_card',
  title: 'Product',
  image: 'product.jpg',
  actions: [{ title: 'Buy', action: 'buy' }]
}
```

### General Bots

```basic
' Text
TALK "Hello!"

' Image
TALK IMAGE "/products/featured.jpg"

' File
TALK FILE "/docs/brochure.pdf"

' Suggestions
ADD SUGGESTION "View Products"
ADD SUGGESTION "Contact Sales"
ADD SUGGESTION "Get Help"
TALK "What would you like to do?"
```

## Knowledge Base Migration

### Botpress Q&A

Limited to question-answer pairs.

### General Bots RAG

Full document support:

```
my-bot.gbkb/
├── products/
│   ├── catalog.pdf
│   ├── specs.xlsx
│   └── pricing.md
├── support/
│   ├── faq.md
│   └── troubleshooting.md
└── company/
    ├── about.md
    └── policies.pdf
```

```basic
USE KB "products"
USE KB "support"
USE KB "company"

answer = LLM customer_question
```

## Channel Migration

### Botpress Channels

Requires separate connector configuration for each channel.

### General Bots

Native multi-channel with same code:

```basic
' Works everywhere: Web, WhatsApp, Teams, Slack, Telegram, SMS
TALK "How can I help?"
HEAR question
answer = LLM question
TALK answer
```

## Database and State

### Botpress State

```javascript
// Botpress user state
event.state.user.name = 'John';
event.state.session.orderId = '12345';
```

### General Bots

```basic
' Session/conversation memory
SET BOT MEMORY "customer_name", name
SET BOT MEMORY "current_order", order_id

' Retrieve
name = GET BOT MEMORY "customer_name"

' Persistent storage
INSERT "customers", #{name: name, email: email}
customer = FIND "customers", "email = '" + email + "'"
```

## What You Gain

**Simpler Development:** BASIC scripts are more readable than visual flows with scattered code.

**No NLU Training:** LLM understands variations without explicit intent training.

**Native AI:** Full LLM integration without plugins.

**Productivity Suite:** Built-in email, calendar, files, and tasks.

**Unified Platform:** Chat, automation, and productivity in one system.

**True Multi-Channel:** Same code works everywhere without channel-specific configuration.

## Migration Checklist

### Pre-Migration

- [ ] Export Botpress flows and content
- [ ] Document intents and entities
- [ ] List custom actions
- [ ] Export Q&A/knowledge base
- [ ] Note channel configurations

### Migration

- [ ] Set up General Bots environment
- [ ] Create BASIC scripts for main flows
- [ ] Build knowledge base structure
- [ ] Implement entity validation
- [ ] Configure channels
- [ ] Test all flows

### Post-Migration

- [ ] Compare conversation quality
- [ ] Verify integrations
- [ ] Train team
- [ ] Redirect channel endpoints
- [ ] Decommission Botpress

## See Also

- [Dialog Basics](../chapter-06-gbdialog/basics.md) - Script fundamentals
- [HEAR Keyword](../chapter-06-gbdialog/keyword-hear.md) - Input validation
- [Knowledge Base](../chapter-03/README.md) - RAG configuration
- [Platform Comparison](./comparison-matrix.md) - Full comparison