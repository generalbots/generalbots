# KB Statistics Keywords

Knowledge Base Statistics keywords provide real-time information about your Qdrant vector database collections. Use these keywords to monitor document counts, storage usage, and indexing activity.

## Overview

These keywords are useful for:

- **Administration**: Monitor KB health and growth
- **Dashboards**: Display statistics in admin interfaces
- **Automation**: Trigger actions based on KB state
- **Compliance**: Track document retention and storage

## Available Keywords

| Keyword | Returns | Description |
|---------|---------|-------------|
| `KB STATISTICS` | JSON | Complete statistics for all collections |
| `KB COLLECTION STATS` | JSON | Statistics for a specific collection |
| `KB DOCUMENTS COUNT` | Integer | Total document count for bot |
| `KB DOCUMENTS ADDED SINCE` | Integer | Documents added in last N days |
| `KB LIST COLLECTIONS` | Array | List of collection names |
| `KB STORAGE SIZE` | Float | Total storage in MB |

## KB STATISTICS

Returns comprehensive statistics about all knowledge base collections for the current bot.

### Syntax

```basic
stats = KB STATISTICS
```

### Return Value

JSON string containing:

```json
{
  "total_collections": 3,
  "total_documents": 5000,
  "total_vectors": 5000,
  "total_disk_size_mb": 125.5,
  "total_ram_size_mb": 62.3,
  "documents_added_last_week": 150,
  "documents_added_last_month": 620,
  "collections": [
    {
      "name": "kb_bot-id_main",
      "vectors_count": 3000,
      "points_count": 3000,
      "segments_count": 2,
      "disk_data_size": 78643200,
      "ram_data_size": 39321600,
      "indexed_vectors_count": 3000,
      "status": "green"
    }
  ]
}
```

### Example

```basic
REM Get and display KB statistics
stats = KB STATISTICS
statsObj = JSON PARSE stats

TALK "Your knowledge base has " + statsObj.total_documents + " documents"
TALK "Using " + FORMAT(statsObj.total_disk_size_mb, "#,##0.00") + " MB of storage"

IF statsObj.documents_added_last_week > 100 THEN
    TALK "High activity! " + statsObj.documents_added_last_week + " documents added this week"
END IF
```

## KB COLLECTION STATS

Returns detailed statistics for a specific Qdrant collection.

### Syntax

```basic
stats = KB COLLECTION STATS collection_name
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_name` | String | Name of the collection |

### Return Value

JSON string with collection details:

```json
{
  "name": "kb_bot-id_products",
  "vectors_count": 1500,
  "points_count": 1500,
  "segments_count": 1,
  "disk_data_size": 52428800,
  "ram_data_size": 26214400,
  "indexed_vectors_count": 1500,
  "status": "green"
}
```

### Example

```basic
REM Check specific collection health
collections = KB LIST COLLECTIONS

FOR EACH collection IN collections
    stats = KB COLLECTION STATS collection
    collObj = JSON PARSE stats
    
    IF collObj.status <> "green" THEN
        TALK "Warning: Collection " + collection + " status is " + collObj.status
    END IF
NEXT
```

## KB DOCUMENTS COUNT

Returns the total number of documents indexed for the current bot.

### Syntax

```basic
count = KB DOCUMENTS COUNT
```

### Return Value

Integer representing total document count.

### Example

```basic
docCount = KB DOCUMENTS COUNT

IF docCount = 0 THEN
    TALK "Your knowledge base is empty. Upload some documents to get started!"
ELSE
    TALK "You have " + FORMAT(docCount, "#,##0") + " documents in your knowledge base"
END IF
```

## KB DOCUMENTS ADDED SINCE

Returns the number of documents added within the specified number of days.

### Syntax

```basic
count = KB DOCUMENTS ADDED SINCE days
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `days` | Integer | Number of days to look back |

### Return Value

Integer representing documents added in the time period.

### Example

```basic
REM Activity report
lastDay = KB DOCUMENTS ADDED SINCE 1
lastWeek = KB DOCUMENTS ADDED SINCE 7
lastMonth = KB DOCUMENTS ADDED SINCE 30

TALK "Document Activity Report"
TALK "Last 24 hours: " + lastDay + " documents"
TALK "Last 7 days: " + lastWeek + " documents"
TALK "Last 30 days: " + lastMonth + " documents"

REM Calculate daily average
IF lastWeek > 0 THEN
    avgDaily = lastWeek / 7
    TALK "Daily average: " + FORMAT(avgDaily, "#,##0.0")
END IF
```

## KB LIST COLLECTIONS

Returns an array of all collection names belonging to the current bot.

### Syntax

```basic
collections = KB LIST COLLECTIONS
```

### Return Value

Array of collection name strings.

### Example

```basic
collections = KB LIST COLLECTIONS

