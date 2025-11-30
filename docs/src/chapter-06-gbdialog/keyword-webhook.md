# WEBHOOK

Creates an instant HTTP endpoint for your bot. With WEBHOOK, you can expose any BASIC script as an API endpoint that external systems can call - perfect for integrations, notifications, and building custom APIs with LLM-powered responses.

## Why WEBHOOK?

Traditional API development requires:
- Setting up a web framework
- Writing routing code
- Handling HTTP parsing
- Deploying infrastructure

With General Bots WEBHOOK, you write one line and your endpoint is live:

```basic
WEBHOOK "my-endpoint"
```

That's it. Your script is now accessible at `/api/{botname}/webhook/my-endpoint`.

## Syntax

```basic
WEBHOOK "endpoint-name"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| endpoint-name | String | Unique name for the webhook (alphanumeric, hyphens, underscores) |

## Request Data Available

When your webhook is called, these variables are automatically available:

| Variable | Description | Example |
|----------|-------------|---------|
| `params` | Query string parameters | `params.id`, `params.filter` |
| `body` | JSON request body as object | `body.customer.name` |
| `headers` | HTTP headers | `headers.authorization` |
| `method` | HTTP method used | `"POST"`, `"GET"` |
| `path` | Request path | `"/webhook/my-endpoint"` |

## Examples

### 1. Simple Status Endpoint

```basic
' status.bas - Simple health check
WEBHOOK "status"

result_status = "healthy"
result_timestamp = NOW()
result_version = "1.0.0"
```

**Call it:**
```bash
curl https://bot.example.com/api/mybot/webhook/status
```

**Response:**
```json
{"status": "healthy", "timestamp": "2024-01-20T10:30:00Z", "version": "1.0.0"}
```

### 2. WhatsApp Order Notification

Send order confirmations directly to customers on WhatsApp:

```basic
' order-notify.bas - Notify customer via WhatsApp
WEBHOOK "order-notify"

order_id = body.order_id
customer_phone = body.customer_phone
customer_name = body.customer_name
total = body.total
items = body.items

IF order_id = "" OR customer_phone = "" THEN
    result_status = 400
    result_error = "Missing order_id or customer_phone"
    EXIT
END IF

' Build order summary
order_summary = "🛒 *Order Confirmed #" + order_id + "*\n\n"
order_summary = order_summary + "Hi " + customer_name + "!\n\n"
order_summary = order_summary + "Your order has been confirmed.\n"
order_summary = order_summary + "Total: $" + total + "\n\n"
order_summary = order_summary + "We'll notify you when it ships!"

' Send to WhatsApp using TALK TO
TALK TO "whatsapp:" + customer_phone, order_summary

' Save order to database
order_status = "confirmed"
created_at = NOW()
SAVE "orders", order_id, customer_name, customer_phone, total, order_status, created_at

result_status = "ok"
result_order_id = order_id
result_message = "Customer notified via WhatsApp"
```

**Call it:**
```bash
curl -X POST https://bot.example.com/api/mybot/webhook/order-notify \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "ORD-12345",
    "customer_phone": "+5511999887766",
    "customer_name": "João",
    "total": "299.90",
    "items": ["Widget", "Gadget"]
  }'
```

### 3. WhatsApp Document Delivery

Send invoices, reports, or documents to WhatsApp:

```basic
' send-invoice.bas - Generate and send invoice via WhatsApp
WEBHOOK "send-invoice"

order_id = body.order_id
customer_phone = body.customer_phone
customer_name = body.customer_name

IF order_id = "" OR customer_phone = "" THEN
    result_status = 400
    result_error = "Missing order_id or customer_phone"
    EXIT
END IF

' Get order data
order = FIND "orders", "order_id=" + order_id

' Generate PDF invoice
invoice_date = FORMAT(NOW(), "DD/MM/YYYY")
GENERATE PDF "templates/invoice.html", order_id, customer_name, order.total, order.items, invoice_date, "invoices/" + order_id + ".pdf"

