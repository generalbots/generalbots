# Data Operations

This section covers keywords for working with structured data in databases, spreadsheets, and in-memory collections. These keywords enable bots to query, transform, and persist data across various storage backends.

---

## Overview

General Bots provides a complete set of data operation keywords:

| Keyword | Purpose |
|---------|---------|
| [SAVE](keyword-save.md) | Persist data to storage |
| [INSERT](keyword-insert.md) | Add new records to tables |
| [UPDATE](keyword-update.md) | Modify existing records |
| [DELETE](keyword-delete.md) | Remove records from tables |
| [MERGE](keyword-merge.md) | Upsert (insert or update) records |
| [FILL](keyword-fill.md) | Populate templates with data |
| [MAP](keyword-map.md) | Transform collections |
| [FILTER](keyword-filter.md) | Select matching items |
| [AGGREGATE](keyword-aggregate.md) | Sum, count, average operations |
| [JOIN](keyword-join.md) | Combine related datasets |
| [PIVOT](keyword-pivot.md) | Reshape data tables |
| [GROUP BY](keyword-group-by.md) | Group records by field |

---

## Quick Examples

### Database Operations

```basic
' Insert a new record
INSERT INTO "customers" WITH
    name = "John Doe",
    email = "john@example.com",
    created_at = NOW()

' Update existing records
UPDATE "customers" SET status = "active" WHERE email = "john@example.com"

' Delete records
DELETE FROM "customers" WHERE status = "inactive" AND last_login < "2024-01-01"

' Merge (upsert) - insert or update based on key
MERGE INTO "products" ON sku = "SKU-001" WITH
    sku = "SKU-001",
    name = "Widget",
    price = 29.99,
    stock = 100
```

### Collection Transformations

```basic
' Map - transform each item
prices = [10, 20, 30, 40]
with_tax = MAP prices WITH item * 1.1
' Result: [11, 22, 33, 44]

' Filter - select matching items
orders = FIND "orders"
large_orders = FILTER orders WHERE total > 100
' Returns only orders with total > 100

' Aggregate - calculate summaries
total_sales = AGGREGATE orders SUM total
order_count = AGGREGATE orders COUNT
avg_order = AGGREGATE orders AVERAGE total
```

### Data Analysis

```basic
' Group by category
sales_by_category = GROUP BY "sales" ON category
FOR EACH group IN sales_by_category
    TALK group.category + ": $" + group.total
NEXT

' Join related tables
order_details = JOIN "orders" WITH "customers" ON customer_id = id
FOR EACH detail IN order_details
    TALK detail.customer_name + " ordered " + detail.product
NEXT

' Pivot data for reports
monthly_pivot = PIVOT "sales" ROWS month COLUMNS product VALUES SUM(amount)
```

---

## Data Sources

### Supported Backends

| Backend | Use Case | Configuration |
|---------|----------|---------------|
| PostgreSQL | Primary database | `database-url` in config.csv |
| SQLite | Local/embedded | `database-provider,sqlite` |
| In-memory | Temporary data | Default for collections |
| CSV files | Import/export | Via READ/WRITE AS TABLE |
| Excel | Spreadsheet data | Via READ AS TABLE |

### Connection Configuration

```csv
name,value
database-provider,postgres
database-url,postgres://user:pass@localhost/botdb
database-pool-size,10
database-timeout,30
```

### Multiple Connections

```basic
' Use default connection
customers = FIND "customers"

' Use named connection
legacy_data = FIND "orders" ON "legacy_db"
warehouse_stock = FIND "inventory" ON "warehouse_db"
```

---

## Common Patterns

### CRUD Operations

```basic
' CREATE
customer_id = INSERT INTO "customers" WITH
    name = customer_name,
    email = customer_email,
    phone = customer_phone

TALK "Customer created with ID: " + customer_id

' READ
customer = FIND "customers" WHERE id = customer_id
TALK "Found: " + customer.name

' UPDATE
UPDATE "customers" SET 
    last_contact = NOW(),
    contact_count = contact_count + 1
WHERE id = customer_id

' DELETE
DELETE FROM "customers" WHERE id = customer_id AND confirmed = true
```

### Batch Operations

```basic
' Insert multiple records from data source
new_orders = READ "imports/orders.csv" AS TABLE

FOR EACH order IN new_orders
    INSERT INTO "orders" WITH
        product = order.product,
        quantity = order.quantity,
        price = order.price
NEXT

' Bulk update
UPDATE "products" SET on_sale = true WHERE category = "electronics"
```

### Data Transformation Pipeline

```basic
' Load raw data
raw_sales = READ "imports/sales-data.csv" AS TABLE

' Clean and transform
cleaned = FILTER raw_sales WHERE amount > 0 AND date IS NOT NULL

' Enrich with calculations
enriched = MAP cleaned WITH
    tax = item.amount * 0.1,
    total = item.amount * 1.1,
    quarter = QUARTER(item.date)

' Aggregate for reporting
quarterly_totals = GROUP BY enriched ON quarter
summary = AGGREGATE quarterly_totals SUM total

' Save results
WRITE summary TO "reports/quarterly-summary.csv" AS TABLE
INSERT INTO "sales_reports" VALUES summary
```

