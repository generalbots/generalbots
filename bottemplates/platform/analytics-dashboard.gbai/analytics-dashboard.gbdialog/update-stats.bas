REM Analytics Statistics Update
REM Runs hourly to pre-compute dashboard statistics
REM Similar pattern to announcements/update-summary.bas

SET SCHEDULE "0 * * * *"

REM Fetch KB statistics
stats = KB STATISTICS
statsObj = JSON PARSE stats

REM Store document counts
SET BOT MEMORY "analytics_total_docs", statsObj.total_documents
SET BOT MEMORY "analytics_total_vectors", statsObj.total_vectors
SET BOT MEMORY "analytics_storage_mb", statsObj.total_disk_size_mb
SET BOT MEMORY "analytics_collections", statsObj.total_collections

REM Store activity metrics
SET BOT MEMORY "analytics_docs_week", statsObj.documents_added_last_week
SET BOT MEMORY "analytics_docs_month", statsObj.documents_added_last_month

REM Calculate growth rate
IF statsObj.documents_added_last_month > 0 THEN
    weeklyAvg = statsObj.documents_added_last_month / 4
    IF weeklyAvg > 0 THEN
        growthRate = ((statsObj.documents_added_last_week - weeklyAvg) / weeklyAvg) * 100
        SET BOT MEMORY "analytics_growth_rate", growthRate
    END IF
END IF

REM Check collection health
healthyCount = 0
totalCount = 0
FOR EACH coll IN statsObj.collections
    totalCount = totalCount + 1
    IF coll.status = "green" THEN
        healthyCount = healthyCount + 1
    END IF
NEXT

IF totalCount > 0 THEN
    healthPercent = (healthyCount / totalCount) * 100
    SET BOT MEMORY "analytics_health_percent", healthPercent
END IF

REM Store last update timestamp
SET BOT MEMORY "analytics_last_update", NOW()

REM Generate summary for quick display
summary = "📊 " + FORMAT(statsObj.total_documents, "#,##0") + " docs"
summary = summary + " | " + FORMAT(statsObj.total_disk_size_mb, "#,##0.0") + " MB"
summary = summary + " | +" + FORMAT(statsObj.documents_added_last_week, "#,##0") + " this week"
SET BOT MEMORY "analytics_summary", summary
