REM Analytics Dashboard Start Dialog
REM Displays pre-computed statistics from update-stats.bas
REM No heavy computation at conversation start

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
summary = GET BOT MEMORY("analytics_summary")

REM Set contexts for different report types
SET CONTEXT "overview" AS "Total documents: " + totalDocs + ", Storage: " + storageMB + " MB, Collections: " + collections
SET CONTEXT "activity" AS "Documents added this week: " + docsWeek + ", This month: " + docsMonth + ", Growth rate: " + growthRate + "%"
SET CONTEXT "health" AS "System health: " + healthPercent + "%, Last updated: " + lastUpdate

REM Clear and set up suggestions
CLEAR SUGGESTIONS

ADD SUGGESTION "overview" AS "Show overview"
ADD SUGGESTION "overview" AS "Storage usage"
ADD SUGGESTION "activity" AS "Recent activity"
ADD SUGGESTION "activity" AS "Growth trends"
ADD SUGGESTION "health" AS "System health"
ADD SUGGESTION "health" AS "Collection status"

REM Add tools for detailed reports
ADD TOOL "detailed-report"
ADD TOOL "export-stats"

REM Welcome message with pre-computed summary
IF summary <> "" THEN
    TALK summary
    TALK ""
END IF

TALK "📊 **Analytics Dashboard**"
TALK ""

IF totalDocs <> "" THEN
    TALK "**Knowledge Base Overview**"
    TALK "• Documents: " + FORMAT(totalDocs, "#,##0")
    TALK "• Vectors: " + FORMAT(totalVectors, "#,##0")
    TALK "• Storage: " + FORMAT(storageMB, "#,##0.00") + " MB"
    TALK "• Collections: " + collections
    TALK ""

    TALK "**Recent Activity**"
    TALK "• This week: +" + FORMAT(docsWeek, "#,##0") + " documents"
    TALK "• This month: +" + FORMAT(docsMonth, "#,##0") + " documents"

    IF growthRate <> "" THEN
        IF growthRate > 0 THEN
            TALK "• Trend: 📈 +" + FORMAT(growthRate, "#,##0.0") + "% vs average"
        ELSE
            TALK "• Trend: 📉 " + FORMAT(growthRate, "#,##0.0") + "% vs average"
        END IF
    END IF
    TALK ""

    IF healthPercent <> "" THEN
        IF healthPercent = 100 THEN
            TALK "✅ All systems healthy"
        ELSE
            TALK "⚠️ System health: " + FORMAT(healthPercent, "#,##0") + "%"
        END IF
    END IF
ELSE
    TALK "Statistics are being computed. Please check back in a few minutes."
    TALK "Run the update-stats schedule to refresh data."
END IF

TALK ""
TALK "Ask me about any metric or select a topic above."
