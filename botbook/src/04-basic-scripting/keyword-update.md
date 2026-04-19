# UPDATE

The `UPDATE` keyword modifies existing records in database tables, enabling bots to change stored data based on conditions.

---

## Syntax

```basic
UPDATE "table_name" SET field1 = value1 WHERE condition
UPDATE "table_name" SET field1 = value1, field2 = value2 WHERE condition
UPDATE "table_name" ON connection SET field1 = value1 WHERE condition
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `table_name` | String | Name of the target database table |
| `SET` | Clause | Field-value pairs to update |
| `WHERE` | Clause | Condition to select records to update |
| `ON connection` | String | Optional named database connection |

---

## Description

`UPDATE` modifies existing records in a database table that match the specified `WHERE` condition. The `SET` clause specifies which fields to change and their new values. Without a `WHERE` clause, all records in the table would be updated (which is usually not desired).

Use cases include:
- Updating user profiles
- Changing order status
- Recording timestamps for actions
- Incrementing counters
- Marking items as read/processed

---

## Examples

### Basic Update

```basic
' Update a customer's email
UPDATE "customers" SET email = "new.email@example.com" WHERE id = 123

TALK "Email updated successfully!"
```

### Update Multiple Fields

```basic
' Update multiple fields at once
UPDATE "orders" SET
    status = "shipped",
    shipped_at = NOW(),
    tracking_number = tracking_id
WHERE id = order_id

TALK "Order #" + order_id + " marked as shipped"
```

### Update with Variable Values

```basic
' Update from conversation data
TALK "What is your new phone number?"
HEAR new_phone

UPDATE "customers" SET phone = new_phone WHERE id = user.id

TALK "Your phone number has been updated to " + new_phone
```

### Increment Counter

```basic
' Increment a counter field
UPDATE "products" SET view_count = view_count + 1 WHERE id = product_id
```

### Update Based on Condition

```basic
' Mark old sessions as expired
UPDATE "sessions" SET
    status = "expired",
    expired_at = NOW()
WHERE last_activity < DATEADD(NOW(), -30, "minute")

TALK "Inactive sessions have been expired"
```

### Update with Named Connection

```basic
' Update on specific database
UPDATE "audit_log" ON "analytics_db" SET
    reviewed = true,
    reviewed_by = admin.id
WHERE id = log_entry_id
```

---

## Common Use Cases

### Update User Profile

```basic
' User wants to update their profile
TALK "What would you like to update? (name, email, phone)"
HEAR field_to_update

TALK "What is the new value?"
HEAR new_value

SWITCH field_to_update
    CASE "name"
        UPDATE "users" SET name = new_value WHERE id = user.id
    CASE "email"
        UPDATE "users" SET email = new_value WHERE id = user.id
    CASE "phone"
        UPDATE "users" SET phone = new_value WHERE id = user.id
    CASE ELSE
        TALK "Unknown field. Please choose name, email, or phone."
END SWITCH

TALK "Your " + field_to_update + " has been updated!"
```

### Change Order Status

```basic
' Update order through its lifecycle
UPDATE "orders" SET
    status = "processing",
    processed_at = NOW()
WHERE id = order_id AND status = "pending"

TALK "Order is now being processed"
```

### Mark as Read

```basic
' Mark notification as read
UPDATE "notifications" SET
    read = true,
    read_at = NOW()
WHERE user_id = user.id AND id = notification_id

TALK "Notification marked as read"
```

### Record Last Activity

```basic
' Update last activity timestamp
UPDATE "users" SET last_active = NOW() WHERE id = user.id
```

### Soft Delete

```basic
' Soft delete (mark as deleted without removing)
UPDATE "records" SET
    deleted = true,
    deleted_at = NOW(),
    deleted_by = user.id
WHERE id = record_id

TALK "Record archived"
```

### Batch Update

```basic
' Update multiple records matching condition
UPDATE "subscriptions" SET
    status = "active",
    renewed_at = NOW()
WHERE expires_at > NOW() AND auto_renew = true

TALK "Active subscriptions renewed"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

UPDATE "customers" SET email = new_email WHERE id = customer_id

IF ERROR THEN
    PRINT "Update failed: " + ERROR_MESSAGE
    
    IF INSTR(ERROR_MESSAGE, "duplicate") > 0 THEN
        TALK "This email is already in use by another account."
    ELSE IF INSTR(ERROR_MESSAGE, "constraint") > 0 THEN
        TALK "The value you entered is not valid."
    ELSE
        TALK "Sorry, I couldn't update your information. Please try again."
    END IF
ELSE
    TALK "Information updated successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `DUPLICATE_KEY` | Unique constraint violated | Value already exists |
| `CHECK_VIOLATION` | Value fails check constraint | Validate before update |
| `NOT_NULL_VIOLATION` | Setting required field to null | Provide a value |
| `NO_ROWS_AFFECTED` | WHERE matched no records | Verify condition |

---

## Safety Considerations

### Always Use WHERE Clause

```basic
' DANGEROUS - updates ALL records!
' UPDATE "users" SET status = "inactive"

' SAFE - updates only matching records
UPDATE "users" SET status = "inactive" WHERE last_login < "2024-01-01"
```

### Verify Before Update

```basic
' Check record exists before updating
record = FIND "orders" WHERE id = order_id

IF record THEN
    UPDATE "orders" SET status = "cancelled" WHERE id = order_id
    TALK "Order cancelled"
ELSE
    TALK "Order not found"
END IF
```

### Limit Scope

```basic
' Update only records the user owns
UPDATE "documents" SET
    title = new_title
WHERE id = document_id AND owner_id = user.id
```

---

## UPDATE vs MERGE

| Keyword | Purpose | Use When |
|---------|---------|----------|
| `UPDATE` | Modify existing records | Record definitely exists |
| `MERGE` | Insert or update | Record may or may not exist |

```basic
' UPDATE - Only modifies if exists
UPDATE "users" SET name = "John" WHERE email = "john@example.com"

' MERGE - Creates if not exists, updates if exists
MERGE INTO "users" ON email = "john@example.com" WITH
    email = "john@example.com",
    name = "John"
```

---

## Configuration

Database connection is configured in `config.csv`:

```csv
name,value
database-provider,postgres
database-pool-size,10
database-timeout,30
```

Database credentials are stored in Vault, not in config files.

---

## Implementation Notes

- Implemented in Rust under `src/database/operations.rs`
- Uses parameterized queries to prevent SQL injection
- Returns number of affected rows
- WHERE clause is required by default for safety
- Supports all comparison operators (=, <, >, <=, >=, <>)
- Supports AND/OR in WHERE conditions

---

## Related Keywords

- [INSERT](keyword-insert.md) — Add new records
- [DELETE](keyword-delete.md) — Remove records
- [MERGE](keyword-merge.md) — Insert or update (upsert)
- [FIND](keyword-find.md) — Query records
- [TABLE](keyword-table.md) — Create tables

---

## Summary

`UPDATE` modifies existing database records that match a WHERE condition. Use it to change user data, update statuses, record timestamps, and modify stored information. Always include a WHERE clause to avoid accidentally updating all records. For cases where you're unsure if a record exists, consider using `MERGE` instead.