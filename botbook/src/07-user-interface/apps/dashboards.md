# Dashboards

> **Build custom data visualizations with AI-powered insights**

---

## Overview

Dashboards is the business intelligence and data visualization app in General Bots Suite. Create custom dashboards with drag-and-drop widgets, connect to multiple data sources, and use natural language queries to explore your data. Dashboards helps you monitor KPIs, track metrics, and make data-driven decisions.

**Key Capabilities:**
- **Visual Dashboard Builder** - Drag-and-drop widgets and layouts
- **Multiple Data Sources** - Connect databases, APIs, files, and cloud warehouses
- **Conversational Analytics** - Ask questions in natural language
- **Real-time Updates** - Auto-refresh with configurable intervals
- **Templates** - Pre-built dashboards for common use cases
- **Sharing & Export** - Share dashboards or export as PDF/PNG

---

## Features

### Dashboard Builder

Create dashboards visually with a flexible grid system:

**Layout Options:**

| Layout | Columns | Best For |
|--------|---------|----------|
| **12 Columns** | 12 | Complex dashboards with many widgets |
| **6 Columns** | 6 | Medium complexity |
| **4 Columns** | 4 | Simple, mobile-friendly dashboards |

**Widget Types:**

| Category | Widgets |
|----------|---------|
| **Charts** | Line, Bar, Pie, Donut, Area, Scatter, Heatmap |
| **Data Display** | KPI Card, Table, Gauge, Map |
| **Content** | Text, Image, Embed (iframe) |
| **Filters** | Dropdown Filter, Date Range Picker |

---

### Widget Configuration

Each widget can be configured with:

**Chart Widgets:**
- X-axis and Y-axis field mapping
- Multiple data series
- Colors and legend position
- Animation settings
- Stacked or grouped display

**KPI Widgets:**
- Value field
- Comparison (previous period, target, YoY)
- Thresholds for color coding
- Prefix/suffix formatting

**Table Widgets:**
- Column selection and ordering
- Sorting and filtering
- Pagination settings
- Export options

---

### Data Sources

Connect to various data sources:

| Source Type | Examples |
|-------------|----------|
| **Databases** | PostgreSQL, MySQL, SQL Server, MongoDB |
| **Cloud Warehouses** | BigQuery, Snowflake, Redshift |
| **APIs** | REST API, GraphQL API |
| **Files** | CSV, Excel, Google Sheets |
| **Internal** | GB Suite internal tables |

**Connection Setup:**

1. Click **Add Data Source** in sidebar
2. Select source type
3. Enter connection details:
   - Host, port, database name
   - Username and password (stored in vault)
   - SSL configuration
4. Click **Test Connection**
5. Click **Add Source**

---

### Conversational Analytics

Ask questions about your data in natural language:

**Example Queries:**

| Query | Result |
|-------|--------|
| "Show me sales by region for last quarter" | Bar chart with regional breakdown |
| "What's the trend in customer signups?" | Line chart with trend analysis |
| "Top 10 products by revenue" | Table with ranking |
| "Compare this month vs last month" | KPI cards with comparison |
| "Which day had the most orders?" | Single value with date |

**How It Works:**

1. Type your question in the query box
2. Select target data source (or "All")
3. AI translates to SQL/query
4. Results displayed as visualization
5. Save as widget to dashboard

---

### Templates

Pre-built dashboard templates for common use cases:

| Template | Widgets Included |
|----------|------------------|
| **Sales Dashboard** | Revenue KPI, Sales by Region, Top Products, Pipeline |
| **Marketing Dashboard** | Leads, Conversion Rate, Campaign Performance |
| **Operations Dashboard** | Uptime, Response Time, Error Rate, Active Users |
| **Finance Dashboard** | Revenue, Expenses, Cash Flow, P&L |
| **HR Dashboard** | Headcount, Turnover, Hiring Pipeline |

---

### Sharing & Permissions

**Visibility Options:**

| Setting | Description |
|---------|-------------|
| **Private** | Only you can view and edit |
| **Team** | Team members can view |
| **Organization** | Everyone in org can view |
| **Public** | Anyone with link can view |

**Sharing Actions:**
- Copy link to clipboard
- Embed in other applications
- Schedule email reports
- Export as PDF or PNG

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `N` | New dashboard |
| `E` | Edit mode toggle |
| `R` | Refresh data |
| `S` | Save dashboard |
| `W` | Add widget |
| `/` | Focus query input |
| `Escape` | Close modal/exit edit mode |
| `Ctrl+D` | Duplicate widget |
| `Delete` | Delete selected widget |

---

## Tips & Tricks

### Building Effective Dashboards

💡 **Start with KPIs** - Place most important metrics at the top

💡 **Use consistent colors** - Same color = same meaning across widgets

💡 **Group related widgets** - Organize by topic or data source

💡 **Add context** - Include text widgets to explain metrics

### Performance Tips

💡 **Limit widgets** - 10-15 widgets per dashboard for best performance

💡 **Use date filters** - Narrow data range to speed up queries

💡 **Cache queries** - Enable caching for slow data sources

💡 **Aggregate data** - Pre-aggregate when possible

### Data Source Best Practices

💡 **Use read replicas** - Don't query production databases

💡 **Create views** - Pre-join tables for simpler queries