' Send PDF to WhatsApp with caption
SEND FILE TO "whatsapp:" + customer_phone, "invoices/" + order_id + ".pdf", "📄 Invoice #" + order_id + " - Thank you for your purchase!"

' Also send a follow-up message
TALK TO "whatsapp:" + customer_phone, "If you have any questions about your order, just reply to this message! 😊"

result_status = "ok"
result_message = "Invoice sent to WhatsApp"
```

### 4. WhatsApp Support Ticket System

Create support tickets and notify via WhatsApp:

```basic
' support-ticket.bas - Create ticket and notify customer
WEBHOOK "support-ticket"

customer_phone = body.phone
customer_name = body.name
issue = body.issue
priority = body.priority

IF customer_phone = "" OR issue = "" THEN
    result_status = 400
    result_error = "Missing phone or issue description"
    EXIT
END IF

IF priority = "" THEN
    priority = "normal"
END IF

' Create ticket
ticket_id = "TKT-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
ticket_status = "open"
created_at = NOW()

SAVE "support_tickets", ticket_id, customer_name, customer_phone, issue, priority, ticket_status, created_at

' Notify customer via WhatsApp
confirmation = "🎫 *Support Ticket Created*\n\n"
confirmation = confirmation + "Ticket: #" + ticket_id + "\n"
confirmation = confirmation + "Priority: " + priority + "\n\n"
confirmation = confirmation + "We received your request:\n_" + issue + "_\n\n"
confirmation = confirmation + "Our team will respond within 24 hours."

TALK TO "whatsapp:" + customer_phone, confirmation

' Notify support team
team_msg = "🆕 New ticket #" + ticket_id + "\n"
team_msg = team_msg + "From: " + customer_name + " (" + customer_phone + ")\n"
team_msg = team_msg + "Priority: " + priority + "\n"
team_msg = team_msg + "Issue: " + issue

TALK TO "whatsapp:+5511999000001", team_msg

result_status = "ok"
result_ticket_id = ticket_id
```

### 5. AI-Powered WhatsApp Assistant

Create an API that uses AI and responds via WhatsApp:

```basic
' ai-assistant.bas - AI assistant that responds via WhatsApp
WEBHOOK "ask-ai"

question = body.question
customer_phone = body.phone
context_type = body.context

IF question = "" OR customer_phone = "" THEN
    result_status = 400
    result_error = "Missing question or phone"
    EXIT
END IF

' Load appropriate knowledge base
IF context_type = "products" THEN
    USE KB "product-catalog"
ELSE IF context_type = "support" THEN
    USE KB "support-docs"
ELSE
    USE KB "general-faq"
END IF

' Set AI context
SET CONTEXT "You are a helpful assistant. Be concise and friendly. Use emojis occasionally."

' Get AI response
answer = LLM question

' Send response via WhatsApp
TALK TO "whatsapp:" + customer_phone, answer

' Log the interaction
log_question = question
log_answer = answer
log_phone = customer_phone
log_context = context_type
log_timestamp = NOW()

INSERT "ai_conversations", log_question, log_answer, log_phone, log_context, log_timestamp

result_status = "ok"
result_answer = answer
```

### 6. WhatsApp Broadcast for Promotions

Send promotional messages to multiple customers:

```basic
' promo-broadcast.bas - Send promotions to customer list
WEBHOOK "send-promo"

promo_title = body.title
promo_message = body.message
promo_image = body.image_url
customer_segment = body.segment

IF promo_message = "" THEN
    result_status = 400
    result_error = "Missing promotion message"
    EXIT
END IF

IF customer_segment = "" THEN
    customer_segment = "all"
END IF

' Get customers for this segment
customers = FIND "customers", "segment=" + customer_segment + " AND whatsapp_optin=true"

sent_count = 0
error_count = 0

