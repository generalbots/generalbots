# Transfer to Human

The `TRANSFER TO HUMAN` keyword enables seamless handoff from bot conversations to human attendants. This is a critical feature for hybrid support workflows where complex issues require human intervention.

## Overview

When a conversation requires human attention—whether due to customer request, issue complexity, or emotional escalation—the bot can transfer the conversation to a human attendant using the `TRANSFER TO HUMAN` keyword.

The system sets `needs_human = true` in the session context, which routes all subsequent messages from that customer to human attendants instead of the bot.

## How It Works

```
Customer Message → Check needs_human
                        ↓
        ┌───────────────┴───────────────┐
        ↓                               ↓
  needs_human=false               needs_human=true
        ↓                               ↓
   Bot Processing              Human Attendant
        ↓                               ↓
  TRANSFER TO HUMAN?            Respond via
        ↓                       Console/WhatsApp
  Set needs_human=true                ↓
        ↓                       /resolve command
  Notify Attendants                   ↓
                              needs_human=false
                                      ↓
                              Back to Bot
```

## Configuration

### Enable CRM Features

Add the following to your bot's `config.csv`:

```csv
name,value

# Required: Enable CRM/Transfer functionality
crm-enabled,true

# Optional: Enable LLM-assisted attendant features
attendant-llm-tips,true
attendant-polish-message,true
attendant-smart-replies,true
attendant-auto-summary,true
attendant-sentiment-analysis,true
```

The `crm-enabled` setting activates:
- Transfer to human functionality
- Attendant queue management
- WebSocket notifications
- LLM assist features (if configured)

### Configure Attendants

Create `attendant.csv` in your bot's `.gbai` folder:

```csv
id,name,channel,preferences,department,aliases
att-001,John Smith,all,sales,commercial,john;johnny;js
att-002,Jane Doe,web,support,customer-service,jane
att-003,Bob Wilson,whatsapp,technical,engineering,bob;bobby
att-004,Maria Santos,all,collections,finance,maria
```

| Column | Description |
|--------|-------------|
| `id` | Unique identifier for the attendant |
| `name` | Display name shown to customers |
| `channel` | Channel they handle: `all`, `web`, `whatsapp`, `teams`, etc. |
| `preferences` | Type of work they prefer |
| `department` | Department for routing |
| `aliases` | Semicolon-separated nicknames for name matching |

---

## The `needs_human` Flag

When `TRANSFER TO HUMAN` is called, the system sets `needs_human = true` in the session's context data. This flag controls message routing:

| `needs_human` Value | Behavior |
|---------------------|----------|
| `false` (default) | Messages go to bot for processing |
| `true` | Messages go to human attendant |

### Checking the Flag in BASIC

```basic
' Check if conversation needs human
IF session.needs_human THEN
    TALK "You're connected to our support team."
ELSE
    TALK "I'm your AI assistant. How can I help?"
END IF
```

### Manual Flag Control (Advanced)

```basic
' Force transfer without using keyword
SET SESSION "needs_human", true
SET SESSION "transfer_reason", "Customer requested human"

' Return to bot mode (usually done by attendant via /resolve)
SET SESSION "needs_human", false
```

---

## Basic Usage

### Transfer to Any Available Attendant

```basic
' Simple transfer to next available human
TRANSFER TO HUMAN

TALK result.message
```

### Transfer to Specific Person

```basic
' Transfer to a specific attendant by name
TRANSFER TO HUMAN "John Smith"

' Also works with aliases
TRANSFER TO HUMAN "johnny"

' Or by ID
TRANSFER TO HUMAN "att-001"
```

### Transfer to Department

```basic
' Transfer to sales department
TRANSFER TO HUMAN "sales"

' Transfer with priority
result = TRANSFER TO HUMAN "support", "high"

IF result.success THEN
    TALK "You are now connected to " + result.assigned_to_name
ELSE
    TALK result.message
END IF
```

### Transfer with Context

```basic
' Transfer with department, priority, and context
TRANSFER TO HUMAN "technical", "urgent", "Customer needs help with API integration"
```

---

## Advanced Usage

