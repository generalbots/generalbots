# POST

The `POST` keyword sends HTTP POST requests to external APIs and web services, enabling bots to create resources, submit data, and integrate with third-party systems.

---

## Syntax

```basic
result = POST url, data
result = POST url, data, content_type
POST url, param1, param2, param3, ...
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | String | The target URL endpoint |
| `data` | String/Object | Request body (JSON string or object) |
| `content_type` | String | Optional content type (default: `application/json`) |
| `param1, param2, ...` | Any | Positional parameters for form-style requests |

---

## Description

`POST` sends data to a specified URL using the HTTP POST method. This is the primary keyword for:

- Creating new resources in REST APIs
- Submitting form data
- Triggering webhooks
- Sending notifications to external services
- Integrating with third-party platforms

The response is returned as a parsed JSON object when possible, or as a string for other content types.

---

## Examples

### Basic JSON POST

```basic
' Create a new user via API
data = '{"name": "John Doe", "email": "john@example.com"}'
result = POST "https://api.example.com/users", data

TALK "User created with ID: " + result.id
```

### Using WITH Syntax

```basic
' Create order using WITH keyword
result = POST "https://api.store.com/orders" WITH
    customer_id = "cust-123",
    items = ["item-1", "item-2"],
    total = 99.99

TALK "Order " + result.order_id + " placed successfully!"
```

### Form-Style Parameters

```basic
' Submit with positional parameters
POST "https://warehouse.internal/api/orders", order_id, items, shipping_address, "express"
```

### With Custom Headers

```basic
' Set authorization header first
SET HEADER "Authorization", "Bearer " + api_token
SET HEADER "X-Request-ID", request_id

result = POST "https://api.service.com/data", payload

' Clear headers after request
SET HEADER "Authorization", ""
```

### Webhook Integration

```basic
' Send Slack notification
POST "https://hooks.slack.com/services/xxx/yyy/zzz" WITH
    channel = "#alerts",
    text = "New order received: " + order_id,
    username = "Order Bot"
```

### Creating Records

```basic
' Create a support ticket
result = POST "https://helpdesk.example.com/api/tickets" WITH
    title = "Customer inquiry",
    description = user_message,
    priority = "medium",
    customer_email = customer.email

IF result.id THEN
    TALK "Ticket #" + result.id + " created. Our team will respond within 24 hours."
ELSE
    TALK "Sorry, I couldn't create the ticket. Please try again."
END IF
```

---

## Handling Responses

### Check Response Status

```basic
result = POST "https://api.example.com/resource", data

IF result.error THEN
    TALK "Error: " + result.error.message
ELSE IF result.id THEN
    TALK "Success! Created resource: " + result.id
END IF
```

### Parse Nested Response

```basic
result = POST "https://api.payment.com/charge", payment_data

IF result.status = "succeeded" THEN
    TALK "Payment of $" + result.amount + " processed!"
    TALK "Transaction ID: " + result.transaction_id
ELSE
    TALK "Payment failed: " + result.failure_reason
END IF
```

---

## Common Use Cases

### Send Email via API

```basic
POST "https://api.mailservice.com/send" WITH
    to = customer_email,
    subject = "Order Confirmation",
    body = "Thank you for your order #" + order_id
```

### Create Calendar Event

```basic
result = POST "https://calendar.api.com/events" WITH
    title = "Meeting with " + contact_name,
    start = meeting_time,
    duration = 60,
    attendees = [contact_email]

TALK "Meeting scheduled! Calendar invite sent."
```

### Log Analytics Event

```basic
' Track user action
POST "https://analytics.example.com/track" WITH
    event = "purchase_completed",
    user_id = user.id,
    order_value = total,
    items_count = LEN(cart)
```

### CRM Integration

```basic
' Create lead in CRM
result = POST "https://crm.example.com/api/leads" WITH
    first_name = first_name,
    last_name = last_name,
    email = email,
    phone = phone,
    source = "chatbot",
    notes = "Initial inquiry: " + user_query

SET USER MEMORY "crm_lead_id", result.id
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = POST "https://api.example.com/resource", data

IF ERROR THEN
    PRINT "POST failed: " + ERROR_MESSAGE
    ' Try backup endpoint
    result = POST "https://backup-api.example.com/resource", data
END IF

IF result.error THEN
    TALK "The service returned an error. Please try again later."
ELSE
    TALK "Request successful!"
END IF
```

---

## Content Types

| Content Type | Use Case |
|--------------|----------|
| `application/json` | Default, most REST APIs |
| `application/x-www-form-urlencoded` | HTML form submissions |
| `multipart/form-data` | File uploads (use UPLOAD instead) |
| `text/xml` | SOAP services (use SOAP instead) |

```basic
' Explicit content type
result = POST "https://legacy.api.com/submit", form_data, "application/x-www-form-urlencoded"
```

---

## Configuration

### Timeouts

Configure request timeout in `config.csv`:

```csv
name,value
http-timeout,30
http-retry-count,3
http-retry-delay,1000
```

### Base URL

Set a base URL for all HTTP requests:

```csv
name,value
http-base-url,https://api.mycompany.com
```

Then use relative paths:

```basic
result = POST "/users", user_data  ' Resolves to https://api.mycompany.com/users
```

---

## Implementation Notes

- Implemented in Rust under `src/web_automation/http.rs`
- Uses `reqwest` library with async runtime
- Automatically serializes objects to JSON
- Handles redirects (up to 10 hops)
- Validates SSL certificates by default
- Supports gzip/deflate response compression

---

## Related Keywords

- [GET](keyword-get.md) — Retrieve data from URLs
- [PUT](keyword-put.md) — Update existing resources
- [PATCH](keyword-patch.md) — Partial resource updates
- [DELETE HTTP](keyword-delete-http.md) — Remove resources
- [SET HEADER](keyword-set-header.md) — Set request headers
- [GRAPHQL](keyword-graphql.md) — GraphQL queries and mutations

---

## Summary

`POST` is essential for integrating bots with external services. Use it to create resources, submit data, trigger webhooks, and connect to any REST API. Combined with `SET HEADER` for authentication, it enables powerful integrations with CRMs, payment systems, notification services, and more.