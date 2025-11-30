' =============================================================================
' Platform Overview - Key Metrics Summary
' Analytics Bot Dialog for General Bots
' =============================================================================

TALK "Generating platform overview..."

HEAR timeRange AS TEXT WITH "Select time range (1h, 6h, 24h, 7d, 30d):" DEFAULT "24h"

' Query platform metrics from time-series database
messages = QUERY METRICS "messages" FOR timeRange
sessions = QUERY METRICS "active_sessions" FOR timeRange
responseTime = QUERY METRICS "response_time" FOR timeRange
errors = QUERY METRICS "errors" FOR timeRange
tokens = QUERY METRICS "llm_tokens" FOR timeRange

' Calculate totals
totalMessages = SUM(messages, "count")
avgSessions = AVG(sessions, "count")
avgResponseTime = AVG(responseTime, "duration_ms")
totalErrors = SUM(errors, "count")
totalTokens = SUM(tokens, "total_tokens")

' Calculate trends compared to previous period
prevMessages = QUERY METRICS "messages" FOR timeRange OFFSET 1
prevSessions = QUERY METRICS "active_sessions" FOR timeRange OFFSET 1
messagesTrend = ((totalMessages - SUM(prevMessages, "count")) / SUM(prevMessages, "count")) * 100
sessionsTrend = ((avgSessions - AVG(prevSessions, "count")) / AVG(prevSessions, "count")) * 100

TALK "Platform Overview for " + timeRange
TALK ""
TALK "Messages"
TALK "  Total: " + FORMAT(totalMessages, "#,###")
TALK "  Trend: " + FORMAT(messagesTrend, "+#.#") + "%"
TALK ""
TALK "Sessions"
TALK "  Average Active: " + FORMAT(avgSessions, "#,###")
TALK "  Trend: " + FORMAT(sessionsTrend, "+#.#") + "%"
TALK ""
TALK "Performance"
TALK "  Avg Response Time: " + FORMAT(avgResponseTime, "#.##") + " ms"
TALK ""
TALK "Errors"
TALK "  Total: " + FORMAT(totalErrors, "#,###")
TALK "  Error Rate: " + FORMAT((totalErrors / totalMessages) * 100, "#.##") + "%"
TALK ""
TALK "LLM Usage"
TALK "  Total Tokens: " + FORMAT(totalTokens, "#,###")
TALK ""

HEAR action AS TEXT WITH "Options: (D)etail, (E)xport report, (A)lerts, (B)ack"

SELECT CASE UCASE(action)
    CASE "D", "DETAIL"
        TALK "Select metric for detailed view:"
        TALK "1. Messages breakdown by channel"
        TALK "2. Sessions by bot"
        TALK "3. Response time distribution"
        TALK "4. Error breakdown by type"

        HEAR detailChoice AS INTEGER

        SELECT CASE detailChoice
            CASE 1
                CALL "message-analytics.bas"
            CASE 2
                CALL "user-analytics.bas"
            CASE 3
                CALL "performance-metrics.bas"
            CASE 4
                CALL "error-analysis.bas"
        END SELECT

    CASE "E", "EXPORT"
        HEAR exportFormat AS TEXT WITH "Export format (PDF, CSV, XLSX):" DEFAULT "PDF"

        report = {
            "title": "Platform Overview Report",
            "generated_at": NOW(),
            "time_range": timeRange,
            "metrics": {
                "total_messages": totalMessages,
                "messages_trend": messagesTrend,
                "avg_sessions": avgSessions,
                "sessions_trend": sessionsTrend,
                "avg_response_time": avgResponseTime,
                "total_errors": totalErrors,
                "error_rate": (totalErrors / totalMessages) * 100,
                "total_tokens": totalTokens
            }
        }

        filename = "platform_overview_" + FORMAT(NOW(), "YYYYMMDD_HHmmss")

        SELECT CASE UCASE(exportFormat)
            CASE "PDF"
                GENERATE PDF filename + ".pdf" WITH TEMPLATE "analytics_report" DATA report
            CASE "CSV"
                WRITE filename + ".csv", CSV(report.metrics)
            CASE "XLSX"
                WRITE filename + ".xlsx", EXCEL(report)
        END SELECT

        TALK "Report exported: " + filename + "." + LCASE(exportFormat)
        TALK "The file is available in your Drive."

    CASE "A", "ALERTS"
        CALL "configure-alerts.bas"

    CASE "B", "BACK"
        CALL "start.bas"

    CASE ELSE
        CALL "start.bas"
END SELECT
