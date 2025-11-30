# TALK

Sends a message to the current conversation or to a specific recipient on any supported channel.

## Syntax

```basic
TALK message

TALK TO recipient, message
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| message | String | The message to send |
| recipient | String | Channel and address in format `channel:address` |

## Description

`TALK` is the primary keyword for sending messages in General Bots BASIC.

- **TALK message** - Sends to the current conversation (web chat, WhatsApp, etc.)
- **TALK TO recipient, message** - Sends to a specific recipient on any channel

## TALK - Current Conversation

Send a message to whoever is currently chatting with the bot:

```basic
TALK "Hello! How can I help you today?"

name = "João"
TALK "Welcome, " + name + "!"

total = 299.90
TALK "Your total is $" + total
```

## TALK TO - Specific Recipients

Send messages to specific people on specific channels using the format `channel:address`.

### Supported Channels

| Channel | Format | Example |
|---------|--------|---------|
| WhatsApp | `whatsapp:+phone` | `whatsapp:+5511999887766` |
| Teams | `teams:user@domain` | `teams:john@company.com` |
| Email | `email:address` | `email:customer@example.com` |
| Web Session | `web:session_id` | `web:abc123` |
| Instagram | `instagram:username` | `instagram:@myuser` |

### WhatsApp Examples

```basic
' Send simple message
TALK TO "whatsapp:+5511999887766", "Your order has been shipped!"

' Send with variables
phone = "+5511999887766"
customer_name = "Maria"
TALK TO "whatsapp:" + phone, "Hello " + customer_name + "! Your order is ready."

' Send formatted message (WhatsApp supports markdown-like formatting)
message = "*Order Confirmed* ✅\n\n"
message = message + "Order: #12345\n"
message = message + "Total: R$ 299,90\n\n"
message = message + "_Thank you for your purchase!_"
TALK TO "whatsapp:" + customer_phone, message
```

### WhatsApp Message Formatting

WhatsApp supports rich text formatting:

| Format | Syntax | Result |
|--------|--------|--------|
| Bold | `*text*` | **text** |
| Italic | `_text_` | *text* |
| Strikethrough | `~text~` | ~~text~~ |
| Monospace | `` `text` `` | `text` |
| Line break | `\n` | New line |

```basic
' Example with all formatting
msg = "🎉 *PROMOTION!*\n\n"
msg = msg + "~R$ 199,90~ *R$ 149,90*\n"
msg = msg + "_Limited time offer!_\n\n"
msg = msg + "Use code: `PROMO2024`"

TALK TO "whatsapp:" + phone, msg
```

### Microsoft Teams Examples

```basic
' Send to Teams user
TALK TO "teams:john.smith@company.com", "Meeting reminder: 3pm today"

' Send with formatting (Teams supports markdown)
msg = "**Project Update**\n\n"
msg = msg + "- Task 1: ✅ Complete\n"
msg = msg + "- Task 2: 🔄 In Progress\n"
msg = msg + "- Task 3: ⏳ Pending"

TALK TO "teams:" + manager_email, msg
```

### Email Examples

```basic
' Simple email (uses SEND MAIL internally for full email)
TALK TO "email:customer@example.com", "Your password has been reset."

' For full email with subject, use SEND MAIL instead
SEND MAIL "customer@example.com", "Password Reset", "Your password has been reset successfully."
```

## Complete Examples

### Order Notification System

```basic
WEBHOOK "order-status"

order_id = body.order_id
customer_phone = body.phone
status = body.status

SELECT CASE status
    CASE "confirmed"
        msg = "✅ *Order Confirmed*\n\n"
        msg = msg + "Order #" + order_id + "\n"
        msg = msg + "We're preparing your order!"
        
    CASE "shipped"
        tracking = body.tracking_number
        msg = "📦 *Order Shipped*\n\n"
        msg = msg + "Order #" + order_id + "\n"
        msg = msg + "Tracking: " + tracking + "\n"
        msg = msg + "Track at: https://track.example.com/" + tracking
        
    CASE "delivered"
        msg = "🎉 *Order Delivered*\n\n"
        msg = msg + "Order #" + order_id + "\n"
        msg = msg + "Enjoy your purchase!\n\n"
        msg = msg + "_Rate your experience: reply 1-5_"
        
    CASE ELSE
        msg = "Order #" + order_id + " status: " + status
END SELECT

TALK TO "whatsapp:" + customer_phone, msg

result_status = "ok"
```

### Support Ticket Notifications

```basic
SUB NotifyCustomer(phone, ticket_id, message)
    full_msg = "🎫 *Ticket #" + ticket_id + "*\n\n"
    full_msg = full_msg + message
    TALK TO "whatsapp:" + phone, full_msg
END SUB

SUB NotifyAgent(agent_email, ticket_id, customer_name, issue)
    msg = "New ticket assigned:\n\n"
    msg = msg + "Ticket: #" + ticket_id + "\n"
    msg = msg + "Customer: " + customer_name + "\n"
    msg = msg + "Issue: " + issue
    TALK TO "teams:" + agent_email, msg
END SUB

' Usage
CALL NotifyCustomer("+5511999887766", "TKT-001", "Your ticket has been created. We'll respond within 24 hours.")
CALL NotifyAgent("support@company.com", "TKT-001", "João Silva", "Payment issue")
```

### Multi-Channel Broadcast

```basic
SUB Broadcast(message, channels)
    FOR EACH channel IN channels
        TALK TO channel, message
        WAIT 1  ' Rate limiting
    NEXT channel
END SUB

' Send to multiple recipients
promo = "🎉 *Flash Sale!* 50% off everything today only!"

recipients = [
    "whatsapp:+5511999887766",
    "whatsapp:+5511888776655",
    "teams:marketing@company.com"
]

CALL Broadcast(promo, recipients)
```

### Appointment Reminders

```basic
WEBHOOK "send-reminder"

appointment_id = body.id
appointment = FIND "appointments", "id=" + appointment_id

phone = appointment.customer_phone
name = appointment.customer_name
service = appointment.service
date_time = FORMAT(appointment.datetime, "DD/MM/YYYY HH:mm")

reminder = "📅 *Appointment Reminder*\n\n"
reminder = reminder + "Hi " + name + "!\n\n"
reminder = reminder + "You have an appointment scheduled:\n\n"
reminder = reminder + "📋 " + service + "\n"
reminder = reminder + "🗓️ " + date_time + "\n\n"
reminder = reminder + "Reply *CONFIRM* to confirm or *CANCEL* to cancel."

TALK TO "whatsapp:" + phone, reminder

result_status = "ok"
```

## Notes

- **TALK** sends to the current active conversation
- **TALK TO** can send to any supported channel
- WhatsApp requires phone numbers in international format with country code
- Teams requires valid email addresses from your organization
- Message formatting varies by channel (WhatsApp uses different syntax than Teams)
- Rate limiting may apply - use `WAIT` between bulk messages

## Related Keywords

- [SEND FILE TO](./keyword-send-file-to.md) - Send files to specific recipients
- [SEND MAIL](./keyword-send-mail.md) - Send emails with subject and attachments
- [HEAR](./keyword-hear.md) - Receive input from users
- [PRINT](./keyword-print.md) - Alias for TALK (debug output)

## See Also

- [Universal Messaging](./universal-messaging.md) - Multi-channel messaging overview
- [WEBHOOK](./keyword-webhook.md) - Create API endpoints