### Lookup and Reference

```basic
' Simple lookup
product = FIND "products" WHERE sku = user_sku
IF product THEN
    TALK "Price: $" + product.price
ELSE
    TALK "Product not found"
END IF

' Lookup with join
order_with_customer = FIND "orders" 
    JOIN "customers" ON orders.customer_id = customers.id
    WHERE orders.id = order_id

TALK "Order for " + order_with_customer.customer_name
```

---

## Query Syntax

### WHERE Clauses

```basic
' Equality
FIND "users" WHERE status = "active"

' Comparison
FIND "orders" WHERE total > 100
FIND "products" WHERE stock <= 10

' Multiple conditions
FIND "customers" WHERE country = "US" AND created_at > "2024-01-01"
FIND "items" WHERE category = "electronics" OR category = "accessories"

' NULL checks
FIND "leads" WHERE assigned_to IS NULL
FIND "orders" WHERE shipped_at IS NOT NULL

' Pattern matching
FIND "products" WHERE name LIKE "%widget%"

' IN lists
FIND "orders" WHERE status IN ["pending", "processing", "shipped"]
```

### ORDER BY

```basic
' Single column sort
FIND "products" ORDER BY price ASC

' Multiple column sort
FIND "orders" ORDER BY priority DESC, created_at ASC

' With limit
recent_orders = FIND "orders" ORDER BY created_at DESC LIMIT 10
```

### Aggregations

```basic
' Count records
total_customers = AGGREGATE "customers" COUNT

' Sum values
total_revenue = AGGREGATE "orders" SUM total

' Average
avg_order_value = AGGREGATE "orders" AVERAGE total

' Min/Max
cheapest = AGGREGATE "products" MIN price
most_expensive = AGGREGATE "products" MAX price

' With grouping
sales_by_region = AGGREGATE "sales" SUM amount GROUP BY region
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = INSERT INTO "orders" VALUES order_data

IF ERROR THEN
    PRINT "Database error: " + ERROR_MESSAGE
    
    IF INSTR(ERROR_MESSAGE, "duplicate") > 0 THEN
        TALK "This order already exists."
    ELSE IF INSTR(ERROR_MESSAGE, "constraint") > 0 THEN
        TALK "Invalid data. Please check all fields."
    ELSE
        TALK "Sorry, I couldn't save your order. Please try again."
    END IF
ELSE
    TALK "Order saved successfully!"
END IF
```

### Transaction Handling

```basic
' Start transaction
BEGIN TRANSACTION

' Multiple operations
INSERT INTO "orders" VALUES order_data
UPDATE "inventory" SET stock = stock - quantity WHERE product_id = product_id
INSERT INTO "order_items" VALUES items

' Commit if all succeeded
IF NOT ERROR THEN
    COMMIT
    TALK "Order completed!"
ELSE
    ROLLBACK
    TALK "Order failed. All changes reverted."
END IF
```

---

## Performance Tips

### Use Indexes

Ensure database tables have appropriate indexes for frequently queried columns:

```sql
-- In database setup
CREATE INDEX idx_orders_customer ON orders(customer_id);
CREATE INDEX idx_orders_date ON orders(created_at);
CREATE INDEX idx_products_sku ON products(sku);
```

### Limit Results

```basic
' Avoid loading entire tables
' Bad:
all_orders = FIND "orders"

' Good:
recent_orders = FIND "orders" WHERE created_at > date_limit LIMIT 100
```

### Batch Operations

```basic
' Process large datasets in batches
page = 0
batch_size = 100

WHILE true
    batch = FIND "records" LIMIT batch_size OFFSET page * batch_size
    
    IF LEN(batch) = 0 THEN
        EXIT WHILE
    END IF
    
    FOR EACH record IN batch
        ' Process record
    NEXT
    
    page = page + 1
WEND
```

---

## Configuration

Configure data operations in `config.csv`:

```csv
name,value
database-provider,postgres
database-url,postgres://localhost/botdb
database-pool-size,10
database-timeout,30
database-log-queries,false
database-max-rows,10000
```

---

## Security Considerations

1. **Parameterized queries** — All keywords use parameterized queries to prevent SQL injection
2. **Row limits** — Default limit on returned rows prevents memory exhaustion
3. **Access control** — Bots can only access their own data by default
4. **Audit logging** — All data modifications logged for compliance
5. **Encryption** — Sensitive data encrypted at rest

---

## See Also

- [SAVE](keyword-save.md) — Persist data
- [INSERT](keyword-insert.md) — Add records
- [UPDATE](keyword-update.md) — Modify records
- [DELETE](keyword-delete.md) — Remove records
- [MERGE](keyword-merge.md) — Upsert operations
- [FILL](keyword-fill.md) — Template population
- [MAP](keyword-map.md) — Transform collections
- [FILTER](keyword-filter.md) — Select items
- [AGGREGATE](keyword-aggregate.md) — Summaries
- [JOIN](keyword-join.md) — Combine datasets
- [PIVOT](keyword-pivot.md) — Reshape data
- [GROUP BY](keyword-group-by.md) — Group records
- [TABLE](keyword-table.md) — Create tables