# Vector Collections

A **vector collection** is a set of documents that have been transformed into vector embeddings for fast semantic similarity search. Each collection lives under a `.gbkb` folder and is identified by a unique name.

## Creating a Collection

Use the `USE_KB` keyword in a dialog script:

```basic
USE_KB "company-policies"
```

This creates a new collection named `company-policies` in the bot’s knowledge base.

## Adding Documents

Documents can be added directly from files or by crawling a website:

```basic
USE_KB "company-policies"   ' loads and embeds all files from .gbkb/company-policies/ folder
ADD_WEBSITE "https://example.com/policies"
```

The system will download the content, split it into chunks, generate embeddings using the default LLM model, and store them in the collection.

## Managing Collections

- `USE_KB "collection-name"` – loads and embeds files from the `.gbkb/collection-name` folder into the vector database, making them available for semantic search in the current session.
- `CLEAR_KB "collection-name"` – removes the collection from the current session (files remain embedded in vector database).

## Use in Dialogs

When a KB is added to a session, the vector database is queried to retrieve relevant document chunks/excerpts that are automatically injected into LLM prompts, providing context-aware responses.

```basic
USE_KB "company-policies"
FIND "vacation policy" INTO RESULT
TALK RESULT
```

## Technical Details

- Embeddings are generated with the BGE‑small‑en‑v1.5 model.
- Vectors are stored in VectorDB (see Chapter 04).
- Each document is chunked into 500‑token pieces for efficient retrieval.
