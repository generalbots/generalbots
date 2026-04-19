# Talk to Data Template

The Talk to Data template enables natural language queries against your structured data, transforming plain English questions into SQL queries and visualizations. It's like having a data analyst available 24/7.

## Topic: Natural Language Data Analysis

This template is perfect for:
- Business intelligence dashboards
- Self-service analytics
- Report generation on demand
- Data exploration without SQL knowledge
- Executive summaries and KPI tracking

## The Code

```basic
ADD TOOL "query-data"
ADD TOOL "create-chart"
ADD TOOL "export-data"
ADD TOOL "notify-latest-orders"

SET ANSWER MODE "sql"

CLEAR SUGGESTIONS

ADD SUGGESTION "products" AS "Top products chart"
ADD SUGGESTION "sales" AS "Sales across years"
ADD SUGGESTION "orders" AS "Latest orders"
ADD SUGGESTION "chart" AS "Create a chart"
ADD SUGGESTION "export" AS "Export data"

SET CONTEXT "talk-to-data" AS "You are a data analyst assistant helping users query and visualize their data. Convert natural language questions into SQL queries and generate charts. Be helpful and suggest visualizations."

BEGIN TALK
**Talk To Data**

I can help you analyze your data with natural language queries.

**Examples:**
• Show me top products in a rainbow colored pie chart
• Sales across years
• Latest orders this month
• Compare revenue by region

Just ask me anything about your data.
END TALK

BEGIN SYSTEM PROMPT
You are a data analysis assistant that converts natural language to SQL queries.

Chart types:
- timeseries: For data over time
- bar: For comparisons
- pie/donut: For proportions
- line: For trends

When users ask about data:
1. Understand the intent
2. Generate appropriate SQL
3. Suggest relevant visualizations
4. Offer to export if needed

Always use LOWER() for text comparisons.
Use LIKE with %% for partial matches.
Return clear, actionable insights.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the Talk to Data template works in real-world scenarios.

### Dialog 1: Simple Data Query

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Data Analyst</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Data Analyst</div>
      <p><strong>Talk To Data</strong></p>
      <p>I can help you analyze your data with natural language queries.</p>
      <p>Just ask me anything about your data.</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me the top 5 selling products this month</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Top 5 Products - January 2025</strong></p>
      <p></p>
      <p>1. 🏆 Wireless Headphones - 1,234 units ($61,700)</p>
      <p>2. 🥈 USB-C Cable - 987 units ($9,870)</p>
      <p>3. 🥉 Phone Case - 876 units ($8,760)</p>
      <p>4. Bluetooth Speaker - 654 units ($32,700)</p>
      <p>5. Screen Protector - 543 units ($2,715)</p>
      <p></p>
      <p>Would you like me to create a chart or export this data?</p>
      <div class="wa-time">10:01</div>
    </div>
  </div>
</div>

### Dialog 2: Creating a Visualization

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Data Analyst</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me sales by region in a pie chart with rainbow colors</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎨 <strong>Sales by Region</strong></p>
      <p>[PIE CHART]</p>
      <p>🔴 North: 35% ($1.2M)</p>
      <p>🟠 South: 28% ($980K)</p>
      <p>🟡 East: 22% ($770K)</p>
      <p>🟢 West: 15% ($525K)</p>
      <p></p>
      <p><strong>Insight:</strong> North region leads with 35% of total sales. Consider replicating North's strategies in other regions.</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

### Dialog 3: Time Series Analysis

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Data Analyst</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Compare revenue over the last 3 years</p>
      <div class="wa-time">16:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📈 <strong>Revenue Comparison (2022-2024)</strong></p>
      <p>[LINE CHART]</p>
      <p></p>
      <p>• 2022: $2.4M</p>
      <p>• 2023: $3.1M (+29%)</p>
      <p>• 2024: $4.2M (+35%)</p>
      <p></p>
      <p><strong>Key Insights:</strong></p>
      <p>📈 Consistent year-over-year growth</p>
      <p>🎯 2024 exceeded growth rate of 2023</p>
      <p>📅 Q4 strongest quarter in all years</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Break it down by quarter</p>
      <div class="wa-time">16:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Quarterly Revenue Breakdown</strong></p>
      <p></p>
      <p><strong>2024:</strong></p>
      <p>Q1: $850K | Q2: $980K | Q3: $1.1M | Q4: $1.27M</p>
      <p></p>
      <p><strong>2023:</strong></p>
      <p>Q1: $650K | Q2: $720K | Q3: $800K | Q4: $930K</p>
      <p></p>
      <p><strong>2022:</strong></p>
      <p>Q1: $480K | Q2: $560K | Q3: $620K | Q4: $740K</p>
      <p></p>
      <p>Want me to export this to Excel?</p>
      <div class="wa-time">16:01</div>
    </div>
  </div>
</div>

### Dialog 4: Latest Orders Notification

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Data Analyst</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me the latest orders</p>
      <div class="wa-time">09:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🛒 <strong>Latest Orders</strong></p>
      <p></p>
      <p><strong>#ORD-2025-1234</strong> - 5 min ago</p>
      <p>Customer: John Smith | $245.00 | Processing</p>
      <p></p>
      <p><strong>#ORD-2025-1233</strong> - 12 min ago</p>
      <p>Customer: Maria Garcia | $89.50 | Shipped</p>
      <p></p>
      <p><strong>#ORD-2025-1232</strong> - 28 min ago</p>
      <p>Customer: Carlos Santos | $567.00 | Delivered</p>
      <p></p>
      <p><strong>Today's Summary:</strong> 47 orders, $12,450 revenue</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register data query and visualization tools |
| `SET ANSWER MODE` | Configure SQL query generation mode |
| `SET CONTEXT` | Define the data analyst role |
| `ADD SUGGESTION` | Create quick query buttons |
| `BEGIN TALK` | Welcome message with examples |
| `BEGIN SYSTEM PROMPT` | Instructions for SQL generation |

## How It Works

1. **Natural Language Input**: User asks a question in plain English
2. **Intent Understanding**: AI interprets what data is needed
3. **SQL Generation**: Query is automatically generated
4. **Data Retrieval**: SQL executes against your database
5. **Visualization**: Results are formatted or charted
6. **Insights**: AI provides analysis and recommendations

## Query Tool: query-data.bas

```basic
PARAM query AS STRING LIKE "top 10 products by revenue" DESCRIPTION "Natural language data query"
PARAM format AS STRING LIKE "table" DESCRIPTION "Output format: table, chart, export" OPTIONAL

