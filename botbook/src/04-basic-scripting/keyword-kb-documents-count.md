# KB DOCUMENTS COUNT

The `KB DOCUMENTS COUNT` keyword returns the total number of documents stored in the bot's knowledge base.

---

## Syntax

```basic
count = KB DOCUMENTS COUNT
```

---

## Parameters

None. Returns the count for the current bot's knowledge base.

---

## Description

`KB DOCUMENTS COUNT` queries the database to return the total number of documents that have been indexed in the bot's knowledge base. This is a lightweight operation compared to `KB STATISTICS` when you only need the document count.

Use cases include:
- Checking if knowledge base has content
- Displaying document counts in conversations
- Conditional logic based on KB size
- Simple monitoring and logging

---

## Return Value

Returns an integer representing the total document count. Returns `0` if no documents exist or if an error occurs.

---

## Examples

### Basic Count Check

```basic
' Check how many documents are in KB
doc_count = KB DOCUMENTS COUNT

TALK "The knowledge base contains " + doc_count + " documents."
```

### Conditional KB Usage

```basic
' Only use KB if it has content
doc_count = KB DOCUMENTS COUNT

IF doc_count > 0 THEN
    USE KB
    answer = SEARCH user_question
    TALK answer
ELSE
    TALK "The knowledge base is empty. Please add some documents first."
END IF
```

### Admin Status Report

```basic
' Quick status check for administrators
doc_count = KB DOCUMENTS COUNT

IF doc_count = 0 THEN
    status = "⚠️ Empty - No documents indexed"
ELSE IF doc_count < 10 THEN
    status = "📄 Minimal - " + doc_count + " documents"
ELSE IF doc_count < 100 THEN
    status = "📚 Growing - " + doc_count + " documents"
ELSE
    status = "✅ Robust - " + doc_count + " documents"
END IF

TALK "Knowledge Base Status: " + status
```

### Monitoring Growth

```basic
' Log document count for tracking
doc_count = KB DOCUMENTS COUNT
timestamp = FORMAT(NOW(), "YYYY-MM-DD HH:mm")

PRINT "[" + timestamp + "] KB document count: " + doc_count

' Store for trending
INSERT "kb_count_log", #{
    "timestamp": NOW(),
    "count": doc_count
}
```

### Before/After Import Check

```basic
' Check count before and after document import
before_count = KB DOCUMENTS COUNT

' Import new documents
IMPORT "new-documents.zip"

after_count = KB DOCUMENTS COUNT
added = after_count - before_count

TALK "Import complete! Added " + added + " new documents."
TALK "Total documents now: " + after_count
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

count = KB DOCUMENTS COUNT

IF ERROR THEN
    PRINT "Error getting document count: " + ERROR_MESSAGE
    count = 0
END IF

IF count > 0 THEN
    TALK "Found " + count + " documents in the knowledge base."
ELSE
    TALK "No documents found or unable to query knowledge base."
END IF
```

---

## Related Keywords

- [KB STATISTICS](keyword-kb-statistics.md) — Get comprehensive KB statistics
- [KB DOCUMENTS ADDED SINCE](keyword-kb-documents-added-since.md) — Count recently added documents
- [KB STORAGE SIZE](keyword-kb-storage-size.md) — Get storage usage
- [KB LIST COLLECTIONS](keyword-kb-list-collections.md) — List all collections
- [CLEAR KB](keyword-clear-kb.md) — Clear knowledge base
- [USE KB](keyword-use-kb.md) — Enable KB for queries

---

## Implementation Notes

- Implemented in Rust under `src/basic/keywords/kb_statistics.rs`
- Queries PostgreSQL `kb_documents` table
- Filters by current bot ID
- Returns 0 on error (does not throw)
- Very fast operation (single COUNT query)

---

## Summary

`KB DOCUMENTS COUNT` provides a quick way to get the total number of documents in the knowledge base. Use it for simple checks, conditional logic, and lightweight monitoring. For more detailed statistics, use `KB STATISTICS` instead.