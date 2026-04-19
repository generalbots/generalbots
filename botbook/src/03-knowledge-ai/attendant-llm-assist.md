# LLM-Assisted Attendant Features

General Bots provides AI-powered assistance to human attendants during customer conversations. These features help attendants respond faster, more professionally, and with better context awareness.

## Overview

When the bot transfers a conversation to a human attendant (via `TRANSFER TO HUMAN`), the LLM orchestrator continues working in the background to assist the human. This creates a hybrid experience where AI augments human capability rather than replacing it.

The system uses the same `PROMPT.md` and bot personality configured for the bot, ensuring consistency between bot responses and attendant assistance.

## Features

| Feature | Config Key | Description |
|---------|-----------|-------------|
| **Real-time Tips** | `attendant-llm-tips` | Contextual tips when customer messages arrive |
| **Message Polish** | `attendant-polish-message` | Improve grammar and tone before sending |
| **Smart Replies** | `attendant-smart-replies` | Generate 3 contextual reply suggestions |
| **Auto Summary** | `attendant-auto-summary` | Summarize conversation when attendant joins |
| **Sentiment Analysis** | `attendant-sentiment-analysis` | Real-time emotional state tracking |

## Configuration

Add these settings to your bot's `config.csv`:

```csv
name,value

# Enable all LLM assist features
attendant-llm-tips,true
attendant-polish-message,true
attendant-smart-replies,true
attendant-auto-summary,true
attendant-sentiment-analysis,true

# Optional: Set bot personality for context
bot-system-prompt,You are a friendly customer service assistant for Acme Corp
bot-description,Premium support for enterprise customers
```

### Selective Enablement

Enable only the features you need:

```csv
name,value
attendant-smart-replies,true
attendant-sentiment-analysis,true
```

---

## Feature Details

### 1. Real-time Tips (`attendant-llm-tips`)

When a customer sends a message, the LLM analyzes it and provides actionable tips to the attendant.

#### Tip Types

| Type | Icon | Description |
|------|------|-------------|
| `intent` | 🎯 | What the customer wants |
| `action` | ✅ | Suggested action to take |
| `warning` | ⚠️ | Sentiment or escalation concern |
| `knowledge` | 📚 | Relevant info to share |
| `history` | 📜 | Insight from conversation history |
| `general` | 💡 | General helpful advice |

#### Example Tips

Customer says: *"This is ridiculous! I've been waiting 3 days for a response!"*

Tips generated:
- ⚠️ Customer is frustrated - use empathetic language and apologize
- 🎯 Customer has been waiting for support response
- ✅ Acknowledge the delay and provide immediate assistance

#### API Usage

```basic
' Internal API - automatically called by UI
POST /api/attendance/llm/tips
{
    "session_id": "uuid",
    "customer_message": "message text",
    "history": [{"role": "customer", "content": "..."}]
}
```

---

### 2. Message Polish (`attendant-polish-message`)

Before sending, attendants can polish their message with one click. The LLM improves grammar, clarity, and tone while preserving the original meaning.

#### Supported Tones

- `professional` (default)
- `friendly`
- `empathetic`
- `formal`

#### Example

**Original:** *"ya we can do that but u need to wait til tmrw"*

**Polished:** *"Yes, we can certainly help with that! Please allow until tomorrow for us to process your request."*

**Changes:** Fixed grammar, improved clarity, added professional tone

#### API Usage

```basic
POST /api/attendance/llm/polish
{
    "session_id": "uuid",
    "message": "original message",
    "tone": "professional"
}
```

**Response:**
```json
{
    "success": true,
    "original": "ya we can do that...",
    "polished": "Yes, we can certainly...",
    "changes": ["Fixed grammar", "Improved tone"]
}
```

---

### 3. Smart Replies (`attendant-smart-replies`)

Generate three contextually appropriate reply suggestions based on the conversation history and bot personality.

#### Reply Categories

- `greeting` - Opening responses
- `answer` - Direct answers to questions
- `acknowledgment` - Empathetic acknowledgments
- `solution` - Problem-solving responses
- `follow_up` - Continuation questions
- `closing` - Conversation wrap-up

#### Example

Customer: *"How do I reset my password?"*

**Suggested Replies:**

1. **Empathetic:** "I understand how frustrating it can be when you can't access your account. I'll help you reset your password right away."

