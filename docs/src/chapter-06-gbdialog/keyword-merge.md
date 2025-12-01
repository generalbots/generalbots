# MERGE

Combines data from multiple sources or upserts records into a database table.

## Syntax

```basic
MERGE table, data, key_column
MERGE table, data, key_columns, update_columns
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `table` | String | Target database table name |
| `data` | Array/Object | Data to merge (single record or array of records) |
| `key_column` | String | Column(s) to match existing records |
| `update_columns` | Array | Optional specific columns to update on match |

## Description

`MERGE` performs an "upsert" operation: it inserts new records or updates existing ones based on matching key columns. This is useful for synchronizing data from external sources, importing bulk data, or maintaining data consistency.

## Examples

### Basic Merge (Single Record)

```basic
contact = #{
    email: "john@example.com",
    name: "John Smith",
    phone: "+1-555-0123"
}

MERGE "contacts", contact, "email"
TALK "Contact merged successfully"
```

### Bulk Merge

```basic
new_products = GET "https://api.supplier.com/products"

MERGE "products", new_products, "sku"
TALK "Merged " + LEN(new_products) + " products"
```

### Merge with Specific Update Columns

```basic
price_updates = [
    #{sku: "ABC123", price: 29.99},
    #{sku: "DEF456", price: 49.99},
    #{sku: "GHI789", price: 19.99}
]

MERGE "products", price_updates, "sku", ["price"]
TALK "Prices updated"
```

### Composite Key Match

```basic
attendance = #{
    employee_id: "EMP001",
    date: TODAY(),
    status: "present",
    check_in: NOW()
}

MERGE "attendance", attendance, "employee_id,date"
```

### Sync from External API

```basic
SET SCHEDULE "every 6 hours"

' Fetch latest data from CRM
customers = GET "https://crm.example.com/api/customers"

' Merge into local database
MERGE "customers", customers, "crm_id"

TALK "Synced " + LEN(customers) + " customer records"
```

## Return Value

Returns an object with merge statistics:

| Property | Description |
|----------|-------------|
| `inserted` | Number of new records created |
| `updated` | Number of existing records updated |
| `unchanged` | Number of records that matched but had no changes |
| `total` | Total records processed |

```basic
result = MERGE "products", data, "sku"
TALK "Inserted: " + result.inserted + ", Updated: " + result.updated
```

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Sync customer data from the CRM</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔄 Syncing customer data...</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Customer sync complete!</p>
      <p></p>
      <p>📊 Results:</p>
      <p>• New customers: 12</p>
      <p>• Updated records: 45</p>
      <p>• Unchanged: 238</p>
      <p>• Total processed: 295</p>
      <div class="wa-time">14:01</div>
    </div>
  </div>
</div>

## Behavior

### On Match (Key Exists)
- Updates all columns in the data (or only `update_columns` if specified)
- Preserves columns not in the data
- Updates `updated_at` timestamp if column exists

### On No Match (New Record)
- Inserts new row with all provided columns
- Sets `created_at` timestamp if column exists

## Common Patterns

### Daily Data Import

```basic
SET SCHEDULE "every day at 2am"

data = GET "https://data.provider.com/daily-export"
result = MERGE "imported_data", data, "external_id"

IF result.inserted > 0 THEN
    SEND MAIL "admin@company.com", "Data Import", 
        "Imported " + result.inserted + " new records"
END IF
```

### Inventory Sync

```basic
inventory = GET "https://warehouse.api/stock-levels"
MERGE "products", inventory, "sku", ["quantity", "last_restock"]
```

### User Profile Updates

```basic
profile = #{
    user_id: current_user_id,
    preferences: user_preferences,
    last_active: NOW()
}
MERGE "user_profiles", profile, "user_id"
```

## See Also

- [INSERT](./keyword-insert.md) - Insert new records only
- [UPDATE](./keyword-update.md) - Update existing records only
- [SAVE](./keyword-save.md) - Simple data persistence
- [FIND](./keyword-find.md) - Query data before merging

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:500px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-message{margin-bottom:10px}
.wa-message.user{text-align:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;display:inline-block;text-align:left}
.wa-message.bot .wa-bubble{background-color:#fff;display:inline-block}
.wa-bubble{padding:8px 12px;border-radius:8px;box-shadow:0 1px .5px rgba(0,0,0,.13);max-width:85%}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
</style>