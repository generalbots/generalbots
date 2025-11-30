# PRINT

Debug output keyword. `PRINT` is an alias for `TALK` - both send messages to the current conversation.

> **Note:** `PRINT` and `TALK` are equivalent. Use `TALK` for user-facing messages and `PRINT` for debug output during development. In production, prefer `TALK` for clarity.

## Syntax

```basic
PRINT expression
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| expression | Any | The value to output (string, number, or any expression) |

## Description

`PRINT` outputs a message to the current conversation. It is functionally identical to `TALK` but conventionally used for debugging and logging purposes.

For sending messages to specific recipients on other channels, use:
- `TALK TO` - Send to a specific recipient (WhatsApp, Teams, etc.)
- `SEND FILE TO` - Send files to a specific recipient

## Examples

### Basic Debug Output

```basic
x = 10
y = 20
PRINT "Debug: x = " + x + ", y = " + y

result = x + y
PRINT "Result: " + result
```

### Logging During Processing

```basic
WEBHOOK "process-order"

order_id = body.order_id
PRINT "Processing order: " + order_id

' Process the order
customer = FIND "customers", "id=" + body.customer_id
PRINT "Found customer: " + customer.name

' More processing...
PRINT "Order processing complete"

result_status = "ok"
```

### Variable Inspection

```basic
data = GET "https://api.example.com/data"
PRINT "API Response: " + data

items = FIND "products", "stock < 10"
PRINT "Low stock items count: " + UBOUND(items)
```

## Equivalent Keywords

| Keyword | Description |
|---------|-------------|
| `TALK` | Same as PRINT - send message to conversation |
| `TALK TO` | Send message to specific recipient |

## Example: PRINT vs TALK TO

```basic
' Debug output (goes to current conversation)
PRINT "Starting order notification..."

' User-facing message to specific WhatsApp number
TALK TO "whatsapp:+5511999887766", "Your order is confirmed!"

' More debug output
PRINT "Notification sent successfully"
```

## Best Practices

1. **Use TALK for production** - More readable for user-facing messages
2. **Use PRINT for debugging** - Makes it clear this is debug/log output
3. **Use TALK TO for channels** - When sending to specific recipients

```basic
' Good: Clear intent
PRINT "Debug: Processing started"
TALK "Welcome! How can I help you?"
TALK TO "whatsapp:" + phone, "Your order is ready!"

' Also valid but less clear:
PRINT "Welcome! How can I help you?"  ' Works but TALK is clearer
```

## See Also

- [TALK](./keyword-talk.md) - Primary message output keyword
- [TALK TO](./keyword-talk.md#talk-to) - Send to specific recipients
- [SEND FILE TO](./keyword-send-file-to.md) - Send files to recipients