# Knowledge Base 🟡 BETA

The Knowledge Base (KB) system enables semantic search and document retrieval for intelligent bot responses.

## Quick Overview

| Feature | Description |
|---------|-------------|
| **Storage** | S3-compatible drive + PostgreSQL metadata + Qdrant vectors |
| **Search** | Dense vector search, optionally fused with keyword matching — chosen per bot by `rag-mode` |
| **Formats** | PDF, DOCX, DOC, XLSX, XLS, ODS, PPTX, PPT, ODP, EPUB, ODT, TXT, MD, HTML, CSV, JSON, YAML, TOML, and 30+ text-based formats |
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
- **Hybrid Search** - Combines dense (semantic) retrieval with term matching, fused by Reciprocal Rank Fusion
- **Context Management** - Relevant chunks injected into LLM prompts

## Document Organization

```
bot.gbkb/
├── policies/      → USE KB "policies"
├── products/      → USE KB "products"
└── support/       → USE KB "support"
```

## Configuration

Retrieval strategy is one per-bot setting:

```csv
name,value
rag-mode,hybrid
```

`rag-mode` accepts `standard`, `hybrid`, `corrective`, `graph`, `agentic` or
`multimodal`, and defaults to `standard`. It is read from the bot's configuration
row (environment fallback `RAG_MODE`), not from `config.csv`. The older
`rag-hybrid-enabled` / `rag-dense-weight` / `rag-sparse-weight` keys are read only
by an unconnected crate and have no effect — see
[Retrieval and RAG](./hybrid-search.md).

## Performance Tips

1. **Organize collections** by topic for precise activation
2. **Clear unused KBs** to free memory: `CLEAR KB "old-docs"`
3. **Enable caching** for repeated queries
4. **Tune weights** based on content type (technical vs conversational)

## Learn More

- **[KB System Architecture](../03-knowledge-ai/README.md)** - Technical deep dive
- **[Retrieval and RAG](../03-knowledge-ai/hybrid-search.md)** - How retrieval works
- **[Document Indexing](../03-knowledge-ai/indexing.md)** - Processing pipeline
- **[Retrieval and RAG](./hybrid-search.md)** - The `rag-mode` setting and the six retrieval modes
- **[USE KB Keyword](../04-basic-scripting/keyword-use-kb.md)** - Complete reference
- **[.gbkb Package](../02-architecture-packages/gbkb.md)** - Folder structure