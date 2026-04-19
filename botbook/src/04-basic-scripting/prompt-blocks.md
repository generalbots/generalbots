# Prompt Blocks: BEGIN SYSTEM PROMPT & BEGIN TALK

Prompt blocks are special multi-line constructs in General Bots BASIC that define AI behavior and formatted user messages. Unlike regular keywords, these blocks preserve formatting, line breaks, and support rich content.

## Overview

| Block | Purpose | When Processed |
|-------|---------|----------------|
| `BEGIN SYSTEM PROMPT` | Define AI personality, rules, and capabilities | Bot initialization |
| `BEGIN TALK` | Display formatted multi-line messages | Runtime |

---

## BEGIN SYSTEM PROMPT / END SYSTEM PROMPT

Defines the AI's behavior, personality, constraints, and available capabilities. This is the "instruction manual" for the LLM.

### Syntax

```basic
BEGIN SYSTEM PROMPT
Your system prompt content here.
Multiple lines are supported.
Formatting is preserved.
END SYSTEM PROMPT
```

### Purpose

The system prompt sets the AI's persona and tone, defines rules and constraints, lists available tools and capabilities, specifies response formats, and provides domain knowledge. This block serves as the foundation for how the AI will interact with users throughout the conversation.

### Complete Example

```basic
' start.bas - with comprehensive system prompt

ADD TOOL "create-order"
ADD TOOL "track-shipment"
ADD TOOL "lookup-product"

USE KB "products"
USE KB "policies"

BEGIN SYSTEM PROMPT
You are a helpful e-commerce assistant for AcmeStore.

## Your Persona
- Friendly but professional
- Patient with confused customers
- Proactive in offering help

## Your Capabilities
You have access to these tools:
- create-order: Create new orders for customers
- track-shipment: Track order shipments by order ID
- lookup-product: Search product catalog

## Rules
1. Always greet customers warmly
2. Never discuss competitor products
3. For refunds, collect order number first
4. Prices are in USD unless customer specifies otherwise
5. If unsure, ask clarifying questions rather than guessing

## Response Format
- Keep responses under 100 words unless detailed explanation needed
- Use bullet points for lists
- Include relevant product links when available

## Escalation
If customer is frustrated or issue is complex, offer to connect with human support.

## Knowledge
You have access to:
- Complete product catalog (products KB)
- Return and shipping policies (policies KB)
- Current promotions and discounts
END SYSTEM PROMPT

' Continue with welcome message...
```

### Best Practices

A well-crafted system prompt should be specific about capabilities and constraints. Rather than writing a vague prompt like "You are a helpful assistant," provide detailed context about available tools, behavioral rules, and response expectations:

```basic
BEGIN SYSTEM PROMPT
You are a medical appointment scheduler.

Available tools:
- book-appointment: Schedule new appointments
- cancel-appointment: Cancel existing appointments
- check-availability: View available time slots

Rules:
1. Always confirm patient identity before accessing records
2. Appointments require 24-hour advance notice
3. Emergency cases should be directed to call 911

You can access patient records and doctor schedules through the connected systems.
END SYSTEM PROMPT
```

### Placement

Place `BEGIN SYSTEM PROMPT` near the top of `start.bas`, after tool and KB registration. Register tools first, then load knowledge bases, then define the system prompt, and finally include your welcome message. This ordering ensures all capabilities are available when the system prompt references them.

---

## BEGIN TALK / END TALK

Displays formatted multi-line messages to users with preserved formatting, Markdown support, and emoji rendering.

### Syntax

```basic
BEGIN TALK
Your message content here.
Multiple lines supported.
**Markdown** formatting works.
Emojis render: 🎉 ✅ 📦
END TALK
```

### Purpose

Use `BEGIN TALK` for welcome messages, formatted instructions, multi-line responses, messages with bullet points or structure, and content with emojis or special formatting.

### Basic Example

```basic
BEGIN TALK
**Welcome to AcmeStore!** 🛒

I can help you with:
• Browsing products
• Placing orders
• Tracking shipments
• Returns and refunds

What would you like to do today?
END TALK
```

### Markdown Support

`BEGIN TALK` supports common Markdown syntax:

```basic
BEGIN TALK
# Main Heading

## Section Heading

**Bold text** and *italic text*

- Bullet point 1
- Bullet point 2
- Bullet point 3

1. Numbered item
2. Another item

> Quoted text block

`inline code`

---

[Link text](https://example.com)
END TALK
```

### Dynamic Content

Combine with variables for dynamic messages:

```basic
customerName = "John"
orderCount = 5

BEGIN TALK
Hello, **${customerName}**! 👋

You have ${orderCount} orders in your history.

What would you like to do?
END TALK
```

### Comparison: TALK vs BEGIN TALK

| Feature | `TALK` | `BEGIN TALK` |
|---------|--------|--------------|
| Single line | ✅ | ❌ |
| Multiple lines | Concatenate with + | ✅ Native |
| Formatting preserved | ❌ | ✅ |
| Markdown | Limited | ✅ Full |
| Emojis | ✅ | ✅ |
| Variables | `TALK "Hi " + name` | `${name}` |

