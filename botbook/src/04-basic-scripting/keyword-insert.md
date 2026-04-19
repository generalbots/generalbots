# INSERT

The `INSERT` keyword adds new records to database tables, enabling bots to store data collected from conversations and integrations.

---

## Syntax

```basic
INSERT INTO "table_name" WITH field1 = value1, field2 = value2
result = INSERT INTO "table_name" WITH field1 = value1, field2 = value2
INSERT INTO "table_name" ON connection WITH field1 = value1
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `table_name` | String | Name of the target database table |
| `WITH` | Clause | Field-value pairs for the new record |
| `ON connection` | String | Optional named database connection |

---

## Description

`INSERT` creates a new record in a database table. The `WITH` clause specifies the field names and values for the new row. The keyword returns the newly created record, including any auto-generated fields like `id`.

Use cases include:
- Storing user information collected during conversations
- Logging interactions and events
- Creating orders, tickets, or other business records
- Saving form submissions

---

## Examples

### Basic Insert

```basic
' Insert a new customer record
INSERT INTO "customers" WITH
    name = "John Doe",
    email = "john@example.com",
    phone = "+1-555-0100"

TALK "Customer record created!"
```

### Insert with Return Value

```basic
' Insert and capture the new record
result = INSERT INTO "customers" WITH
    name = customer_name,
    email = customer_email,
    created_at = NOW()

TALK "Customer created with ID: " + result.id
```

### Insert from Conversation

```basic
' Collect data from user and insert
TALK "What is your name?"
HEAR user_name

TALK "What is your email?"
HEAR user_email

TALK "What is your phone number?"
HEAR user_phone

result = INSERT INTO "contacts" WITH
    name = user_name,
    email = user_email,
    phone = user_phone,
    source = "chatbot",
    created_at = NOW()

TALK "Thanks " + user_name + "! Your contact ID is " + result.id
```

### Insert Order

```basic
' Create a new order
result = INSERT INTO "orders" WITH
    customer_id = user.id,
    product_id = selected_product.id,
    quantity = order_quantity,
    total = selected_product.price * order_quantity,
    status = "pending",
    created_at = NOW()

TALK "Order #" + result.id + " created for $" + result.total
```

### Insert with Foreign Key

```basic
' Insert related records
customer = INSERT INTO "customers" WITH
    name = customer_name,
    email = customer_email

address = INSERT INTO "addresses" WITH
    customer_id = customer.id,
    street = street_address,
    city = city_name,
    postal_code = zip_code,
    country = "US"

TALK "Customer and address saved!"
```

### Insert to Named Connection

```basic
' Insert to a specific database
INSERT INTO "audit_log" ON "analytics_db" WITH
    event = "user_signup",
    user_id = user.id,
    timestamp = NOW(),
    ip_address = session.ip
```

---

## Batch Insert

```basic
' Insert multiple records from a data source
new_contacts = READ "imports/contacts.csv" AS TABLE

inserted_count = 0

FOR EACH contact IN new_contacts
    INSERT INTO "contacts" WITH
        name = contact.name,
        email = contact.email,
        phone = contact.phone,
        imported_at = NOW()
    
    inserted_count = inserted_count + 1
NEXT

TALK "Imported " + inserted_count + " contacts"
```

---

## Common Use Cases

### Log User Interaction

```basic
' Log every conversation for analytics
INSERT INTO "conversation_logs" WITH
    user_id = user.id,
    session_id = session.id,
    message = user_message,
    response = bot_response,
    timestamp = NOW()
```

### Create Support Ticket

```basic
' Create a support ticket from conversation
result = INSERT INTO "tickets" WITH
    customer_id = user.id,
    subject = ticket_subject,
    description = ticket_description,
    priority = "medium",
    status = "open",
    created_at = NOW()

TALK "Ticket #" + result.id + " created. Our team will respond within 24 hours."
```

### Save Form Submission

```basic
' Save a lead form submission
result = INSERT INTO "leads" WITH
    first_name = form.first_name,
    last_name = form.last_name,
    email = form.email,
    company = form.company,
    interest = form.product_interest,
    source = "website_chatbot",
    created_at = NOW()

' Notify sales team
SEND MAIL "sales@company.com", "New Lead: " + form.first_name, "A new lead has been captured via chatbot."
```

### Record Event

```basic
' Record a business event
INSERT INTO "events" WITH
    event_type = "purchase",
    user_id = user.id,
    data = '{"product_id": "' + product_id + '", "amount": ' + amount + '}',
    occurred_at = NOW()
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = INSERT INTO "customers" WITH
    name = customer_name,
    email = customer_email

IF ERROR THEN
    PRINT "Insert failed: " + ERROR_MESSAGE
    
    IF INSTR(ERROR_MESSAGE, "duplicate") > 0 THEN
        TALK "This email is already registered."
    ELSE IF INSTR(ERROR_MESSAGE, "constraint") > 0 THEN
        TALK "Please provide all required information."
    ELSE
        TALK "Sorry, I couldn't save your information. Please try again."
    END IF
ELSE
    TALK "Information saved successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `DUPLICATE_KEY` | Unique constraint violated | Check for existing record first |
| `NOT_NULL_VIOLATION` | Required field missing | Include all required fields |
| `FOREIGN_KEY_VIOLATION` | Referenced record doesn't exist | Verify foreign key values |
| `CHECK_VIOLATION` | Value fails check constraint | Validate data before insert |
| `TABLE_NOT_FOUND` | Table doesn't exist | Verify table name |

---

## Validation Before Insert

```basic
' Validate data before inserting
IF LEN(email) < 5 OR INSTR(email, "@") = 0 THEN
    TALK "Please provide a valid email address."
ELSE IF LEN(name) < 2 THEN
    TALK "Please provide your full name."
ELSE
    result = INSERT INTO "contacts" WITH
        name = name,
        email = email,
        created_at = NOW()
    
    TALK "Contact saved!"
END IF
```

---

## INSERT vs MERGE

| Keyword | Purpose | Use When |
|---------|---------|----------|
| `INSERT` | Create new record | Adding new data |
| `MERGE` | Insert or update | Record may already exist |

```basic
' INSERT - Always creates new record (may fail if duplicate)
INSERT INTO "users" WITH email = "john@example.com", name = "John"

' MERGE - Creates or updates based on key
MERGE INTO "users" ON email = "john@example.com" WITH
    email = "john@example.com",
    name = "John Updated"
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
- Auto-generates `id` if not specified (serial/UUID)
- Timestamps can be set with `NOW()` function
- Returns the complete inserted record including defaults

---

## Related Keywords

- [UPDATE](keyword-update.md) — Modify existing records
- [DELETE](keyword-delete.md) — Remove records
- [MERGE](keyword-merge.md) — Insert or update (upsert)
- [FIND](keyword-find.md) — Query records
- [TABLE](keyword-table.md) — Create tables

---

## Summary

`INSERT` creates new records in database tables. Use it to store user data, log events, create orders, and save form submissions. Always validate data before inserting and handle potential errors like duplicates and constraint violations. For cases where a record may already exist, consider using `MERGE` instead.