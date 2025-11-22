## gbkb Reference
The knowledge‑base package provides three main commands:

- **USE_KB** – Loads and embeds files from the `.gbkb/collection-name` folder into the vector database, making them available for semantic search in the current session. Multiple KBs can be active simultaneously.
- **CLEAR_KB** – Removes a knowledge base from the current session (files remain embedded in the vector database).
- **ADD_WEBSITE** – Crawl a website and add its pages to a collection.

**Example:**
```bas
' Add support docs KB - files from work/botname/botname.gbkb/support_docs/ are embedded
USE_KB "support_docs"

' Add multiple KBs to the same session
USE_KB "policies"
USE_KB "procedures"

' Remove a specific KB from session
CLEAR_KB "policies"

' Remove all KBs from session
CLEAR_KB
```

The vector database retrieves relevant chunks/excerpts from active KBs and injects them into LLM prompts automatically, providing context-aware responses.
