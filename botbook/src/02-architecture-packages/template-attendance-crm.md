# Attendance CRM Template (attendance-crm.gbai)

A hybrid AI + Human support template that combines intelligent bot routing with human attendant management and full CRM automation. This template demonstrates the power of General Bots as an LLM orchestrator for customer service operations.

---

## Overview

The Attendance CRM template provides:

- **Intelligent Routing** - Bot analyzes sentiment and auto-transfers frustrated customers
- **LLM-Assisted Attendants** - AI tips, message polish, smart replies for human agents
- **Queue Management** - Automated queue monitoring and load balancing
- **CRM Automations** - Follow-ups, collections, lead nurturing, pipeline management
- **Multi-Channel Support** - Works on WhatsApp, Web, and other channels

## Key Features

| Feature | Description |
|---------|-------------|
| **Sentiment-Based Transfer** | Auto-transfers when customer frustration is detected |
| **AI Copilot for Attendants** | Real-time tips, smart replies, message polishing |
| **Queue Health Monitoring** | Auto-reassign stale conversations, alert supervisors |
| **Automated Follow-ups** | 1-day, 3-day, 7-day follow-up sequences |
| **Collections Workflow** | Payment reminders from due date to legal escalation |
| **Lead Scoring & Nurturing** | Score leads and re-engage cold prospects |
| **Pipeline Management** | Weekly reviews, stale opportunity alerts |

---

## Package Structure

```
attendance-crm.gbai/
├── attendance-crm.gbdialog/
│   ├── start.bas                 # Main entry - intelligent routing
│   ├── queue-monitor.bas         # Queue health monitoring (scheduled)
│   ├── attendant-helper.bas      # LLM assist tools for attendants
│   └── crm-automations.bas       # Follow-ups, collections, nurturing
├── attendance-crm.gbot/
│   └── config.csv                # Bot configuration
└── attendant.csv                 # Attendant team configuration
```

---

## Configuration

### config.csv

```csv
name,value

# Bot Identity
bot-name,Attendance CRM Bot
bot-description,Hybrid AI + Human support with CRM integration

# CRM / Human Handoff - Required
crm-enabled,true

# LLM Assist Features for Attendants
attendant-llm-tips,true
attendant-polish-message,true
attendant-smart-replies,true
attendant-auto-summary,true
attendant-sentiment-analysis,true

# Bot Personality (used for LLM assist context)
bot-system-prompt,You are a professional customer service assistant. Be helpful and empathetic.

# Auto-transfer triggers
auto-transfer-on-frustration,true
auto-transfer-threshold,3

# Queue Settings
queue-timeout-minutes,30
queue-notify-interval,5

# Lead Scoring
lead-score-threshold-hot,70
lead-score-threshold-warm,50

# Follow-up Automation
follow-up-1-day,true
follow-up-3-day,true
follow-up-7-day,true

# Collections Automation
collections-enabled,true
collections-grace-days,3

# Working Hours
business-hours-start,09:00
business-hours-end,18:00
business-days,1-5

# Notifications
notify-on-vip,true
notify-on-escalation,true
notify-email,support@company.com
```

### attendant.csv

Attendants can be identified by **any channel**: WhatsApp phone, email, Microsoft Teams, or Google account.

```csv
id,name,channel,preferences,department,aliases,phone,email,teams,google
att-001,João Silva,all,sales,commercial,joao;js;silva,+5511999990001,joao.silva@company.com,joao.silva@company.onmicrosoft.com,joao.silva@company.com
att-002,Maria Santos,whatsapp,support,customer-service,maria;ms,+5511999990002,santos@company.com,santos@company.onmicrosoft.com,santos@gmail.com
att-003,Pedro Costa,web,technical,engineering,pedro;pc;tech,+5511999990003,pedro.costa@company.com,pedro.costa@company.onmicrosoft.com,pedro.costa@company.com
att-004,Ana Oliveira,all,collections,finance,ana;ao;cobranca,+5511999990004,ana.oliveira@company.com,ana.oliveira@company.onmicrosoft.com,ana.oliveira@company.com
att-005,Carlos Souza,whatsapp,sales,commercial,carlos;cs,+5511999990005,carlos.souza@company.com,carlos.souza@company.onmicrosoft.com,carlos.souza@gmail.com
```

#### Column Reference

| Column | Description | Example |
|--------|-------------|---------|
| `id` | Unique attendant ID | `att-001` |
| `name` | Display name | `João Silva` |
| `channel` | Preferred channels (`all`, `whatsapp`, `web`, `teams`) | `all` |
| `preferences` | Specialization area | `sales`, `support`, `technical` |
| `department` | Department for routing | `commercial`, `engineering` |
| `aliases` | Semicolon-separated nicknames for matching | `joao;js;silva` |
| `phone` | WhatsApp number (E.164 format) | `+5511999990001` |
| `email` | Email address for notifications | `joao@company.com` |
| `teams` | Microsoft Teams UPN | `joao@company.onmicrosoft.com` |
| `google` | Google Workspace email | `joao@company.com` |

