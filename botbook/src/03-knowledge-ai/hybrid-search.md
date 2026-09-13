# Retrieval and RAG 🟡 BETA

Retrieval decides what the model knows when it answers. This page is the authoritative description of how General Bots retrieves text today, which retrieval strategies exist, and which ones the field has moved on to that this platform does not implement.

> **Verified September 2026** against `botserver/src/core/bot/kb_context/`. If a claim here disagrees with the code, the code is right — please open an issue.

## Retrieval modes

Retrieval is selected per bot with a single setting, `rag-mode`. Six modes are implemented and all of them are reachable from chat.

| Mode | What it actually does | Cost |
|------|----------------------|------|
| `standard` | Embeds the query, searches Qdrant by cosine similarity, drops results scoring below **0.20** | One embedding call |
| `hybrid` | Runs the dense search and a keyword search, then merges both with **Reciprocal Rank Fusion**, `k = 60`. Falls back to whichever side returned results | One embedding call + one scroll |
| `corrective` | Asks the LLM for 2–3 search variants, searches each, merges and de-duplicates, then has the LLM **grade every chunk 0–10** and keeps those scoring 3 or better. If nothing is found locally it tries a **web crawl** (depth 1, up to 3 pages) | 1 + N embedding calls, one grading call per chunk |
| `graph` | Has the LLM extract up to 5 named entities, searches each entity separately, then merges them with the original query's results. This is **entity-expansion retrieval**, not a knowledge graph — see the limits below | 1 + N embedding calls |
| `agentic` | Has the LLM **decompose** the query into 2–3 focused sub-queries, searches each, then merges and re-ranks by score | 1 + N embedding calls |
| `multimodal` | Expands the query into three variants biased toward visual language, searches each, then boosts chunks containing terms like `diagram`, `chart`, `figure` or `screenshot` | 1 + 3 embedding calls |

Every mode degrades rather than failing: if the LLM is unavailable, the LLM-assisted modes fall back to heuristic splitting (`expand_query`, `decompose_query`) instead of returning nothing.

### Which mode to choose

| Situation | Start with |
|---|---|
| General questions over a clean knowledge base | `standard` |
| Documents full of exact terms — part numbers, product codes, names | `hybrid` |
| Users ask vague or badly-phrased questions | `corrective` |
| Questions naming several things at once | `graph` |
| Complex questions that span several documents | `agentic` |
| Knowledge base with diagrams and screenshots | `multimodal` |

`standard` is the default. Raising the mode raises both latency and token cost — `corrective` in particular makes one LLM call per candidate chunk, so it is the expensive one.

## Configuring the mode

The mode is a per-bot value named `rag-mode`, read through `ConfigManager::get_config`, which resolves keys in this order:

1. the bot's **configuration row in the database** — the source of truth;
2. the environment variable `RAG_MODE` (`rag-mode` upper-cased, `-` → `_`);
3. the default.

| Key | Values | Default |
|---|---|---|
| `rag-mode` | `standard`, `hybrid`, `corrective`, `graph`, `agentic`, `multimodal` | `standard` |

`rag-mode` is **not** part of the LLM secret block, so it is **not** read from Vault, and it is **not** a `config.csv` key — setting it in either place has no effect. An unrecognised value is treated as `standard` rather than failing.

The same setting applies to knowledge bases and to websites registered with `USE WEBSITE`.

> A second, richer configuration surface exists in `botqdrant/src/hybrid_search.rs`
> (`dense_weight`, `sparse_weight`, `reranker_*`, `rrf_k`, `bm25_*`, read from bot
> config). Nothing constructs it, so those keys have no effect on retrieval. Do not
> configure against them — see the note under Limits.

## The pipeline underneath

```
User question
      │
      ▼
Query embedding ──► if no embedding model is configured
      │             └─► keyword search (Qdrant scroll + term matching)
      ▼
Qdrant vector search (cosine)  ──► drop score < 0.20
      │
      ▼
Mode-specific step (fusion, grading, entity merge, decomposition, boost)
      │
      ▼
Top results, up to 10 per source
      │
      ▼
Context assembled into the model prompt
```

### The dense step

Every mode except pure keyword fallback starts here, and `standard` is nothing else:

| Stage | Operation | Value |
|-------|-----------|-------|
| Embedding | Query to vector | Local `all-MiniLM-L6-v2` (384-d), or `text-embedding-3-small` (1536-d) when an API key is set |
| Search | Vector similarity | Qdrant, cosine distance |
| Candidates | Per source | 10 |
| Relevance cut | Score floor | 0.20 — anything below is discarded |

