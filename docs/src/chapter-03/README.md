# Chapter 03: Knowledge Base System

Vector search and semantic retrieval for intelligent document querying.

## Overview

The Knowledge Base (gbkb) transforms documents into searchable semantic representations, enabling natural language queries against your organization's content.

## Architecture

<img src="../assets/chapter-03/kb-architecture-pipeline.svg" alt="KB Architecture Pipeline" style="max-height: 400px; width: 100%; object-fit: contain;">

The pipeline processes documents through extraction, chunking, embedding, and storage to enable semantic search.

## Supported Formats

| Format | Features |
|--------|----------|
| PDF | Text, OCR, tables |
| DOCX | Formatted text, styles |
| HTML | DOM parsing |
| Markdown | GFM, tables, code |
| CSV/JSON | Structured data |
| TXT | Plain text |

## Quick Start

```basic
' Activate knowledge base
USE KB "company-docs"

' Bot now answers from your documents
TALK "How can I help you?"
```

## Key Concepts

### Document Processing
1. **Extract** - Pull text from files
2. **Chunk** - Split into ~500 token segments
3. **Embed** - Generate vectors (BGE model)
4. **Store** - Save to Qdrant

### Semantic Search
- Query converted to vector embedding
- Cosine similarity finds relevant chunks
- Top results injected into LLM context
- No explicit search code needed

### Storage Requirements

Vector databases need ~3.5x original document size:
- Embeddings: ~2x
- Indexes: ~1x
- Metadata: ~0.5x

## Configuration

```csv
name,value
embedding-url,http://localhost:8082
embedding-model,bge-small-en-v1.5
rag-hybrid-enabled,true
rag-top-k,10
```

## Chapter Contents

- [KB and Tools System](./kb-and-tools.md) - Integration patterns
- [Vector Collections](./vector-collections.md) - Collection management
- [Document Indexing](./indexing.md) - Processing pipeline
- [Semantic Search](./semantic-search.md) - Search mechanics
- [Context Compaction](./context-compaction.md) - Token management
- [Semantic Caching](./caching.md) - Performance optimization

## See Also

- [.gbkb Package](../chapter-02/gbkb.md) - Folder structure
- [USE KB Keyword](../chapter-06-gbdialog/keyword-use-kb.md) - Keyword reference
- [Hybrid Search](../chapter-11-features/hybrid-search.md) - RAG 2.0