The system can find an attendant by **any identifier** - phone, email, Teams UPN, Google account, name, or alias.
```

---

## Scripts

### start.bas - Intelligent Routing

The main entry point analyzes every customer message and decides routing:

```basic
' Analyze sentiment immediately
sentiment = ANALYZE SENTIMENT session.id, message

' Track frustration
IF sentiment.overall = "negative" THEN
    frustration_count = frustration_count + 1
END IF

' Auto-transfer on high escalation risk
IF sentiment.escalation_risk = "high" THEN
    tips = GET TIPS session.id, message
    result = TRANSFER TO HUMAN "support", "urgent", context_summary
END IF
```

**Key behaviors:**
- Analyzes sentiment on every message
- Tracks frustration count across conversation
- Auto-transfers on explicit request ("falar com humano", "talk to human")
- Auto-transfers when escalation risk is high
- Auto-transfers after 3+ negative messages
- Passes AI tips to attendant during transfer

### queue-monitor.bas - Queue Health

Scheduled job that runs every 5 minutes:

```basic
SET SCHEDULE "queue-monitor", "*/5 * * * *"
```

**What it does:**
- Finds conversations waiting >10 minutes → auto-assigns
- Finds inactive assigned conversations → reminds attendant
- Finds conversations with offline attendants → reassigns
- Detects abandoned conversations → sends follow-up, then resolves
- Generates queue metrics for dashboard
- Alerts supervisor if queue gets long or no attendants online

### attendant-helper.bas - LLM Assist Tools

Provides AI-powered assistance to human attendants:

```basic
' Get tips for current conversation
tips = USE TOOL "attendant-helper", "tips", session_id, message

' Polish a message before sending
polished = USE TOOL "attendant-helper", "polish", session_id, message, "empathetic"

' Get smart reply suggestions
replies = USE TOOL "attendant-helper", "replies", session_id

' Get conversation summary
summary = USE TOOL "attendant-helper", "summary", session_id

' Analyze sentiment with recommendations
sentiment = USE TOOL "attendant-helper", "sentiment", session_id, message

' Check if transfer is recommended
should_transfer = USE TOOL "attendant-helper", "suggest_transfer", session_id
```

### crm-automations.bas - Business Workflows

Scheduled CRM automations:

```basic
' Daily follow-ups at 9am weekdays
SET SCHEDULE "follow-ups", "0 9 * * 1-5"

' Daily collections at 8am weekdays
SET SCHEDULE "collections", "0 8 * * 1-5"

' Daily lead nurturing at 10am weekdays
SET SCHEDULE "lead-nurture", "0 10 * * 1-5"

' Weekly pipeline review Friday 2pm
SET SCHEDULE "pipeline-review", "0 14 * * 5"
```

---

## BASIC Keywords Used

### Queue Management

| Keyword | Description | Example |
|---------|-------------|---------|
| `GET QUEUE` | Get queue status and items | `queue = GET QUEUE` |
| `NEXT IN QUEUE` | Get next waiting conversation | `next = NEXT IN QUEUE` |
| `ASSIGN CONVERSATION` | Assign to attendant | `ASSIGN CONVERSATION session_id, "att-001"` |
| `RESOLVE CONVERSATION` | Mark as resolved | `RESOLVE CONVERSATION session_id, "Fixed"` |
| `SET PRIORITY` | Change priority | `SET PRIORITY session_id, "urgent"` |

### Attendant Management

| Keyword | Description | Example |
|---------|-------------|---------|
| `GET ATTENDANTS` | List attendants | `attendants = GET ATTENDANTS "online"` |
| `GET ATTENDANT STATS` | Get performance metrics | `stats = GET ATTENDANT STATS "att-001"` |
| `SET ATTENDANT STATUS` | Change status | `SET ATTENDANT STATUS "att-001", "busy"` |

### LLM Assist

| Keyword | Description | Example |
|---------|-------------|---------|
| `GET TIPS` | Generate AI tips | `tips = GET TIPS session_id, message` |
| `POLISH MESSAGE` | Improve message | `polished = POLISH MESSAGE text, "empathetic"` |
| `GET SMART REPLIES` | Get reply suggestions | `replies = GET SMART REPLIES session_id` |
| `GET SUMMARY` | Conversation summary | `summary = GET SUMMARY session_id` |
| `ANALYZE SENTIMENT` | Sentiment analysis | `sentiment = ANALYZE SENTIMENT session_id, message` |

### Customer Journey

| Keyword | Description | Example |
|---------|-------------|---------|
| `TAG CONVERSATION` | Add tags | `TAG CONVERSATION session_id, "vip"` |
| `ADD NOTE` | Add internal note | `ADD NOTE session_id, "Customer is VIP"` |
| `GET CUSTOMER HISTORY` | Previous interactions | `history = GET CUSTOMER HISTORY user_id` |
| `TRANSFER TO HUMAN` | Handoff to human | `TRANSFER TO HUMAN "support", "high", context` |

---

## Sample Conversations

### Auto-Transfer on Frustration

```
Customer: Olá, preciso de ajuda com meu pedido

