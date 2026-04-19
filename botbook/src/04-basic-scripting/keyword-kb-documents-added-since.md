# KB DOCUMENTS ADDED SINCE

The `KB DOCUMENTS ADDED SINCE` keyword returns the count of documents added to the knowledge base within a specified number of days, useful for tracking ingestion activity and monitoring growth.

---

## Syntax

```basic
count = KB DOCUMENTS ADDED SINCE days
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `days` | Number | Number of days to look back |

---

## Description

`KB DOCUMENTS ADDED SINCE` queries the database to count how many documents were added to the bot's knowledge base within the specified time window. This is useful for tracking ingestion rates, monitoring content growth, and generating activity reports.

Use cases include:
- Tracking daily/weekly document ingestion
- Monitoring automated content pipelines
- Activity reports and dashboards
- Alert systems for low/high activity
- Growth trend analysis

---

## Return Value

Returns an integer representing the number of documents added within the specified period.

---

## Examples

### Basic Usage

```basic
' Count documents added in last 7 days
weekly_count = KB DOCUMENTS ADDED SINCE 7

TALK "Documents added this week: " + weekly_count
```

### Daily Activity Check

```basic
' Check today's ingestion
today_count = KB DOCUMENTS ADDED SINCE 1

IF today_count = 0 THEN
    TALK "No new documents added today"
ELSE
    TALK today_count + " documents added today"
END IF
```

### Growth Comparison

```basic
' Compare recent activity periods
last_week = KB DOCUMENTS ADDED SINCE 7
last_month = KB DOCUMENTS ADDED SINCE 30

weekly_average = last_month / 4

IF last_week > weekly_average * 1.5 THEN
    TALK "Document ingestion is above average this week!"
ELSE IF last_week < weekly_average * 0.5 THEN
    TALK "Document ingestion is below average this week"
ELSE
    TALK "Document ingestion is on track"
END IF
```

### Activity Alert System

```basic
' Alert if no documents added recently
recent_docs = KB DOCUMENTS ADDED SINCE 3

IF recent_docs = 0 THEN
    SEND MAIL admin_email,
        "KB Activity Alert",
        "No documents have been added to the knowledge base in the last 3 days. Please check content pipelines.",
        []
    TALK "Alert sent - no recent KB activity"
END IF
```

### Scheduled Activity Report

```basic
' Weekly ingestion report (run via SET SCHEDULE)
day_1 = KB DOCUMENTS ADDED SINCE 1
day_7 = KB DOCUMENTS ADDED SINCE 7
day_30 = KB DOCUMENTS ADDED SINCE 30

report = "KB Ingestion Report\n\n"
report = report + "Last 24 hours: " + day_1 + " documents\n"
report = report + "Last 7 days: " + day_7 + " documents\n"
report = report + "Last 30 days: " + day_30 + " documents\n"
report = report + "\nDaily average (30 days): " + FORMAT(day_30 / 30, "#,##0.0") + "\n"
report = report + "Weekly average (30 days): " + FORMAT(day_30 / 4, "#,##0.0")

SEND MAIL admin_email, "Weekly KB Ingestion Report", report, []
```

### Pipeline Monitoring

```basic
' Monitor automated document pipeline
expected_daily = 50  ' Expected documents per day
tolerance = 0.2  ' 20% tolerance

yesterday_count = KB DOCUMENTS ADDED SINCE 1
min_expected = expected_daily * (1 - tolerance)
max_expected = expected_daily * (1 + tolerance)

IF yesterday_count < min_expected THEN
    TALK "Warning: Only " + yesterday_count + " documents ingested yesterday (expected ~" + expected_daily + ")"
    LOG_WARN "Low document ingestion: " + yesterday_count
ELSE IF yesterday_count > max_expected THEN
    TALK "Note: High ingestion yesterday - " + yesterday_count + " documents"
    LOG_INFO "High document ingestion: " + yesterday_count
ELSE
    TALK "Document pipeline operating normally: " + yesterday_count + " documents yesterday"
END IF
```

---

## Use with Other KB Keywords

```basic
' Comprehensive KB activity check
total_docs = KB DOCUMENTS COUNT
recent_docs = KB DOCUMENTS ADDED SINCE 7
storage_mb = KB STORAGE SIZE

TALK "Knowledge Base Status:"
TALK "  Total documents: " + FORMAT(total_docs, "#,##0")
TALK "  Added this week: " + recent_docs
TALK "  Storage used: " + FORMAT(storage_mb, "#,##0.00") + " MB"

IF recent_docs > 0 THEN
    pct_new = (recent_docs / total_docs) * 100
    TALK "  " + FORMAT(pct_new, "#,##0.0") + "% of KB is from this week"
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

count = KB DOCUMENTS ADDED SINCE 7

IF ERROR THEN
    PRINT "Failed to get document count: " + ERROR_MESSAGE
    count = 0
END IF

TALK "Documents added recently: " + count
```

---

## Related Keywords

- [KB STATISTICS](keyword-kb-statistics.md) — Comprehensive KB statistics
- [KB DOCUMENTS COUNT](keyword-kb-documents-count.md) — Total document count
- [KB COLLECTION STATS](keyword-kb-collection-stats.md) — Per-collection statistics
- [KB STORAGE SIZE](keyword-kb-storage-size.md) — Storage usage
- [KB LIST COLLECTIONS](keyword-kb-list-collections.md) — List collections

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries PostgreSQL `kb_documents` table by `created_at` timestamp
- Filters by current bot ID
- Returns 0 if no documents found or on error
- Days parameter is converted to interval for SQL query

---

## Summary

`KB DOCUMENTS ADDED SINCE` provides a simple way to track recent document ingestion activity. Use it for monitoring content pipelines, generating activity reports, and creating alerts for unusual activity levels. Combine with other KB keywords for comprehensive knowledge base monitoring.