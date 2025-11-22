# USE_KB Keyword

The **USE_KB** keyword loads and embeds files from a `.gbkb` folder into the vector database, making them available for semantic search in the current conversation session.

---

## Syntax

```basic
USE_KB "kb-name"
```

---

## Parameters

- `"kb-name"` — The name of the knowledge base folder inside `.gbkb/`.  
  Files from `work/{bot_name}/{bot_name}.gbkb/{kb-name}/` will be embedded and made available.

---

## Description

When executed, `USE_KB` performs the following:

1. **Locates the KB folder**: Finds `work/{bot_name}/{bot_name}.gbkb/{kb-name}/`
2. **Embeds documents**: Reads all files (PDF, TXT, MD, DOCX, etc.) and converts them to vector embeddings
3. **Stores in VectorDB**: Saves embeddings in the vector database (Qdrant or compatible)
4. **Activates for session**: Makes this KB available for the current conversation session
5. **LLM context injection**: Relevant chunks from this KB are automatically retrieved and injected into LLM prompts

**Multiple KBs**: You can add multiple KBs to the same session. The vector database will search across all active KBs.

**Automatic retrieval**: When the LLM receives a user query, the system automatically:
- Searches all active KBs for relevant content
- Retrieves the top matching chunks/excerpts
- Injects them into the LLM prompt as context
- LLM generates a response based on the retrieved knowledge

---

## Folder Structure

```
work/
  mybot/
    mybot.gbkb/
      policies/           ← USE_KB "policies"
        vacation.pdf
        benefits.docx
      procedures/         ← USE_KB "procedures"
        onboarding.md
        safety.txt
      faqs/              ← USE_KB "faqs"
        common.txt
```

---

## Examples

### Example 1: Add Single KB

```basic
' Load company policies KB
USE_KB "policies"

' Now LLM queries will automatically use policy documents as context
TALK "Ask me about our vacation policy"
```

### Example 2: Add Multiple KBs

```basic
' Load multiple knowledge bases
USE_KB "policies"
USE_KB "procedures"
USE_KB "faqs"

' All three KBs are now active and will be searched for relevant content
TALK "Ask me anything about our company"
```

### Example 3: Dynamic KB Selection (in a tool)

```basic
' In start.bas or any tool
PARAM subject as string
DESCRIPTION "Called when user wants to change conversation topic."

' Dynamically choose KB based on user input
kbname = LLM "Return one word: policies, procedures, or faqs based on: " + subject
USE_KB kbname

TALK "You have chosen to discuss " + subject + "."
```

### Example 4: Switch KBs

```basic
' Clear current KB and load a different one
CLEAR_KB "policies"
USE_KB "procedures"

TALK "Now focused on procedures"
```

---

## Implementation Notes

- **File types supported**: PDF, TXT, MD, DOCX, HTML, and more
- **Embedding model**: Uses configured embedding model (OpenAI, local, etc.)
- **Chunk size**: Documents are split into chunks for optimal retrieval
- **Vector database**: Stores embeddings in Qdrant or compatible VectorDB
- **Session isolation**: Each session maintains its own list of active KBs
- **Persistence**: KB embeddings persist across sessions (only session associations are cleared)

---

## Related Keywords

- [`CLEAR_KB`](keyword-clear-kb.md) — Remove KB from current session
- [`USE_TOOL`](keyword-add-tool.md) — Make a tool available in the session
- [`CLEAR_TOOLS`](keyword-clear-tools.md) — Remove all tools from session
- [`FIND`](keyword-find.md) — Manually search within active KBs

---

## Summary

`USE_KB` is the primary way to give your bot access to document knowledge. It embeds files from `.gbkb` folders into the vector database and automatically retrieves relevant content to enhance LLM responses with context-aware information.