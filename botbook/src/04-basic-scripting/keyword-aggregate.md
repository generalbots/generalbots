# AGGREGATE

The `AGGREGATE` keyword performs calculations on collections of data, computing sums, counts, averages, and other statistical operations.

---

## Syntax

```basic
result = AGGREGATE collection SUM field
result = AGGREGATE collection COUNT
result = AGGREGATE collection AVERAGE field
result = AGGREGATE collection MIN field
result = AGGREGATE collection MAX field
result = AGGREGATE "table_name" SUM field WHERE condition
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `collection` | Array/String | Data array or table name |
| `SUM` | Operation | Calculate total of numeric field |
| `COUNT` | Operation | Count number of items |
| `AVERAGE` | Operation | Calculate arithmetic mean |
| `MIN` | Operation | Find minimum value |
| `MAX` | Operation | Find maximum value |
| `field` | String | Field name to aggregate |
| `WHERE` | Clause | Optional filter condition |

---

## Description

`AGGREGATE` performs mathematical and statistical calculations on data collections. It can work with in-memory arrays or query database tables directly. Use it to compute totals, counts, averages, and find extreme values.

Use cases include:
- Calculating order totals
- Counting records
- Computing averages for reports
- Finding highest/lowest values
- Summarizing data for dashboards

---

## Examples

### Sum Values

```basic
' Calculate total sales
orders = FIND "orders" WHERE status = "completed"
total_sales = AGGREGATE orders SUM amount

TALK "Total sales: $" + FORMAT(total_sales, "#,##0.00")
```

### Count Records

```basic
' Count active users
active_count = AGGREGATE "users" COUNT WHERE status = "active"

TALK "We have " + active_count + " active users"
```

### Calculate Average

```basic
' Calculate average order value
avg_order = AGGREGATE "orders" AVERAGE amount WHERE created_at > "2025-01-01"

TALK "Average order value: $" + FORMAT(avg_order, "#,##0.00")
```

### Find Minimum and Maximum

```basic
' Find price range
products = FIND "products" WHERE category = "electronics"

min_price = AGGREGATE products MIN price
max_price = AGGREGATE products MAX price

TALK "Prices range from $" + min_price + " to $" + max_price
```

### Multiple Aggregations

```basic
' Calculate multiple statistics
orders = FIND "orders" WHERE customer_id = user.id

total_spent = AGGREGATE orders SUM amount
order_count = AGGREGATE orders COUNT
avg_order = AGGREGATE orders AVERAGE amount
largest_order = AGGREGATE orders MAX amount

TALK "Your order summary:"
TALK "- Total orders: " + order_count
TALK "- Total spent: $" + FORMAT(total_spent, "#,##0.00")
TALK "- Average order: $" + FORMAT(avg_order, "#,##0.00")
TALK "- Largest order: $" + FORMAT(largest_order, "#,##0.00")
```

---

## Common Use Cases

### Sales Dashboard

```basic
' Calculate sales metrics
today = FORMAT(NOW(), "YYYY-MM-DD")
this_month = FORMAT(NOW(), "YYYY-MM") + "-01"

today_sales = AGGREGATE "orders" SUM amount WHERE DATE(created_at) = today
month_sales = AGGREGATE "orders" SUM amount WHERE created_at >= this_month
today_count = AGGREGATE "orders" COUNT WHERE DATE(created_at) = today
month_count = AGGREGATE "orders" COUNT WHERE created_at >= this_month

TALK "📊 Sales Dashboard"
TALK "Today: $" + FORMAT(today_sales, "#,##0.00") + " (" + today_count + " orders)"
TALK "This month: $" + FORMAT(month_sales, "#,##0.00") + " (" + month_count + " orders)"
```

### Inventory Summary

```basic
' Calculate inventory metrics
total_items = AGGREGATE "products" COUNT
total_value = AGGREGATE "products" SUM (price * stock)
low_stock = AGGREGATE "products" COUNT WHERE stock < 10
out_of_stock = AGGREGATE "products" COUNT WHERE stock = 0

TALK "Inventory Summary:"
TALK "- Total products: " + total_items
TALK "- Total value: $" + FORMAT(total_value, "#,##0.00")
TALK "- Low stock items: " + low_stock
TALK "- Out of stock: " + out_of_stock
```

### Customer Metrics

```basic
' Calculate customer statistics
total_customers = AGGREGATE "customers" COUNT
new_this_month = AGGREGATE "customers" COUNT WHERE created_at >= this_month
avg_lifetime_value = AGGREGATE "customers" AVERAGE lifetime_value

TALK "Customer Metrics:"
TALK "- Total customers: " + total_customers
TALK "- New this month: " + new_this_month
TALK "- Avg lifetime value: $" + FORMAT(avg_lifetime_value, "#,##0.00")
```

### Rating Analysis

```basic
' Analyze product ratings
reviews = FIND "reviews" WHERE product_id = product.id