### Extended Transfer with Named Parameters

```basic
' Using transfer_to_human_ex for full control
params = #{
    name: "John",
    department: "support",
    priority: "high",
    reason: "Complex billing issue",
    context: "Customer has been a member since 2020, premium tier"
}

result = transfer_to_human_ex(params)

IF result.success THEN
    TALK "Transferring you to " + result.assigned_to_name
    TALK "Estimated wait time: " + result.estimated_wait_seconds + " seconds"
ELSE
    TALK "Sorry, " + result.message
END IF
```

### Conditional Transfer

```basic
' Transfer based on conversation context
sentiment = ANALYZE SENTIMENT conversation

IF sentiment.score < -0.5 THEN
    ' Frustrated customer - high priority
    TRANSFER TO HUMAN "support", "urgent", "Customer appears frustrated"
ELSE IF topic = "billing" THEN
    TRANSFER TO HUMAN "billing"
ELSE IF topic = "technical" THEN
    TRANSFER TO HUMAN "technical"
ELSE
    TRANSFER TO HUMAN
END IF
```

### Check Availability Before Transfer

```basic
' Check if any attendants are available
attendants = GET "/api/attendance/attendants"

available = 0
FOR EACH att IN attendants
    IF att.status = "online" THEN
        available = available + 1
    END IF
NEXT

IF available > 0 THEN
    TRANSFER TO HUMAN
ELSE
    TALK "Our team is currently unavailable. Would you like to:"
    TALK "1. Leave a message"
    TALK "2. Schedule a callback"
    TALK "3. Continue with our AI assistant"
    HEAR choice
END IF
```

---

## Transfer Result

The `TRANSFER TO HUMAN` keyword returns a result object:

| Property | Type | Description |
|----------|------|-------------|
| `success` | Boolean | Whether the transfer was successful |
| `status` | String | Status: `queued`, `assigned`, `connected`, `no_attendants`, `crm_disabled`, `attendant_not_found`, `error` |
| `queue_position` | Integer | Position in queue (if queued) |
| `assigned_to` | String | Attendant ID (if assigned) |
| `assigned_to_name` | String | Attendant name (if assigned) |
| `estimated_wait_seconds` | Integer | Estimated wait time |
| `message` | String | Human-readable message |

### Handling Different Statuses

```basic
result = TRANSFER TO HUMAN "sales"

SELECT CASE result.status
    CASE "assigned"
        TALK "Great news! " + result.assigned_to_name + " will be with you shortly."
        
    CASE "queued"
        TALK "You are #" + result.queue_position + " in line."
        TALK "Estimated wait: " + (result.estimated_wait_seconds / 60) + " minutes."
        
    CASE "connected"
        TALK "You are now connected with " + result.assigned_to_name
        
    CASE "no_attendants"
        TALK "No attendants are currently available."
        TALK "Would you like to leave a message?"
        
    CASE "attendant_not_found"
        TALK "That person is not available. Let me find someone else."
        TRANSFER TO HUMAN
        
    CASE "crm_disabled"
        TALK "I'm sorry, human support is not configured for this bot."
        
    CASE ELSE
        TALK "Something went wrong. Please try again."
END SELECT
```

---

## LLM Tool Integration

The `TRANSFER TO HUMAN` keyword is automatically registered as an LLM tool, allowing the AI to decide when to transfer:

### Tool Schema

```json
{
    "name": "transfer_to_human",
    "description": "Transfer the conversation to a human attendant. Use when the customer explicitly asks to speak with a person, when the issue is too complex, or when emotional support is needed.",
    "parameters": {
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "If someone wants to talk to somebody specific, provide their name or alias"
            },
            "department": {
                "type": "string",
                "description": "Department to transfer to: sales, support, technical, billing, etc."
            },
            "priority": {
                "type": "string",
                "enum": ["normal", "high", "urgent"],
                "default": "normal"
            },
            "reason": {
                "type": "string",
                "description": "Brief reason for the transfer"
            }
        }
    }
}
```

### AI-Initiated Transfer Example

When a customer says "I want to talk to a real person," the LLM can automatically invoke:

```json
{
    "tool": "transfer_to_human",
    "arguments": {
        "reason": "Customer requested human assistance"
    }
}
```

---

## Priority Levels

| Priority | Value | Use Case |
|----------|-------|----------|
| `low` | 0 | Non-urgent inquiries |
| `normal` | 1 | Standard requests (default) |
| `high` | 2 | Important customers, time-sensitive issues |
| `urgent` | 3 | Escalations, complaints, VIP customers |

Higher priority conversations are served first in the queue.

---

## Attendant Status

Attendants can have the following statuses:

| Status | Description |
|--------|-------------|
| `online` | Available and ready for conversations |
| `busy` | Currently handling conversations |
| `away` | Temporarily unavailable |
| `offline` | Not working |

Only `online` attendants receive new conversation assignments.

---

## Queue Status

Conversations in the queue have these statuses:

| Status | Description |
|--------|-------------|
| `waiting` | Waiting for an attendant |
| `assigned` | Assigned but not yet active |
| `active` | Conversation in progress |
| `resolved` | Conversation completed |
| `abandoned` | Customer left before assignment |

---

## REST API Endpoints

### Queue Management

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/attendance/queue` | GET | List conversations in queue |
| `/api/attendance/attendants` | GET | List all attendants |
| `/api/attendance/assign` | POST | Assign conversation to attendant |
| `/api/attendance/transfer` | POST | Transfer between attendants |
| `/api/attendance/resolve/:session_id` | POST | Mark conversation resolved |
| `/api/attendance/insights` | GET | Get queue insights |

### Example: Manual Transfer via API

```basic
' Transfer using direct API call
body = #{
    session_id: session.id,
    from_attendant_id: "att-001",
    to_attendant_id: "att-002",
    reason: "Specialist needed for technical issue"
}

result = POST "/api/attendance/transfer", body
```

---

## Attendant Console

When CRM is enabled, the **Attendant Console** becomes available at `/suite/attendant/`. This provides a full-featured interface for human agents:

### Features

- **Queue Management**: View and filter waiting conversations
- **Real-time Updates**: WebSocket-powered live updates
- **AI Insights**: Sentiment analysis, intent detection, suggested replies
- **Transfer**: Transfer conversations between attendants
- **Customer Details**: View customer history and information
- **Quick Responses**: Pre-configured response templates

### Accessing the Console

1. Enable `crm-enabled,true` in config.csv
2. Create `attendant.csv` with your team
3. Navigate to `/suite/attendant/` or click "Attendant" in the Suite menu

---

## WhatsApp Attendant Mode

Attendants can manage conversations directly from WhatsApp using commands:

### Queue Commands

| Command | Description |
|---------|-------------|
| `/queue` or `/fila` | View waiting conversations |
| `/take` or `/pegar` | Take next conversation |
| `/status [online\|busy\|away\|offline]` | Set availability |
| `/transfer @name` or `/transferir` | Transfer to another attendant |
| `/resolve` or `/resolver` | Mark complete, set `needs_human=false` |

### AI Assist Commands

| Command | Description |
|---------|-------------|
| `/tips` or `/dicas` | Get AI tips for current conversation |
| `/polish <msg>` or `/polir` | Polish message before sending |
| `/replies` or `/respostas` | Get 3 smart reply suggestions |
| `/summary` or `/resumo` | Get conversation summary |
| `/help` or `/ajuda` | Show all commands |

### Example WhatsApp Session

```
Attendant: /queue
Bot: 📋 Queue (2 waiting)
     1. João Silva (whatsapp) - Status: waiting
     2. Maria Santos (web) - Status: waiting

Attendant: /take
Bot: ✅ Conversation assigned
     Customer: João Silva
     Session: abc12345

[Customer message arrives]
Customer: Preciso de ajuda com meu pedido

Attendant: /tips
Bot: 💡 Tips:
     🎯 Customer needs help with order
     ✅ Ask for order number
     📚 Check order status in system

Attendant: /polish oi joao, vou verificar seu pedido agora
Bot: ✨ Polished:
     "Olá João! Vou verificar seu pedido agora mesmo."