2. **Solution-focused:** "To reset your password, please go to the login page and click 'Forgot Password'. You'll receive an email with reset instructions."

3. **Follow-up:** "I can help you with that! Are you trying to reset the password for your main account or a sub-account?"

#### API Usage

```basic
POST /api/attendance/llm/smart-replies
{
    "session_id": "uuid",
    "history": [
        {"role": "customer", "content": "How do I reset my password?"},
        {"role": "attendant", "content": "Hi! Let me help you with that."}
    ]
}
```

---

### 4. Auto Summary (`attendant-auto-summary`)

When an attendant takes a conversation, they receive an instant summary of what's happened so far. This is especially useful for:

- Long conversations
- Transfers between attendants
- Complex multi-issue discussions

#### Summary Contents

| Field | Description |
|-------|-------------|
| `brief` | One-sentence overview |
| `key_points` | Main discussion points |
| `customer_needs` | What the customer wants |
| `unresolved_issues` | Open items |
| `sentiment_trend` | Improving/stable/declining |
| `recommended_action` | What to do next |
| `message_count` | Number of messages |
| `duration_minutes` | Conversation length |

#### Example Summary

```json
{
    "brief": "Customer requesting refund for damaged product received yesterday",
    "key_points": [
        "Order #12345 arrived damaged",
        "Customer sent photos as proof",
        "Previous agent offered replacement"
    ],
    "customer_needs": [
        "Full refund instead of replacement",
        "Confirmation email"
    ],
    "unresolved_issues": [
        "Refund approval pending"
    ],
    "sentiment_trend": "stable",
    "recommended_action": "Escalate to supervisor for refund approval"
}
```

#### API Usage

```basic
GET /api/attendance/llm/summary/{session_id}
```

---

### 5. Sentiment Analysis (`attendant-sentiment-analysis`)

Real-time analysis of customer emotional state to help attendants respond appropriately.

#### Analysis Components

| Component | Values | Description |
|-----------|--------|-------------|
| `overall` | positive, neutral, negative | General sentiment |
| `score` | -1.0 to 1.0 | Numeric sentiment score |
| `emotions` | List | Detected emotions with intensity |
| `escalation_risk` | low, medium, high | Risk of escalation |
| `urgency` | low, normal, high, urgent | Message urgency |
| `emoji` | 😊😐😟 | Visual indicator |

#### Example Analysis

**Customer message:** *"I've been trying to get help for TWO WEEKS! This is absolutely unacceptable!"*

```json
{
    "overall": "negative",
    "score": -0.8,
    "emotions": [
        {"name": "frustration", "intensity": 0.9},
        {"name": "anger", "intensity": 0.7}
    ],
    "escalation_risk": "high",
    "urgency": "high",
    "emoji": "😟"
}
```

The UI shows a warning: ⚠️ High escalation risk detected

#### API Usage

```basic
POST /api/attendance/llm/sentiment
{
    "session_id": "uuid",
    "message": "customer message",
    "history": []
}
```

---

## WhatsApp Attendant Commands

Attendants using WhatsApp can access LLM assist features via commands:

### Queue Management

| Command | Description |
|---------|-------------|
| `/queue` or `/fila` | View waiting conversations |
| `/take` or `/pegar` | Take next conversation |
| `/status [online\|busy\|away\|offline]` | Set your status |
| `/transfer @name` or `/transferir` | Transfer conversation |
| `/resolve` or `/resolver` | Mark as resolved |

### AI Assistance

| Command | Description |
|---------|-------------|
| `/tips` or `/dicas` | Get tips for current conversation |
| `/polish <message>` or `/polir` | Polish a message before sending |
| `/replies` or `/respostas` | Get smart reply suggestions |
| `/summary` or `/resumo` | Get conversation summary |
| `/help` or `/ajuda` | Show all commands |

### Example Usage

```
Attendant: /queue
Bot: 📋 Queue (3 waiting)
     1. John Smith (whatsapp)
        Status: waiting | ID: abc123
     2. Maria Santos (web)
        Status: waiting | ID: def456
     ...

Attendant: /take
Bot: ✅ Conversation assigned
     Customer: John Smith
     Session: abc123
     
Attendant: /tips
Bot: 💡 Tips for this conversation

     🎯 Customer is asking about billing
     ✅ Check their account status before responding
     ⚠️ Previous conversation ended unresolved
     
Attendant: /polish thank u for waiting ill check ur account now
Bot: ✨ Polished message

     "Thank you for your patience! I'll check your account right now."
     
     Changes:
     • Fixed grammar
     • Improved professionalism
     
Attendant: /resolve
Bot: ✅ Conversation resolved
     Session abc123 marked as resolved.
```

