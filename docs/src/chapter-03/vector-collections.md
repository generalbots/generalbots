# Vector Collections

A **vector collection** is a set of documents that have been transformed into vector embeddings for fast semantic similarity search. Each collection lives under a `.gbkb` folder and is identified by a unique name.

## Creating a Collection

Use the `ADD_KB` keyword in a dialog script:

```basic
ADD_KB "company-policies"
```

This creates a new collection named `company-policies` in the bot’s knowledge base.

## Adding Documents

Documents can be added directly from files or by crawling a website:

```basic
ADD_KB "company-policies"   ' adds a new empty collection
ADD_WEBSITE "https://example.com/policies"
```

The system will download the content, split it into chunks, generate embeddings using the default LLM model, and store them in the collection.

## Managing Collections

- `SET_KB "collection-name"` – selects the active collection for subsequent `ADD_KB` or `FIND` calls.
- `LIST_KB` – (not a keyword, but you can query via API) lists all collections.

## Use in Dialogs

When a collection is active, the `FIND` keyword searches across its documents, and the `GET_BOT_MEMORY` keyword can retrieve relevant snippets to inject into LLM prompts.

```basic
SET_KB "company-policies"
FIND "vacation policy" INTO RESULT
TALK RESULT
```

## Technical Details

- Embeddings are generated with the BGE‑small‑en‑v1.5 model.
- Vectors are stored in Qdrant (see Chapter 04).
- Each document is chunked into 500‑token pieces for efficient retrieval.