avg_rating = AGGREGATE reviews AVERAGE rating
review_count = AGGREGATE reviews COUNT
five_stars = AGGREGATE reviews COUNT WHERE rating = 5

TALK "Product rating: " + FORMAT(avg_rating, "#.#") + " stars"
TALK "Based on " + review_count + " reviews"
TALK five_stars + " customers gave 5 stars"
```

---

## Aggregate from Array

```basic
' Aggregate in-memory data
prices = [29.99, 49.99, 19.99, 99.99, 39.99]

total = AGGREGATE prices SUM
count = AGGREGATE prices COUNT
average = AGGREGATE prices AVERAGE
minimum = AGGREGATE prices MIN
maximum = AGGREGATE prices MAX

TALK "Sum: $" + FORMAT(total, "#,##0.00")
TALK "Count: " + count
TALK "Average: $" + FORMAT(average, "#,##0.00")
TALK "Range: $" + minimum + " - $" + maximum
```

---

## Aggregate with Expressions

```basic
' Calculate computed values
total_revenue = AGGREGATE "order_items" SUM (quantity * unit_price)
total_discount = AGGREGATE "order_items" SUM (quantity * unit_price * discount_percent / 100)
net_revenue = total_revenue - total_discount

TALK "Gross revenue: $" + FORMAT(total_revenue, "#,##0.00")
TALK "Discounts: $" + FORMAT(total_discount, "#,##0.00")
TALK "Net revenue: $" + FORMAT(net_revenue, "#,##0.00")
```

---

## Conditional Aggregation

```basic
' Aggregate with different conditions
pending_total = AGGREGATE "orders" SUM amount WHERE status = "pending"
shipped_total = AGGREGATE "orders" SUM amount WHERE status = "shipped"
delivered_total = AGGREGATE "orders" SUM amount WHERE status = "delivered"

TALK "Order totals by status:"
TALK "- Pending: $" + FORMAT(pending_total, "#,##0.00")
TALK "- Shipped: $" + FORMAT(shipped_total, "#,##0.00")
TALK "- Delivered: $" + FORMAT(delivered_total, "#,##0.00")
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

total = AGGREGATE "orders" SUM amount WHERE customer_id = user.id

IF ERROR THEN
    PRINT "Aggregation failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't calculate your totals."
ELSE IF total = 0 THEN
    TALK "You haven't placed any orders yet."
ELSE
    TALK "Your total purchases: $" + FORMAT(total, "#,##0.00")
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `INVALID_FIELD` | Field doesn't exist | Check field name spelling |
| `TYPE_ERROR` | Non-numeric field for SUM/AVG | Use numeric fields only |
| `EMPTY_COLLECTION` | No data to aggregate | Handle zero/null results |
| `TABLE_NOT_FOUND` | Table doesn't exist | Verify table name |

---

## Null Handling

```basic
' AGGREGATE ignores NULL values by default
avg_rating = AGGREGATE "products" AVERAGE rating
' NULL ratings are not included in the average

' Count non-null values
rated_count = AGGREGATE "products" COUNT WHERE rating IS NOT NULL
total_count = AGGREGATE "products" COUNT

TALK rated_count + " of " + total_count + " products have ratings"
```

---

## Performance Tips

1. **Use WHERE clauses** — Filter before aggregating for better performance
2. **Index aggregate fields** — Ensure database indexes on frequently aggregated columns
3. **Limit data scope** — Aggregate only the date range or subset needed
4. **Cache results** — Store aggregated values for expensive calculations

```basic
' Efficient: Filter first
total = AGGREGATE "orders" SUM amount WHERE date > "2025-01-01"

' Less efficient: Aggregate all, then filter
' all_orders = FIND "orders"
' recent = FILTER all_orders WHERE date > "2025-01-01"
' total = AGGREGATE recent SUM amount
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

- Implemented in Rust under `src/database/aggregate.rs`
- Uses SQL aggregate functions when querying tables
- Handles NULL values according to SQL standards
- Supports expressions in aggregate calculations
- Returns 0 for COUNT on empty sets, NULL for SUM/AVG/MIN/MAX

---

## Related Keywords

- [FIND](keyword-find.md) — Query data before aggregating
- [GROUP BY](keyword-group-by.md) — Group data before aggregating
- [FILTER](keyword-filter.md) — Filter in-memory collections
- [MAP](keyword-map.md) — Transform data before aggregating

---

## Summary

`AGGREGATE` calculates sums, counts, averages, and min/max values from data collections. Use it for dashboards, reports, and any situation where you need to summarize data. It works with both database tables (using SQL) and in-memory arrays. Always handle empty results and use WHERE clauses to improve performance on large datasets.