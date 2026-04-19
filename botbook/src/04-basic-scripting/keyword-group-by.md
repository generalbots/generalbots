# GROUP BY

Groups data by specified columns and optionally applies aggregate functions.

## Syntax

```basic
result = GROUP BY data, column
result = GROUP BY data, column, aggregates
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `data` | Array | The data array to group |
| `column` | String | Column name to group by |
| `aggregates` | Object | Optional aggregate functions to apply |

## Description

`GROUP BY` organizes rows of data into groups based on matching values in a specified column. When combined with aggregate functions, it calculates summary values for each group.

## Examples

### Basic Grouping

```basic
orders = FIND "orders", "status=completed"
grouped = GROUP BY orders, "category"

FOR EACH group IN grouped
    TALK "Category: " + group.key + " - Count: " + group.count
NEXT
```

### With Aggregates

```basic
sales = FIND "sales", "year=2025"
summary = GROUP BY sales, "region", #{
    total: "SUM(amount)",
    average: "AVG(amount)",
    count: "COUNT(*)"
}

FOR EACH region IN summary
    TALK region.key + ": $" + region.total + " (" + region.count + " sales)"
NEXT
```

### Multiple Level Grouping

```basic
' First group by category, then by month
products = FIND "orders", "year=2025"
by_category = GROUP BY products, "category"

FOR EACH cat IN by_category
    TALK "Category: " + cat.key
    by_month = GROUP BY cat.items, "month"
    FOR EACH month IN by_month
        TALK "  " + month.key + ": " + month.count + " orders"
    NEXT
NEXT
```

## Return Value

Returns an array of group objects, each containing:

| Property | Description |
|----------|-------------|
| `key` | The grouping value |
| `items` | Array of items in this group |
| `count` | Number of items in group |
| Additional | Any requested aggregates |

## Supported Aggregates

| Function | Description |
|----------|-------------|
| `SUM(column)` | Sum of values |
| `AVG(column)` | Average of values |
| `MIN(column)` | Minimum value |
| `MAX(column)` | Maximum value |
| `COUNT(*)` | Number of rows |

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me sales by region</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 Sales by Region (2025)</p>
      <p></p>
      <p><strong>North America:</strong> $245,000 (127 sales)</p>
      <p><strong>Europe:</strong> $189,500 (98 sales)</p>
      <p><strong>Asia Pacific:</strong> $156,200 (84 sales)</p>
      <p><strong>Latin America:</strong> $67,300 (42 sales)</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

## See Also

- [AGGREGATE](./keyword-aggregate.md) - Single aggregate calculations
- [PIVOT](./keyword-pivot.md) - Cross-tabulation of data
- [FILTER](./keyword-filter.md) - Filter data before grouping
- [FIND](./keyword-find.md) - Retrieve data to group

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