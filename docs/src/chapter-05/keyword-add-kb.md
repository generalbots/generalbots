# ADD_KB Keyword

**Syntax**

```
ADD_KB "collection-name"
```

**Parameters**

- `"collection-name"` – Identifier for a new knowledge‑base folder inside the bot’s `.gbkb` directory.

**Description**

`ADD_KB` does not create a physical folder on disk; instead it informs the **context manager** (found in `src/context/`) that a new vector‑DB collection, identified by the given name, should be considered part of the current conversation’s context. The collection is treated as a drive‑based knowledge source that can be queried by the prompt processor.

Multiple `ADD_KB` calls can be issued in a single dialog or by a tool invoked through its trigger phrase. Each call adds the named collection to the session’s **additional_kb_collections** list, which the prompt processor later reads to retrieve relevant documents. This makes it possible for a user to say things like “talk about the latest policy” after a tool has prepared a new collection.

The keyword is typically used at the start of a conversation or inside a tool that changes context (e.g., after uploading a set of files to a drive or after an email attachment is processed). By adding the collection to the context, subsequent `FIND` or `LLM` calls automatically incorporate the newly available knowledge.

**Example**

```basic
ADD_KB "company-policies"
TALK "Knowledge base 'company-policies' is now part of the conversation context."
```

After execution, the `company-policies` collection is registered with the context manager and will be consulted for any future queries in this session.