DESCRIPTION "Query data using natural language and return results"

' Convert natural language to SQL using AI
sql = LLM "Convert this to SQL for our database: " + query + ". Tables: products, orders, customers, order_items."

' Execute query
results = SQL sql

IF UBOUND(results) = 0 THEN
    TALK "No data found for your query. Try rephrasing or ask what data is available."
    RETURN NULL
END IF

' Format output based on request
IF format = "chart" OR INSTR(LOWER(query), "chart") > 0 THEN
    ' Determine chart type
    IF INSTR(LOWER(query), "pie") > 0 THEN
        chartType = "pie"
    ELSE IF INSTR(LOWER(query), "line") > 0 OR INSTR(LOWER(query), "trend") > 0 THEN
        chartType = "line"
    ELSE IF INSTR(LOWER(query), "bar") > 0 THEN
        chartType = "bar"
    ELSE
        chartType = "bar"  ' Default
    END IF
    
    chart = CREATE CHART chartType, results
    TALK chart
ELSE
    ' Display as table
    TALK TABLE results
END IF

' Offer insights
IF UBOUND(results) > 5 THEN
    insight = LLM "Provide a brief insight about this data: " + TOJSON(results)
    TALK "💡 **Insight:** " + insight
END IF

RETURN results
```

## Chart Tool: create-chart.bas

```basic
PARAM data_query AS STRING LIKE "sales by month" DESCRIPTION "Data to visualize"
PARAM chart_type AS STRING LIKE "bar" DESCRIPTION "Chart type: bar, line, pie, donut, timeseries"
PARAM title AS STRING LIKE "Monthly Sales" DESCRIPTION "Chart title" OPTIONAL
PARAM colors AS STRING LIKE "rainbow" DESCRIPTION "Color scheme: rainbow, blue, green, custom" OPTIONAL

DESCRIPTION "Create a visualization from data query"

' Get the data
results = CALL query-data(data_query, "raw")

IF NOT results THEN
    TALK "Could not retrieve data for chart."
    RETURN NULL
END IF

' Set chart options
WITH chartOptions
    type = chart_type
    title = IIF(title, title, data_query)
    colorScheme = IIF(colors, colors, "default")
    showLegend = TRUE
    showValues = TRUE
END WITH

' Generate chart
chart = CREATE CHART chartOptions.type, results, chartOptions

TALK chart

' Provide chart summary
TALK "📊 Chart shows " + UBOUND(results) + " data points."

RETURN chart
```

## Notify Latest Orders: notify-latest-orders.bas

```basic
PARAM since AS STRING LIKE "1 hour" DESCRIPTION "Time period for orders" OPTIONAL
PARAM notify AS STRING LIKE "sales@company.com" DESCRIPTION "Email to notify" OPTIONAL

DESCRIPTION "Get latest orders and optionally send notification"

IF NOT since THEN
    since = "1 hour"
END IF

' Calculate time filter
cutoff = DATEADD(NOW(), -1, "hours")
IF INSTR(since, "day") > 0 THEN
    cutoff = DATEADD(NOW(), -1, "days")
ELSE IF INSTR(since, "week") > 0 THEN
    cutoff = DATEADD(NOW(), -7, "days")
END IF

' Query orders
orders = SQL "SELECT * FROM orders WHERE created_at >= '" + FORMAT(cutoff, "YYYY-MM-DD HH:mm:ss") + "' ORDER BY created_at DESC LIMIT 10"