Use `TALK` for simple messages like `TALK "Hello!"` or `TALK "Your order is: " + orderId`. Use `BEGIN TALK` for complex formatted messages that benefit from multiple lines, Markdown formatting, and preserved whitespace.

---

## Real-World Examples

### Customer Service Bot

```basic
' start.bas

ADD TOOL "create-ticket"
ADD TOOL "check-status"
ADD TOOL "escalate"

USE KB "faq"
USE KB "troubleshooting"

BEGIN SYSTEM PROMPT
You are a customer service representative for TechSupport Inc.

## Persona
- Empathetic and patient
- Solution-oriented
- Professional but warm

## Available Tools
- create-ticket: Create support tickets for issues
- check-status: Check existing ticket status
- escalate: Escalate to human agent

## Workflow
1. Greet customer and understand their issue
2. Check if issue is in FAQ/troubleshooting KB
3. If solvable, provide solution
4. If not, create ticket or escalate

## Rules
- Never share internal system information
- Always provide ticket numbers for reference
- Offer escalation if customer requests human help
- Response time SLA: acknowledge within 30 seconds

## Escalation Triggers
- Customer explicitly requests human
- Issue unresolved after 3 exchanges
- Billing disputes over $100
- Account security concerns
END SYSTEM PROMPT

CLEAR SUGGESTIONS
ADD SUGGESTION "Technical Issue"
ADD SUGGESTION "Billing Question"
ADD SUGGESTION "Account Help"

BEGIN TALK
**Welcome to TechSupport!** 🛠️

I'm here to help you with:
• Technical issues and troubleshooting
• Billing and account questions
• Product information

I can also connect you with a human agent if needed.

How can I assist you today?
END TALK
```

### E-commerce Bot

```basic
' start.bas

ADD TOOL "search-products"
ADD TOOL "add-to-cart"
ADD TOOL "checkout"
ADD TOOL "track-order"

USE KB "catalog"
USE KB "promotions"

BEGIN SYSTEM PROMPT
You are a shopping assistant for FashionMart.

## Capabilities
- search-products: Find products by name, category, or description
- add-to-cart: Add items to shopping cart
- checkout: Process payment and create order
- track-order: Track shipment status

## Knowledge
- Full product catalog with prices and availability
- Current promotions and discount codes
- Shipping policies and delivery times

## Sales Approach
- Be helpful, not pushy
- Mention relevant promotions naturally
- Suggest complementary products when appropriate
- Always confirm before checkout

## Constraints
- Cannot modify prices or create custom discounts
- Returns handled through separate process
- Cannot access customer payment details directly

## Response Style
- Use product images when available
- Include prices in responses
- Mention stock levels for popular items
END SYSTEM PROMPT

BEGIN TALK
**Welcome to FashionMart!** 👗👔

🔥 **Today's Deals:**
• 20% off summer collection
• Free shipping on orders over $50

I can help you:
• Find the perfect outfit
• Check sizes and availability
• Track your orders

What are you looking for today?
END TALK
```

### Data Sync Bot (Scheduled)

Automated bots that run on a schedule don't need welcome messages since there's no user interaction:

```basic
' sync.bas - No welcome needed, runs on schedule

SET SCHEDULE "0 0 6 * * *"  ' Daily at 6 AM

BEGIN SYSTEM PROMPT
You are a data synchronization agent.

## Purpose
Sync product data from external ERP to local database.

## Process
1. Fetch products from ERP API
2. Compare with local database
3. Update changed records
4. Report statistics

## Error Handling
- Log all errors
- Continue processing on individual failures
- Send summary email to admin

## No user interaction required.
END SYSTEM PROMPT

' No BEGIN TALK needed - this is automated
SEND EMAIL admin1, "Starting daily sync..."

' ... sync logic ...

SEND EMAIL admin1, "Sync complete: " + REPORT
```

---

## Common Patterns

### Role-Based Prompts

You can dynamically set different system prompts based on user role:

```basic
role = GET USER "role"

SWITCH role
    CASE "admin"
        BEGIN SYSTEM PROMPT
        You are an admin assistant with full system access.
        You can manage users, view logs, and modify settings.
        END SYSTEM PROMPT
    
    CASE "customer"
        BEGIN SYSTEM PROMPT
        You are a customer service assistant.
        You can help with orders and general questions.
        END SYSTEM PROMPT
    
    DEFAULT
        BEGIN SYSTEM PROMPT
        You are a general assistant with limited access.
        END SYSTEM PROMPT
END SWITCH
```

### Conditional Welcome

Personalize welcome messages based on context:

```basic
hour = HOUR(NOW)

IF hour < 12 THEN
    greeting = "Good morning"
ELSE IF hour < 18 THEN
    greeting = "Good afternoon"
ELSE
    greeting = "Good evening"
END IF

BEGIN TALK
**${greeting}!** 👋

Welcome to our service. How can I help you today?
END TALK
```

---

## See Also

The [SET CONTEXT](./keyword-set-context.md) keyword provides dynamic context setting during runtime. The [TALK](./keyword-talk.md) keyword handles simple message output for single-line messages. Review [Script Execution Flow](./script-execution-flow.md) to understand the execution lifecycle. The [Tools System](./keyword-use-tool.md) documentation explains tool registration that works with system prompts.