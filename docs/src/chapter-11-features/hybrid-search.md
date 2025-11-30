# Hybrid RAG Search

General Bots implements a hybrid search system that combines dense (semantic) and sparse (keyword) retrieval methods for improved search quality. This approach, known as RAG 2.0, provides better recall and precision than either method alone.

## Overview

Traditional RAG systems use either:
- **Dense retrieval** - Semantic similarity via vector embeddings
- **Sparse retrieval** - Keyword matching (BM25, TF-IDF)

Hybrid search combines both approaches using Reciprocal Rank Fusion (RRF), getting the best of both worlds:

- **Dense** excels at understanding meaning and synonyms
- **Sparse** excels at exact matches and rare terms

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     User Query                               │
│                 "customer refund policy"                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
          ┌───────────┴───────────┐
          ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  Dense Search   │     │  Sparse Search  │
│   (Semantic)    │     │     (BM25)      │
│                 │     │                 │
│ Weight: 0.7     │     │ Weight: 0.3     │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │   Results + Scores    │
         └───────────┬───────────┘
                     ▼
         ┌───────────────────┐
         │ Reciprocal Rank   │
         │     Fusion        │
         │                   │
         │ RRF(d) = Σ 1/(k+r)│
         └─────────┬─────────┘
                   ▼
         ┌───────────────────┐
         │   Optional LLM    │
         │    Reranking      │
         └─────────┬─────────┘
                   ▼
         ┌───────────────────┐
         │   Final Results   │
         └───────────────────┘
```

## Configuration

### Enable Hybrid Search

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
rag-reranker-model,quality
rag-top-k,10
rag-rrf-k,60
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `rag-hybrid-enabled` | `true` | Enable hybrid search |
| `rag-dense-weight` | `0.7` | Weight for semantic results (0-1) |
| `rag-sparse-weight` | `0.3` | Weight for keyword results (0-1) |
| `rag-reranker-enabled` | `false` | Enable LLM reranking |
| `rag-reranker-model` | `quality` | Model for reranking |
| `rag-top-k` | `10` | Number of results to return |
| `rag-rrf-k` | `60` | RRF smoothing constant |

## Usage in BASIC

Hybrid search is automatic when enabled. No code changes needed.

### Basic Search

```basic
' Load knowledge base
USE KB "company-policies"

' Search uses hybrid method automatically
result = FIND "refund policy for damaged items"
TALK result
```

### Search with Context

```basic
' Multiple KBs for comprehensive search
USE KB "product-docs"
USE KB "support-articles"
USE KB "faq"

' Query searches all KBs with hybrid method
answer = FIND "how to reset password"
TALK answer
```

### Contextual Search

```basic
' Set context to improve search relevance
SET CONTEXT "department" AS "billing"
SET CONTEXT "customer_tier" AS "premium"

