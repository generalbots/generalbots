# KB LIST COLLECTIONS

The `KB LIST COLLECTIONS` keyword returns a list of all knowledge base collection names associated with the current bot.

---

## Syntax

```basic
collections = KB LIST COLLECTIONS
```

---

## Parameters

None. Returns collections for the current bot.

---

## Description

`KB LIST COLLECTIONS` queries Qdrant to retrieve all collection names that belong to the current bot. Collections are filtered by the bot ID prefix (`kb_{bot_id}`), returning only collections owned by the calling bot.

Use cases include:
- Discovering available knowledge domains
- Building dynamic collection selection interfaces
- Admin dashboards and monitoring
- Iterating over collections for batch operations
- Validating collection existence before operations

---

## Return Value

Returns an array of collection name strings. Returns an empty array if no collections exist.

Example return value:
```basic
["kb_products", "kb_faqs", "kb_policies", "kb_support"]
```

---

## Examples

### Basic Collection Listing

```basic
' List all KB collections
collections = KB LIST COLLECTIONS

TALK "Available knowledge bases:"
FOR EACH coll IN collections
    TALK "  - " + coll
END FOR
```

### Check Collection Existence

```basic
' Verify a collection exists before using it
collections = KB LIST COLLECTIONS
target_collection = "kb_products"

found = false
FOR EACH coll IN collections
    IF coll = target_collection THEN
        found = true
        EXIT FOR
    END IF
END FOR

IF found THEN
    TALK "Products knowledge base is available"
    USE KB target_collection
ELSE
    TALK "Products knowledge base not found"
END IF
```

### Admin Collection Overview

```basic
' Generate overview of all collections
collections = KB LIST COLLECTIONS

IF LEN(collections) = 0 THEN
    TALK "No knowledge base collections found."
ELSE
    TALK "Found " + LEN(collections) + " collections:"
    
    FOR EACH coll IN collections
        stats_json = KB COLLECTION STATS coll
        stats = PARSE_JSON(stats_json)
        
        disk_mb = stats.disk_data_size / 1024 / 1024
        TALK "  " + coll + ": " + stats.points_count + " docs (" + FORMAT(disk_mb, "#,##0.00") + " MB)"
    END FOR
END IF
```

### Dynamic Collection Selection

```basic
' Let user choose a knowledge base
collections = KB LIST COLLECTIONS

TALK "Which knowledge base would you like to search?"
TALK "Available options:"

idx = 1
FOR EACH coll IN collections
    ' Remove kb_ prefix for display
    display_name = REPLACE(coll, "kb_", "")
    TALK idx + ". " + display_name
    idx = idx + 1
END FOR

HEAR choice AS NUMBER

IF choice > 0 AND choice <= LEN(collections) THEN
    selected = collections[choice - 1]
    USE KB selected
    TALK "Now searching in: " + selected
ELSE
    TALK "Invalid selection"
END IF
```

### Batch Operations on All Collections

```basic
' Get stats for all collections
collections = KB LIST COLLECTIONS

total_docs = 0
total_size = 0

FOR EACH coll IN collections
    stats_json = KB COLLECTION STATS coll
    stats = PARSE_JSON(stats_json)
    
    total_docs = total_docs + stats.points_count
    total_size = total_size + stats.disk_data_size
END FOR

TALK "Across " + LEN(collections) + " collections:"
TALK "  Total documents: " + FORMAT(total_docs, "#,##0")
TALK "  Total size: " + FORMAT(total_size / 1024 / 1024, "#,##0.00") + " MB"
```

### Collection Health Check

```basic
' Check health of all collections
collections = KB LIST COLLECTIONS
issues = []

FOR EACH coll IN collections
    stats_json = KB COLLECTION STATS coll
    stats = PARSE_JSON(stats_json)
    
    IF stats.status <> "green" THEN
        issues = issues + [coll + " (" + stats.status + ")"]
    END IF
END FOR

IF LEN(issues) > 0 THEN
    TALK "Collections with issues:"
    FOR EACH issue IN issues
        TALK "  ⚠️ " + issue
    END FOR
ELSE
    TALK "✅ All " + LEN(collections) + " collections are healthy"
END IF
```

### Collection-Based Routing

```basic
' Route query to appropriate collection based on topic
collections = KB LIST COLLECTIONS

' Determine best collection for user's question
topic = LLM "Classify this question into one category: products, support, policies, or general. Question: " + user_question
topic = TRIM(LOWER(topic))

target = "kb_" + topic

' Check if collection exists
collection_found = false
FOR EACH coll IN collections
    IF coll = target THEN
        collection_found = true
        EXIT FOR
    END IF
END FOR

IF collection_found THEN
    USE KB target
    answer = SEARCH user_question
ELSE
    ' Fall back to searching all collections
    USE KB
    answer = SEARCH user_question
END IF

TALK answer
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

collections = KB LIST COLLECTIONS

IF ERROR THEN
    PRINT "Failed to list collections: " + ERROR_MESSAGE
    collections = []
END IF

IF LEN(collections) = 0 THEN
    TALK "No knowledge base collections available"
ELSE
    TALK "Found " + LEN(collections) + " knowledge base collections"
END IF
```

---

## Related Keywords

- [KB STATISTICS](keyword-kb-statistics.md) — Comprehensive KB statistics
- [KB COLLECTION STATS](keyword-kb-collection-stats.md) — Stats for specific collection
- [KB DOCUMENTS COUNT](keyword-kb-documents-count.md) — Total document count
- [KB STORAGE SIZE](keyword-kb-storage-size.md) — Storage usage in MB
- [USE KB](keyword-use-kb.md) — Enable KB for queries
- [CLEAR KB](keyword-clear-kb.md) — Clear knowledge base content

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries Qdrant REST API at `/collections`
- Filters results by bot ID prefix (`kb_{bot_id}`)
- Returns array of Dynamic strings for easy iteration
- Empty array returned if no collections or on error
- Collection names include the full prefix (e.g., `kb_products`)

---

## Summary

`KB LIST COLLECTIONS` provides a way to discover all knowledge base collections belonging to the current bot. Use it for dynamic collection selection, admin dashboards, batch operations, or validating collection existence before performing operations. Combine with `KB COLLECTION STATS` to get detailed information about each collection.