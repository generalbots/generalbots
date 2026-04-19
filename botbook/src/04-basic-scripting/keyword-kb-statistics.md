# KB STATISTICS

The `KB STATISTICS` keyword retrieves comprehensive statistics about the bot's knowledge base, including document counts, vector counts, storage usage, and collection information from the Qdrant vector database.

---

## Syntax

```basic
stats = KB STATISTICS
```

---

## Parameters

None. Returns statistics for the current bot's knowledge base.

---

## Description

`KB STATISTICS` queries the Qdrant vector database to gather comprehensive metrics about the bot's knowledge base. This is useful for monitoring KB health, planning capacity, generating admin reports, and tracking document ingestion over time.

The keyword returns a JSON object containing:
- Total collections count
- Total documents across all collections
- Total vectors stored
- Disk and RAM usage
- Documents added in the last week/month
- Per-collection statistics

Use cases include:
- Admin dashboards and monitoring
- Capacity planning and alerts
- Usage reporting and analytics
- Knowledge base health checks
- Cost tracking for vector storage

---

## Return Value

Returns a JSON string with the following structure:

| Property | Type | Description |
|----------|------|-------------|
| `total_collections` | Number | Number of KB collections for this bot |
| `total_documents` | Number | Total document count across collections |
| `total_vectors` | Number | Total vectors stored in Qdrant |
| `total_disk_size_mb` | Number | Disk storage usage in MB |
| `total_ram_size_mb` | Number | RAM usage in MB |
| `documents_added_last_week` | Number | Documents added in past 7 days |
| `documents_added_last_month` | Number | Documents added in past 30 days |
| `collections` | Array | Detailed stats per collection |

### Collection Stats Object

Each collection in the `collections` array contains:

| Property | Type | Description |
|----------|------|-------------|
| `name` | String | Collection name |
| `vectors_count` | Number | Vectors in this collection |
| `points_count` | Number | Points (documents) count |
| `segments_count` | Number | Storage segments |
| `disk_data_size` | Number | Disk size in bytes |
| `ram_data_size` | Number | RAM size in bytes |
| `indexed_vectors_count` | Number | Indexed vectors |
| `status` | String | Collection status (green/yellow/red) |

---

## Examples

### Basic Statistics Retrieval

```basic
' Get KB statistics
stats_json = KB STATISTICS

' Parse the JSON response
stats = PARSE_JSON(stats_json)

TALK "Your knowledge base has:"
TALK "  - " + stats.total_documents + " documents"
TALK "  - " + stats.total_vectors + " vectors"
TALK "  - " + FORMAT(stats.total_disk_size_mb, "#,##0.00") + " MB on disk"
```

### Admin Dashboard Report

```basic
' Generate KB health report for administrators
stats_json = KB STATISTICS
stats = PARSE_JSON(stats_json)

report = "## Knowledge Base Report\n\n"
report = report + "**Generated:** " + FORMAT(NOW(), "YYYY-MM-DD HH:mm") + "\n\n"
report = report + "### Summary\n"
report = report + "- Collections: " + stats.total_collections + "\n"
report = report + "- Total Documents: " + FORMAT(stats.total_documents, "#,##0") + "\n"
report = report + "- Total Vectors: " + FORMAT(stats.total_vectors, "#,##0") + "\n"
report = report + "- Disk Usage: " + FORMAT(stats.total_disk_size_mb, "#,##0.00") + " MB\n"
report = report + "- RAM Usage: " + FORMAT(stats.total_ram_size_mb, "#,##0.00") + " MB\n\n"
report = report + "### Recent Activity\n"
report = report + "- Added this week: " + stats.documents_added_last_week + "\n"
report = report + "- Added this month: " + stats.documents_added_last_month + "\n"

TALK report
```

### Storage Alert System

```basic
' Check KB storage and alert if threshold exceeded
stats_json = KB STATISTICS
stats = PARSE_JSON(stats_json)

storage_threshold_mb = 1000  ' 1 GB warning threshold
critical_threshold_mb = 5000  ' 5 GB critical threshold

IF stats.total_disk_size_mb > critical_threshold_mb THEN
    SEND MAIL admin_email, 
        "CRITICAL: KB Storage Alert",
        "Knowledge base storage is at " + FORMAT(stats.total_disk_size_mb, "#,##0") + " MB. Immediate action required.",
        []
    TALK "Critical storage alert sent to administrator"
ELSE IF stats.total_disk_size_mb > storage_threshold_mb THEN
    SEND MAIL admin_email,
        "Warning: KB Storage Growing",
        "Knowledge base storage is at " + FORMAT(stats.total_disk_size_mb, "#,##0") + " MB. Consider cleanup.",
        []
    TALK "Storage warning sent to administrator"
ELSE
    TALK "Storage levels are healthy: " + FORMAT(stats.total_disk_size_mb, "#,##0") + " MB"
END IF
```

### Collection Health Check

```basic
' Check health of each collection
stats_json = KB STATISTICS
stats = PARSE_JSON(stats_json)

unhealthy_collections = []

FOR EACH collection IN stats.collections
    IF collection.status <> "green" THEN
        unhealthy_collections = unhealthy_collections + [collection.name]
        PRINT "Warning: Collection " + collection.name + " status is " + collection.status
    END IF
END FOR

IF LEN(unhealthy_collections) > 0 THEN
    TALK "Found " + LEN(unhealthy_collections) + " collections needing attention"
ELSE
    TALK "All " + stats.total_collections + " collections are healthy"
END IF
```

### Scheduled Statistics Report

```basic
' Weekly KB statistics email (run via SET SCHEDULE)
stats_json = KB STATISTICS
stats = PARSE_JSON(stats_json)

' Calculate week-over-week growth
weekly_growth = stats.documents_added_last_week
monthly_growth = stats.documents_added_last_month
avg_weekly = monthly_growth / 4

body = "Weekly Knowledge Base Statistics\n\n"
body = body + "Total Documents: " + FORMAT(stats.total_documents, "#,##0") + "\n"
body = body + "Documents Added This Week: " + weekly_growth + "\n"
body = body + "4-Week Average: " + FORMAT(avg_weekly, "#,##0.0") + "\n"
body = body + "Storage Used: " + FORMAT(stats.total_disk_size_mb, "#,##0.00") + " MB\n"
body = body + "\nCollections:\n"

FOR EACH coll IN stats.collections
    body = body + "  - " + coll.name + ": " + FORMAT(coll.points_count, "#,##0") + " docs\n"
END FOR

SEND MAIL admin_email, "Weekly KB Report - " + FORMAT(NOW(), "YYYY-MM-DD"), body, []
```

### Usage Analytics Integration

```basic
' Log KB stats to analytics system
stats_json = KB STATISTICS
stats = PARSE_JSON(stats_json)

' Store metrics for trending
metrics = #{
    "timestamp": FORMAT(NOW(), "YYYY-MM-DDTHH:mm:ss"),
    "bot_id": bot_id,
    "total_docs": stats.total_documents,
    "total_vectors": stats.total_vectors,
    "disk_mb": stats.total_disk_size_mb,
    "ram_mb": stats.total_ram_size_mb,
    "collections": stats.total_collections
}

INSERT "kb_metrics", metrics

PRINT "KB metrics logged at " + metrics.timestamp
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

stats_json = KB STATISTICS

IF ERROR THEN
    PRINT "Failed to get KB statistics: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't retrieve knowledge base statistics right now."
ELSE
    IF stats_json = "" THEN
        TALK "No knowledge base data available yet."
    ELSE
        stats = PARSE_JSON(stats_json)
        TALK "KB contains " + stats.total_documents + " documents"
    END IF
END IF
```

---

## Related Keywords

- [KB COLLECTION STATS](keyword-kb-collection-stats.md) — Get stats for a specific collection
- [KB DOCUMENTS COUNT](keyword-kb-documents-count.md) — Get total document count
- [KB DOCUMENTS ADDED SINCE](keyword-kb-documents-added-since.md) — Count recently added documents
- [KB LIST COLLECTIONS](keyword-kb-list-collections.md) — List all KB collections
- [KB STORAGE SIZE](keyword-kb-storage-size.md) — Get storage usage in MB
- [CLEAR KB](keyword-clear-kb.md) — Clear knowledge base content
- [USE KB](keyword-use-kb.md) — Enable knowledge base for queries

---

## Configuration

No specific configuration required. The keyword uses the Qdrant connection configured at the system level.

Ensure Qdrant is running and accessible:

```csv
name,value
qdrant-url,https://localhost:6334
```

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries Qdrant REST API for collection statistics
- Filters collections by bot ID prefix (`kb_{bot_id}`)
- Document counts from both Qdrant and PostgreSQL
- Returns JSON string for flexible parsing
- May take 1-2 seconds for large knowledge bases

---

## Summary

`KB STATISTICS` provides comprehensive metrics about the bot's knowledge base, enabling administrators to monitor health, track growth, and plan capacity. Use it for dashboards, alerts, and reporting. For simpler queries, use the specialized keywords like `KB DOCUMENTS COUNT` or `KB STORAGE SIZE`.