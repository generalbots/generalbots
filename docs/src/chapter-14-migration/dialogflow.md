# Dialogflow Migration Guide

Migrating chatbots and conversational agents from Dialogflow to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Dialogflow is Google's conversational AI platform for building chatbots with intent-based NLU. General Bots provides a simpler, more powerful approach using LLM-based understanding and BASIC scripting—without cloud lock-in or complex intent management.

## Why Migrate

| Aspect | Dialogflow | General Bots |
|--------|------------|--------------|
| Hosting | Google Cloud only | Self-hosted |
| Pricing | Per-request fees | No per-request costs |
| NLU Approach | Intent + entity training | LLM-based (zero training) |
| Fulfillment | Cloud Functions/webhooks | Native BASIC scripts |
| Knowledge Base | Limited connector | Full RAG system |
| Channels | Via integrations | Native multi-channel |
| Customization | Limited | Full source access |
| Maintenance | Intent training required | LLM handles variations |

## Cost Comparison

### Dialogflow Pricing

| Edition | Cost |
|---------|------|
| ES (Standard) | Free tier + $0.002/request |
| CX | $0.007/request |
| Mega Agent | $0.06/request |

**10,000 requests/month:** $20-600/month

### General Bots

| Component | Cost |
|-----------|------|
| Software | $0 |
| Infrastructure | $50-200/month |
| LLM API | Usage-based (typically lower) |

## Architecture Comparison

### Dialogflow Architecture

```
User → Dialogflow Agent → Intent Matching → Fulfillment Webhook → Response
                ↓
         Entity Extraction
                ↓
         Context Management
```

### General Bots Architecture

```
User → BASIC Script → LLM Processing → Response
             ↓
      Knowledge Base (RAG)
             ↓
      Direct Actions (DB, API, etc.)
```

## Concept Mapping

### Intents to BASIC

| Dialogflow Concept | General Bots Equivalent |
|--------------------|------------------------|
| Intent | LLM understanding + conditions |
| Training Phrases | Not needed (LLM handles) |
| Entity | `HEAR AS <type>` |
| Context | `SET CONTEXT` / `SET BOT MEMORY` |
| Fulfillment | Direct BASIC code |
| Follow-up Intent | Conversation flow |
| Event | `ON` triggers |
| Knowledge Connector | `USE KB` |

### Entity Types

| Dialogflow Entity | General Bots HEAR AS |
|-------------------|---------------------|
| @sys.date | `HEAR AS DATE` |
| @sys.time | `HEAR AS HOUR` |
| @sys.number | `HEAR AS INTEGER` / `FLOAT` |
| @sys.email | `HEAR AS EMAIL` |
| @sys.phone-number | `HEAR AS MOBILE` |
| @sys.currency-name | `HEAR AS MONEY` |
| @sys.person | `HEAR AS NAME` |
| Custom entity | Menu options or LLM extraction |

## Migration Examples

### Simple FAQ Bot

**Dialogflow:**
- Intent: "hours" with training phrases
- Response: "We're open 9 AM to 5 PM"

**General Bots:**

```basic
USE KB "company-info"

SET CONTEXT "You are a helpful assistant for Acme Corp. Answer questions about our business."

TALK "Hi! How can I help you today?"
HEAR question
answer = LLM question
TALK answer
```

The LLM understands "hours", "when are you open", "opening times", etc. without explicit training.

### Order Status Bot

**Dialogflow:**
```
Intent: order.status
Training phrases: "where is my order", "track order", "order status"
Entity: @order_number
Fulfillment: Webhook to order API
```

**General Bots:**

```basic
' order-status.bas
SET CONTEXT "You help customers check their order status."

TALK "I can help you track your order. What's your order number?"
HEAR order_number

' Direct API call - no webhook needed
SET HEADER "Authorization", "Bearer " + GET CONFIG "orders-api-key"
order = GET "https://api.company.com/orders/" + order_number

IF order.error THEN
    TALK "I couldn't find that order. Please check the number and try again."
ELSE
    TALK "Your order #" + order_number + " is " + order.status + "."
    
    IF order.status = "shipped" THEN
        TALK "Tracking number: " + order.tracking
        TALK "Expected delivery: " + FORMAT(order.delivery_date, "MMMM d")
    END IF
END IF

TALK "Is there anything else I can help with?"
```

