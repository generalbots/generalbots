# SEND TEMPLATE Keywords

Send templated messages across multiple channels (email, WhatsApp, SMS, Telegram, push notifications).

## Keywords

| Keyword | Purpose |
|---------|---------|
| `SEND_TEMPLATE` | Send template to single recipient |
| `SEND_TEMPLATE_TO` | Send template to multiple recipients |
| `CREATE_TEMPLATE` | Create a new message template |
| `GET_TEMPLATE` | Retrieve template by name |

## SEND_TEMPLATE

```basic
result = SEND_TEMPLATE "welcome", "user@example.com", "email"
```

With variables:

```basic
vars = {"name": "John", "order_id": "12345"}
result = SEND_TEMPLATE "order_confirmation", "+1234567890", "whatsapp", vars
```

## SEND_TEMPLATE_TO

Send to multiple recipients:

```basic
recipients = ["user1@example.com", "user2@example.com", "user3@example.com"]
result = SEND_TEMPLATE_TO "newsletter", recipients, "email"

TALK "Sent: " + result.sent + ", Failed: " + result.failed
```

## Supported Channels

| Channel | Recipient Format |
|---------|------------------|
| `email` | Email address |
| `whatsapp` | Phone number with country code |
| `sms` | Phone number with country code |
| `telegram` | Telegram user ID or username |
| `push` | Device token or user ID |

## CREATE_TEMPLATE

```basic
template_body = "Hello {{name}}, your order {{order_id}} has shipped!"
result = CREATE_TEMPLATE "shipping_notification", template_body, "transactional"
```

## Template Variables

Use `{{variable_name}}` syntax in templates:

```basic
vars = {
    "customer_name": "Alice",
    "amount": "$99.00",
    "date": "March 15, 2024"
}
result = SEND_TEMPLATE "receipt", "alice@example.com", "email", vars
```

## Example: Order Notification

```basic
' Send order confirmation across multiple channels
order_vars = {
    "order_id": order.id,
    "total": order.total,
    "items": order.item_count
}

SEND_TEMPLATE "order_placed", customer.email, "email", order_vars
SEND_TEMPLATE "order_placed", customer.phone, "whatsapp", order_vars
```

## Response Object

```json
{
    "success": true,
    "message_id": "msg_123abc",
    "channel": "email",
    "recipient": "user@example.com"
}
```

For batch sends:

```json
{
    "total": 100,
    "sent": 98,
    "failed": 2,
    "errors": [...]
}
```

## See Also

- [SEND MAIL](./keyword-send-mail.md)
- [SEND SMS](./keyword-sms.md)