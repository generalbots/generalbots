# Business Intelligence Template (bi.gbai)

A General Bots template for automated business intelligence reporting and data visualization.

---

## Overview

The BI template provides scheduled analytics reporting with automatic chart generation and delivery. It's designed for organizations that need automated consumption reports, category analysis, and customer-specific insights.

## Features

- **Scheduled Reporting** - Automated report generation on configurable schedules
- **Time-Series Charts** - Monthly consumption trends visualization
- **Category Analysis** - Product category breakdown with donut charts
- **Per-Customer Reports** - Individual customer consumption analysis
- **Multi-Channel Delivery** - Send reports via chat, email, or messaging platforms

---

## Package Structure

```
bi.gbai/
├── bi.gbai/
│   ├── bi-admin.bas      # Administrative scheduled reports
│   └── bi-user.bas       # Per-customer report generation
```

## Scripts

| File | Description |
|------|-------------|
| `bi-admin.bas` | Scheduled job for generating platform-wide analytics reports |
| `bi-user.bas` | Loop through customers to generate individual consumption reports |

---

## Configuration

Configure the template in your bot's `config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Schedule` | Cron expression for report timing | `1 * * * * *` |
| `Data Source` | Table/view for billing data | `Orders` |

---

## Usage

### Administrative Reports

The `bi-admin.bas` script runs on a schedule and generates:

1. **Monthly Consumption Chart** - Time-series showing spending trends
2. **Product Category Breakdown** - Donut chart of spending by category

```basic
SET SCHEDULE "1 * * * * *"

billing = FIND "Orders"

' Monthly consumption
data = SELECT SUM(UnitPrice * Quantity) as Value, 
       MONTH(OrderDate)+'/'+YEAR(OrderDate) 
       FROM billing 
       GROUP BY MONTH(OrderDate), YEAR(OrderDate)

img = CHART "timeseries", data
SEND FILE img, "Monthly Consumption"
```

### Per-Customer Reports

The `bi-user.bas` script iterates through customers to generate personalized reports:

```basic
customers = FIND "Customers"

FOR EACH c IN customers
    data = SELECT SUM(UnitPrice * Quantity) AS Value, 
           MONTH(OrderDate)+'/'+YEAR(OrderDate) 
           FROM billing
           JOIN Customers ON billing.CustomerID = Customers.CustomerID
           GROUP BY MONTH(OrderDate), YEAR(OrderDate)
           WHERE Customers.CustomerID = c.CustomerID

    img = CHART "timeseries", data
    SEND FILE img, "Monthly Consumption"
END FOR
```

---

## Chart Types

The template supports various chart types:

| Type | Use Case |
|------|----------|
| `timeseries` | Trends over time (monthly, weekly, daily) |
| `donut` | Category distribution |
| `bar` | Comparative analysis |
| `pie` | Percentage breakdowns |

---

## Data Requirements

### Orders Table Schema

The template expects a billing/orders data source with:

- `OrderDate` - Date of the transaction
- `UnitPrice` - Price per unit
- `Quantity` - Number of units
- `ProductID` - Foreign key to products
- `CustomerID` - Foreign key to customers

### Products Table Schema

- `ProductID` - Primary key
- `CategoryID` - Foreign key to categories
- `ProductName` - Product name

### Categories Table Schema

- `CategoryID` - Primary key
- `CategoryName` - Category display name

---

## Example Output

### Monthly Consumption Report

```
📊 Monthly Consumption Report
-----------------------------
Generated: 2024-01-15 08:00

[Time Series Chart Image]

Total Revenue: $125,430
Top Month: December ($18,500)
Growth Rate: +12% MoM
```

### Category Breakdown

```
📊 Product Category Distribution
--------------------------------

[Donut Chart Image]

Electronics: 35%
Clothing: 28%
Home & Garden: 22%
Other: 15%
```

---

## Customization

### Adding New Reports

Create additional `.bas` files in the `bi.gbai` folder:

```basic
' sales-by-region.bas
SET SCHEDULE "0 9 * * 1"  ' Every Monday at 9 AM

data = SELECT Region, SUM(Amount) as Total 
       FROM Sales 
       GROUP BY Region

img = CHART "bar", data
SEND FILE img, "Weekly Regional Sales"
```

### Customizing Delivery

Send reports to specific users or channels:

```basic
' Send to specific user
SEND FILE img TO "manager@company.com", "Weekly Report"

' Send to WhatsApp
SEND FILE img TO "+1234567890", "Your monthly report"

' Send to team channel
TALK TO "sales-team", img
```

---

## Scheduling Options

| Schedule | Cron Expression | Description |
|----------|-----------------|-------------|
| Every minute | `1 * * * * *` | Testing/real-time |
| Hourly | `0 0 * * * *` | Frequent updates |
| Daily 8 AM | `0 0 8 * * *` | Morning reports |
| Weekly Monday | `0 0 9 * * 1` | Weekly summaries |
| Monthly 1st | `0 0 8 1 * *` | Monthly reports |

---

## Integration Examples

### With CRM

```basic
' Combine with CRM data
opportunities = FIND "opportunities.csv"
revenue = SELECT stage, SUM(amount) FROM opportunities GROUP BY stage

img = CHART "funnel", revenue
SEND FILE img, "Sales Pipeline"
```

### With ERP

```basic
' Inventory analysis
inventory = FIND "inventory.csv"
low_stock = SELECT product, quantity FROM inventory WHERE quantity < reorder_level

img = CHART "bar", low_stock
SEND FILE img, "Low Stock Alert"
```

---

## Best Practices

1. **Schedule appropriately** - Don't run heavy reports too frequently
2. **Filter data** - Use date ranges to limit data volume
3. **Cache results** - Store computed metrics for faster access
4. **Log activities** - Track report generation for auditing
5. **Handle errors** - Wrap queries in error handling

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Empty charts | Verify data source has records |
| Schedule not running | Check cron syntax |
| Slow reports | Add date filters, optimize queries |
| Missing data | Verify JOIN conditions |

---

## Related Templates

- [Platform Analytics](./template-analytics.md) - Platform metrics and monitoring
- [Talk to Data](./template-talk-to-data.md) - Natural language data queries
- [CRM](./template-crm.md) - CRM with built-in reporting

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide