# FIND

Search and retrieve data from database tables using filter criteria.

## Syntax

```basic
result = FIND "table_name", "filter_criteria"
```

## Parameters

- `table_name` - The name of the database table to search
- `filter_criteria` - Filter expression in the format "field=value"

## Description

FIND searches database tables for records matching specified criteria. It returns an array of matching records that can be iterated over using FOR EACH loops.

## Examples

### Basic Search
```basic
' Find records with specific action
items = FIND "gb.rob", "ACTION=EMUL"

FOR EACH item IN items
    TALK "Found: " + item.company
NEXT
```

### Single Field Filter
```basic
' Find pending orders
orders = FIND "orders", "status=pending"

FOR EACH order IN orders
    TALK "Order #" + order.id + " is pending"
NEXT
```

### Working with Results
```basic
' Find and process customer records
customers = FIND "customers", "city=Seattle"

FOR EACH customer IN customers
    TALK customer.name + " from " + customer.address
    
    ' Access fields with dot notation
    email = customer.email
    phone = customer.phone
    
    ' Update related data
    SET "contacts", "id=" + customer.id, "last_contacted=" + NOW()
NEXT
```

## Return Value

FIND returns an array of records from the specified table. Each record is an object with fields accessible via dot notation.

- Returns empty array if no matches found
- Returns array of matching records if successful
- Each record contains all columns from the table

## Field Access

Access fields in returned records using dot notation:

```basic
items = FIND "products", "category=electronics"

FOR EACH item IN items
    ' Access fields directly
    TALK item.name
    TALK item.price
    TALK item.description
    
    ' Use null coalescing for optional fields
    website = item.website ?? ""
    
    ' Check field existence
    IF item.discount != "" THEN
        TALK "On sale: " + item.discount + "% off"
    END IF
NEXT
```

## Common Patterns

### Process All Matching Records
```basic
tasks = FIND "tasks", "status=open"

FOR EACH task IN tasks
    ' Process each task
    TALK "Processing task: " + task.title
    
    ' Update task status
    SET "tasks", "id=" + task.id, "status=in_progress"
NEXT
```

### Check If Records Exist
```basic
users = FIND "users", "email=john@example.com"

IF LENGTH(users) > 0 THEN
    TALK "User exists"
ELSE
    TALK "User not found"
END IF
```

### Data Enrichment
```basic
companies = FIND "companies", "needs_update=true"

FOR EACH company IN companies
    ' Get additional data
    website = company.website ?? ""
    
    IF website == "" THEN
        ' Look up website
        website = WEBSITE OF company.name
        
        ' Update record
        SET "companies", "id=" + company.id, "website=" + website
    END IF
    
    ' Fetch and process website data
    page = GET website
    ' Process page content...
NEXT
```

### Batch Processing with Delays
```basic
emails = FIND "email_queue", "sent=false"

FOR EACH email IN emails
    ' Send email
    SEND MAIL email.to, email.subject, email.body
    
    ' Mark as sent
    SET "email_queue", "id=" + email.id, "sent=true"
    
    ' Rate limiting
    WAIT 1000
NEXT
```

## Filter Expressions

The filter parameter uses simple equality expressions:

- `"field=value"` - Match exact value
- Multiple conditions must be handled in BASIC code after retrieval

```basic
' Get all records then filter in BASIC
all_orders = FIND "orders", "status=active"

FOR EACH order IN all_orders
    ' Additional filtering in code
    IF order.amount > 1000 AND order.priority == "high" THEN
        ' Process high-value orders
        TALK "Priority order: " + order.id
    END IF
NEXT
```

## Working with Different Data Types

```basic
products = FIND "products", "active=true"

FOR EACH product IN products
    ' String fields
    name = product.name
    
    ' Numeric fields
    price = product.price
    quantity = product.quantity
    
    ' Date fields
    created = product.created_at
    
    ' Boolean-like fields (stored as strings)
    IF product.featured == "true" THEN
        TALK "Featured: " + name
    END IF
NEXT
```

## Error Handling

```basic
' Handle potential errors
items = FIND "inventory", "warehouse=main"

IF items == null THEN
    TALK "Error accessing inventory data"
ELSE IF LENGTH(items) == 0 THEN
    TALK "No items found in main warehouse"
ELSE
    TALK "Found " + LENGTH(items) + " items"
    ' Process items...
END IF
```

## Performance Considerations

1. **Limit Results**: The system automatically limits to 10 results for safety
2. **Use Specific Filters**: More specific filters reduce processing time
3. **Avoid Full Table Scans**: Always provide a filter criterion
4. **Process in Batches**: For large datasets, process in chunks

```basic
' Process records in batches
batch = FIND "large_table", "processed=false"

count = 0
FOR EACH record IN batch
    ' Process record
    SET "large_table", "id=" + record.id, "processed=true"
    
    count = count + 1
    IF count >= 10 THEN
        EXIT FOR  ' Process max 10 at a time
    END IF
NEXT
```

## Integration with Other Keywords

### With SET for Updates
```basic
users = FIND "users", "newsletter=true"

FOR EACH user IN users
    ' Update last_notified field
    SET "users", "id=" + user.id, "last_notified=" + NOW()
NEXT
```

### With LLM for Processing
```basic
articles = FIND "articles", "needs_summary=true"

FOR EACH article IN articles
    summary = LLM "Summarize: " + article.content
    SET "articles", "id=" + article.id, "summary=" + summary
NEXT
```

### With CREATE SITE
```basic
companies = FIND "companies", "needs_site=true"

FOR EACH company IN companies
    alias = LLM "Create URL alias for: " + company.name
    CREATE SITE alias, "template", "Create site for " + company.name
    SET "companies", "id=" + company.id, "site_url=" + alias
NEXT
```

## Limitations

- Maximum 10 records returned per query (system limit)
- Filter supports simple equality only
- Complex queries require post-processing in BASIC
- Table must exist in the database
- User must have read permissions on the table

## Best Practices

✅ **Always check results** - Verify FIND returned data before processing  
✅ **Use specific filters** - Reduce result set size with precise criteria  
✅ **Handle empty results** - Check LENGTH before iterating  
✅ **Update as you go** - Mark records as processed to avoid reprocessing  

❌ **Don't assume order** - Results may not be sorted  
❌ **Don't ignore limits** - Remember the 10-record limit  
❌ **Don't use without filter** - Always provide filter criteria  

## See Also

- [SET](./keyword-set.md) - Update database records
- [GET](./keyword-get.md) - Retrieve single values
- [FOR EACH](./keyword-for-each.md) - Iterate over results
- [LLM](./keyword-llm.md) - Process found data with AI