' Build promo message with formatting
full_message = "🎉 *" + promo_title + "*\n\n"
full_message = full_message + promo_message + "\n\n"
full_message = full_message + "_Reply STOP to unsubscribe_"

FOR EACH customer IN customers
    ' Send to each customer
    IF promo_image <> "" THEN
        SEND FILE TO "whatsapp:" + customer.phone, promo_image, full_message
    ELSE
        TALK TO "whatsapp:" + customer.phone, full_message
    END IF
    
    sent_count = sent_count + 1
    
    ' Rate limiting - wait between messages
    WAIT 1
NEXT customer

' Log the campaign
campaign_id = "CAMP-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
campaign_title = promo_title
campaign_sent = sent_count
campaign_date = NOW()

INSERT "campaigns", campaign_id, campaign_title, campaign_sent, customer_segment, campaign_date

result_status = "ok"
result_campaign_id = campaign_id
result_sent = sent_count
```

### 7. Payment Notification with WhatsApp Receipt

Handle payment webhooks and notify customers:

```basic
' payment-webhook.bas - Handle payment and notify via WhatsApp
WEBHOOK "payment"

event_type = body.type
payment_id = body.data.object.id
amount = body.data.object.amount
customer_id = body.data.object.customer

SELECT CASE event_type
    CASE "payment_intent.succeeded"
        ' Get customer info
        customer = FIND "customers", "stripe_id=" + customer_id
        
        ' Update order status
        order_status = "paid"
        paid_at = NOW()
        UPDATE "orders", "payment_id=" + payment_id, order_status, paid_at
        
        ' Format amount (cents to dollars)
        amount_formatted = amount / 100
        
        ' Send WhatsApp receipt
        receipt = "✅ *Payment Received*\n\n"
        receipt = receipt + "Amount: $" + amount_formatted + "\n"
        receipt = receipt + "Payment ID: " + payment_id + "\n"
        receipt = receipt + "Date: " + FORMAT(NOW(), "DD/MM/YYYY HH:mm") + "\n\n"
        receipt = receipt + "Thank you for your purchase! 🙏"
        
        TALK TO "whatsapp:" + customer.phone, receipt
        
    CASE "payment_intent.payment_failed"
        customer = FIND "customers", "stripe_id=" + customer_id
        
        ' Notify customer of failure
        failure_msg = "⚠️ *Payment Failed*\n\n"
        failure_msg = failure_msg + "We couldn't process your payment.\n"
        failure_msg = failure_msg + "Please try again or use a different payment method.\n\n"
        failure_msg = failure_msg + "Need help? Reply to this message!"
        
        TALK TO "whatsapp:" + customer.phone, failure_msg
        
    CASE ELSE
        ' Log unhandled event
        TALK "Unhandled payment event: " + event_type
END SELECT

result_received = TRUE
```

### 8. Appointment Reminder System

Webhook to trigger appointment reminders:

```basic
' appointment-reminder.bas - Send appointment reminders via WhatsApp
WEBHOOK "send-reminder"

appointment_id = body.appointment_id
hours_before = body.hours_before

IF appointment_id = "" THEN
    result_status = 400
    result_error = "Missing appointment_id"
    EXIT
END IF

IF hours_before = "" THEN
    hours_before = 24
END IF

' Get appointment details
appointment = FIND "appointments", "id=" + appointment_id

' Format date/time nicely
appt_date = FORMAT(appointment.datetime, "dddd, MMMM DD")
appt_time = FORMAT(appointment.datetime, "HH:mm")

' Build reminder message
reminder = "📅 *Appointment Reminder*\n\n"
reminder = reminder + "Hi " + appointment.customer_name + "!\n\n"
reminder = reminder + "This is a reminder of your upcoming appointment:\n\n"
reminder = reminder + "📍 *" + appointment.service + "*\n"
reminder = reminder + "🗓️ " + appt_date + "\n"
reminder = reminder + "🕐 " + appt_time + "\n"
reminder = reminder + "📌 " + appointment.location + "\n\n"
reminder = reminder + "Reply *CONFIRM* to confirm or *CANCEL* to cancel."

