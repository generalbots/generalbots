# Platform Analytics Template (analytics.gbai)

A General Bots template for platform metrics, performance monitoring, and custom report generation.

## Overview

The Analytics template provides comprehensive platform analytics capabilities, allowing administrators and stakeholders to monitor usage, track performance, analyze trends, and generate custom reports through conversational AI.

## Features

- **Platform Overview** - Key metrics summary with trend analysis
- **Message Analytics** - Conversation statistics by channel and bot
- **User Analytics** - Active users, sessions, and engagement
- **Performance Metrics** - Response times and throughput monitoring
- **LLM Usage Tracking** - Token consumption and cost analysis
- **Storage Analytics** - Disk usage and file statistics
- **Error Analysis** - Error patterns and troubleshooting insights
- **Custom Report Generator** - Build and schedule personalized reports
- **AI-Powered Insights** - Automatic trend analysis and recommendations

## Package Structure

```
analytics.gbai/
├── README.md
├── analytics.gbdialog/
│   ├── start.bas              # Main menu and navigation
│   ├── platform-overview.bas  # Key metrics dashboard
│   └── custom-report.bas      # Custom report builder
└── analytics.gbot/
    └── config.csv             # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `start.bas` | Main entry point with analytics menu options |
| `platform-overview.bas` | Platform-wide metrics with trends and export |
| `custom-report.bas` | Interactive custom report generator with scheduling |

## Available Analytics

### 1. Platform Overview

High-level metrics summary including:
- Total messages and trend percentage
- Average active sessions
- Response time performance
- Error rates
- LLM token consumption

### 2. Message Analytics

- Message volume by time period
- Breakdown by channel (WhatsApp, Web, API)
- Bot-specific conversation stats
- Peak usage times

### 3. User Analytics

- Active users (daily, weekly, monthly)
- Session duration analysis
- User retention rates
- Geographic distribution

### 4. Performance Metrics

- Average response time
- Response time distribution
- Throughput (messages per second)
- Latency percentiles (p50, p95, p99)

### 5. LLM Usage

- Token consumption by model
- Cost analysis
- Usage patterns
- Optimization recommendations

### 6. Storage Analytics

- Disk usage trends
- File count and types
- Storage by bot/package
- Cleanup recommendations

### 7. Error Analysis

- Error frequency and patterns
- Error categorization
- Root cause analysis
- Resolution suggestions

### 8. Custom Reports

- Select metrics to include
- Choose time ranges
- Set grouping intervals
- Export in multiple formats
- Schedule automatic delivery

## Usage

### Starting Analytics

```
User: Show me analytics
Bot: Welcome to the Analytics Center. I can help you understand your platform metrics and generate reports.

What would you like to analyze?
1. Platform Overview - Key metrics summary
2. Message Analytics - Conversation statistics
3. User Analytics - Active users and sessions
4. Performance Metrics - Response times and throughput
5. LLM Usage - Token consumption and costs
6. Storage Analytics - Disk usage and file statistics
7. Error Analysis - Error patterns and trends
8. Generate Custom Report
```

### Platform Overview

```
User: 1
Bot: Generating platform overview...
     Select time range (1h, 6h, 24h, 7d, 30d): 24h

     Platform Overview for 24h
     
     📊 Messages
       Total: 15,432
       Trend: +12.5%
     
     👥 Sessions
       Average Active: 234
       Trend: +5.2%
     
     ⚡ Performance
       Avg Response Time: 145 ms
     
     ❌ Errors
       Total: 23
       Error Rate: 0.15%
     
     🤖 LLM Usage
       Total Tokens: 1,245,678
```

### Custom Report Generation

```
User: I need a custom report
Bot: Custom Report Generator
     What would you like to name this report?

User: Weekly Performance Summary

Bot: Select the time range for your report:
     1. Last Hour
     2. Last 24 Hours
     3. Last 7 Days
     4. Last 30 Days
     5. Last 90 Days
     6. Custom Range

User: 3

Bot: Select metrics to include (enter numbers separated by commas):
     1. Message Volume
     2. Active Sessions
     3. Response Time
     4. LLM Token Usage
     5. Error Rate
     ...

User: 1,3,4,5

Bot: Select grouping interval:
     1. Hourly
     2. Daily
     3. Weekly
     4. Monthly

User: 2

Bot: Generating your custom report...
     
     Report generated successfully.
     Report Name: Weekly Performance Summary
     Time Range: Last 7 Days
     Download: https://example.com/reports/report_xxx.pdf
     
     Would you like to receive this report via email?
```

## Metrics Query System

The template uses a metrics query system:

```basic
' Query metrics for a time range
messages = QUERY METRICS "messages" FOR "24h"
totalMessages = SUM(messages, "count")

' Query with grouping
hourlyMessages = QUERY METRICS "messages" FOR "24h" BY "1h"

' Query with offset for comparison
prevMessages = QUERY METRICS "messages" FOR "24h" OFFSET 1
trend = ((totalMessages - SUM(prevMessages, "count")) / SUM(prevMessages, "count")) * 100
```

## Export Formats

Reports can be exported in multiple formats:

| Format | Description |
|--------|-------------|
| PDF | Formatted report with charts |
| XLSX | Excel spreadsheet |
| CSV | Raw data export |
| JSON | Structured data format |

## Scheduled Reports

Configure automatic report delivery:

| Schedule | Cron Expression | Description |
|----------|-----------------|-------------|
| Daily | `0 8 * * *` | Every day at 8 AM |
| Weekly | `0 8 * * 1` | Monday at 8 AM |
| Monthly | `0 8 1 * *` | 1st of month at 8 AM |

```basic
SET SCHEDULE "0 8 * * 1", "generate-scheduled-report.bas"
```

## Configuration

Configure in `analytics.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Default Time Range` | Default period for queries | `7d` |
| `Data Retention Days` | How long to keep metrics | `90` |
| `Admin Email` | Email for scheduled reports | `admin@company.com` |
| `Enable AI Insights` | Auto-generate insights | `true` |
| `Export Path` | Report storage location | `/reports/` |

## Customization

### Adding Custom Metrics

```basic
' Track custom events
INSERT INTO "custom_metrics" VALUES {
    "name": "feature_usage",
    "value": 1,
    "tags": {"feature": "chat", "plan": "pro"},
    "timestamp": NOW()
}

' Query custom metrics
usage = QUERY METRICS "feature_usage" FOR "30d" WHERE tags.feature = "chat"
```

### Custom Dashboard Widgets

```basic
' Add to start.bas
TALK "Custom Metrics:"
TALK "9. Feature Usage"
TALK "10. Revenue Analytics"
TALK "11. Customer Health Score"

' Handle custom options
CASE 9
    CALL "feature-usage.bas"
CASE 10
    CALL "revenue-analytics.bas"
```

### AI-Powered Insights

```basic
' Generate AI insights from metrics
SET CONTEXT "You are an analytics expert. Generate executive insights from this data."

insights = LLM "Analyze this platform data and provide 3-5 key insights: " + JSON(report_data)
```

## Integration

### With Alerting

```basic
' Set up alerts based on metrics
errorRate = SUM(errors, "count") / SUM(messages, "count") * 100

IF errorRate > 5 THEN
    SEND EMAIL admin_email, "High Error Rate Alert", 
        "Error rate is " + errorRate + "%, above 5% threshold."
END IF
```

### With External BI Tools

```basic
' Export data for external tools
data = QUERY METRICS "messages" FOR "30d" BY "1d"
WRITE "analytics_export.csv", CSV(data)

' Or send to webhook
POST "https://bi-tool.example.com/webhook", data
```

## Best Practices

1. **Set appropriate time ranges** - Don't query more data than needed
2. **Use caching** - Cache expensive queries
3. **Schedule off-peak** - Run heavy reports during low traffic
4. **Monitor the monitor** - Track analytics query performance
5. **Archive old data** - Move historical data to cold storage
6. **Validate insights** - Review AI-generated insights for accuracy

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Slow queries | Large time range | Reduce time range or add filters |
| Missing data | Metrics not being collected | Verify instrumentation |
| Export fails | Large report size | Export in chunks or use CSV |
| Stale data | Cache not refreshed | Clear cache or reduce TTL |
| Incorrect trends | Insufficient historical data | Wait for more data collection |

## Related Templates

- `bi.gbai` - Business Intelligence reporting
- `talk-to-data.gbai` - Natural language data queries
- `crm.gbai` - CRM analytics and pipeline reports

## Use Cases

- **Operations Teams** - Monitor platform health and performance
- **Product Managers** - Track feature usage and engagement
- **Executives** - High-level KPI dashboards
- **Support Teams** - Identify error patterns
- **Finance** - LLM cost tracking and optimization

## Data Privacy

- Analytics data is aggregated and anonymized
- User-level data requires appropriate permissions
- Respect data retention policies
- Comply with GDPR/CCPA as applicable

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots