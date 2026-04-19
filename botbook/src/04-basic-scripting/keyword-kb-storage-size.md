# KB STORAGE SIZE

The `KB STORAGE SIZE` keyword returns the total disk storage used by the bot's knowledge base in megabytes.

---

## Syntax

```basic
size_mb = KB STORAGE SIZE
```

---

## Parameters

None. Returns the storage size for the current bot's knowledge base.

---

## Description

`KB STORAGE SIZE` queries the Qdrant vector database to calculate the total disk storage consumed by all of the bot's knowledge base collections. This is useful for monitoring storage usage, capacity planning, and cost management.

Use cases include:
- Storage monitoring and alerts
- Capacity planning
- Cost tracking for vector storage
- Admin dashboards
- Cleanup decisions

---

## Return Value

Returns a floating-point number representing storage size in megabytes (MB).

---

## Examples

### Basic Storage Check

```basic
' Get current KB storage usage
storage_mb = KB STORAGE SIZE

TALK "Knowledge base is using " + FORMAT(storage_mb, "#,##0.00") + " MB of storage"
```

### Storage Threshold Alert

```basic
' Alert if storage exceeds threshold
storage_mb = KB STORAGE SIZE
max_storage_mb = 1000  ' 1 GB limit

IF storage_mb > max_storage_mb THEN
    SEND MAIL admin_email,
        "KB Storage Alert",
        "Knowledge base storage (" + FORMAT(storage_mb, "#,##0") + " MB) has exceeded the " + max_storage_mb + " MB threshold.",
        []
    TALK "Storage alert sent to administrator"
ELSE
    remaining = max_storage_mb - storage_mb
    TALK "Storage OK: " + FORMAT(storage_mb, "#,##0") + " MB used, " + FORMAT(remaining, "#,##0") + " MB remaining"
END IF
```

### Storage Tiers Display

```basic
' Display storage status with tier indicators
storage_mb = KB STORAGE SIZE

IF storage_mb < 100 THEN
    tier = "🟢 Light"
ELSE IF storage_mb < 500 THEN
    tier = "🟡 Moderate"
ELSE IF storage_mb < 1000 THEN
    tier = "🟠 Heavy"
ELSE
    tier = "🔴 Critical"
END IF

TALK "Storage Status: " + tier
TALK "Current usage: " + FORMAT(storage_mb, "#,##0.00") + " MB"
```

### Cost Estimation

```basic
' Estimate storage costs (example pricing)
storage_mb = KB STORAGE SIZE
storage_gb = storage_mb / 1024

cost_per_gb = 0.25  ' Example: $0.25 per GB per month
monthly_cost = storage_gb * cost_per_gb

TALK "Current storage: " + FORMAT(storage_gb, "#,##0.00") + " GB"
TALK "Estimated monthly cost: $" + FORMAT(monthly_cost, "#,##0.00")
```

### Storage Growth Tracking

```basic
' Log storage for trend analysis
storage_mb = KB STORAGE SIZE
doc_count = KB DOCUMENTS COUNT

' Calculate average size per document
IF doc_count > 0 THEN
    avg_size_kb = (storage_mb * 1024) / doc_count
    TALK "Average document size: " + FORMAT(avg_size_kb, "#,##0.00") + " KB"
END IF

' Store for trending
INSERT "storage_metrics", #{
    "timestamp": NOW(),
    "storage_mb": storage_mb,
    "doc_count": doc_count,
    "avg_size_kb": avg_size_kb
}
```

### Comprehensive Storage Report

```basic
' Generate storage report
storage_mb = KB STORAGE SIZE
doc_count = KB DOCUMENTS COUNT
recent_docs = KB DOCUMENTS ADDED SINCE 30

' Calculate metrics
storage_gb = storage_mb / 1024
avg_doc_kb = IF(doc_count > 0, (storage_mb * 1024) / doc_count, 0)

report = "## KB Storage Report\n\n"
report = report + "**Date:** " + FORMAT(NOW(), "YYYY-MM-DD") + "\n\n"
report = report + "### Storage Metrics\n"
report = report + "- Total Storage: " + FORMAT(storage_mb, "#,##0.00") + " MB"
report = report + " (" + FORMAT(storage_gb, "#,##0.00") + " GB)\n"
report = report + "- Total Documents: " + FORMAT(doc_count, "#,##0") + "\n"
report = report + "- Avg Size per Doc: " + FORMAT(avg_doc_kb, "#,##0.00") + " KB\n"
report = report + "- Docs Added (30 days): " + recent_docs + "\n"

TALK report
```

### Cleanup Decision Helper

```basic
' Help decide if cleanup is needed
storage_mb = KB STORAGE SIZE
max_storage = 2000  ' 2 GB limit

usage_pct = (storage_mb / max_storage) * 100

IF usage_pct > 80 THEN
    TALK "⚠️ Storage at " + FORMAT(usage_pct, "#0.0") + "% capacity"
    TALK "Consider cleaning up old or unused documents"
    TALK "Use CLEAR KB to remove content if needed"
ELSE IF usage_pct > 60 THEN
    TALK "📊 Storage at " + FORMAT(usage_pct, "#0.0") + "% capacity"
    TALK "Storage is healthy but monitor growth"
ELSE
    TALK "✅ Storage at " + FORMAT(usage_pct, "#0.0") + "% capacity"
    TALK "Plenty of room for more documents"
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

storage_mb = KB STORAGE SIZE

IF ERROR THEN
    PRINT "Error getting storage size: " + ERROR_MESSAGE
    storage_mb = 0.0
END IF

IF storage_mb > 0 THEN
    TALK "Storage usage: " + FORMAT(storage_mb, "#,##0.00") + " MB"
ELSE
    TALK "Unable to determine storage usage"
END IF
```

---

## Related Keywords

- [KB STATISTICS](keyword-kb-statistics.md) — Comprehensive KB statistics including storage
- [KB DOCUMENTS COUNT](keyword-kb-documents-count.md) — Total document count
- [KB DOCUMENTS ADDED SINCE](keyword-kb-documents-added-since.md) — Recently added documents
- [KB COLLECTION STATS](keyword-kb-collection-stats.md) — Per-collection statistics
- [KB LIST COLLECTIONS](keyword-kb-list-collections.md) — List all collections
- [CLEAR KB](keyword-clear-kb.md) — Clear knowledge base content

---

## Configuration

No specific configuration required. Uses the Qdrant connection configured at the system level.

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries Qdrant REST API for collection sizes
- Aggregates disk usage across all bot collections
- Returns value in megabytes (MB) as float
- Returns 0.0 on error (does not throw)
- May take 1-2 seconds for large knowledge bases

---

## Summary

`KB STORAGE SIZE` provides a quick way to check how much disk storage the knowledge base is consuming. Use it for monitoring, capacity planning, cost estimation, and cleanup decisions. For more detailed storage breakdown by collection, use `KB STATISTICS` instead.