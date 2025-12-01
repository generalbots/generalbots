# Chapter 03 – Knowledge‑Base (VectorDB) Documentation Overview

This chapter explains how GeneralBots manages knowledge‑base collections, indexing, caching, and semantic search. The implementation uses vector databases for semantic search and highlights the use of the **.gbdrive** package for storage when needed.

| Document | File | Description |
|----------|------|-------------|
| **README** | [README.md](README.md) | High‑level reference for the `.gbkb` package and its core commands (`USE KB`, `CLEAR KB`, `USE WEBSITE`). |
| **Caching** | [caching.md](caching.md) | Optional in‑memory and persistent SQLite caching to speed up frequent `FIND` queries. |
| **Context Compaction** | [context-compaction.md](context-compaction.md) | Techniques to keep the LLM context window within limits (summarization, memory pruning, sliding window). |
| **Indexing** | [indexing.md](indexing.md) | Process of extracting, chunking, embedding, and storing document vectors in the VectorDB. |
| **Semantic Caching** | [caching.md](caching.md) | Intelligent caching for LLM responses, including semantic similarity matching. |
| **Semantic Search** | [semantic-search.md](semantic-search.md) | How the `FIND` keyword performs meaning‑based retrieval using the VectorDB. |
| **Vector Collections** | [vector-collections.md](vector-collections.md) | Definition and management of vector collections, including creation, document addition, and usage in dialogs. |

## How to Use This Overview

- **Navigate**: Click the file links to read the full documentation for each topic.
- **Reference**: Use this table as a quick lookup when developing or extending knowledge‑base functionality.
- **Update**: When the underlying storage or VectorDB implementation changes, edit the corresponding markdown files and keep this summary in sync.

---

*This summary was added to provide a cohesive overview of Chapter 03, aligning terminology with the current architecture (VectorDB, .gbdrive, etc.).*