IF UBOUND(orders) = 0 THEN
    TALK "No new orders in the last " + since + "."
    RETURN NULL
END IF

' Calculate totals
totalRevenue = 0
FOR EACH order IN orders
    totalRevenue = totalRevenue + order.total
NEXT

' Display orders
TALK "🛒 **Latest Orders** (Last " + since + ")"
TALK ""

FOR EACH order IN orders
    timeAgo = DATEDIFF(NOW(), order.created_at, "minutes")
    TALK "**#" + order.order_number + "** - " + timeAgo + " min ago"
    TALK "Customer: " + order.customer_name + " | $" + FORMAT(order.total, "#,##0.00") + " | " + order.status
    TALK ""
NEXT

TALK "**Summary:** " + UBOUND(orders) + " orders, $" + FORMAT(totalRevenue, "#,##0.00") + " revenue"

' Send notification if requested
IF notify THEN
    emailBody = "New orders in the last " + since + ":\n\n"
    emailBody = emailBody + "Total Orders: " + UBOUND(orders) + "\n"
    emailBody = emailBody + "Total Revenue: $" + FORMAT(totalRevenue, "#,##0.00")
    
    SEND MAIL notify, "Order Update - " + UBOUND(orders) + " new orders", emailBody
    TALK "📧 Notification sent to " + notify
END IF

RETURN orders
```

## Setting Up Your Data

### Connecting to Data Sources

The Talk to Data template works with various data sources:

```basic
' CSV files
data = FIND "sales.csv"

' Excel files  
data = FIND "reports.xlsx", "Sheet1"

' SQL databases
data = SQL "SELECT * FROM products"

' External APIs
data = GET "https://api.example.com/sales"
```

### Schema Configuration

For best results, configure your data schema:

```basic
SET CONTEXT "data-schema" AS "
Available tables:
- products: id, name, category, price, stock
- orders: id, customer_id, total, status, created_at
- customers: id, name, email, region
- order_items: order_id, product_id, quantity, price
"
```

## Customization Ideas

### Add Scheduled Reports

```basic
PARAM reportType AS STRING

IF reportType = "daily summary" THEN
    SET SCHEDULE "0 8 * * *"  ' Run at 8 AM daily
    
    results = CALL query-data("sales summary for yesterday")
    SEND MAIL "team@company.com", "Daily Sales Summary", results
    
    TALK "Daily report sent."
END IF

IF reportType = "weekly dashboard" THEN
    SET SCHEDULE "0 9 * * 1"  ' Run at 9 AM on Mondays
    
    results = CALL query-data("weekly sales by region")
    chart = CALL create-chart("weekly sales", "bar")
    
    SEND MAIL "executives@company.com", "Weekly Dashboard", chart
END IF
```

### Add Natural Language Filters

```basic
' Enhanced query understanding
PARAM question AS STRING

' Extract time filters
IF INSTR(LOWER(question), "yesterday") > 0 THEN
    dateFilter = "date = '" + FORMAT(NOW() - 1, "YYYY-MM-DD") + "'"
ELSE IF INSTR(LOWER(question), "last week") > 0 THEN
    dateFilter = "date >= '" + FORMAT(NOW() - 7, "YYYY-MM-DD") + "'"
ELSE IF INSTR(LOWER(question), "this month") > 0 THEN
    dateFilter = "MONTH(date) = " + MONTH(NOW())
END IF

' Apply to query
sql = sql + " WHERE " + dateFilter
```

### Add Comparative Analysis

```basic
PARAM metric AS STRING LIKE "revenue"
PARAM compare AS STRING LIKE "this month vs last month"

DESCRIPTION "Compare metrics across time periods"

' Parse comparison periods
IF INSTR(compare, "month") > 0 THEN
    current = SQL "SELECT SUM(" + metric + ") FROM sales WHERE MONTH(date) = " + MONTH(NOW())
    previous = SQL "SELECT SUM(" + metric + ") FROM sales WHERE MONTH(date) = " + (MONTH(NOW()) - 1)
    
    change = ((current - previous) / previous) * 100
    
    TALK "📊 **" + metric + " Comparison**"
    TALK "This month: $" + FORMAT(current, "#,##0")
    TALK "Last month: $" + FORMAT(previous, "#,##0")
    
    IF change > 0 THEN
        TALK "📈 Change: +" + FORMAT(change, "#,##0.0") + "%"
    ELSE
        TALK "📉 Change: " + FORMAT(change, "#,##0.0") + "%"
    END IF
END IF
```

## Best Practices

1. **Define Your Schema**: Provide clear table and column descriptions in context
2. **Use Examples**: Include example queries in the welcome message
3. **Handle Edge Cases**: Always check for empty results
4. **Provide Insights**: Don't just show data—interpret it
5. **Offer Next Steps**: Suggest related queries or visualizations

## Related Templates

- [ai-search.bas](./ai-search.md) - Search documents with AI
- [analytics-dashboard.bas](./analytics-dashboard.md) - System monitoring
- [erp.bas](./erp.md) - Enterprise resource planning

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>