### Appointment Booking

**Dialogflow:**
```
Intent: book.appointment
Entities: @sys.date, @sys.time, @service_type
Slot filling for required parameters
Fulfillment: Calendar API webhook
```

**General Bots:**

```basic
' appointment-booking.bas
SET CONTEXT "You help customers book appointments."

TALK "I'd be happy to help you book an appointment."

TALK "What type of service do you need?"
HEAR service AS "Consultation", "Follow-up", "New Patient", "Urgent Care"

TALK "What date works for you?"
HEAR appointment_date AS DATE

TALK "And what time?"
HEAR appointment_time AS HOUR

' Check availability
available = GET "https://api.calendar.com/check?date=" + appointment_date + "&time=" + appointment_time

IF available.open THEN
    ' Book directly
    BOOK service + " Appointment" AT appointment_date + " " + appointment_time
    
    TALK "Perfect! Your " + service + " appointment is confirmed for " + FORMAT(appointment_date, "MMMM d") + " at " + appointment_time
    
    ' Send confirmation
    TALK "What email should I send the confirmation to?"
    HEAR email AS EMAIL
    
    SEND MAIL TO email SUBJECT "Appointment Confirmation" BODY "Your " + service + " is scheduled for " + appointment_date
ELSE
    TALK "That time isn't available. How about " + available.next_slot + "?"
    HEAR confirm AS BOOLEAN
    ' ... continue flow
END IF
```

### Multi-Turn Conversation

**Dialogflow:**
- Follow-up intents
- Context management
- Lifespan settings

**General Bots:**

```basic
' pizza-order.bas
SET CONTEXT "You help customers order pizza."

TALK "Welcome to Pizza Bot! What would you like to order?"

' Size
TALK "What size pizza?"
HEAR size AS "Small", "Medium", "Large", "Extra Large"

' Type
TALK "What type would you like?"
HEAR pizza_type AS "Pepperoni", "Margherita", "Supreme", "Hawaiian", "Custom"

IF pizza_type = "Custom" THEN
    TALK "What toppings would you like? (comma separated)"
    HEAR toppings
END IF

' Confirm
TALK "So that's a " + size + " " + pizza_type + " pizza. Is that correct?"
HEAR confirmed AS BOOLEAN

IF confirmed THEN
    ' Store order
    order_id = INSERT "orders", #{
        size: size,
        type: pizza_type,
        toppings: toppings,
        status: "pending",
        created_at: NOW()
    }
    
    TALK "Great! Your order #" + order_id + " has been placed."
    TALK "Would you like to add anything else?"
    HEAR add_more AS BOOLEAN
    
    IF add_more THEN
        ' Continue ordering
    ELSE
        TALK "What's your delivery address?"
        HEAR address
        ' ... complete order
    END IF
ELSE
    TALK "No problem, let's start over."
END IF
```

## Migrating Fulfillment Code

### Dialogflow Webhook

```javascript
// Dialogflow fulfillment
exports.webhook = (req, res) => {
  const intent = req.body.queryResult.intent.displayName;
  const params = req.body.queryResult.parameters;
  
  if (intent === 'order.status') {
    const orderId = params.order_number;
    // Call API
    fetch(`https://api.example.com/orders/${orderId}`)
      .then(response => response.json())
      .then(order => {
        res.json({
          fulfillmentText: `Your order is ${order.status}`
        });
      });
  }
};
```

### General Bots Equivalent

```basic
' The logic is inline - no separate webhook needed
order = GET "https://api.example.com/orders/" + order_id
TALK "Your order is " + order.status
```

## Knowledge Base Migration

### Dialogflow Knowledge Connector

Limited to FAQ format, requires Google Cloud.

### General Bots Knowledge Base

Full document support with RAG:

```
my-bot.gbkb/
├── products/
│   ├── catalog.pdf
│   └── specifications.xlsx
├── support/
│   ├── faq.md
│   └── troubleshooting.md
└── policies/
    ├── returns.pdf
    └── warranty.md
