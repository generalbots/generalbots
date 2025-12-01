# MAP

Transforms each element of an array by applying a function or expression.

## Syntax

```basic
result = MAP(array, expression)
result = MAP(array, field)
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `array` | Array | The source array to transform |
| `expression` | String | Expression to apply to each element, or field name to extract |

## Description

`MAP` creates a new array by applying a transformation to each element of the input array. This is useful for extracting specific fields from objects, formatting data, or performing calculations on each item.

## Examples

### Extract Field from Objects

```basic
users = FIND "users", "status=active"
names = MAP(users, "name")

TALK "Active users: " + JOIN(names, ", ")
```

### Transform Values

```basic
prices = [100, 200, 300, 400]
with_tax = MAP(prices, "item * 1.1")

FOR EACH price IN with_tax
    TALK "Price with tax: $" + price
NEXT
```

### Format Data

```basic
orders = FIND "orders", "date=today"
summaries = MAP(orders, "'Order #' + item.id + ': $' + item.total")

FOR EACH summary IN summaries
    TALK summary
NEXT
```

### Extract Nested Properties

```basic
contacts = FIND "contacts", "company=Acme"
emails = MAP(contacts, "email")

email_list = JOIN(emails, "; ")
TALK "Emails: " + email_list
```

### Uppercase Names

```basic
products = ["widget", "gadget", "gizmo"]
upper_products = MAP(products, "UPPER(item)")

TALK JOIN(upper_products, ", ")
' Output: "WIDGET, GADGET, GIZMO"
```

## Return Value

Returns a new array with the same length as the input, containing transformed values.

- Original array is not modified
- Null values in the source are preserved as null
- If transformation fails for an element, that element becomes null

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>List all customer emails</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📧 Customer Emails:</p>
      <p></p>
      <p>• john@example.com</p>
      <p>• sarah@company.co</p>
      <p>• mike@business.org</p>
      <p>• lisa@startup.io</p>
      <p></p>
      <p>Found 4 customer emails.</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
</div>

## Common Patterns

### Extract IDs for API Calls

```basic
records = FIND "items", "sync=pending"
ids = MAP(records, "id")
' Use ids for batch API operations
```

### Create Display Labels

```basic
products = FIND "products", "in_stock=true"
labels = MAP(products, "item.name + ' ($' + item.price + ')'")
```

### Calculate Derived Values

```basic
line_items = FIND "cart_items", "cart_id=123"
totals = MAP(line_items, "item.quantity * item.unit_price")
```

## See Also

- [FILTER](./keyword-filter.md) - Filter array elements
- [FOR EACH](./keyword-for-each.md) - Iterate with more control
- [JOIN](./keyword-join.md) - Combine mapped results into string
- [AGGREGATE](./keyword-aggregate.md) - Calculate summary from mapped values

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