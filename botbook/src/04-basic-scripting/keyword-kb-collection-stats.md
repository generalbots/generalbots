# KB COLLECTION STATS

The `KB COLLECTION STATS` keyword retrieves detailed statistics for a specific knowledge base collection, allowing granular monitoring of individual collections within the bot's KB.

---

## Syntax

```basic
stats = KB COLLECTION STATS "collection_name"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_name` | String | Name of the collection to query |

---

## Description

`KB COLLECTION STATS` queries Qdrant for detailed metrics about a specific collection. This is useful when you need information about a particular knowledge domain rather than the entire KB.

Returns a JSON object containing:
- Collection name
- Vector and point counts
- Storage metrics (disk and RAM)
- Segment information
- Index status
- Collection health status

---

## Return Value

Returns a JSON string with the following structure:

| Property | Type | Description |
|----------|------|-------------|
| `name` | String | Collection name |
| `vectors_count` | Number | Total vectors in collection |
| `points_count` | Number | Total points (documents) |
| `segments_count` | Number | Number of storage segments |
| `disk_data_size` | Number | Disk usage in bytes |
| `ram_data_size` | Number | RAM usage in bytes |
| `indexed_vectors_count` | Number | Vectors that are indexed |
| `status` | String | Collection status (green/yellow/red) |

---

## Examples

### Basic Collection Stats

```basic
' Get stats for a specific collection
stats_json = KB COLLECTION STATS "kb_products"
stats = PARSE_JSON(stats_json)

TALK "Products collection has " + stats.points_count + " documents"
TALK "Storage: " + FORMAT(stats.disk_data_size / 1024 / 1024, "#,##0.00") + " MB"
```

### Compare Multiple Collections

```basic
' Compare stats across collections
collections = ["kb_products", "kb_faqs", "kb_policies"]

TALK "Collection Statistics:"
FOR EACH coll_name IN collections
    stats_json = KB COLLECTION STATS coll_name
    stats = PARSE_JSON(stats_json)
    
    disk_mb = stats.disk_data_size / 1024 / 1024
    TALK "  " + coll_name + ": " + stats.points_count + " docs, " + FORMAT(disk_mb, "#,##0.00") + " MB"
END FOR
```

### Collection Health Monitoring

```basic
' Check if collection is healthy
stats_json = KB COLLECTION STATS collection_name
stats = PARSE_JSON(stats_json)

IF stats.status = "green" THEN
    TALK "Collection " + collection_name + " is healthy"
ELSE IF stats.status = "yellow" THEN
    TALK "Warning: Collection " + collection_name + " needs optimization"
ELSE
    TALK "Error: Collection " + collection_name + " has issues - status: " + stats.status
END IF
```

### Index Coverage Check

```basic
' Verify all vectors are indexed
stats_json = KB COLLECTION STATS "kb_main"
stats = PARSE_JSON(stats_json)

index_coverage = (stats.indexed_vectors_count / stats.vectors_count) * 100

IF index_coverage < 100 THEN
    TALK "Warning: Only " + FORMAT(index_coverage, "#0.0") + "% of vectors are indexed"
    TALK "Search performance may be degraded"
ELSE
    TALK "All vectors are fully indexed"
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

stats_json = KB COLLECTION STATS "kb_" + collection_name

IF ERROR THEN
    IF INSTR(ERROR_MESSAGE, "not found") > 0 THEN
        TALK "Collection '" + collection_name + "' does not exist"
    ELSE
        TALK "Error retrieving collection stats: " + ERROR_MESSAGE
    END IF
ELSE
    stats = PARSE_JSON(stats_json)
    TALK "Collection has " + stats.points_count + " documents"
END IF
```

---

## Related Keywords

- [KB STATISTICS](keyword-kb-statistics.md) — Get overall KB statistics
- [KB LIST COLLECTIONS](keyword-kb-list-collections.md) — List all collections
- [KB DOCUMENTS COUNT](keyword-kb-documents-count.md) — Get total document count
- [KB STORAGE SIZE](keyword-kb-storage-size.md) — Get storage usage

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries Qdrant REST API at `/collections/{name}`
- Collection name should match exactly (case-sensitive)
- Returns empty if collection doesn't exist

---

## Summary

`KB COLLECTION STATS` provides detailed metrics for a specific knowledge base collection. Use it for granular monitoring, comparing collections, or checking health of individual knowledge domains. For overall KB statistics, use `KB STATISTICS` instead.