```

```basic
USE KB "products"
USE KB "support"
USE KB "policies"

answer = LLM customer_question
```

## Context Migration

### Dialogflow Contexts

```javascript
// Setting context in fulfillment
outputContexts: [{
  name: `projects/.../contexts/order-context`,
  lifespanCount: 5,
  parameters: { orderId: '12345' }
}]
```

### General Bots Memory

```basic
' Store context
SET BOT MEMORY "current_order_id", order_id
SET BOT MEMORY "customer_name", customer_name

' Retrieve context
order_id = GET BOT MEMORY "current_order_id"
```

## Multi-Channel Deployment

### Dialogflow Integrations

Requires separate configuration for each channel:
- Web: Dialogflow Messenger
- Telephony: CCAI
- Other: Custom integrations

### General Bots

Same code works everywhere:

```basic
' Works on Web, WhatsApp, Teams, Slack, Telegram, SMS
TALK "How can I help?"
HEAR question
USE KB "support"
answer = LLM question
TALK answer
```

## Advanced Features

### Small Talk

**Dialogflow:** Enable small talk prebuilt agent

**General Bots:** LLM handles naturally

```basic
SET CONTEXT "You are a friendly assistant. Engage in casual conversation when appropriate while staying helpful."

' LLM naturally handles:
' - "Hello"
' - "How are you?"
' - "Thanks"
' - "Goodbye"
```

### Sentiment Analysis

**Dialogflow:** Enable sentiment in settings

**General Bots:**

```basic
HEAR customer_message

sentiment = LLM "Analyze the sentiment of this message and respond with: positive, neutral, or negative. Message: " + customer_message

IF sentiment = "negative" THEN
    SET CONTEXT "The customer seems frustrated. Be extra helpful and empathetic."
    ' Or escalate
    CREATE TASK "Review negative sentiment conversation" 
END IF

answer = LLM customer_message
TALK answer
```

### Rich Responses

**Dialogflow:** Card, suggestion chips, etc.

**General Bots:**

```basic
' Suggestions
ADD SUGGESTION "Check Order"
ADD SUGGESTION "Track Shipment"
ADD SUGGESTION "Contact Support"
TALK "What would you like to do?"

' Images
TALK IMAGE "/products/featured.jpg"

' Files
TALK FILE "/documents/brochure.pdf"
```

## Migration Checklist

### Pre-Migration

- [ ] Export Dialogflow agent (JSON)
- [ ] Document all intents and training phrases
- [ ] List entities and their values
- [ ] Map fulfillment webhooks
- [ ] Identify knowledge connectors
- [ ] Note channel integrations

### Migration

- [ ] Set up General Bots environment
- [ ] Create knowledge base from FAQs/docs
- [ ] Build BASIC scripts for main flows
- [ ] Implement entity validation with HEAR AS
- [ ] Convert fulfillment logic to BASIC
- [ ] Configure channels

### Post-Migration

- [ ] Test all conversation flows
- [ ] Compare response quality
- [ ] Verify API integrations
- [ ] Train team on new system
- [ ] Redirect channel integrations
- [ ] Decommission Dialogflow agent

## What You Gain

**No Intent Training:** LLM understands variations without explicit training phrases.

**Simpler Architecture:** Logic lives in BASIC scripts, not spread across intents and webhooks.

**Self-Hosted:** No Google Cloud dependency or per-request fees.

**Native Integrations:** Direct API calls and database access without webhook complexity.

**Full RAG:** Rich knowledge base support beyond simple FAQ.

**Multi-Channel Native:** Deploy everywhere with one codebase.

## See Also

- [HEAR Keyword](../chapter-06-gbdialog/keyword-hear.md) - Input validation (replaces entities)
- [SET CONTEXT](../chapter-06-gbdialog/keyword-set-context.md) - AI behavior configuration
- [Knowledge Base](../chapter-03/README.md) - RAG setup
- [Platform Comparison](./comparison-matrix.md) - Full feature comparison