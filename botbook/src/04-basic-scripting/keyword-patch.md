# PATCH

The `PATCH` keyword sends HTTP PATCH requests to external APIs, used for partial updates to existing resources.

---

## Syntax

```basic
result = PATCH url, data
PATCH url WITH field1 = value1, field2 = value2
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

`PATCH` sends partial data to a specified URL using the HTTP PATCH method. In REST APIs, PATCH is used to:

- Update specific fields without affecting others
- Make incremental changes to resources
- Modify only what has changed

Unlike `PUT` which replaces the entire resource, `PATCH` only updates the fields you specify.

---

## Examples

### Basic PATCH Request

```basic
' Update only the user's email
result = PATCH "https://api.example.com/users/123" WITH
    email = "new.email@example.com"

IF result.success THEN
    TALK "Email updated successfully!"
ELSE
    TALK "Update failed: " + result.error
END IF
```

### Update Status Only

```basic
' Change order status without modifying other fields
PATCH "https://api.orders.com/orders/" + order_id WITH
    status = "shipped"

TALK "Order status updated to shipped"
```

### Update Multiple Fields

```basic
' Update several fields at once
result = PATCH "https://api.example.com/products/SKU-001" WITH
    price = 39.99,
    stock = 150,
    on_sale = true

TALK "Product updated: price, stock, and sale status"
```

### With Authentication

```basic
' Set authorization header first
SET HEADER "Authorization", "Bearer " + api_token
SET HEADER "Content-Type", "application/json"

' Make authenticated PATCH request
result = PATCH "https://api.service.com/resources/456" WITH
    title = "Updated Title"

' Clear headers after request
SET HEADER "Authorization", ""
```

### Using JSON String

```basic
' PATCH with JSON string body
json_body = '{"status": "archived", "archived_at": "2025-01-15T10:00:00Z"}'
result = PATCH "https://api.example.com/documents/789", json_body

TALK "Document archived!"
```

---

## PATCH vs PUT

| Aspect | PATCH | PUT |
|--------|-------|-----|
| **Purpose** | Update specific fields | Replace entire resource |
| **Body Contains** | Only changed fields | All resource fields |
| **Missing Fields** | Unchanged | May be set to null |
| **Use When** | Changing 1-2 fields | Replacing whole object |

```basic
' PATCH - Only update what changed
result = PATCH "https://api.example.com/users/123" WITH
    phone = "+1-555-0200"
' Only phone is updated, name/email/etc unchanged

' PUT - Must include all fields
result = PUT "https://api.example.com/users/123" WITH
    name = "John Doe",
    email = "john@example.com",
    phone = "+1-555-0200",
    status = "active"
' All fields required, replaces entire user
```

---

## Common Use Cases

### Toggle Feature Flag

```basic
' Enable a single feature
PATCH "https://api.example.com/users/" + user.id + "/settings" WITH
    dark_mode = true

TALK "Dark mode enabled!"
```

### Update Profile Field

```basic
' User wants to change their display name
TALK "What would you like your new display name to be?"
HEAR new_name

result = PATCH "https://api.example.com/users/" + user.id WITH
    display_name = new_name

TALK "Your display name is now: " + new_name
```

### Mark as Read

```basic
' Mark notification as read
PATCH "https://api.example.com/notifications/" + notification_id WITH
    read = true,
    read_at = FORMAT(NOW(), "ISO8601")

TALK "Notification marked as read"
```

### Update Progress

```basic
' Update task completion percentage
PATCH "https://api.tasks.com/tasks/" + task_id WITH
    progress = 75,
    last_updated = FORMAT(NOW(), "ISO8601")

TALK "Task progress updated to 75%"
```

### Increment Counter

```basic
' Update view count (if API supports increment)
result = PATCH "https://api.content.com/articles/" + article_id WITH
    views = current_views + 1

' Or if API has increment syntax
PATCH "https://api.content.com/articles/" + article_id WITH
    increment_views = 1
```

### Soft Delete

```basic
' Mark record as deleted without removing it
PATCH "https://api.example.com/records/" + record_id WITH
    deleted = true,
    deleted_at = FORMAT(NOW(), "ISO8601"),
    deleted_by = user.id

TALK "Record archived (can be restored if needed)"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = PATCH "https://api.example.com/resource/123" WITH
    status = "updated"

IF ERROR THEN
    PRINT "PATCH request failed: " + ERROR_MESSAGE
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
| 200 | Success, updated resource returned | Process response |
| 204 | Success, no content returned | Update complete |
| 400 | Bad request | Check field names/values |
| 401 | Unauthorized | Check authentication |
| 404 | Resource not found | Verify URL/ID |
| 409 | Conflict | Resource was modified by another |
| 422 | Validation error | Check field constraints |

---

## Best Practices

1. **Update only changed fields** — Don't include unchanged data
2. **Check response** — Verify the update was applied correctly
3. **Handle conflicts** — Be prepared for concurrent modification errors
4. **Use optimistic locking** — Include version/etag if API supports it

```basic
' With version checking (if API supports it)
SET HEADER "If-Match", current_etag

result = PATCH "https://api.example.com/resource/123" WITH
    field = new_value

IF result.status = 409 THEN
    TALK "Someone else modified this. Please refresh and try again."
END IF
```

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
- Content-Type defaults to `application/json`

---

## Related Keywords

- [GET](keyword-get.md) — Retrieve data from URLs
- [POST](keyword-post.md) — Create new resources
- [PUT](keyword-put.md) — Replace entire resources
- [DELETE HTTP](keyword-delete-http.md) — Remove resources
- [SET HEADER](keyword-set-header.md) — Set request headers

---

## Summary

`PATCH` updates specific fields of a resource via HTTP PATCH requests. Use it when you only need to change one or a few fields without affecting the rest of the resource. This is more efficient than PUT and reduces the risk of accidentally overwriting data. Always specify only the fields that need to change.