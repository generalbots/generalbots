# Semantic Search

Semantic search finds relevant content by meaning, not just keywords. When a user asks "How many days off do I get?", the system matches documents about "vacation policy" or "PTO allowance" even though the words differ.

<img src="../assets/chapter-03/search-pipeline.svg" alt="Search Pipeline" style="max-height: 400px; width: 100%; object-fit: contain;">

## How It Works

1. **Query embedding** - Convert question to vector using same model as documents
2. **Similarity search** - Find document chunks with closest embeddings (cosine distance)
3. **Result selection** - Take top-k results above relevance threshold
4. **Context injection** - Add retrieved text to LLM prompt

## Automatic Integration

Semantic search requires no explicit coding. Just activate knowledge bases:

```basic
USE KB "policies"
USE KB "products"

' Now all user questions automatically search both collections
TALK "How can I help you?"
```

The system handles query embedding, vector search, ranking, and context assembly transparently.

## Search Pipeline Details

| Stage | Operation | Default |
|-------|-----------|---------|
| Embedding | Convert query to vector | BGE model |
| Search | Vector similarity lookup | Qdrant |
| Distance | Cosine similarity | 0.0-1.0 |
| Top-k | Results returned | 5 |
| Threshold | Minimum relevance | 0.7 |

## Multiple Collections

When multiple KBs are active, the system searches all and combines best results:

```basic
USE KB "hr-docs"      ' Active
USE KB "it-docs"      ' Active
USE KB "finance"      ' Active

' Query searches all three, returns best matches regardless of source
```

Use `CLEAR KB` to deactivate collections when switching topics.

## Performance

- **Cold search**: 100-200ms (first query)
- **Warm search**: 20-50ms (cached embeddings)
- **Indexing**: One-time cost per document

Optimizations:
- Embedding cache for repeated queries
- HNSW index for fast vector search
- Only active collections consume resources

## Optimizing Quality

**Document factors:**
- Clear, descriptive text produces better matches
- Use vocabulary similar to how users ask questions
- Avoid jargon-heavy content when possible

**Collection factors:**
- Focused collections (one topic) beat catch-all collections
- Fewer active collections = less noise in results
- Split large document sets by domain area

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| No results | Collection not active | Call `USE KB "name"` |
| Wrong results | Too many collections | Clear irrelevant KBs |
| Missing matches | Document not indexed | Check file is in `.gbkb` folder |
| Poor relevance | Content mismatch | Review document quality |

## Configuration

Semantic search uses sensible defaults. Two settings affect context:

```csv
name,value
episodic-memory-history,2      # Previous exchanges to include
episodic-memory-threshold,4      # When to compress older context
```

## See Also

- [Hybrid Search](../chapter-11-features/hybrid-search.md) - Combining semantic + keyword search
- [Document Indexing](./indexing.md) - How documents are processed
- [Vector Collections](./vector-collections.md) - Technical vector DB details
- [USE KB](../chapter-06-gbdialog/keyword-use-kb.md) - Keyword reference