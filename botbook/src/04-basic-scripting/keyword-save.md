# SAVE

Saves data to a database table using upsert (insert or update) semantics.

## Syntax

### Form 1: Save with object (classic)

```basic
SAVE "table", id, data
```

### Form 2: Save with variables (direct)

```basic
SAVE "table", id, field1, field2, field3, ...
```

The variable names are used as column names automatically.

## Parameters

### Form 1 (with object)

| Parameter | Type | Description |
|-----------|------|-------------|
| table | String | The name of the database table |
| id | String/Number | The unique identifier for the record |
| data | Object | A map/object containing field names and values |

### Form 2 (with variables)

| Parameter | Type | Description |
|-----------|------|-------------|
| table | String | The name of the database table |
| id | String/Number | The unique identifier for the record |
| field1, field2, ... | Any | Variable references (names become column names) |

## Description

`SAVE` performs an upsert operation:
- If a record with the given `id` exists, it updates the record
- If no record exists, it inserts a new one

The `id` parameter maps to the `id` column in the table.

### Form 1 vs Form 2

**Form 1** (with object) is useful when you need custom column names or complex data structures:

```basic
data = #{
    "customer_name": "João Silva",
    "email": "joao@example.com"
}
SAVE "customers", "CUST-001", data
```

**Form 2** (with variables) is simpler - variable names become column names:

```basic
customerName = "João Silva"
email = "joao@example.com"
phone = "+5511999887766"
SAVE "customers", "CUST-001", customerName, email, phone
' Creates columns: customerName, email, phone
```

This eliminates the need for WITH blocks when variable names match your desired column names.

### Perfect for TOOL Functions

**This is especially useful for TOOL functions** where variables are automatically filled by user input and can be saved directly without needing WITH blocks:

```basic
' TOOL function parameters - automatically filled by LLM
PARAM nome AS STRING LIKE "João Silva" DESCRIPTION "Nome completo"
PARAM email AS EMAIL LIKE "joao@example.com" DESCRIPTION "Email"
PARAM telefone AS STRING LIKE "(21) 98888-8888" DESCRIPTION "Telefone"

' Generate unique ID
customerId = "CUST-" + FORMAT(NOW(), "yyyyMMddHHmmss")

' Save directly - variable names become column names automatically!
' No need for WITH block - just pass the variables directly
SAVE "customers", customerId, nome, email, telefone

RETURN customerId
```

In TOOL functions, the parameters (variables like `nome`, `email`, `telefone`) are automatically extracted from user input by the LLM. The direct SAVE syntax allows you to persist these variables immediately without manual object construction.

## Examples

### Basic Save with Object (Form 1)

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

### Save with Variables - No WITH Block Needed (Form 2)

```basic
' Variable names become column names automatically
casamentoId = "CAS-20250117-1234"
protocolo = "CAS123456"
nomeNoivo = "Carlos Eduardo"
nomeNoiva = "Juliana Cristina"
telefoneNoivo = "(21) 98888-8888"
telefoneNoiva = "(21) 97777-7777"
emailNoivo = "carlos@example.com"
emailNoiva = "juliana@example.com"
tipoCasamento = "RELIGIOSO_COM_EFEITO_CIVIL"
dataPreferencial = "2026-12-15"
horarioPreferencial = "16:00"

' Save directly without WITH block
SAVE "casamentos", casamentoId, protocolo, nomeNoivo, nomeNoiva, telefoneNoivo, telefoneNoiva, emailNoivo, emailNoiva, tipoCasamento, dataPreferencial, horarioPreferencial
```

### Save Order Data (Direct Syntax - No Object)

```basic
order_id = "ORD-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
customer_id = "CUST-001"
customer_name = "João Silva"
total = 150.50
status = "pending"

' Save directly - variable names become columns
SAVE "orders", order_id, customer_id, customer_name, total, status

TALK "Order " + order_id + " saved successfully!"
```

### Save Event Registration

```basic
' Event registration form data
eventId = "EVT-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
nome = "Maria Santos"
email = "maria@example.com"
telefone = "(11) 91234-5678"
dataEvento = "2025-03-15"
quantidadePessoas = 3
observacoes = "Precisa de cadeira de rodas"

' Direct save - no WITH block needed
SAVE "eventos", eventId, nome, email, telefone, dataEvento, quantidadePessoas, observacoes

TALK "Inscrição confirmada! ID: " + eventId
```

### Update Existing Record

```basic
' If order exists, this updates it; otherwise creates it
order_id = "ORD-20250117-0001"
status = "shipped"
shipped_at = NOW()
tracking_number = "TRACK123456"

' Use object for updates to specific columns
update_data = #{
    "status": status,
    "shipped_at": shipped_at,
    "tracking_number": tracking_number
}

SAVE "orders", order_id, update_data
```

### With WhatsApp Notification

```basic
WEBHOOK "new-customer"

customer_id = "CUST-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
phone = body.phone
name = body.name
source = "webhook"

' Direct save with variables
SAVE "customers", customer_id, phone, name, source

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
customer_id = body.customer_id
total = body.total
status = "pending"

SAVE "orders", order_id, customer_id, total, status

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

### Comparison: WITH Block vs Direct Syntax

**Old way (WITH block):**
```basic
WITH casamento
    id = casamentoId
    protocolo = protocolo
    noivo = nomeNoivo
    noiva = nomeNoiva
END WITH
SAVE "casamentos", casamento
```

**New way (direct):**
```basic
' Variable names become column names automatically
SAVE "casamentos", casamentoId, protocolo, nomeNoivo, nomeNoiva
```

The direct syntax is cleaner and avoids the intermediate object creation. Use it when your variable names match your desired column names.

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