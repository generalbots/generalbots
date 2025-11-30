# Template Variables Reference

> Documentation for SEND TEMPLATE variables and built-in placeholders

## Overview

Templates support variable substitution using double curly braces `{{variable_name}}`. Variables are replaced at send time with values from the provided data object.

## Built-in Variables

These variables are automatically available in all templates:

| Variable | Description | Example Output |
|----------|-------------|----------------|
| `{{recipient}}` | Recipient email/phone | `john@example.com` |
| `{{to}}` | Alias for recipient | `john@example.com` |
| `{{date}}` | Current date (YYYY-MM-DD) | `2025-01-22` |
| `{{time}}` | Current time (HH:MM) | `14:30` |
| `{{datetime}}` | Date and time | `2025-01-22 14:30` |
| `{{year}}` | Current year | `2025` |
| `{{month}}` | Current month name | `January` |

## Custom Variables

Pass custom variables via the variables parameter:

```basic
WITH vars
    .name = "John"
    .company = "Acme Corp"
    .product = "Pro Plan"
    .discount = "20%"
END WITH

SEND TEMPLATE "welcome", "email", "john@example.com", vars
```

Template content:
```
Hello {{name}},

Welcome to {{company}}! You've signed up for {{product}}.

As a special offer, use code WELCOME for {{discount}} off your first purchase.

Best regards,
The Team
```

## Channel-Specific Templates

### Email Templates

Email templates support `Subject:` line extraction:

```
Subject: Welcome to {{company}}, {{name}}!

Hello {{name}},

Thank you for joining us...
```

### WhatsApp Templates

WhatsApp templates must be pre-approved by Meta. Use numbered placeholders:

```
Hello {{1}}, your order {{2}} has shipped. Track at {{3}}
```

Map variables:
```basic
WITH vars
    .1 = customer_name
    .2 = order_id
    .3 = tracking_url
END WITH

SEND TEMPLATE "order-shipped", "whatsapp", phone, vars
```

### SMS Templates

Keep SMS templates under 160 characters for single segment:

```
Hi {{name}}, your code is {{code}}. Valid for 10 minutes.
```

## Template Examples

### Welcome Email

```
Subject: Welcome to {{company}}!

Hi {{name}},

Thanks for signing up on {{date}}. Here's what you can do next:

1. Complete your profile
2. Explore our features
3. Join our community

Questions? Reply to this email.

Best,
{{company}} Team
```

### Order Confirmation

```
Subject: Order #{{order_id}} Confirmed

Hi {{name}},

Your order has been confirmed!

Order: #{{order_id}}
Date: {{date}}
Total: {{total}}

Items:
{{items}}

Shipping to:
{{address}}

Track your order: {{tracking_url}}
```

### Lead Nurture

```
Subject: {{name}}, here's your exclusive resource

Hi {{name}},

As a {{company}} professional, we thought you'd find this helpful:

{{resource_title}}

{{resource_description}}

Download now: {{resource_url}}

Best,
{{sender_name}}
```

### Appointment Reminder

```
Subject: Reminder: {{appointment_type}} tomorrow

Hi {{name}},

This is a reminder of your upcoming appointment:

Date: {{appointment_date}}
Time: {{appointment_time}}
Location: {{location}}

Need to reschedule? Reply to this email or call {{phone}}.

See you soon!
```

## Creating Templates

### Via BASIC

```basic
CREATE TEMPLATE "welcome", "email", "Welcome {{name}}!", "Hello {{name}}, thank you for joining {{company}}!"
```

### Via Database

Templates are stored in `message_templates` table:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Template ID |
| `bot_id` | UUID | Bot owner |
| `name` | TEXT | Template name |
| `channel` | TEXT | email/whatsapp/sms/telegram/push |
| `subject` | TEXT | Email subject (nullable) |
| `body` | TEXT | Template body |
| `variables` | JSONB | List of variable names |
| `is_active` | BOOL | Active status |

## Variable Extraction

Variables are automatically extracted from template body:

```basic
body = "Hello {{name}}, your order {{order_id}} is {{status}}."
' Extracted variables: ["name", "order_id", "status"]
```

Built-in variables (recipient, date, time, etc.) are excluded from extraction.

## Fallback Values

Use NVL for fallback values in your code:

```basic
WITH vars
    .name = NVL(user_name, "Friend")
    .company = NVL(user_company, "your organization")
END WITH
```

## Multi-Channel Example

Send same template to multiple channels:

```basic
WITH vars
    .name = "John"
    .message = "Your appointment is confirmed"
END WITH

' Send to all channels
SEND TEMPLATE "appointment-confirm", "email,sms,whatsapp", recipient, vars

' Or send separately with channel-specific content
SEND TEMPLATE "appointment-email", "email", email, vars
SEND TEMPLATE "appointment-sms", "sms", phone, vars
```

## Best Practices

1. **Keep variable names simple**: Use `name` not `customer_first_name`
2. **Provide fallbacks**: Always handle missing variables
3. **Test templates**: Verify all variables are populated
4. **Channel limits**: SMS 160 chars, WhatsApp requires approval
5. **Personalization**: Use `{{name}}` for better engagement
6. **Unsubscribe**: Include unsubscribe link in marketing emails