IF LEN(collections) = 0 THEN
    TALK "No collections found"
ELSE
    TALK "Your collections:"
    FOR EACH name IN collections
        TALK "  - " + name
    NEXT
END IF
```

## KB STORAGE SIZE

Returns the total disk storage used by all collections in megabytes.

### Syntax

```basic
sizeMB = KB STORAGE SIZE
```

### Return Value

Float representing storage size in MB.

### Example

```basic
storageMB = KB STORAGE SIZE

TALK "Storage used: " + FORMAT(storageMB, "#,##0.00") + " MB"

REM Alert if storage is high
IF storageMB > 1000 THEN
    TALK "Warning: Knowledge base exceeds 1 GB. Consider archiving old documents."
END IF
```

## Complete Example: KB Dashboard

```basic
REM Knowledge Base Dashboard
REM Displays comprehensive statistics

DESCRIPTION "View knowledge base statistics and health"

TALK "📊 **Knowledge Base Dashboard**"
TALK ""

REM Get overall statistics
stats = KB STATISTICS
statsObj = JSON PARSE stats

REM Summary section
TALK "**Summary**"
TALK "Collections: " + statsObj.total_collections
TALK "Documents: " + FORMAT(statsObj.total_documents, "#,##0")
TALK "Vectors: " + FORMAT(statsObj.total_vectors, "#,##0")
TALK ""

REM Storage section
TALK "**Storage**"
TALK "Disk: " + FORMAT(statsObj.total_disk_size_mb, "#,##0.00") + " MB"
TALK "RAM: " + FORMAT(statsObj.total_ram_size_mb, "#,##0.00") + " MB"
TALK ""

REM Activity section
TALK "**Recent Activity**"
TALK "Last 7 days: " + FORMAT(statsObj.documents_added_last_week, "#,##0") + " documents"
TALK "Last 30 days: " + FORMAT(statsObj.documents_added_last_month, "#,##0") + " documents"

REM Calculate growth rate
IF statsObj.documents_added_last_month > 0 THEN
    growthRate = (statsObj.documents_added_last_week / (statsObj.documents_added_last_month / 4)) * 100 - 100
    IF growthRate > 0 THEN
        TALK "Growth trend: +" + FORMAT(growthRate, "#,##0") + "% vs average"
    ELSE
        TALK "Growth trend: " + FORMAT(growthRate, "#,##0") + "% vs average"
    END IF
END IF

REM Health check
TALK ""
TALK "**Health Status**"
allHealthy = true
FOR EACH coll IN statsObj.collections
    IF coll.status <> "green" THEN
        TALK "⚠️ " + coll.name + ": " + coll.status
        allHealthy = false
    END IF
NEXT

IF allHealthy THEN
    TALK "✅ All collections healthy"
END IF

REM Store for dashboard
SET BOT MEMORY "kb_last_check", NOW()
SET BOT MEMORY "kb_total_docs", statsObj.total_documents
SET BOT MEMORY "kb_storage_mb", statsObj.total_disk_size_mb
```

## Use Cases

### 1. Admin Monitoring Bot

```basic
REM Daily KB health check
SET SCHEDULE "kb-health" TO "0 8 * * *"
    stats = KB STATISTICS
    statsObj = JSON PARSE stats
    
    IF statsObj.total_disk_size_mb > 5000 THEN
        SEND MAIL "admin@example.com", "KB Storage Alert", 
            "Knowledge base storage exceeds 5 GB: " + statsObj.total_disk_size_mb + " MB"
    END IF
END SCHEDULE
```

### 2. User-Facing Statistics

```basic
REM Show user their document count
docCount = KB DOCUMENTS COUNT
TALK "Your bot has learned from " + docCount + " documents"
TALK "Ask me anything about your content!"
```

### 3. Compliance Reporting

```basic
REM Monthly compliance report
lastMonth = KB DOCUMENTS ADDED SINCE 30
storageSize = KB STORAGE SIZE

report = "Monthly KB Report\n"
report = report + "Documents added: " + lastMonth + "\n"
report = report + "Total storage: " + FORMAT(storageSize, "#,##0.00") + " MB\n"

SEND MAIL "compliance@example.com", "Monthly KB Report", report
```

## Notes

- Statistics are fetched in real-time from Qdrant
- Large collections may have slight delays in statistics updates
- Document counts from the database may differ slightly from vector counts if indexing is in progress
- Collection names follow the pattern `kb_{bot_id}_{collection_name}`

## See Also

- [USE KB](./keyword-use-kb.md) - Load knowledge base for queries
- [CLEAR KB](./keyword-clear-kb.md) - Clear knowledge base
- [Vector Collections](../chapter-03/vector-collections.md) - Understanding collections