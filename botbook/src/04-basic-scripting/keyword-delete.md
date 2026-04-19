# DELETE

The `DELETE` keyword is a unified command that automatically detects context and handles HTTP requests, database operations, and file deletions through a single interface.

---

## Syntax

```basic
' HTTP DELETE - auto-detected by URL
DELETE "https://api.example.com/resource/123"

' Database DELETE - table with filter
DELETE "table_name", "filter_condition"

' File DELETE - path without URL
DELETE "path/to/file.txt"
```

---

## Parameters

| Context | Parameter 1 | Parameter 2 | Description |
|---------|-------------|-------------|-------------|
| HTTP | URL (string) | - | DELETE request to the URL |
| Database | Table name | Filter condition | Delete matching records |
| File | File path | - | Delete the file |

---

## Description

`DELETE` is a smart, unified keyword that detects what you want to delete based on the arguments:

1. **HTTP DELETE**: If the first argument starts with `http://` or `https://`, sends an HTTP DELETE request
2. **Database DELETE**: If two arguments are provided (table, filter), performs SQL DELETE
3. **File DELETE**: Otherwise, treats the argument as a file path

This eliminates the need for separate `DELETE HTTP`, `DELETE FILE` commands - just use `DELETE`.

---

## Examples

### HTTP DELETE

```basic
' Delete a resource via REST API
DELETE "https://api.example.com/users/123"

TALK "User deleted from API"
```

```basic
' Delete with authentication (set headers first)
SET HEADER "Authorization", "Bearer " + api_token
DELETE "https://api.example.com/posts/" + post_id
CLEAR HEADERS

TALK "Post deleted"
```

### Database DELETE

```basic
' Delete by ID
DELETE "customers", "id = 123"

TALK "Customer deleted"
```

```basic
' Delete with variable
DELETE "orders", "id = " + order_id + " AND user_id = " + user.id

TALK "Order cancelled"
```

```basic
' Delete with multiple conditions
DELETE "sessions", "user_id = " + user.id + " AND status = 'expired'"

TALK "Expired sessions cleared"
```

```basic
' Delete old records
DELETE "logs", "created_at < '2024-01-01'"

TALK "Old logs purged"
```

### File DELETE

```basic
' Delete a file
DELETE "temp/report.pdf"

TALK "File deleted"
```

```basic
' Delete uploaded file
DELETE "uploads/" + filename

TALK "Upload removed"
```

---

## Common Use Cases

### REST API Resource Deletion

```basic
' Delete item from external service
TALK "Removing item from inventory system..."

SET HEADER "Authorization", "Bearer " + inventory_api_key
SET HEADER "Content-Type", "application/json"

result = DELETE "https://inventory.example.com/api/items/" + item_id

CLEAR HEADERS

IF result THEN
    TALK "Item removed from inventory"
ELSE
    TALK "Failed to remove item"
END IF
```

### User Account Deletion

```basic
' Complete account deletion flow
TALK "Are you sure you want to delete your account? Type 'DELETE' to confirm."
HEAR confirmation

IF confirmation = "DELETE" THEN
    ' Delete related records first
    DELETE "orders", "customer_id = " + user.id
    DELETE "addresses", "customer_id = " + user.id
    DELETE "preferences", "user_id = " + user.id
    
    ' Delete the user
    DELETE "users", "id = " + user.id
    
    TALK "Your account has been deleted."
ELSE
    TALK "Account deletion cancelled."
END IF
```

### Cleanup Temporary Files

```basic
' Clean up temp files after processing
temp_files = ["temp/doc1.pdf", "temp/doc2.pdf", "temp/merged.pdf"]

FOR EACH f IN temp_files
    DELETE f
NEXT

TALK "Temporary files cleaned up"
```

### Cancel Order via API

```basic
' Cancel order in external system
order_api_url = "https://orders.example.com/api/orders/" + order_id

SET HEADER "Authorization", "Bearer " + api_key
DELETE order_api_url
CLEAR HEADERS

' Also remove from local database
DELETE "local_orders", "external_id = '" + order_id + "'"

TALK "Order cancelled"
```

### Remove Expired Data