' Send via WhatsApp
TALK TO "whatsapp:" + appointment.customer_phone, reminder

' Update reminder sent status
reminder_sent_at = NOW()
UPDATE "appointments", "id=" + appointment_id, reminder_sent_at

result_status = "ok"
result_message = "Reminder sent"
```

### 9. Form Submission with WhatsApp Follow-up

Handle web form submissions and follow up on WhatsApp:

```basic
' contact-form.bas - Handle contact form and follow up via WhatsApp
WEBHOOK "contact"

name = body.name
email = body.email
phone = body.phone
message = body.message
source = body.source

IF name = "" OR message = "" THEN
    result_status = 400
    result_error = "Name and message are required"
    EXIT
END IF

' Use AI to categorize and generate response
SET CONTEXT "Categorize this message as: sales, support, feedback, or other. Then write a friendly acknowledgment."

ai_prompt = "Customer: " + name + "\nMessage: " + message
ai_response = LLM ai_prompt

' Save the submission
submission_id = "SUB-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
submission_status = "new"
created_at = NOW()

SAVE "submissions", submission_id, name, email, phone, message, source, ai_response, submission_status, created_at

' If phone provided, send WhatsApp confirmation
IF phone <> "" THEN
    whatsapp_msg = "👋 Hi " + name + "!\n\n"
    whatsapp_msg = whatsapp_msg + "Thanks for reaching out! We received your message:\n\n"
    whatsapp_msg = whatsapp_msg + "_" + message + "_\n\n"
    whatsapp_msg = whatsapp_msg + "We'll get back to you soon. In the meantime, feel free to reply here if you have any questions!"
    
    TALK TO "whatsapp:" + phone, whatsapp_msg
END IF

' Send email confirmation too
IF email <> "" THEN
    SEND MAIL email, "We received your message", "Hi " + name + ",\n\nThank you for contacting us. We'll respond within 24 hours.\n\nBest regards"
END IF

result_status = "ok"
result_submission_id = submission_id
```

### 10. Multi-Channel Notification Hub

Single webhook that routes to multiple channels:

```basic
' notify.bas - Multi-channel notification hub
WEBHOOK "notify"

channel = body.channel
recipient = body.recipient
message = body.message
file_url = body.file
caption = body.caption

IF recipient = "" OR message = "" THEN
    result_status = 400
    result_error = "Missing recipient or message"
    EXIT
END IF

IF channel = "" THEN
    channel = "whatsapp"
END IF

' Route to appropriate channel
SELECT CASE channel
    CASE "whatsapp"
        IF file_url <> "" THEN
            SEND FILE TO "whatsapp:" + recipient, file_url, caption
        ELSE
            TALK TO "whatsapp:" + recipient, message
        END IF
        
    CASE "email"
        subject = body.subject
        IF subject = "" THEN
            subject = "Notification"
        END IF
        
        IF file_url <> "" THEN
            SEND MAIL recipient, subject, message, file_url
        ELSE
            SEND MAIL recipient, subject, message
        END IF
        
    CASE "teams"
        TALK TO "teams:" + recipient, message
        
    CASE "web"
        ' Send to web session
        TALK TO "web:" + recipient, message
        
    CASE ELSE
        result_status = 400
        result_error = "Unknown channel: " + channel
        EXIT
END SELECT

' Log notification
log_channel = channel
log_recipient = recipient
log_message = message
log_timestamp = NOW()

INSERT "notification_log", log_channel, log_recipient, log_message, log_timestamp

