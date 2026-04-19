# Chapter 03 – Knowledge Base System Overview

This chapter explains how botserver manages knowledge base collections, indexing, caching, semantic search, and conversation memory. The implementation uses vector databases for semantic search and intelligent memory management for context optimization.

| Document | File | Description |
|----------|------|-------------|
| **README** | [README.md](README.md) | High-level reference for the `.gbkb` package and its core commands (`USE KB`, `CLEAR KB`, `USE WEBSITE`). |
| **KB and Tools** | [kb-and-tools.md](kb-and-tools.md) | Integration patterns for knowledge bases and tool systems. |
| **Vector Collections** | [vector-collections.md](vector-collections.md) | Definition and management of vector collections, including creation, document addition, and usage in dialogs. |
| **Document Indexing** | [indexing.md](indexing.md) | Process of extracting, chunking, embedding, and storing document vectors in the VectorDB. |
| **Semantic Search** | [semantic-search.md](semantic-search.md) | How semantic search performs meaning-based retrieval using vector embeddings. |
| **Episodic Memory** | [episodic-memory.md](episodic-memory.md) | Automatic conversation history management, context compaction, and intelligent summarization. |
| **Semantic Caching** | [caching.md](caching.md) | Intelligent caching for LLM responses, including semantic similarity matching. |

## Key Configuration Parameters

### Knowledge Base

| Parameter | Default | Description |
|-----------|---------|-------------|
| `embedding-url` | `http://localhost:8082` | Embedding service endpoint |
| `embedding-model` | `bge-small-en-v1.5` | Model for vector embeddings |
| `rag-hybrid-enabled` | `true` | Enable hybrid search |
| `rag-top-k` | `10` | Number of results to retrieve |

### Episodic Memory

| Parameter | Default | Description |
|-----------|---------|-------------|
| `episodic-memory-enabled` | `true` | Enable/disable episodic memory |
| `episodic-memory-threshold` | `4` | Exchanges before compaction |
| `episodic-memory-history` | `2` | Recent exchanges to keep |
| `episodic-memory-model` | `fast` | Model for summarization |
| `episodic-memory-max-episodes` | `100` | Max episodes per user |
| `episodic-memory-retention-days` | `365` | Days to keep episodes |
| `episodic-memory-auto-summarize` | `true` | Auto-summarize conversations |

### LLM Cache

| Parameter | Default | Description |
|-----------|---------|-------------|
| `llm-cache` | `false` | Enable/disable response caching |
| `llm-cache-ttl` | `3600` | Cache time-to-live in seconds |
| `llm-cache-semantic` | `true` | Use semantic similarity matching |
| `llm-cache-threshold` | `0.95` | Similarity threshold for cache hits |

## How to Use This Overview

- **Navigate**: Click the file links to read the full documentation for each topic.
- **Reference**: Use the parameter tables for quick configuration lookup.
- **Update**: When the underlying implementation changes, edit the corresponding markdown files and keep this summary in sync.

## See Also

- [.gbkb Package](../02-architecture-packages/gbkb.md) - Folder structure for knowledge bases
- [LLM Configuration](../10-configuration-deployment/llm-config.md) - Model and provider settings
- [Hybrid Search](../03-knowledge-ai/hybrid-search.md) - Advanced RAG techniques