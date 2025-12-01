# Intercom Migration Guide

Migrating customer messaging and support from Intercom to General Bots.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Intercom is a customer messaging platform with live chat, chatbots, and help desk features. General Bots provides equivalent capabilities with self-hosting, no per-seat pricing, and native AI integration.

## Why Migrate

| Aspect | Intercom | General Bots |
|--------|----------|--------------|
| Pricing | $39-139/seat/month | No per-seat fees |
| Hosting | Cloud only | Self-hosted |
| AI Features | Fin AI ($0.99/resolution) | Native LLM (any provider) |
| Channels | Web, email, mobile | Web, WhatsApp, Teams, Slack, SMS, more |
| Automation | Limited workflows | Full BASIC scripting |
| Knowledge Base | Included | Built-in RAG |
| Data Ownership | Their servers | Your infrastructure |
| Customization | Limited | Full source access |

## Cost Comparison

### Intercom Pricing (per seat/month)

| Plan | Cost | Features |
|------|------|----------|
| Essential | $39 | Basic chat, inbox |
| Advanced | $99 | Automation, reporting |
| Expert | $139 | Full platform |
| Fin AI | $0.99/resolution | AI answers |

For a team of 10 support agents, Intercom costs between $990-1,390 per month plus AI costs.

### General Bots

| Component | Cost |
|-----------|------|
| Software | $0 |
| Infrastructure | $50-200/month |
| LLM API (optional) | Usage-based |

The same 10-agent team would spend approximately $100-300 per month total with General Bots.

## Feature Mapping

### Core Features

| Intercom Feature | General Bots Equivalent |
|------------------|------------------------|
| Messenger | Web chat widget |
| Inbox | Conversation management |
| Help Center | Knowledge base (.gbkb) |
| Bots | BASIC dialog scripts |
| Product Tours | Guided conversations |
| Outbound Messages | Automated messaging |

### Bot Capabilities

| Intercom Bots | General Bots Equivalent |
|---------------|------------------------|
| Custom Bots | BASIC scripts |
| Resolution Bot | LLM + USE KB |
| Task Bots | Automated workflows |
| Qualification Bots | HEAR AS + lead scoring |
| Article Suggestions | RAG responses |

## Migration Process

### Step 1: Export Intercom Data

Begin by exporting your data from Intercom. Navigate to Settings, then Data Management, and export conversations, contacts, and articles. Download your Help Center articles separately and export any custom attributes and tags you've configured.

### Step 2: Migrate Knowledge Base

Convert your Help Center articles to a General Bots knowledge base structure:

```
my-bot.gbkb/
├── getting-started/
│   ├── quick-start.md
│   └── setup-guide.md
├── features/
│   ├── feature-overview.md
│   └── tutorials.md
├── troubleshooting/
│   ├── common-issues.md
│   └── faq.md
└── billing/
    ├── plans.md
    └── payments.md
```

### Step 3: Create Support Bot

```basic
' support-bot.bas
' Main customer support entry point

USE KB "getting-started"
USE KB "features"
USE KB "troubleshooting"
USE KB "billing"

SET CONTEXT "You are a friendly customer support assistant.
- Be helpful and concise
- If you cannot answer, offer to connect with a human
- Always maintain a professional, positive tone"

TALK "Hi! I'm here to help. What can I assist you with today?"

LOOP
    HEAR question
    
    ' Check for handoff request
    IF CONTAINS(LOWER(question), "human") OR CONTAINS(LOWER(question), "agent") OR CONTAINS(LOWER(question), "person") THEN
        CALL REQUEST_HUMAN_HANDOFF()
        EXIT LOOP
    END IF
    
    answer = LLM question
    TALK answer
    
    TALK "Is there anything else I can help you with?"
LOOP
```

### Step 4: Implement Human Handoff

```basic
SUB REQUEST_HUMAN_HANDOFF()
    TALK "I'll connect you with a support agent. Let me gather some information first."
    
    TALK "What's your email address?"
    HEAR email AS EMAIL
    
    TALK "Please briefly describe your issue:"
    HEAR issue_summary
    
    ' Create support ticket
    ticket_id = INSERT "support_tickets", #{
        customer_email: email,
        summary: issue_summary,
        conversation_id: session.id,
        status: "pending",
        created_at: NOW()
    }
    
    ' Notify support team
    SEND MAIL TO "support@company.com" SUBJECT "New Support Request #" + ticket_id BODY "Customer: " + email + "\n\nIssue: " + issue_summary
    
    POST GET CONFIG "slack-support", #{
        text: "New support request from " + email + ": " + issue_summary
    }
    
    TALK "Thanks! A support agent will reach out to you at " + email + " shortly. Your ticket number is #" + ticket_id
END SUB
```