```basic
' Scheduled cleanup task
' Delete expired tokens
DELETE "tokens", "expires_at < NOW()"

' Delete old notifications
DELETE "notifications", "read = true AND created_at < DATEADD(NOW(), -90, 'day')"

' Delete abandoned carts
DELETE "carts", "updated_at < DATEADD(NOW(), -7, 'day') AND checkout_completed = false"

TALK "Cleanup complete"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

DELETE "orders", "id = " + order_id

IF ERROR THEN
    error_msg = ERROR MESSAGE
    
    IF INSTR(error_msg, "foreign key") > 0 THEN
        TALK "Cannot delete: this record is referenced by other data."
    ELSE IF INSTR(error_msg, "not found") > 0 THEN
        TALK "Record not found."
    ELSE IF INSTR(error_msg, "permission") > 0 THEN
        TALK "You don't have permission to delete this."
    ELSE
        TALK "Delete failed: " + error_msg
    END IF
ELSE
    TALK "Deleted successfully!"
END IF

ON ERROR GOTO 0
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FOREIGN_KEY_VIOLATION` | Database record referenced elsewhere | Delete child records first |
| `FILE_NOT_FOUND` | File doesn't exist | Check file path |
| `HTTP 404` | API resource not found | Verify URL and resource ID |
| `HTTP 401/403` | Authentication failed | Check API credentials |
| `PERMISSION_DENIED` | Insufficient privileges | Check permissions |

---

## Context Detection

The `DELETE` keyword automatically detects context:

| Argument Pattern | Detected As |
|------------------|-------------|
| `"https://..."` or `"http://..."` | HTTP DELETE |
| Two arguments: `"table", "filter"` | Database DELETE |
| Single argument without URL prefix | File DELETE |

```basic
' HTTP - starts with http/https
DELETE "https://api.example.com/resource/1"

' Database - two arguments
DELETE "users", "id = 123"

' File - single argument, no URL prefix
DELETE "temp/file.txt"
```

---

## Safety Considerations

### Always Use Filters for Database

```basic
' DANGEROUS - would delete all records!
' DELETE "users", ""

' SAFE - specific condition
DELETE "users", "id = " + user_id
```

### Verify Before Deleting

```basic
' Check record exists and belongs to user
record = FIND "documents", "id = " + doc_id + " AND owner_id = " + user.id

IF record THEN
    DELETE "documents", "id = " + doc_id
    TALK "Document deleted"
ELSE
    TALK "Document not found or access denied"
END IF
```

### Confirm Destructive Actions

```basic
TALK "Delete " + item_name + "? This cannot be undone. Type 'yes' to confirm."
HEAR confirmation

IF LOWER(confirmation) = "yes" THEN
    DELETE "items", "id = " + item_id
    TALK "Deleted"
ELSE
    TALK "Cancelled"
END IF
```

### Consider Soft Delete

```basic
' Instead of permanent delete, mark as deleted
UPDATE "records", #{ "deleted": true, "deleted_at": NOW() }, "id = " + record_id

TALK "Record archived (can be restored)"
```

---

## Return Values

| Context | Returns |
|---------|---------|
| HTTP | Response body as string |
| Database | Number of deleted rows |
| File | `true` on success, error message on failure |

---

## Configuration

No specific configuration required. Uses:
- HTTP: Standard HTTP client
- Database: Connection from `config.csv`
- Files: Bot's `.gbdrive` storage

---

## Implementation Notes

- Implemented in `data_operations.rs`
- Auto-detects URL vs table vs file
- HTTP DELETE supports custom headers via `SET HEADER`
- Database DELETE uses parameterized queries (SQL injection safe)
- File DELETE works within bot's storage sandbox

---

## Related Keywords

- [INSERT](keyword-insert.md) — Add new records
- [UPDATE](keyword-update.md) — Modify existing records
- [FIND](keyword-find.md) — Query records
- [POST](keyword-post.md) — HTTP POST requests
- [PUT](keyword-put.md) — HTTP PUT requests
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write file contents

---

## Summary

`DELETE` is a unified keyword that intelligently handles HTTP API deletions, database record removal, and file deletion through a single interface. It auto-detects context based on arguments: URLs trigger HTTP DELETE, table+filter triggers database DELETE, and paths trigger file DELETE. Always use filters for database operations, verify ownership before deleting user data, and confirm destructive actions. For recoverable deletions, consider soft delete instead.