' Search considers context
result = FIND "payment options"
' Results prioritized for billing + premium context
```

## Reciprocal Rank Fusion (RRF)

RRF combines rankings from multiple retrieval methods:

```
RRF_score(d) = Σ 1 / (k + rank_i(d))
```

Where:
- `d` = document
- `k` = smoothing constant (default: 60)
- `rank_i(d)` = rank of document in result list i

### Why RRF?

- **Rank-based** - Works regardless of score scales
- **Robust** - Handles missing documents gracefully
- **Simple** - No training required
- **Effective** - Proven in information retrieval research

### Example

Document appears at:
- Dense search: rank 2
- Sparse search: rank 5

RRF score = 1/(60+2) + 1/(60+5) = 0.0161 + 0.0154 = 0.0315

## Dense Search (Semantic)

Uses vector embeddings to find semantically similar content.

### Strengths

- Understands synonyms ("car" matches "automobile")
- Captures semantic meaning
- Handles paraphrasing
- Works across languages (with multilingual models)

### Configuration

```csv
name,value
embedding-model,all-MiniLM-L6-v2
embedding-dimension,384
vector-db,qdrant
vector-similarity,cosine
```

### When Dense Excels

- "What's your return policy?" matches "refund guidelines"
- "How do I contact support?" matches "reach customer service"
- Conceptual queries without exact keywords

## Sparse Search (BM25)

Uses keyword matching with term frequency weighting.

### Strengths

- Exact term matching
- Handles rare/unique terms
- Fast and efficient
- No embedding computation needed

### BM25 Formula

```
score(D,Q) = Σ IDF(qi) · (f(qi,D) · (k1 + 1)) / (f(qi,D) + k1 · (1 - b + b · |D|/avgdl))
```

Where:
- `IDF(qi)` = Inverse document frequency of term
- `f(qi,D)` = Term frequency in document
- `k1`, `b` = Tuning parameters
- `|D|` = Document length
- `avgdl` = Average document length

### Configuration

```csv
name,value
bm25-k1,1.2
bm25-b,0.75
bm25-stemming,true
bm25-stopwords,true
```

### When BM25 Excels

- Product codes: "SKU-12345"
- Technical terms: "NullPointerException"
- Proper nouns: "John Smith"
- Exact phrases: "Terms of Service"

## Reranking

Optional LLM-based reranking for highest quality results.

### How It Works

1. Hybrid search returns top-N candidates
2. LLM scores each candidate for query relevance
3. Results reordered by LLM scores

### Enable Reranking

```csv
name,value
rag-reranker-enabled,true
rag-reranker-model,quality
rag-reranker-top-n,20
```

### Trade-offs

| Aspect | Without Reranking | With Reranking |
|--------|-------------------|----------------|
| Latency | ~50ms | ~500ms |
| Quality | Good | Excellent |
| Cost | None | LLM API cost |
| Use case | Real-time chat | Quality-critical |

### When to Use Reranking

- Legal/compliance queries
- Medical information
- Financial advice
- Any high-stakes answers

## Tuning Weights

### Default (Balanced)

```csv
rag-dense-weight,0.7
rag-sparse-weight,0.3
```

Good for general-purpose search.

### Semantic-Heavy

```csv
rag-dense-weight,0.9
rag-sparse-weight,0.1
```

Best for:
- Conversational queries
- Concept-based search
- Multilingual content

### Keyword-Heavy

```csv
rag-dense-weight,0.4
rag-sparse-weight,0.6
```

Best for:
- Technical documentation
- Code search
- Product catalogs

### Equal Weight

```csv
rag-dense-weight,0.5
rag-sparse-weight,0.5
```

When you're unsure which method works better.

## Performance Optimization

### Index Configuration

```csv
name,value
vector-index-type,hnsw
vector-ef-construct,200
vector-m,16
bm25-index-shards,4
```

### Caching

```csv
name,value
rag-cache-enabled,true
rag-cache-ttl,3600
rag-cache-max-size,10000
```

### Batch Processing

```basic
' Process multiple queries efficiently
queries = ["policy question", "pricing info", "support contact"]

FOR EACH query IN queries
    result = FIND query
    SAVE "results.csv", query, result
NEXT query
```

## Monitoring

### Search Quality Metrics

| Metric | Description | Target |
|--------|-------------|--------|
| MRR | Mean Reciprocal Rank | > 0.7 |
| Recall@10 | Relevant docs in top 10 | > 0.9 |
| Latency P95 | 95th percentile latency | < 200ms |
| Cache Hit Rate | Queries served from cache | > 40% |

### Logging

```csv
name,value
rag-logging-enabled,true
rag-log-queries,true
rag-log-scores,true
```

### Debug Mode

```basic
' Enable debug output for search
PRINT "Searching: " + query
result = FIND query
PRINT "Dense score: " + result.dense_score
PRINT "Sparse score: " + result.sparse_score
PRINT "Final score: " + result.rrf_score
```

## Comparison with Pure Methods

| Aspect | Dense Only | Sparse Only | Hybrid |
|--------|------------|-------------|--------|
| Semantic understanding | ✅ Excellent | ❌ Poor | ✅ Excellent |
| Exact matching | ❌ Poor | ✅ Excellent | ✅ Excellent |
| Rare terms | ❌ Poor | ✅ Excellent | ✅ Good |
| Synonyms | ✅ Excellent | ❌ Poor | ✅ Excellent |
| Latency | Medium | Fast | Medium |
| Overall quality | Good | Good | Best |

## Troubleshooting

### Poor Search Results

1. Check weights match your content type
2. Verify embeddings are generated correctly
3. Test with both pure dense and pure sparse
4. Enable reranking for critical queries

### High Latency

1. Reduce `rag-top-k` value
2. Enable caching
3. Use faster embedding model
4. Consider disabling reranking

### Missing Expected Results

1. Check document is indexed
2. Verify no filters excluding it
3. Test with exact keyword match
4. Check chunk size isn't too large

## Best Practices

1. **Start with defaults** - 0.7/0.3 works well for most cases
2. **Monitor and tune** - Use metrics to guide weight adjustments
3. **Use reranking selectively** - Only for quality-critical paths
4. **Cache aggressively** - Many queries repeat
5. **Test both methods** - Understand where each excels
6. **Keep chunks reasonable** - 500-1000 tokens optimal
7. **Update indices regularly** - Fresh content needs reindexing

## See Also

- [Knowledge Base](./knowledge-base.md) - KB setup and management
- [Vector Collections](../chapter-03/vector-collections.md) - Vector DB details
- [Semantic Search](../chapter-03/semantic-search.md) - Dense search deep dive
- [Document Indexing](../chapter-03/indexing.md) - How documents are indexed
- [LLM Configuration](../chapter-08-config/llm-config.md) - Reranker model setup