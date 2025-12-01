# Hybrid RAG Search

Hybrid search combines dense (semantic) and sparse (keyword) retrieval for better search quality than either method alone.

## Overview

| Method | Strengths | Weaknesses |
|--------|-----------|------------|
| **Dense (Semantic)** | Synonyms, meaning, paraphrasing | Rare terms, exact matches |
| **Sparse (BM25)** | Exact terms, product codes, names | No semantic understanding |
| **Hybrid** | Best of both | Slightly more computation |

## How It Works

```
User Query
    │
    ├──────────────────┐
    ▼                  ▼
Dense Search      Sparse Search
(Weight: 0.7)     (Weight: 0.3)
    │                  │
    └────────┬─────────┘
             ▼
    Reciprocal Rank Fusion
             │
             ▼
    Optional Reranking
             │
             ▼
       Final Results
```

**Reciprocal Rank Fusion (RRF):**
```
RRF_score(d) = Σ 1 / (k + rank_i(d))
```

## Configuration

In `config.csv`:

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-top-k,10
rag-rrf-k,60
rag-reranker-enabled,false
```

## Weight Tuning

| Content Type | Dense | Sparse | Use Case |
|--------------|-------|--------|----------|
| **Balanced** | 0.7 | 0.3 | General purpose |
| **Semantic-Heavy** | 0.9 | 0.1 | Conversational, multilingual |
| **Keyword-Heavy** | 0.4 | 0.6 | Technical docs, product catalogs |
| **Equal** | 0.5 | 0.5 | When unsure |

## Reranking

Optional LLM-based reranking for highest quality:

```csv
name,value
rag-reranker-enabled,true
rag-reranker-model,quality
rag-reranker-top-n,20
```

| Aspect | Without | With Reranking |
|--------|---------|----------------|
| Latency | ~50ms | ~500ms |
| Quality | Good | Excellent |
| Cost | None | LLM API cost |

**Use for:** Legal, medical, financial, compliance-critical queries.

## Usage

Hybrid search is automatic when enabled. No code changes needed:

```basic
USE KB "company-policies"
' Queries automatically use hybrid search
```

## Performance

| Metric | Target |
|--------|--------|
| MRR (Mean Reciprocal Rank) | > 0.7 |
| Recall@10 | > 0.9 |
| Latency P95 | < 200ms |
| Cache Hit Rate | > 40% |

### Caching

```csv
name,value
rag-cache-enabled,true
rag-cache-ttl,3600
rag-cache-max-size,10000
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Poor results | Adjust weights for content type |
| High latency | Reduce `rag-top-k`, enable caching, disable reranking |
| Missing expected results | Check document indexed, verify no filters excluding it |

## See Also

- [Semantic Search](../chapter-03/semantic-search.md) - Dense search details
- [Document Indexing](../chapter-03/indexing.md) - How documents are processed
- [Knowledge Base](./knowledge-base.md) - KB overview