This is the half that finds a document about "vacation policy" when the user asked about "days off". Keyword fusion is what recovers exact terms that dense similarity misses.

### Embeddings and their fallbacks

| Path | Model | Notes |
|---|---|---|
| Local embedding service | `sentence-transformers/all-MiniLM-L6-v2` | Default. Input truncated to 600 tokens |
| OpenAI | `text-embedding-3-small` | Used when an API key is supplied |
| **Hash embedding** | none — deterministic hash | Last resort when embedding generation fails. It is **not semantic**; matching becomes effectively random. If retrieval results look nonsensical, this is the first thing to check |

A knowledge base only searches semantically when an embedding model is configured. Without one, retrieval silently becomes keyword matching — worth knowing before blaming the corpus. Both fallbacks are logged; if results look wrong rather than thin, check the logs before assuming the documents are at fault.

### Ingestion

Files are indexed from Drive into Qdrant collections with cosine distance, carrying `bucket`, `file_path`, `file_name`, `file_type` and `tags` as payload filters. Email is indexed separately when the `mail` feature is compiled in.

## Limits — what these modes are not

Being precise here matters more than looking advanced:

| Technique | Status |
|---|---|
| Real graph RAG (a graph index over entities and relations, with traversal) | **Not implemented.** `graph` mode is entity-expansion over the same vector index; there is no graph store |
| Cross-encoder or LLM **re-ranking model** | **Not implemented** in the retrieval path. `corrective` grades chunks with an LLM, which is a relevance filter, not a re-ranker |
| Late-interaction retrieval (ColBERT-style multi-vector) | **Not implemented** |
| Learned fusion weights | **Not implemented.** Fusion is fixed-weight RRF at `k = 60` |
| Contextual chunking (chunk-aware summaries or surrounding context) | **Not implemented** |
| Semantic chunking of files | **Not implemented.** Files are streamed and indexed as whole records; the 16 MB constant in the ingester is a *read buffer*, not a chunk size |
| Multi-hop retrieval with verification between hops | **Not implemented.** `agentic` decomposes once, then merges — it does not iterate on its own results |
| Retrieval evaluation harness (MRR, Recall@k, golden sets) | **Not implemented.** There is no measured retrieval-quality number to quote |

There is a second, **unconnected** hybrid implementation in `botqdrant/src/hybrid_search.rs` (BM25 index with `k1`/`b` tuning, stemming and stopword options, a re-ranker configuration and an LLM query decomposer). Nothing outside that crate constructs it, so none of it runs. Do not configure against it — if `bm25-*` or `rag-*` keys are set anywhere, they have no effect on the live path.

## Where the field is, and where this is heading

The 2026–2027 direction of retrieval is less about the search call and more about what surrounds it. The honest position for this platform:

- **Retrieval is already agentic in shape.** `agentic`, `corrective` and `graph` modes decompose, grade and merge — they are early forms of what the field now calls agentic RAG.
- **Re-ranking is the cheapest quality win available.** A cross-encoder re-ranker over the top 50 candidates typically beats any query-rewriting trick, and the configuration surface for it already exists in the unwired crate.
- **Graph retrieval needs a graph.** Entity expansion is not a substitute; anything advertised as GraphRAG requires an actual graph index and traversal, which does not exist here yet.
- **Evaluation is the gap that blocks everything else.** Without a golden set and a measured metric, mode selection is guesswork. This is the first thing worth building.
- **Chunking strategy is unexplored.** Whole-file records mean long documents lose precision. Chunking with overlap is standard practice and is not done yet.

## Troubleshooting

| Symptom | Likely cause |
|---|---|
| No results at all | No embedding model configured and the keyword fallback found nothing; or the collection is empty |
| Results unrelated to the question | Falling back to hash embeddings — check whether embedding generation is failing in the logs |
| Exact terms missing from results | Use `hybrid`, which adds keyword matching |
| Answers miss documents that exist | Document was never indexed, or sits in a collection that is not active |
| Slow responses | A mode that makes several LLM calls per question — `corrective` and `agentic`. Try `standard` or `hybrid` first |

## See Also

- [Knowledge Base](./knowledge-base.md) - Working with KB collections
- [Document Indexing](./indexing.md) - How documents get into the index
- [Vector Collections](./vector-collections.md) - Collection structure
- [USE KB](../04-basic-scripting/keyword-use-kb.md) - Keyword reference
