# Template Variables

Templates support variable substitution using double curly braces `{{variable_name}}`. Variables are replaced at send time with values from the provided data object.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Built-in Variables

These variables are automatically available in all templates without explicit declaration:

| Variable | Description | Example |
|----------|-------------|---------|
| `{{recipient}}` | Recipient email or phone | `john@example.com` |
| `{{to}}` | Alias for recipient | `john@example.com` |
| `{{date}}` | Current date (YYYY-MM-DD) | `2025-01-22` |
| `{{time}}` | Current time (HH:MM) | `14:30` |
| `{{datetime}}` | Combined date and time | `2025-01-22 14:30` |
| `{{year}}` | Current year | `2025` |
| `{{month}}` | Current month name | `January` |

## Custom Variables

Pass custom variables via the variables parameter in `SEND TEMPLATE`:

```basic
WITH vars
    .name = "John"
    .company = "Acme Corp"
    .product = "Pro Plan"
    .discount = "20%"
END WITH

SEND TEMPLATE "welcome", "email", "john@example.com", vars
```

The template content would reference these variables:

```
Hello {{name}},

Welcome to {{company}}! You've signed up for {{product}}.

As a special offer, use code WELCOME for {{discount}} off your first purchase.

Best regards,
The Team
```

## Channel-Specific Templates

### Email Templates

Email templates support automatic `Subject:` line extraction. Place the subject on the first line:

```
Subject: Welcome to {{company}}, {{name}}!

Hello {{name}},

Thank you for joining us...
```

The system extracts the subject line and uses the remainder as the body.

### WhatsApp Templates

WhatsApp templates must be pre-approved by Meta. Use numbered placeholders as required by the WhatsApp Business API:

```
Hello {{1}}, your order {{2}} has shipped. Track at {{3}}
```

Map variables using numeric keys:

```basic
WITH vars
    .1 = customer_name
    .2 = order_id
    .3 = tracking_url
END WITH

SEND TEMPLATE "order-shipped", "whatsapp", phone, vars
```

### SMS Templates

Keep SMS templates under 160 characters for single-segment delivery:

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

### Retrieving Templates

```basic
template = GET TEMPLATE "welcome"
TALK "Template body: " + template.body
```

## Variable Extraction

Variables are automatically extracted from template content when the template is created. The system identifies all `{{variable}}` patterns and stores them for validation. Built-in variables (recipient, date, time, etc.) are excluded from the extraction.

## Fallback Values

Handle missing variables using `NVL` in your code:

```basic
WITH vars
    .name = NVL(user_name, "Friend")
    .company = NVL(user_company, "your organization")
END WITH

SEND TEMPLATE "greeting", "email", email, vars
```

## Multi-Channel Delivery

Send the same template to multiple channels in one call:

```basic
WITH vars
    .name = "John"
    .message = "Your appointment is confirmed"
END WITH

SEND TEMPLATE "appointment-confirm", "email,sms,whatsapp", recipient, vars
```

Or send channel-specific versions:

```basic
SEND TEMPLATE "appointment-email", "email", email, vars
SEND TEMPLATE "appointment-sms", "sms", phone, vars
```

## Bulk Sending

Send templates to multiple recipients:

```basic
recipients = ["a@example.com", "b@example.com", "c@example.com"]
count = SEND TEMPLATE "newsletter" TO "email" recipients, #{month: "January"}
TALK "Sent to " + count + " recipients"
```

## Best Practices

**Keep variable names simple.** Use `name` rather than `customer_first_name_from_database`. Shorter names are easier to maintain.

**Provide fallbacks.** Always handle the case where a variable might be missing or empty.

**Test templates.** Verify all variables populate correctly before deploying to production.

**Respect channel limits.** SMS has a 160-character single-segment limit. WhatsApp templates require Meta approval.

**Personalize thoughtfully.** Using `{{name}}` improves engagement, but avoid over-personalization that feels intrusive.

**Include unsubscribe options.** Marketing emails should always provide an unsubscribe mechanism.

## Database Storage

Templates are stored in the `message_templates` table:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Template identifier |
| `bot_id` | UUID | Owning bot |
| `name` | TEXT | Template name |
| `channel` | TEXT | email/whatsapp/sms/telegram/push |
| `subject` | TEXT | Email subject (nullable) |
| `body` | TEXT | Template body |
| `variables` | JSONB | List of variable names |
| `is_active` | BOOL | Active status |

## See Also

- [SEND TEMPLATE Keyword](./keywords.md) - Full keyword reference
- [SET SCHEDULE](./keyword-set-schedule.md) - Scheduled template delivery
- [Universal Messaging](./universal-messaging.md) - Multi-channel patterns