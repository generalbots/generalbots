# PUT

The `PUT` keyword sends HTTP PUT requests to external APIs, used for replacing or updating entire resources.

---

## Syntax

```basic
result = PUT url, data
PUT url WITH field1 = value1, field2 = value2
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | String | The target URL endpoint |
| `data` | String | JSON string for request body |
| `WITH` | Clause | Field-value pairs for the request body |

---

## Description

`PUT` sends data to a specified URL using the HTTP PUT method. In REST APIs, PUT is used to:

- Replace an entire resource with new data
- Create a resource at a specific URL if it doesn't exist
- Update all fields of an existing resource

Unlike `PATCH` which updates partial data, `PUT` typically replaces the entire resource.

---

## Examples

### Basic PUT Request

```basic
' Update entire user profile
result = PUT "https://api.example.com/users/123" WITH
    name = "John Doe",
    email = "john.doe@example.com",
    phone = "+1-555-0100",
    status = "active"

IF result.success THEN
    TALK "Profile updated successfully!"
ELSE
    TALK "Update failed: " + result.error
END IF
```

### Replace Configuration

```basic
' Replace entire configuration object
result = PUT "https://api.example.com/config/bot-settings" WITH
    theme = "dark",
    language = "en",
    notifications = true,
    auto_reply = false

TALK "Configuration saved"
```

### Update Product

```basic
' Replace product details
result = PUT "https://api.store.com/products/SKU-001" WITH
    name = "Premium Widget",
    price = 49.99,
    stock = 100,
    category = "electronics",
    description = "High-quality widget with premium features"

TALK "Product updated: " + result.name
```

### With Authentication

```basic
' Set authorization header first
SET HEADER "Authorization", "Bearer " + api_token
SET HEADER "Content-Type", "application/json"

' Make authenticated PUT request
result = PUT "https://api.service.com/resources/456" WITH
    title = "Updated Title",
    content = new_content,
    updated_by = user.id

' Clear headers after request
SET HEADER "Authorization", ""
```

### Using JSON String

```basic
' PUT with JSON string body
json_body = '{"name": "Updated Name", "status": "published"}'
result = PUT "https://api.example.com/articles/789", json_body

TALK "Article updated!"
```

---

## PUT vs PATCH vs POST

| Method | Purpose | Body Contains |
|--------|---------|---------------|
| `POST` | Create new resource | New resource data |
| `PUT` | Replace entire resource | Complete resource data |
| `PATCH` | Update partial resource | Only changed fields |

```basic
' POST - Create new
result = POST "https://api.example.com/users" WITH
    name = "New User",
    email = "new@example.com"
' Creates user, returns new ID

' PUT - Replace entire resource
result = PUT "https://api.example.com/users/123" WITH
    name = "Updated Name",
    email = "updated@example.com",
    phone = "+1-555-0100"
' All fields required, replaces entire user

' PATCH - Update specific fields
result = PATCH "https://api.example.com/users/123" WITH
    phone = "+1-555-0200"
' Only phone is updated, other fields unchanged
```

---

## Common Use Cases

### Update User Settings

```basic
' Save all user preferences
result = PUT "https://api.example.com/users/" + user.id + "/settings" WITH
    email_notifications = true,
    sms_notifications = false,
    timezone = "America/New_York",
    language = "en"

TALK "Your settings have been saved!"
```

### Replace Document

```basic
' Upload new version of document (replaces existing)
document_content = READ "templates/contract.md"

result = PUT "https://api.docs.com/documents/" + doc_id WITH
    title = "Service Agreement v2.0",
    content = document_content,
    version = "2.0",
    last_modified = FORMAT(NOW(), "ISO8601")

TALK "Document replaced with new version"
```

### Update Order Status

```basic
' Replace order with updated status
result = PUT "https://api.orders.com/orders/" + order_id WITH
    customer_id = order.customer_id,
    items = order.items,
    total = order.total,
    status = "shipped",
    tracking_number = tracking_id,
    shipped_at = FORMAT(NOW(), "ISO8601")

TALK "Order marked as shipped!"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = PUT "https://api.example.com/resource/123" WITH
    field1 = value1,
    field2 = value2

IF ERROR THEN
    PRINT "PUT request failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't update that information."
ELSE IF result.error THEN
    TALK "Update failed: " + result.error.message
ELSE
    TALK "Update successful!"
END IF
```

### Common HTTP Status Codes

| Status | Meaning | Action |
|--------|---------|--------|
| 200 | Success, resource updated | Process response |
| 201 | Created (resource didn't exist) | New resource created |
| 204 | Success, no content returned | Update complete |
| 400 | Bad request | Check request data |
| 401 | Unauthorized | Check authentication |
| 404 | Resource not found | Verify URL/ID |
| 409 | Conflict | Resource was modified |
| 422 | Validation error | Check field values |

---

## Configuration

Configure HTTP settings in `config.csv`:

```csv
name,value
http-timeout,30
http-retry-count,3
http-retry-delay,1000
```

---

## Implementation Notes

- Implemented in Rust under `src/web_automation/http.rs`
- Automatically serializes WITH clause to JSON
- Supports custom headers via SET HEADER
- Returns parsed JSON response
- Handles redirects (up to 10 hops)

---

## Related Keywords

- [GET](keyword-get.md) — Retrieve data from URLs
- [POST](keyword-post.md) — Create new resources
- [PATCH](keyword-patch.md) — Partial resource updates
- [DELETE HTTP](keyword-delete-http.md) — Remove resources
- [SET HEADER](keyword-set-header.md) — Set request headers

---

## Summary

`PUT` replaces entire resources via HTTP PUT requests. Use it when you need to update all fields of a resource or create a resource at a specific URL. For partial updates where you only change specific fields, use `PATCH` instead. Always include all required fields when using PUT, as missing fields may be set to null or cause errors.