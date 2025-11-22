# Semantic Search

Semantic search enables the bot to retrieve information based on meaning rather than exact keyword matches. It leverages the vector embeddings stored in VectorDB.

## How It Works

1. **Query Embedding** – The user’s query string is converted into a dense vector using the same embedding model as the documents.
2. **Nearest‑Neighbor Search** – VectorDB returns the top‑k vectors that are closest to the query vector.
3. **Result Formatting** – The matching document chunks are concatenated and passed to the LLM as context for the final response.

## Using the `FIND` Keyword

```basic
USE_KB "company-policies"
FIND "how many vacation days do I have?" INTO RESULT
TALK RESULT
```

- `USE_KB` adds the collection to the session.
- `FIND` performs the semantic search.
- `RESULT` receives the best matching snippet.

## Parameters

- **k** – Number of results to return (default 3). Can be overridden with `FIND "query" LIMIT 5 INTO RESULT`.
- **filter** – Optional metadata filter, e.g., `FILTER source="policy.pdf"`.

## Best Practices

- Keep the query concise (1‑2 sentences) for optimal embedding quality.
- Use `FORMAT` to clean up the result before sending to the user.
- Combine with `GET_BOT_MEMORY` to store frequently accessed answers.

## Performance

Semantic search latency is typically < 100 ms for collections under 50 k vectors. Larger collections may require tuning VectorDB’s HNSW parameters.
