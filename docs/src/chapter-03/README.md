## gbkb Reference
The knowledge‑base package provides three main commands:

- **USE KB** – Loads and embeds files from the `.gbkb/collection-name` folder into the vector database, making them available for semantic search in the current session. Multiple KBs can be active simultaneously.
- **CLEAR KB** – Removes a knowledge base from the current session (files remain embedded in the vector database).
- **ADD WEBSITE** – Crawl a website and add its pages to a collection.

**Example:**
```bas
' Add support docs KB - files from work/botname/botname.gbkb/support_docs/ are embedded
USE KB "support_docs"

' Add multiple KBs to the same session
USE KB "policies"
USE KB "procedures"

' Remove a specific KB from session
CLEAR KB "policies"

' Remove all KBs from session
CLEAR KB
```

The vector database retrieves relevant chunks/excerpts from active KBs and makes them available to the system AI automatically, providing context-aware responses during conversations.

## See Also

- [KB and Tools System](./kb-and-tools.md) - Complete reference for knowledge bases and tools
- [Vector Collections](./vector-collections.md) - How vector search works
- [Document Indexing](./indexing.md) - Automatic document processing
- [Semantic Search](./semantic-search.md) - Meaning-based retrieval
- [Context Compaction](./context-compaction.md) - Managing conversation context
- [Caching](./caching.md) - Performance optimization
- [Chapter 2: Packages](../chapter-02/README.md) - Understanding bot components
- [Chapter 5: BASIC Keywords](../chapter-05/README.md) - Complete command reference
- [Chapter 7: Configuration](../chapter-07/config-csv.md) - Bot configuration options
- [Chapter 9: Knowledge Base](../chapter-09/knowledge-base.md) - Advanced KB patterns
