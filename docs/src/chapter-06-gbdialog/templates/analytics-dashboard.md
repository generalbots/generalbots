# Analytics Dashboard Template

The analytics dashboard template provides real-time insights into your knowledge base performance, document statistics, and system health metrics. It uses pre-computed statistics stored in bot memory for fast loading.

## Topic: Knowledge Base Analytics & Monitoring

This template is perfect for:
- Monitoring knowledge base growth
- Tracking document indexing status
- System health monitoring
- Capacity planning

## The Code

```basic
REM Analytics Dashboard Start Dialog
REM Displays pre-computed statistics from update-stats.bas

DESCRIPTION "View knowledge base analytics and statistics"

REM Load pre-computed values from BOT MEMORY
totalDocs = GET BOT MEMORY("analytics_total_docs")
totalVectors = GET BOT MEMORY("analytics_total_vectors")
storageMB = GET BOT MEMORY("analytics_storage_mb")
collections = GET BOT MEMORY("analytics_collections")
docsWeek = GET BOT MEMORY("analytics_docs_week")
docsMonth = GET BOT MEMORY("analytics_docs_month")
growthRate = GET BOT MEMORY("analytics_growth_rate")
healthPercent = GET BOT MEMORY("analytics_health_percent")
lastUpdate = GET BOT MEMORY("analytics_last_update")

REM Set contexts for different report types
SET CONTEXT "overview" AS "Total documents: " + totalDocs + ", Storage: " + storageMB + " MB"
SET CONTEXT "activity" AS "Documents added this week: " + docsWeek + ", Growth rate: " + growthRate + "%"
SET CONTEXT "health" AS "System health: " + healthPercent + "%, Last updated: " + lastUpdate

REM Setup suggestions
CLEAR SUGGESTIONS
ADD SUGGESTION "overview" AS "Show overview"
ADD SUGGESTION "activity" AS "Recent activity"
ADD SUGGESTION "health" AS "System health"

REM Display dashboard
TALK "📊 **Analytics Dashboard**"
TALK ""
TALK "**Knowledge Base Overview**"
TALK "• Documents: " + FORMAT(totalDocs, "#,##0")
TALK "• Vectors: " + FORMAT(totalVectors, "#,##0")
TALK "• Storage: " + FORMAT(storageMB, "#,##0.00") + " MB"
TALK ""
TALK "Ask me about any metric or select a topic above."
```

## Sample Dialogs

These conversations show how the analytics dashboard works in real-world scenarios.

### Dialog 1: Viewing Overview Statistics

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Analytics Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Analytics Bot</div>
      <p>📊 <strong>Analytics Dashboard</strong></p>
      <p><strong>Knowledge Base Overview</strong></p>
      <p>• Documents: 1,234</p>
      <p>• Vectors: 45,678</p>
      <p>• Storage: 256.50 MB</p>
      <p>• Collections: 12</p>
      <p>Ask me about any metric or select a topic above.</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show recent activity</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Recent Activity</strong></p>
      <p>• This week: +87 documents</p>
      <p>• This month: +312 documents</p>
      <p>• Trend: 📈 +15.3% vs average</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>
</div>

### Dialog 2: Checking System Health

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Analytics Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's the system health?</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>All systems healthy</strong></p>
      <p>• Database: Connected</p>
      <p>• Vector Store: Operational</p>
      <p>• Storage: 45% used</p>
      <p>• Last Update: 5 minutes ago</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

### Dialog 3: Statistics Not Yet Computed

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📊</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Analytics Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Analytics Bot</div>
      <p>📊 <strong>Analytics Dashboard</strong></p>
      <p>Statistics are being computed. Please check back in a few minutes.</p>
      <p>Run the update-stats schedule to refresh data.</p>
      <div class="wa-time">08:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `GET BOT MEMORY` | Retrieve pre-computed statistics |
| `SET CONTEXT` | Provide context for AI responses |
| `CLEAR SUGGESTIONS` | Reset quick reply options |
| `ADD SUGGESTION` | Add quick reply buttons |
| `TALK` | Display formatted statistics |
| `FORMAT` | Format numbers with separators |

## How It Works

1. **Load Statistics**: Pre-computed values are retrieved from bot memory
2. **Set Contexts**: Different contexts are set for overview, activity, and health queries
3. **Setup UI**: Quick reply suggestions are configured
4. **Display Dashboard**: Formatted statistics are shown to the user

## The Update Stats Job

Statistics are pre-computed by a scheduled job to ensure fast dashboard loading:

```basic
REM update-stats.bas - Scheduled job to compute analytics
SET SCHEDULE "0 * * * *"  REM Run every hour

REM Compute statistics
totalDocs = KB DOCUMENTS COUNT()
totalVectors = KB STATISTICS().total_vectors
storageMB = KB STORAGE SIZE() / 1024 / 1024
collections = UBOUND(KB LIST COLLECTIONS())

REM Calculate activity
docsWeek = KB DOCUMENTS ADDED SINCE(NOW() - 7)
docsMonth = KB DOCUMENTS ADDED SINCE(NOW() - 30)

REM Store in bot memory
SET BOT MEMORY "analytics_total_docs", totalDocs
SET BOT MEMORY "analytics_total_vectors", totalVectors
SET BOT MEMORY "analytics_storage_mb", storageMB
SET BOT MEMORY "analytics_collections", collections
SET BOT MEMORY "analytics_docs_week", docsWeek
SET BOT MEMORY "analytics_docs_month", docsMonth
SET BOT MEMORY "analytics_last_update", NOW()

TALK "Analytics updated successfully."
```

## Customization Ideas

### Add Export Functionality

```basic
ADD TOOL "export-stats"

REM In export-stats.bas
PARAM format AS STRING LIKE "csv" DESCRIPTION "Export format: csv, json, xlsx"

data = []
data.total_docs = GET BOT MEMORY("analytics_total_docs")
data.total_vectors = GET BOT MEMORY("analytics_total_vectors")
data.storage_mb = GET BOT MEMORY("analytics_storage_mb")

IF format = "csv" THEN
    SAVE "analytics-export.csv", data
    TALK "📥 Analytics exported to analytics-export.csv"
ELSE IF format = "json" THEN
    WRITE "analytics-export.json", TOJSON(data)
    TALK "📥 Analytics exported to analytics-export.json"
END IF
```

### Add Alerting

```basic
REM Check for issues and alert
IF healthPercent < 90 THEN
    SEND MAIL "admin@company.com", "System Health Alert", "Health dropped to " + healthPercent + "%"
END IF

IF storageMB > 900 THEN
    SEND MAIL "admin@company.com", "Storage Warning", "Storage usage at " + storageMB + " MB"
END IF
```

### Add Trend Visualization

```basic
REM Generate a simple trend chart
ADD TOOL "show-trend"

REM Collect historical data
history = FIND "analytics_history.csv", "date > " + FORMAT(NOW() - 30, "YYYY-MM-DD")

REM Create chart
chart = CREATE CHART "line", history, "date", "documents"
TALK chart
```

## Related Templates

- [backup.bas](./backup.md) - Backup management and monitoring
- [start.bas](./start.md) - Basic bot structure

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