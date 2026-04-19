# Knowledge Base

The Knowledge Base (KB) system enables semantic search and document retrieval for intelligent bot responses.

## Quick Overview

| Feature | Description |
|---------|-------------|
| **Storage** | S3-compatible drive + PostgreSQL metadata + Qdrant vectors |
| **Search** | Hybrid (semantic + keyword) with optional reranking |
| **Formats** | PDF, DOCX, TXT, MD, HTML, CSV, JSON |
| **Integration** | Automatic context injection into LLM responses |

## Basic Usage

```basic
' Load knowledge base
USE KB "policies"

' Bot now answers questions using that knowledge
' No explicit search needed - it's automatic
```

## Key Capabilities

- **Semantic Search** - Find content by meaning, not just keywords
- **Multi-Collection** - Organize documents into focused collections
- **Auto-Indexing** - Documents indexed automatically when added
- **Hybrid Search** - Combines dense (semantic) and sparse (BM25) retrieval
- **Context Management** - Relevant chunks injected into LLM prompts

## Document Organization

```
bot.gbkb/
├── policies/      → USE KB "policies"
├── products/      → USE KB "products"
└── support/       → USE KB "support"
```

## Configuration

Key settings in `config.csv`:

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-top-k,10
```

## Performance Tips

1. **Organize collections** by topic for precise activation
2. **Clear unused KBs** to free memory: `CLEAR KB "old-docs"`
3. **Enable caching** for repeated queries
4. **Tune weights** based on content type (technical vs conversational)

## Learn More

- **[KB System Architecture](../03-knowledge-ai/README.md)** - Technical deep dive
- **[Semantic Search](../03-knowledge-ai/semantic-search.md)** - How search works
- **[Document Indexing](../03-knowledge-ai/indexing.md)** - Processing pipeline
- **[Hybrid Search](./hybrid-search.md)** - RAG 2.0 configuration
- **[USE KB Keyword](../04-basic-scripting/keyword-use-kb.md)** - Complete reference
- **[.gbkb Package](../02-architecture-packages/gbkb.md)** - Folder structure