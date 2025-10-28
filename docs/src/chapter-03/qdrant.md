# VectorDB Integration

GeneralBots uses **VectorDB** as the vector database for storing and searching embeddings. The Rust client for the configured VectorDB is used to communicate with the service.

## Configuration

The connection is configured via environment variables:

```env
VECTORDB_URL=http://localhost:6333
VECTORDB_API_KEY=your-api-key   # optional
```

These values are read at startup and passed to the `VectorDBClient`.

## Collection Mapping

Each `.gbkb` collection maps to a VectorDB collection with the same name. For example, a knowledge base named `company-policies` becomes a VectorDB collection `company-policies`.

## Operations

- **Insert** – Performed during indexing (see Chapter 03).
- **Search** – Executed by the `FIND` keyword, which sends a query vector and retrieves the top‑k nearest neighbors.
- **Delete/Update** – When a document is removed or re‑indexed, the corresponding vectors are deleted and replaced.

## Performance Tips

- Keep the number of vectors per collection reasonable (tens of thousands) for optimal latency.
- Adjust VectorDB’s `hnsw` parameters in `VectorDBClient::new` if you need higher recall.
- Use the `FILTER` option to restrict searches by metadata (e.g., source file).

## Example `FIND` Usage

```basic
SET_KB "company-policies"
FIND "vacation policy" INTO RESULT
TALK RESULT
```

The keyword internally:
1. Generates an embedding for the query string.
2. Calls VectorDB’s `search` API.
3. Returns the most relevant chunk as `RESULT`.
