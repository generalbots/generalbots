# Document Indexing

When a document is added to a knowledge‑base collection with `USE_KB` or `ADD_WEBSITE`, the system performs several steps to make it searchable:

1. **Content Extraction** – Files are read and plain‑text is extracted (PDF, DOCX, HTML, etc.).
2. **Chunking** – The text is split into 500‑token chunks to keep embeddings manageable.
3. **Embedding Generation** – Each chunk is sent to the configured LLM embedding model (default **BGE‑small‑en‑v1.5**) to produce a dense vector.
4. **Storage** – Vectors, along with metadata (source file, chunk offset), are stored in VectorDB under the collection’s namespace.
5. **Indexing** – VectorDB builds an IVF‑PQ index for fast approximate nearest‑neighbor search.

## Index Refresh

If a document is updated, the system re‑processes the file and replaces the old vectors. The index is automatically refreshed; no manual action is required.

## Example

```basic
USE_KB "company-policies"
ADD_WEBSITE "https://example.com/policies"
```

After execution, the `company-policies` collection contains indexed vectors ready for semantic search via the `FIND` keyword.