result_status = "ok"
result_channel = channel
result_delivered = TRUE
```

## Response Handling

Control the HTTP response by setting `result_` prefixed variables:

### Simple Response
```basic
result_status = "ok"
result_data = my_data
```

### Custom Status Code
```basic
result_status = 201  ' Created
result_id = new_id
result_created = TRUE
```

### Error Response
```basic
result_status = 400
result_error = "Invalid request"
result_details = "Missing required field: phone"
```

## WhatsApp Message Formatting

WhatsApp supports rich text formatting:

| Format | Syntax | Example |
|--------|--------|---------|
| Bold | `*text*` | `*Important*` |
| Italic | `_text_` | `_note_` |
| Strikethrough | `~text~` | `~old price~` |
| Monospace | `` `text` `` | `` `code` `` |
| Line break | `\n` | `"Line 1\nLine 2"` |

### Example with Formatting
```basic
message = "🎉 *Order Confirmed!*\n\n"
message = message + "Order: #" + order_id + "\n"
message = message + "Total: ~$" + old_price + "~ *$" + new_price + "*\n"
message = message + "_Discount applied!_"

TALK TO "whatsapp:" + phone, message
```

## Security Best Practices

### 1. Validate Webhook Signatures

```basic
WEBHOOK "secure-endpoint"

signature = headers.x_webhook_signature
secret = GET BOT MEMORY "webhook_secret"

IF signature = "" THEN
    TALK "Invalid request - no signature"
    result_status = 401
    result_error = "Missing signature"
    EXIT
END IF

' Continue with verified request...
```

### 2. Validate Phone Numbers

```basic
phone = body.phone

' Remove non-numeric characters
clean_phone = REPLACE(phone, "+", "")
clean_phone = REPLACE(clean_phone, "-", "")
clean_phone = REPLACE(clean_phone, " ", "")

IF LEN(clean_phone) < 10 THEN
    result_status = 400
    result_error = "Invalid phone number"
    EXIT
END IF

' Add country code if missing
IF LEFT(clean_phone, 2) <> "55" THEN
    clean_phone = "55" + clean_phone
END IF

TALK TO "whatsapp:+" + clean_phone, message
```

### 3. Rate Limiting

```basic
WEBHOOK "rate-limited"

client_ip = headers.x_forwarded_for
rate_key = "rate:" + client_ip
current_count = GET BOT MEMORY rate_key

IF current_count = "" THEN
    current_count = 0
END IF

IF current_count > 100 THEN
    result_status = 429
    result_error = "Rate limit exceeded"
    result_retry_after = 60
    EXIT
END IF

SET BOT MEMORY rate_key, current_count + 1
' Process request...
```

## Use Cases Summary

| Use Case | Webhook Name | Description |
|----------|--------------|-------------|
| Order Notifications | `/order-notify` | Confirm orders via WhatsApp |
| Invoice Delivery | `/send-invoice` | Send PDF invoices to WhatsApp |
| Support Tickets | `/support-ticket` | Create tickets, notify via WhatsApp |
| AI Assistant | `/ask-ai` | LLM answers sent to WhatsApp |
| Promotions | `/send-promo` | Broadcast promos to customers |
| Payment Alerts | `/payment` | Payment receipts via WhatsApp |
| Reminders | `/send-reminder` | Appointment reminders |
| Contact Forms | `/contact` | Form follow-up on WhatsApp |
| Multi-Channel | `/notify` | Route to any channel |

## Technical Notes

- Webhooks register during script compilation
- Stored in `system_automations` table with `kind = Webhook`
- Endpoint names must be unique per bot
- Request timeout: 30 seconds (keep processing fast)
- Maximum request body: 10MB
- HTTPS required in production

## See Also

- [TALK TO](./keyword-talk.md#talk-to) - Send messages to specific recipients
- [SEND FILE TO](./keyword-send-file-to.md) - Send files to recipients
- [SET SCHEDULE](./keyword-set-schedule.md) - Time-based automation
- [ON](./keyword-on.md) - Database trigger events
- [LLM](./keyword-llm.md) - Language model queries
- [USE KB](./keyword-use-kb.md) - Knowledge base integration