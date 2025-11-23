## gbkb Reference
The knowledge‑base package provides three main commands:

- **USE KB** – Loads and embeds files from the `.gbkb/collection-name` folder into the vector database, making them available for semantic search in the current session. Multiple KBs can be active simultaneously.
- **CLEAR KB** – Removes a knowledge base from the current session (files remain embedded in the vector database).
- **ADD WEBSITE** – Crawl a website and add its pages to a collection.

**Example:**
```bas
' Add support docs KB - files from work/botname/botname.gbkb/support_docs/ are embedded
USE KB "support_docs"

' Add multiple KBs to the same session
USE KB "policies"
USE KB "procedures"

' Remove a specific KB from session
CLEAR KB "policies"

' Remove all KBs from session
CLEAR KB
```

The vector database retrieves relevant chunks/excerpts from active KBs and injects them into LLM prompts automatically, providing context-aware responses.