Bot: Olá! Claro, ficarei feliz em ajudar com seu pedido.
     Qual é o número do pedido?

Customer: #12345

Bot: Encontrei seu pedido #12345. Está em trânsito e deve
     chegar amanhã. Posso ajudar com mais alguma coisa?

Customer: Amanhã?! Isso é ridículo! Já deveria ter chegado há 3 dias!

Bot: [Sentiment: negative, Escalation Risk: medium]
     Entendo sua frustração. Peço desculpas pelo atraso.
     Se preferir falar com um atendente humano, é só me avisar.

Customer: SIM! Quero falar com alguém de verdade AGORA!

Bot: [Auto-transfer triggered]
     Estou transferindo você para Maria Santos.
     Um momento, por favor.

     [Tips sent to Maria:]
     ⚠️ Customer frustrated - 3 negative messages
     🎯 Issue: Delayed order #12345
     ✅ Offer compensation for delay
```

### Attendant Using LLM Assist

```
[Customer message arrives]
Customer: não consigo acessar minha conta faz 2 dias!!

[AI Tips appear in attendant UI:]
💡 Tips:
   ⚠️ Customer frustrated - use empathetic tone
   🎯 Intent: Account access issue
   ✅ Verify account status, offer password reset

[Attendant types response:]
Attendant: oi, vou verificar sua conta

[Clicks ✨ Polish button:]
Polished: "Olá! Entendo como isso pode ser frustrante.
          Vou verificar sua conta agora mesmo e resolver
          isso para você."

[Attendant sends polished message]
```

---

## Automation Workflows

### Follow-up Sequence

| Day | Action | Template |
|-----|--------|----------|
| 1 | Thank you message | `follow_up_thanks` |
| 3 | Value proposition | `follow_up_value` |
| 7 | Special offer (if score ≥50) | `follow_up_offer` |

### Collections Workflow

| Days Overdue | Action | Escalation |
|--------------|--------|------------|
| 0 (due today) | Friendly reminder | WhatsApp template |
| 3 | First notice | WhatsApp + Email |
| 7 | Second notice | + Notify collections team |
| 15 | Final notice + late fees | + Queue for human call |
| 30+ | Send to legal | + Suspend account |

---

## WhatsApp Templates Required

Configure these in Meta Business Manager:

| Template | Variables | Purpose |
|----------|-----------|---------|
| `follow_up_thanks` | name, interest | 1-day thank you |
| `follow_up_value` | name, interest | 3-day value prop |
| `follow_up_offer` | name, discount | 7-day offer |
| `payment_due_today` | name, invoice_id, amount | Due reminder |
| `payment_overdue_3` | name, invoice_id, amount | 3-day overdue |
| `payment_overdue_7` | name, invoice_id, amount | 7-day overdue |
| `payment_final_notice` | name, invoice_id, total | 15-day final |

---

## Metrics & Analytics

The template automatically tracks:

- **Queue Metrics**: Wait times, queue length, utilization
- **Attendant Performance**: Resolved count, active conversations
- **Sentiment Trends**: Per conversation and overall
- **Automation Results**: Follow-ups sent, collections processed

Access via:
- Dashboard at `/suite/analytics/`
- API at `/api/attendance/insights`
- Stored in `queue_metrics` and `automation_logs` tables

---

## Best Practices

### 1. Configure Sentiment Thresholds

Adjust `auto-transfer-threshold` based on your tolerance:
- `2` = Very aggressive (transfer quickly)
- `3` = Balanced (default)
- `5` = Conservative (try harder with bot)

### 2. Set Business Hours

Configure `business-hours-*` to avoid sending automated messages at night.

### 3. Train Your Team

Ensure attendants know the WhatsApp commands:
- `/tips` - Get AI tips
- `/polish <message>` - Improve message
- `/replies` - Get suggestions
- `/resolve` - Close conversation

### 4. Monitor Queue Health

Set up alerts for:
- Queue > 10 waiting
- No attendants online during business hours
- Average wait > 15 minutes

---

## See Also

- [Transfer to Human](../03-knowledge-ai/transfer-to-human.md) - Handoff details
- [LLM-Assisted Attendant](../03-knowledge-ai/attendant-llm-assist.md) - AI copilot features
- [Sales CRM Template](./template-crm.md) - Full CRM without attendance
- [Attendance Queue Module](../appendix-external-services/attendance-queue.md) - Queue configuration
