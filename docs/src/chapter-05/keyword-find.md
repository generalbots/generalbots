# FIND

**Search for specific data in storage or knowledge bases.** The FIND keyword performs targeted searches in bot memory, databases, or document collections to locate specific information.

## Syntax

```basic
result = FIND(pattern)
result = FIND(pattern, location)
result = FIND(pattern, location, options)
```

## Parameters

- `pattern` - Search pattern or query string
- `location` - Where to search (optional, defaults to current KB)
- `options` - Search options like case sensitivity, limit (optional)

## Description

FIND searches for data matching a pattern. Unlike semantic search with LLM, FIND does exact or pattern-based matching. Useful for structured data, IDs, specific values.

## Search Locations

### Bot Memory
```basic
' Find in bot's permanent storage
user_data = FIND("user_*", "BOT_MEMORY")
' Returns all keys starting with "user_"
```

### Session Variables
```basic
' Find in current session
form_fields = FIND("form_*", "SESSION")
' Returns all form-related variables
```

### Knowledge Base
```basic
' Find specific documents
policies = FIND("*.pdf", "policies")
' Returns all PDFs in policies collection
```

### Database
```basic
' Find in database tables
orders = FIND("status:pending", "orders")
' Returns pending orders
```

## Examples

### Basic Search
```basic
' Find a specific user
user = FIND("email:john@example.com")

if user
  TALK "Found user: " + user.name
else
  TALK "User not found"
end
```

### Pattern Matching
```basic
' Find all items matching pattern
items = FIND("SKU-2024-*")

FOR EACH item IN items
  TALK item.name + ": " + item.price
END
```

### Multi-Criteria Search
```basic
' Complex search with multiple conditions
results = FIND("type:invoice AND status:unpaid AND date>2024-01-01")

total = 0
FOR EACH invoice IN results
  total = total + invoice.amount
END
TALK "Total unpaid: $" + total
```

### Search with Options
```basic
' Limited, case-insensitive search
matches = FIND("john", "customers", {
  case_sensitive: false,
  limit: 10,
  fields: ["name", "email"]
})
```

## Return Values

FIND returns different types based on matches:

- **Single match** - Returns the item directly
- **Multiple matches** - Returns array of items
- **No matches** - Returns null or empty array
- **Error** - Returns null with error in ERROR variable

## Common Patterns

### Check Existence
```basic
exists = FIND("id:" + user_id)
if exists
  TALK "User already registered"
else
  ' Create new user
end
```

### Filter Results
```basic
all_products = FIND("*", "products")
in_stock = []

FOR EACH product IN all_products
  if product.quantity > 0
    in_stock.append(product)
  end
END
```

### Aggregate Data
```basic
sales = FIND("date:" + TODAY(), "transactions")
daily_total = 0

FOR EACH sale IN sales
  daily_total = daily_total + sale.amount
END

TALK "Today's sales: $" + daily_total
```

### Search History
```basic
' Find previous conversations
history = FIND("session:" + user_id, "messages", {
  sort: "timestamp DESC",
  limit: 50
})

TALK "Your last conversation:"
FOR EACH message IN history
  TALK message.timestamp + ": " + message.content
END
```

## Performance Tips

### Use Specific Patterns
```basic
' Good - Specific pattern
orders = FIND("order_2024_01_*")

' Bad - Too broad
everything = FIND("*")
```

### Limit Results
```basic
' Get only what you need
recent = FIND("*", "logs", {limit: 100})
```

### Cache Repeated Searches
```basic
' Cache for session
if not cached_products
  cached_products = FIND("*", "products")
end
' Use cached_products instead of searching again
```

## Error Handling

```basic
try
  results = FIND(user_query)
  if results
    TALK "Found " + LENGTH(results) + " matches"
  else
    TALK "No results found"
  end
catch error
  TALK "Search failed. Please try again."
  LOG "FIND error: " + error
end
```

## Comparison with Other Keywords

| Keyword | Purpose | Use When |
|---------|---------|----------|
| FIND | Exact/pattern search | Looking for specific values |
| LLM | Semantic search | Understanding meaning |
| GET | Direct retrieval | Know exact key |
| USE KB | Activate knowledge | Need document context |

## Advanced Usage

### Dynamic Location
```basic
department = GET user.department
data = FIND("*", department + "_records")
```

### Compound Searches
```basic
' Find in multiple places
local = FIND(query, "local_db")
remote = FIND(query, "remote_api")
results = MERGE(local, remote)
```

### Conditional Fields
```basic
search_fields = ["name"]
if advanced_mode
  search_fields.append(["email", "phone", "address"])
end

results = FIND(term, "contacts", {fields: search_fields})
```

## Best Practices

✅ **Be specific** - Use precise patterns to avoid large result sets  
✅ **Handle empty results** - Always check if FIND returned data  
✅ **Use appropriate location** - Search where data actually lives  
✅ **Limit when possible** - Don't retrieve more than needed  

❌ **Don't search everything** - Avoid FIND("*") without limits  
❌ **Don't assume order** - Results may not be sorted unless specified  
❌ **Don't ignore errors** - Wrap in try/catch for production  

## See Also

- [GET](./keyword-get.md) - Direct key retrieval
- [SET](./keyword-set.md) - Store data
- [USE KB](./keyword-use-kb.md) - Semantic document search
- [LLM](./keyword-llm.md) - AI-powered search