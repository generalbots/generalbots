# PIVOT

Transforms rows into columns, creating a cross-tabulation summary of data.

## Syntax

```basic
result = PIVOT data, row_column, column_column, value_column
result = PIVOT data, row_column, column_column, value_column, aggregate
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `data` | Array | The data array to pivot |
| `row_column` | String | Column to use for row headers |
| `column_column` | String | Column to use for column headers |
| `value_column` | String | Column containing values to aggregate |
| `aggregate` | String | Aggregate function: SUM, AVG, COUNT, MIN, MAX (default: SUM) |

## Description

`PIVOT` reorganizes data from a normalized format into a cross-tabulation format, making it easier to compare values across two dimensions. This is useful for creating summary reports and dashboards.

## Examples

### Basic Pivot

```basic
sales = FIND "sales", "year=2025"
summary = PIVOT sales, "region", "quarter", "amount"

' Result: regions as rows, quarters as columns
' Shows total sales for each region/quarter combination
```

### With Aggregate Function

```basic
orders = FIND "orders", "status=completed"
avg_order = PIVOT orders, "product", "month", "total", "AVG"

FOR EACH row IN avg_order
    TALK row.product + ": Q1=$" + row.Q1 + ", Q2=$" + row.Q2
NEXT
```

### Sales by Region and Product

```basic
data = FIND "sales", "year=2025"
pivot_table = PIVOT data, "region", "product", "revenue", "SUM"

TALK "Revenue by Region and Product:"
FOR EACH region IN pivot_table
    TALK region.row_header + ":"
    TALK "  Widgets: $" + region.Widgets
    TALK "  Gadgets: $" + region.Gadgets
NEXT
```

## Return Value

Returns an array of objects where:
- Each object represents a row
- `row_header` contains the row label
- Dynamic properties contain pivoted column values

## Sample Conversation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show quarterly sales by region</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 Quarterly Sales Report 2025</p>
      <p></p>
      <p><strong>Region</strong> | Q1 | Q2 | Q3 | Q4</p>
      <p>━━━━━━━━━━━━━━━━━━━━━━━━</p>
      <p>North | $125K | $142K | $138K | $167K</p>
      <p>South | $98K | $105K | $112K | $128K</p>
      <p>East | $87K | $92K | $95K | $103K</p>
      <p>West | $156K | $168K | $175K | $189K</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
</div>

## Use Cases

| Scenario | Row | Column | Value |
|----------|-----|--------|-------|
| Sales dashboard | Region | Quarter | Revenue |
| Attendance report | Employee | Month | Days |
| Product comparison | Product | Store | Units sold |
| Time tracking | Project | Week | Hours |

## See Also

- [GROUP BY](./keyword-group-by.md) - Group data by columns
- [AGGREGATE](./keyword-aggregate.md) - Calculate summary values
- [TABLE](./keyword-table.md) - Display formatted tables

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