## Recreating Intercom Features

### Messenger Widget

General Bots provides embeddable chat widgets that you can add to your website:

```html
<!-- Embed in your website -->
<script src="https://your-bot-server/widget.js"></script>
<script>
  GeneralBots.init({
    botId: 'your-bot-id',
    position: 'bottom-right',
    greeting: 'Hi! How can we help?'
  });
</script>
```

### Qualification Bot

Where Intercom uses a qualification workflow, General Bots achieves the same result through BASIC scripts:

```basic
' lead-qualification.bas
PARAM source AS string

DESCRIPTION "Qualify incoming leads"

TALK "Welcome! I'd love to learn more about you."

TALK "What's your name?"
HEAR name AS NAME

TALK "And your work email?"
HEAR email AS EMAIL

TALK "What company are you with?"
HEAR company

TALK "What's your role?"
HEAR role AS "Executive", "Manager", "Individual Contributor", "Student", "Other"

TALK "What brings you here today?"
HEAR interest AS "Product Demo", "Pricing", "Support", "Partnership", "Just Exploring"

' Score the lead
WITH lead_data
    .name = name
    .email = email
    .company = company
    .role = role
    .interest = interest
    .source = source
END WITH

score = SCORE LEAD lead_data

' Route based on qualification
IF score.status = "hot" OR interest = "Product Demo" THEN
    TALK "Great! Let me schedule a demo for you."
    TALK "When works best?"
    HEAR preferred_time
    
    SEND MAIL TO "sales@company.com" SUBJECT "Hot Lead - Demo Request" BODY lead_data
    CREATE TASK "Demo call with " + name DUE DATEADD(NOW(), 1, "day")
    
    TALK "Our team will reach out within 24 hours to confirm your demo!"
    
ELSEIF interest = "Pricing" THEN
    USE KB "pricing"
    pricing_info = LLM "Provide a brief pricing overview"
    TALK pricing_info
    TALK "Would you like to speak with someone about your specific needs?"
    
ELSE
    USE KB "getting-started"
    TALK "Here's what you can do to get started..."
    answer = LLM "Give a brief getting started guide"
    TALK answer
END IF

INSERT "leads", lead_data
```

### Proactive Messages

Intercom's outbound messages translate to scheduled BASIC scripts in General Bots:

```basic
' proactive-engagement.bas
SET SCHEDULE "every hour"

' Find users who might need help
inactive_sessions = FIND "sessions", "last_activity < DATEADD(NOW(), -5, 'minute') AND page_views > 3 AND not contacted"

FOR EACH session IN inactive_sessions
    ' Send proactive message
    SEND TO session.id MESSAGE "Need any help? I'm here if you have questions!"
    UPDATE "sessions", "id = '" + session.id + "'", #{contacted: true}
NEXT session
```

### Resolution Bot (AI Answers)

While Intercom's Fin charges $0.99 per resolution, General Bots provides the same capability at no additional cost:

```basic
' ai-resolution.bas
USE KB "help-center"
USE KB "product-docs"
USE KB "faq"

SET CONTEXT "You are a helpful support assistant. Answer questions accurately based on the knowledge base. If you're not confident in the answer, say so and offer to connect with a human."

TALK "How can I help you today?"
HEAR question

answer = LLM question

' Check confidence (you can implement confidence scoring)
IF CONTAINS(answer, "I'm not sure") OR CONTAINS(answer, "I don't have") THEN
    TALK answer
    TALK "Would you like me to connect you with a support agent?"
    HEAR wants_human AS BOOLEAN
    IF wants_human THEN
        CALL REQUEST_HUMAN_HANDOFF()
    END IF
ELSE
    TALK answer
    
    ' Track resolution
    INSERT "resolutions", #{
        question: question,
        answer: answer,
        resolved: true,
        timestamp: NOW()
    }
END IF
```

### Customer Segments

Intercom's user segments become database queries and scheduled scripts in General Bots:

```basic
' segment-customers.bas
SET SCHEDULE "every day at 6am"

customers = FIND "customers", "1=1"

FOR EACH customer IN customers
    segment = "standard"
    
    IF customer.total_spent > 10000 THEN
        segment = "enterprise"
    ELSEIF customer.total_spent > 1000 THEN
        segment = "premium"
    ELSEIF customer.signup_date > DATEADD(NOW(), -30, "day") THEN
        segment = "new"
    ELSEIF customer.last_activity < DATEADD(NOW(), -90, "day") THEN
        segment = "at-risk"
    END IF
    
    UPDATE "customers", "id = '" + customer.id + "'", #{segment: segment}
NEXT customer
```

### Targeted Campaigns

```basic
' win-back-campaign.bas
SET SCHEDULE "every monday at 10am"

' Find at-risk customers
at_risk = FIND "customers", "segment = 'at-risk' AND not win_back_sent"

FOR EACH customer IN at_risk
    USE KB "product-updates"
    personalized_message = LLM "Write a brief, friendly win-back message for " + customer.name + " who hasn't used our product in 3 months. Mention recent improvements."
    
    SEND MAIL TO customer.email SUBJECT "We miss you, " + customer.name + "!" BODY personalized_message
    
    UPDATE "customers", "id = '" + customer.id + "'", #{win_back_sent: true, win_back_date: NOW()}
NEXT customer
```

## Multi-Channel Support

### Intercom Channels

Intercom supports Web Messenger, Mobile SDK, Email, and SMS as an add-on.

### General Bots Channels

All channels use the same BASIC scripts, making development and maintenance simpler:

```basic
' Same bot works everywhere
USE KB "support"

TALK "How can I help?"
HEAR question
answer = LLM question
TALK answer

' Channel-specific handling if needed
IF channel = "whatsapp" THEN
    ' WhatsApp-specific features
ELSEIF channel = "email" THEN
    ' Email formatting
END IF
```

General Bots supports web chat, WhatsApp Business, Teams, Slack, Telegram, SMS, Email, and voice through LiveKit.

## Reporting and Analytics

### Conversation Metrics

```basic
' daily-metrics.bas
SET SCHEDULE "every day at 11pm"

today = FORMAT(NOW(), "yyyy-MM-dd")

conversations = AGGREGATE "conversations", "COUNT", "id", "DATE(created_at) = '" + today + "'"
resolutions = AGGREGATE "resolutions", "COUNT", "id", "DATE(timestamp) = '" + today + "' AND resolved = true"
avg_response_time = AGGREGATE "conversations", "AVG", "first_response_seconds", "DATE(created_at) = '" + today + "'"

WITH daily_report
    .date = today
    .total_conversations = conversations
    .ai_resolutions = resolutions
    .resolution_rate = ROUND(resolutions / conversations * 100, 1)
    .avg_response_time = ROUND(avg_response_time / 60, 1)
END WITH

INSERT "daily_metrics", daily_report

SEND MAIL TO "support-lead@company.com" SUBJECT "Daily Support Metrics - " + today BODY daily_report
```

## Migration Checklist

### Pre-Migration

Before beginning the migration, export all Intercom data including conversations, contacts, and articles. Document your custom bot workflows so you can recreate them in BASIC. List all integrations that connect to Intercom. Note any custom attributes and tags you use. Set up your General Bots environment with the necessary infrastructure.

### Migration

During the migration phase, convert your Help Center content to the .gbkb structure. Create support bot scripts that replicate your Intercom workflows. Implement the human handoff flow for seamless escalation. Set up notification channels for your support team. Configure the chat widget for your website. Import customer data from your Intercom export.

### Post-Migration

After migration, test all conversation flows to ensure they work correctly. Verify knowledge base accuracy by asking common questions. Train your support team on the new interface. Run parallel support briefly by keeping both systems active. Once validated, redirect the widget embed code to General Bots and cancel your Intercom subscription.

## What You Gain

Migrating to General Bots provides several significant advantages. There is no per-seat pricing, so you can add unlimited agents without increasing costs. Native AI comes without per-resolution fees since you can use any LLM provider. Full customization is possible because you have complete source access to modify any aspect of the system. Data ownership means all conversations stay on your infrastructure. Automation power lets you go beyond simple workflows with full BASIC scripting. Multi-channel support is native, meaning the same bot works across all channels without add-ons.

## See Also

- [Projects](../chapter-11-features/projects.md) - Organizing support queues
- [HEAR Validation](../chapter-06-gbdialog/keyword-hear.md) - Input validation
- [Lead Scoring](../chapter-06-gbdialog/keywords-lead-scoring.md) - Qualification
- [Platform Comparison](./comparison-matrix.md) - Full feature comparison