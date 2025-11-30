# WEBHOOK

Creates a webhook endpoint for event-driven automation. When the webhook URL is called, the script containing the WEBHOOK declaration is executed.

## Syntax

```basic
WEBHOOK "endpoint-name"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| endpoint-name | String | The unique name for the webhook endpoint (alphanumeric, hyphens, underscores) |

## Description

The WEBHOOK keyword registers an HTTP endpoint that triggers script execution when called externally. This enables event-driven automation where external systems can notify your bot of events.

The webhook creates an endpoint at:
```
POST /api/{botname}/webhook/{endpoint-name}
```

When the webhook is triggered:
1. The script containing the WEBHOOK declaration executes
2. Request data is available through special variables:
   - `params` - Query string parameters (e.g., `?id=123`)
   - `body` - JSON request body
   - `headers` - HTTP headers
   - `method` - HTTP method (usually POST)

## Example

### Basic Order Webhook

```basic
' order-received.bas
WEBHOOK "order-received"

' Access request data
order_id = params.order_id
customer_name = body.customer.name
customer_email = body.customer.email
total = body.total
items = body.items

' Log the order
PRINT "Received order: " + order_id

' Save to database
order_data = #{
    "customer_name": customer_name,
    "email": customer_email,
    "total": total,
    "status": "pending",
    "created_at": NOW()
}
SAVE "orders", order_id, order_data

' Send confirmation email
SEND MAIL customer_email, "Order Confirmation", "Thank you for your order #" + order_id

' Return response (optional - script return value becomes response)
result = #{ "status": "ok", "order_id": order_id, "message": "Order received" }
```

### Calling the Webhook

```bash
curl -X POST https://bot.example.com/api/mybot/webhook/order-received \
  -H "Content-Type: application/json" \
  -d '{
    "customer": {
      "name": "John Doe",
      "email": "john@example.com"
    },
    "items": [
      {"product": "Widget", "qty": 2, "price": 29.99}
    ],
    "total": 59.98
  }'
```

### Payment Notification Webhook

```basic
' payment-webhook.bas
WEBHOOK "payment-notification"

' Verify webhook signature (if provided)
signature = headers.x_webhook_signature
IF signature = "" THEN
    PRINT "Warning: No signature provided"
END IF

' Process payment event
event_type = body.event
payment_id = body.payment.id
amount = body.payment.amount
status = body.payment.status

SWITCH event_type
    CASE "payment.completed"
        UPDATE "orders", "payment_id=" + payment_id, #{ "status": "paid", "paid_at": NOW() }
        TALK "Payment " + payment_id + " completed"
    
    CASE "payment.failed"
        UPDATE "orders", "payment_id=" + payment_id, #{ "status": "payment_failed" }
        ' Notify customer
        order = FIND "orders", "payment_id=" + payment_id
        SEND MAIL order.email, "Payment Failed", "Your payment could not be processed."
    
    CASE "payment.refunded"
        UPDATE "orders", "payment_id=" + payment_id, #{ "status": "refunded", "refunded_at": NOW() }
    
    DEFAULT
        PRINT "Unknown event type: " + event_type
END SWITCH

result = #{ "received": true }
```

### GitHub Webhook Integration

```basic
' github-webhook.bas
WEBHOOK "github-push"

' GitHub sends event type in header
event_type = headers.x_github_event
repository = body.repository.full_name
pusher = body.pusher.name

IF event_type = "push" THEN
    branch = body.ref
    commits = body.commits
    commit_count = UBOUND(commits)
    
    ' Log the push
    message = pusher + " pushed " + commit_count + " commit(s) to " + repository + " (" + branch + ")"
    PRINT message
    
    ' Notify team via Slack/Teams
    POST "https://hooks.slack.com/services/xxx", #{ "text": message }
    
    ' Trigger deployment if main branch
    IF branch = "refs/heads/main" THEN
        PRINT "Triggering deployment..."
        POST "https://deploy.example.com/trigger", #{ "repo": repository }
    END IF
END IF

result = #{ "status": "processed" }
```

## Response Handling

The webhook automatically returns a JSON response. You can control the response by setting a `result` variable:

```basic
WEBHOOK "my-endpoint"

' Process request...

' Simple success response
result = #{ "status": "ok" }

' Or with custom status code
result = #{
    "status": 201,
    "body": #{ "id": new_id, "created": true },
    "headers": #{ "X-Custom-Header": "value" }
}
```

## Security Considerations

1. **Validate signatures**: Many services (Stripe, GitHub, etc.) sign webhook payloads
2. **Verify source**: Check request headers or IP addresses when possible
3. **Use HTTPS**: Always use HTTPS endpoints in production
4. **Idempotency**: Design webhooks to handle duplicate deliveries gracefully

```basic
WEBHOOK "secure-webhook"

' Verify HMAC signature
expected_signature = HASH(body, secret_key, "sha256")
IF headers.x_signature != expected_signature THEN
    PRINT "Invalid signature - rejecting request"
    result = #{ "status": 401, "body": #{ "error": "Invalid signature" } }
    EXIT
END IF

' Continue processing...
```

## Use Cases

- **E-commerce**: Order notifications, payment confirmations, inventory updates
- **CI/CD**: Build notifications, deployment triggers
- **CRM**: Lead notifications, deal updates
- **IoT**: Sensor data ingestion, device status updates
- **Third-party integrations**: Slack commands, form submissions, calendar events

## Notes

- Webhook endpoints are registered during script compilation
- Multiple scripts can define different webhooks
- Webhooks are stored in the `system_automations` table
- The endpoint name must be unique per bot
- Request timeout is typically 30 seconds - keep processing fast

## See Also

- [SET SCHEDULE](./keyword-set-schedule.md) - Time-based automation
- [ON](./keyword-on.md) - Database trigger events
- [POST](./keyword-post.md) - Making outbound HTTP requests