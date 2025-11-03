# ADD_KB Keyword

The **ADD_KB** keyword creates or registers a new knowledge base collection in the GeneralBots system.  
It is used to expand the bot’s accessible data sources by adding new document collections for semantic search.

---

## Syntax

```basic
ADD_KB "collection-name"
```

---

## Parameters

- `"collection-name"` — The name of the new knowledge base collection.  
  This identifier is used to reference the collection in subsequent commands such as `SET_KB` or `FIND`.

---

## Description

When executed, `ADD_KB` registers a new vector collection in the bot’s knowledge base.  
Internally, the system creates a logical entry for the collection and prepares it for document indexing.  
If the collection already exists, the command ensures it is properly linked to the current session context.

The collection is stored in the configured VectorDB (e.g., Qdrant or other supported database) and can later be populated with documents using commands like `ADD_WEBSITE` or `ADD_FILE`.

---

## Example

```basic
' Create a new knowledge base for company policies
ADD_KB "company-policies"

' Set it as the active collection
SET_KB "company-policies"

' Add documents from a website
ADD_WEBSITE "https://example.com/policies"
```

---

## Implementation Notes

- The keyword is implemented in Rust under `src/kb/minio_handler.rs` and `src/kb/qdrant_client.rs`.  
- It interacts with the bot’s context manager to register the collection name.  
- The collection metadata is stored in the bot’s internal registry and synchronized with the VectorDB backend.  
- If the VectorDB connection fails, the command logs an error and continues without blocking the session.

---

## Related Keywords

- [`SET_KB`](keyword-set-kb.md) — Selects the active knowledge base.  
- [`ADD_WEBSITE`](keyword-add-website.md) — Adds documents to a collection.  
- [`FIND`](keyword-find.md) — Searches within the active collection.

---

## Summary

`ADD_KB` is the foundational command for creating new knowledge bases in GeneralBots.  
It enables dynamic expansion of the bot’s knowledge domain and supports semantic search across multiple collections.
