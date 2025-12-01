# FILTER

Selects elements from an array that match a specified condition.

## Syntax

```basic
result = FILTER(array, condition)
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `array` | Array | The source array to filter |
| `condition` | String | Expression that evaluates to true/false for each element |

## Description

`FILTER` creates a new array containing only the elements from the input array that satisfy the given condition. The condition is evaluated for each element, and only elements where the condition is true are included in the result.

## Examples

### Filter by Field Value

```basic
orders = FIND "orders", "year=2025"
large_orders = FILTER(orders, "item.total > 1000")

TALK "Found " + LEN(large_orders) + " orders over $1000"
```

### Filter by String Match

```basic
contacts = FIND "contacts", "active=true"
gmail_users = FILTER(contacts, "INSTR(item.email, 'gmail.com') > 0")

FOR EACH contact IN gmail_users
    TALK contact.name + " - " + contact.email
NEXT
```

### Filter by Status

```basic
tasks = FIND "tasks", "assigned_to=me"
pending = FILTER(tasks, "item.status = 'pending'")
completed = FILTER(tasks, "item.status = 'completed'")

TALK "Pending: " + LEN(pending) + ", Completed: " + LEN(completed)
```

### Filter Numbers

```basic
scores = [85, 92, 67, 78, 95, 88, 72]
passing = FILTER(scores, "item >= 70")
honors = FILTER(scores, "item >= 90")

TALK "Passing: " + LEN(passing) + ", Honors: " + LEN(honors)
```

### Complex Conditions

```basic
products = FIND "products", "category=electronics"
featured = FILTER(products, "item.in_stock = true AND item.rating >= 4.0")

TALK "Featured products:"
FOR EACH product IN featured
    TALK "- " + product.name + " (★" + product.rating + ")"
NEXT
```

## Return Value

Returns a new array containing only elements where the condition evaluated to true.

- Original array is not modified
- Returns empty array if no elements match
- Preserves order of matching elements

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me high-priority tasks</p>
      <div class="wa-time">09:45</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔴 High Priority Tasks:</p>
      <p></p>
      <p>1. Review quarterly report (Due: Today)</p>
      <p>2. Client presentation prep (Due: Tomorrow)</p>
      <p>3. Budget approval meeting (Due: Friday)</p>
      <p></p>
      <p>You have 3 high-priority tasks.</p>
      <div class="wa-time">09:45</div>
    </div>
  </div>
</div>

## Condition Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Equals | `"item.status = 'active'"` |
| `!=` | Not equals | `"item.type != 'archived'"` |
| `>` | Greater than | `"item.amount > 100"` |
| `>=` | Greater or equal | `"item.score >= 70"` |
| `<` | Less than | `"item.quantity < 10"` |
| `<=` | Less or equal | `"item.age <= 30"` |
| `AND` | Logical and | `"item.active = true AND item.verified = true"` |
| `OR` | Logical or | `"item.priority = 'high' OR item.urgent = true"` |

## Common Patterns

### Filter then Count

```basic
users = FIND "users", "registered=true"
premium = FILTER(users, "item.plan = 'premium'")
TALK "Premium users: " + LEN(premium)
```

### Filter then Map

```basic
orders = FIND "orders", "status=shipped"
recent = FILTER(orders, "item.ship_date > DATEADD('day', -7, NOW())")
tracking = MAP(recent, "tracking_number")
```

### Chain Multiple Filters

```basic
products = FIND "products", "active=true"
in_stock = FILTER(products, "item.quantity > 0")
on_sale = FILTER(in_stock, "item.discount > 0")
featured = FILTER(on_sale, "item.rating >= 4.5")
```

## See Also

- [FIND](./keyword-find.md) - Retrieve data from database
- [MAP](./keyword-map.md) - Transform filtered results
- [FOR EACH](./keyword-for-each.md) - Iterate over filtered array
- [AGGREGATE](./keyword-aggregate.md) - Calculate summary from filtered data

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