---

## Web Attendant Console

The web attendant console (`/suite/attendant/`) integrates LLM assist features directly:

### Insights Panel

The right sidebar shows:
- **Customer Sentiment** - Real-time with emoji indicator
- **Detected Intent** - What the customer wants
- **Conversation Summary** - Auto-generated when you select a conversation

### Suggested Replies

Below the insights, three AI-generated reply suggestions appear. Click any suggestion to insert it into the message input.

### Polish Button

The ✨ button next to the message input polishes your text before sending.

### Tips Notifications

When a customer sends a message:
1. Tips appear in the insights panel
2. High-priority tips show as toast notifications
3. Warning tips (escalation risk) are highlighted

---

## Bot Personality Integration

LLM assist uses your bot's personality when generating suggestions. Set this in `config.csv`:

```csv
name,value
bot-system-prompt,You are a friendly tech support agent for CloudSoft Inc. Be helpful and patient.
bot-description,Enterprise software support
```

Or in your `start.bas` header:

```basic
REM CloudSoft Support Bot
REM Friendly, patient, and technically knowledgeable
REM Always offer to escalate complex issues

TALK "Welcome to CloudSoft Support!"
```

The LLM reads these comments to understand the bot's personality and applies the same tone to:
- Smart reply suggestions
- Message polishing
- Tips generation

---

## API Reference

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/attendance/llm/tips` | Generate tips |
| POST | `/api/attendance/llm/polish` | Polish message |
| POST | `/api/attendance/llm/smart-replies` | Generate replies |
| GET | `/api/attendance/llm/summary/{session_id}` | Get summary |
| POST | `/api/attendance/llm/sentiment` | Analyze sentiment |
| GET | `/api/attendance/llm/config/{bot_id}` | Get config |

### Check Configuration

```basic
GET /api/attendance/llm/config/{bot_id}

Response:
{
    "tips_enabled": true,
    "polish_enabled": true,
    "smart_replies_enabled": true,
    "auto_summary_enabled": true,
    "sentiment_enabled": true,
    "any_enabled": true
}
```

---

## Fallback Behavior

When LLM is unavailable, the system provides fallback functionality:

| Feature | Fallback |
|---------|----------|
| Tips | Keyword-based analysis (urgent, problem, question) |
| Polish | Returns original message |
| Smart Replies | Generic template replies |
| Summary | Basic message count and duration |
| Sentiment | Keyword-based positive/negative detection |

---

## Best Practices

### 1. Start with Smart Replies

If you're unsure which features to enable, start with `attendant-smart-replies`. It provides immediate value with low overhead.

### 2. Enable Sentiment for High-Volume Support

For teams handling many conversations, `attendant-sentiment-analysis` helps prioritize frustrated customers.

### 3. Use Polish for Quality Consistency

Enable `attendant-polish-message` to ensure consistent, professional communication regardless of individual writing skills.

### 4. Tips for Complex Products

For products with many features or complex workflows, `attendant-llm-tips` helps attendants quickly understand context.

### 5. Summary for Shift Changes

Enable `attendant-auto-summary` if your team has shift changes or frequent transfers between attendants.

---

## Troubleshooting

### "Feature is disabled" Message

Add the feature to your `config.csv`:

```csv
attendant-smart-replies,true
```

### Slow Response Times

LLM calls add latency. If responses are slow:
- Use a faster LLM model
- Enable only essential features
- Check your `llm-url` configuration

### Generic Suggestions

If suggestions seem generic:
- Set `bot-system-prompt` in config.csv
- Add personality comments to `start.bas`
- Ensure conversation history is being passed

### WhatsApp Commands Not Working

1. Verify the attendant is registered in `attendant.csv`
2. Check that the phone number matches exactly
3. Ensure `crm-enabled,true` is set

---

## See Also

- [Transfer to Human](./transfer-to-human.md) - Bot-to-human handoff
- [Attendance Queue](../appendix-external-services/attendance-queue.md) - Queue configuration
- [LLM Configuration](../10-configuration-deployment/llm-config.md) - LLM setup
- [config.csv Format](../10-configuration-deployment/config-csv.md) - Configuration reference