💡 **Index properly** - Ensure indexes on filtered columns

💡 **Limit permissions** - Grant minimum necessary access

---

## Troubleshooting

### Dashboard not loading

**Possible causes:**
1. Data source connection failed
2. Query timeout
3. Authentication expired

**Solution:**
1. Check data source status in sidebar
2. Increase timeout in settings
3. Re-authenticate with data source
4. Check query performance

---

### Widget shows no data

**Possible causes:**
1. Query returns empty result
2. Date filter excludes all data
3. Field mapping incorrect

**Solution:**
1. Test query directly in data source
2. Adjust date range filter
3. Verify field names match schema
4. Check data type compatibility

---

### Slow dashboard performance

**Possible causes:**
1. Too many widgets
2. Complex queries
3. Large data volumes
4. No caching enabled

**Solution:**
1. Split into multiple dashboards
2. Simplify queries, add aggregations
3. Narrow date ranges
4. Enable query caching in settings

---

## BASIC Integration

Use Dashboards features in your dialogs:

### Query Dashboard Data

```basic
' Get data from a dashboard widget
data = QUERY DASHBOARD "sales-dashboard" WIDGET "revenue-kpi"
TALK "Current revenue: $" + data.value
TALK "Change from last period: " + data.change + "%"
```

### Embed Dashboard in Bot

```basic
' Show a dashboard in the conversation
SHOW DASHBOARD "sales-overview" TO user
TALK "Here's your sales dashboard. What would you like to know more about?"
```

### Create Dashboard Programmatically

```basic
' Create a new dashboard from bot
dashboard = CREATE DASHBOARD
    NAME "Weekly Report"
    TEMPLATE "sales"
    DATE_RANGE "last_7_days"

SHARE DASHBOARD dashboard.id WITH user.email
TALK "Your dashboard is ready: " + dashboard.url
```

### Natural Language Query

```basic
' Let user ask questions about data
TALK "What would you like to know about your data?"
HEAR question AS TEXT

result = QUERY DATA question FROM "sales-db"

IF result.type = "chart" THEN
    SHOW CHART result.data
ELSE
    TALK result.answer
END IF
```

---

## API Reference

### Endpoints Summary

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/dashboards` | GET | List all dashboards |
| `/api/dashboards` | POST | Create new dashboard |
| `/api/dashboards/{id}` | GET | Get dashboard details |
| `/api/dashboards/{id}` | PUT | Update dashboard |
| `/api/dashboards/{id}` | DELETE | Delete dashboard |
| `/api/dashboards/{id}/widgets` | POST | Add widget |
| `/api/dashboards/{id}/widgets/{wid}` | PUT | Update widget |
| `/api/dashboards/{id}/widgets/{wid}` | DELETE | Delete widget |
| `/api/dashboards/{id}/widgets/{wid}/data` | GET | Get widget data |
| `/api/dashboards/sources` | GET | List data sources |
| `/api/dashboards/sources` | POST | Add data source |
| `/api/dashboards/sources/{id}/test` | POST | Test connection |
| `/api/dashboards/query` | POST | Natural language query |
| `/api/dashboards/templates` | GET | List templates |

### Example: Create Dashboard

```json
POST /api/dashboards

{
  "name": "Sales Overview",
  "description": "Weekly sales metrics and trends",
  "layout": {
    "columns": 12,
    "row_height": 80,
    "gap": 16
  },
  "is_public": false,
  "tags": ["sales", "weekly"]
}

Response:
{
  "id": "dash_abc123",
  "name": "Sales Overview",
  "created_at": "2025-01-27T10:00:00Z",
  "widgets": [],
  "url": "/dashboards/dash_abc123"
}
```

### Example: Add Widget

```json
POST /api/dashboards/dash_abc123/widgets

{
  "widget_type": "line_chart",
  "title": "Revenue Trend",
  "position": {
    "x": 0,
    "y": 0,
    "width": 6,
    "height": 4
  },
  "data_query": {
    "source_id": "src_sales_db",
    "sql": "SELECT date, SUM(amount) as revenue FROM orders GROUP BY date ORDER BY date",
    "fields": ["date", "revenue"]
  },
  "config": {
    "chart_config": {
      "x_axis": "date",
      "y_axis": "revenue",
      "colors": ["#10b981"]
    }
  }
}
```

### Example: Natural Language Query

```json
POST /api/dashboards/query

{
  "query": "Show me top 5 customers by revenue last month",
  "data_source_id": "src_sales_db"
}

Response:
{
  "query": "SELECT customer_name, SUM(amount) as revenue FROM orders WHERE date >= '2025-01-01' GROUP BY customer_name ORDER BY revenue DESC LIMIT 5",
  "data": [
    {"customer_name": "Acme Corp", "revenue": 150000},
    {"customer_name": "TechStart", "revenue": 120000},
    ...
  ],
  "suggested_visualization": "bar_chart",
  "explanation": "Top 5 customers ranked by total revenue for January 2025"
}
```

---

## See Also

- [Analytics App](./analytics.md) - Built-in analytics and metrics
- [Compliance App](./compliance.md) - Compliance dashboards
- [Research App](./research.md) - Data research tools
- [Sheet App](./sheet.md) - Spreadsheet data source