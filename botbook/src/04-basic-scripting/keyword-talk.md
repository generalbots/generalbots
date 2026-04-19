# TALK

Sends a message to the current conversation or to a specific recipient on any supported channel.

## Syntax

### Single Message

```basic
TALK message

TALK TO recipient, message
```

### Multi-Line Block with Variable Substitution

```basic
BEGIN TALK
Line 1 with ${variable}
Line 2 with ${anotherVariable}
Plain text line
END TALK
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| message | String | The message to send |
| recipient | String | Channel and address in format `channel:address` |
| ${variable} | Expression | Variable substitution within TALK blocks |

## Description

`TALK` is the primary keyword for sending messages in General Bots BASIC.

- **TALK message** - Sends to the current conversation (web chat, WhatsApp, etc.)
- **TALK TO recipient, message** - Sends to a specific recipient on any channel
- **BEGIN TALK / END TALK** - Multi-line block with automatic variable substitution

## BEGIN TALK / END TALK Blocks

The `BEGIN TALK / END TALK` block syntax allows you to write multiple messages with automatic variable substitution using `${variable}` syntax.

### Syntax

```basic
BEGIN TALK
Hello ${name}!
Your order ${orderId} is confirmed.
Total: ${FORMAT(total, "currency")}
Thank you for your purchase!
END TALK
```

Each line within the block becomes a separate `TALK` statement. The `${variable}` syntax is automatically converted to string concatenation.

### How It Works

**Input:**
```basic
nomeNoivo = "Carlos"
protocolo = "CAS123456"

BEGIN TALK
Solicitacao de Casamento enviada com sucesso!
PROTOCOLO: ${protocolo}
Noivo: ${nomeNoivo}
END TALK
```

**Converted to:**
```basic
TALK "Solicitacao de Casamento enviada com sucesso!"
TALK "PROTOCOLO: " + protocolo
TALK "Noivo: " + nomeNoivo
```

### Variable Substitution Rules

- `${variableName}` - Replaced with the variable value using string concatenation
- `${FUNCTION(args)}` - Function calls are evaluated and substituted
- Plain text without `${}` is treated as a string literal
- Special characters like `$` (not followed by `{`) are preserved as-is

### Examples

#### Simple Substitution

```basic
nome = "João"
idade = 30

BEGIN TALK
Olá ${nome}!
Você tem ${idade} anos.
END TALK
```

**Equivalent to:**
```basic
TALK "Olá " + nome + "!"
TALK "Você tem " + idade + " anos."
```

#### With Function Calls

```basic
total = 299.90
numero = 42

BEGIN TALK
Seu pedido: ${numero}
Total: ${FORMAT(total, "currency")}
Obrigado pela preferência!
END TALK
```

#### Mixed Content

```basic
nome = "Maria"
codigo = "PROMO2024"
desconto = 20

BEGIN TALK
🎉 Oferta Especial para ${nome}!

Use o código: ${codigo}
Desconto de ${desconto}%

Aproveite!
END TALK
```

### Real-World Example: Wedding Confirmation

```basic
PARAM nomeNoivo AS STRING LIKE "Carlos" DESCRIPTION "Nome do noivo"
PARAM nomeNoiva AS STRING LIKE "Ana" DESCRIPTION "Nome da noiva"
PARAM protocolo AS STRING LIKE "CAS123456" DESCRIPTION "Protocolo"
PARAM dataCasamento AS DATE LIKE "2026-12-15" DESCRIPTION "Data do casamento"

casamentoId = "CAS-" + FORMAT(NOW(), "yyyyMMddHHmmss")
dataDisplay = FORMAT(dataCasamento, "dd/MM/yyyy")

BEGIN TALK
✅ Solicitação de Casamento enviada com sucesso!

Protocolo: ${protocolo}
ID: ${casamentoId}
Noivo: ${nomeNoivo}
Noiva: ${nomeNoiva}
Data: ${dataDisplay}

Status: Aguardando verificação de disponibilidade
Contato: (21) 4101-0770
END TALK
```

This is much cleaner than writing individual TALK statements with manual concatenation:

**Old way:**
```basic
TALK "Solicitacao de Casamento enviada com sucesso!"
TALK "Protocolo: " + protocolo
TALK "ID: " + casamentoId
TALK "Noivo: " + nomeNoivo
TALK "Noiva: " + nomeNoiva
TALK "Data: " + dataDisplay
TALK "Status: Aguardando verificacao de disponibilidade"
TALK "Contato: (21) 4101-0770"
```

### Advantages

1. **Cleaner Syntax** - No more repetitive `TALK` statements and `+` concatenations
2. **Easier to Read** - Multi-line messages are more natural to write
3. **Less Error-Prone** - Automatic substitution reduces typos in variable names
4. **Template-Like** - Write messages like templates with `${variable}` placeholders
5. **Perfect for TOOL Functions** - Variables are automatically filled by user input

## TALK - Current Conversation

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