Attendant: Olá João! Vou verificar seu pedido agora mesmo.
[Message sent to customer]

Attendant: /resolve
Bot: ✅ Conversation resolved
     Customer returned to bot mode.
```

---

## Best Practices

### 1. Set Clear Expectations

```basic
result = TRANSFER TO HUMAN

IF result.success AND result.status = "queued" THEN
    TALK "You're now in line to speak with a team member."
    TALK "Your position: #" + result.queue_position
    TALK "While you wait, I can still help with simple questions."
END IF
```

### 2. Provide Context to Attendants

```basic
' Build context from conversation
context = "Customer inquiry about: " + detected_topic + ". "
context = context + "Sentiment: " + sentiment + ". "
context = context + "Key entities: " + entities.join(", ")

TRANSFER TO HUMAN "support", "normal", context
```

### 3. Handle Off-Hours

```basic
' Check business hours
hour = HOUR(NOW())
day = WEEKDAY(NOW())

IF day >= 1 AND day <= 5 AND hour >= 9 AND hour < 18 THEN
    TRANSFER TO HUMAN
ELSE
    TALK "Our team is available Monday-Friday, 9 AM - 6 PM."
    TALK "Would you like to leave a message or schedule a callback?"
END IF
```

### 4. VIP Routing

```basic
' Check if customer is VIP
customer = FIND "customers", "email='" + user.email + "'"

IF customer.tier = "premium" OR customer.tier = "enterprise" THEN
    TRANSFER TO HUMAN "vip-support", "high", "Premium customer"
ELSE
    TRANSFER TO HUMAN
END IF
```

---

## Troubleshooting

### "CRM not enabled" Error

Add `crm-enabled,true` to your config.csv file.

### "No attendants configured" Error

Create `attendant.csv` in your bot's `.gbai` folder with at least one attendant.

### Transfer Not Finding Attendant by Name

- Check that the name or alias is spelled correctly
- Verify the attendant exists in `attendant.csv`
- Aliases are case-insensitive and separated by semicolons

### Queue Not Updating

- Ensure WebSocket connection is active
- Check that the attendant status is `online`
- Verify the bot has proper database permissions

---

## Analytics & Insights

The attendance system provides analytics through the API:

### Queue Insights

```basic
GET /api/attendance/insights/{session_id}

Response:
{
    "session_id": "uuid",
    "sentiment": "neutral",
    "message_count": 15,
    "suggested_reply": "How can I help?",
    "key_topics": ["billing", "refund"],
    "priority": "normal",
    "language": "pt"
}
```

### LLM-Powered Analytics

When `attendant-sentiment-analysis` is enabled:

```basic
POST /api/attendance/llm/sentiment

Response:
{
    "overall": "negative",
    "score": -0.6,
    "emotions": [{"name": "frustration", "intensity": 0.8}],
    "escalation_risk": "high",
    "urgency": "high",
    "emoji": "😟"
}
```

---

## Troubleshooting

### Customer Stuck in Human Mode

If a customer is stuck with `needs_human=true` after the issue is resolved:

1. Attendant uses `/resolve` command
2. Or manually via API:
```basic
POST /api/attendance/resolve/{session_id}
```

### Messages Not Reaching Attendant

1. Check `crm-enabled,true` in config.csv
2. Verify attendant.csv exists with valid entries
3. Ensure attendant status is `online`
4. Check WebSocket connection in browser console

### Attendant Commands Not Working on WhatsApp

1. Verify phone number is in attendant.csv
2. Phone must match exactly (with country code)
3. Check that bot is receiving webhooks

---

## See Also

- [LLM-Assisted Attendant](./attendant-llm-assist.md) - AI copilot features
- [Attendance Queue Module](../appendix-external-services/attendance-queue.md) - Full queue configuration
- [Human Approval](../04-basic-scripting/keyword-human-approval.md) - Approval workflows
- [CRM Automations](../appendix-external-services/attendance-queue.md#crm-automations) - Sales, collections, scheduling
- [WhatsApp Setup](../07-user-interface/how-to/connect-whatsapp.md) - Channel configuration