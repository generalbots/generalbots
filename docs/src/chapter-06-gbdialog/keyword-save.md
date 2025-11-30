# SAVE

Saves data to a database table using upsert (insert or update) semantics.

## Syntax

```basic
SAVE "table", id, data
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| table | String | The name of the database table |
| id | String/Number | The unique identifier for the record |
| data | Object | A map/object containing field names and values |

## Description

`SAVE` performs an upsert operation:
- If a record with the given `id` exists, it updates the record
- If no record exists, it inserts a new one

The `id` parameter maps to the `id` column in the table.

## Examples

### Basic Save with Object

```basic
' Create data object using Rhai map syntax
data = #{
    "customer_name": "João Silva",
    "email": "joao@example.com",
    "phone": "+5511999887766",
    "status": "active"
}

SAVE "customers", "CUST-001", data
```

### Save Order Data

```basic
order_id = "ORD-" + FORMAT(NOW(), "YYYYMMDDHHmmss")

order_data = #{
    "customer_id": customer_id,
    "customer_name": customer_name,
    "total": total,
    "status": "pending",
    "created_at": NOW()
}

SAVE "orders", order_id, order_data

TALK "Order " + order_id + " saved successfully!"
```

### Update Existing Record

```basic
' If order exists, this updates it; otherwise creates it
update_data = #{
    "status": "shipped",
    "shipped_at": NOW(),
    "tracking_number": tracking
}

SAVE "orders", order_id, update_data
```

### With WhatsApp Notification

```basic
WEBHOOK "new-customer"

customer_id = "CUST-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
phone = body.phone
name = body.name

customer_data = #{
    "name": name,
    "phone": phone,
    "source": "webhook",
    "created_at": NOW()
}

SAVE "customers", customer_id, customer_data

' Notify via WhatsApp
TALK TO "whatsapp:" + phone, "Welcome " + name + "! Your account has been created."

result_status = "ok"
result_customer_id = customer_id
```

### Building Data Dynamically

```basic
' Start with empty map and add fields
data = #{}
data.name = customer_name
data.email = customer_email
data.phone = customer_phone
data.registered_at = NOW()

IF has_referral THEN
    data.referral_code = referral_code
    data.discount = 10
END IF

SAVE "customers", customer_id, data
```

### Saving Multiple Related Records

```basic
WEBHOOK "create-order"

' Save order
order_id = body.order_id
order_data = #{
    "customer_id": body.customer_id,
    "total": body.total,
    "status": "pending"
}
SAVE "orders", order_id, order_data

' Save each line item
FOR EACH item IN body.items
    line_id = order_id + "-" + item.sku
    line_data = #{
        "order_id": order_id,
        "sku": item.sku,
        "quantity": item.quantity,
        "price": item.price
    }
    SAVE "order_items", line_id, line_data
NEXT item

' Notify customer
TALK TO "whatsapp:" + body.customer_phone, "Order #" + order_id + " confirmed!"

result_status = "ok"
```

## Return Value

Returns an object with:
- `command`: "save"
- `table`: The table name
- `id`: The record ID
- `rows_affected`: Number of rows affected (1 for insert/update)

## Notes

- Table must exist in the database
- The `id` column is used as the primary key for conflict detection
- All string values are automatically sanitized to prevent SQL injection
- Column names are validated to prevent injection

## Comparison with INSERT and UPDATE

| Keyword | Behavior |
|---------|----------|
| `SAVE` | Upsert - inserts if new, updates if exists |
| `INSERT` | Always creates new record (may fail if ID exists) |
| `UPDATE` | Only updates existing records (no-op if not found) |

```basic
' SAVE is preferred for most cases
SAVE "customers", id, data      ' Insert or update

' Use INSERT when you need a new record guaranteed
INSERT "logs", log_entry        ' Always creates new

' Use UPDATE for targeted updates
UPDATE "orders", "status=pending", update_data   ' Update matching rows
```

## See Also

- [INSERT](./keyword-insert.md) - Insert new records
- [UPDATE](./keyword-update.md) - Update existing records
- [DELETE](./keyword-delete.md) - Delete records
- [FIND](./keyword-find.md) - Query records