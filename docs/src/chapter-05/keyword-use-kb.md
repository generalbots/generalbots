# USE KB

Load a knowledge base collection into the current session for semantic search and context.

## Syntax

```basic
USE KB kb_name
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `kb_name` | String | Name of the knowledge base collection to load |

## Description

The `USE KB` keyword loads a knowledge base collection into the current session's context, making its documents searchable via `FIND` and available to the LLM for context-aware responses. Knowledge bases are vector collections stored in Qdrant containing indexed documents, FAQs, or other reference materials.

## Examples

### Load Single Knowledge Base
```basic
USE KB "product-docs"
answer = FIND "installation guide"
TALK answer
```

### Load Multiple Knowledge Bases
```basic
USE KB "company-policies"
USE KB "hr-handbook"
USE KB "benefits-guide"

question = HEAR "What's the vacation policy?"
answer = FIND question
TALK answer
```

### Conditional KB Loading
```basic
department = HEAR "Which department are you from?"
IF department = "engineering" THEN
    USE KB "technical-docs"
    USE KB "api-reference"
ELSE IF department = "sales" THEN
    USE KB "product-catalog"
    USE KB "pricing-guide"
ELSE
    USE KB "general-info"
END IF
```

### Dynamic KB Selection
```basic
topic = DETECT_TOPIC(user_message)
kb_name = "kb_" + topic
USE KB kb_name
```

## How It Works

1. **Collection Loading**: Connects to Qdrant vector database
2. **Index Verification**: Checks collection exists and is indexed
3. **Session Association**: Links KB to current user session
4. **Context Building**: Makes documents available for search
5. **Memory Management**: Maintains list of active KBs

## Technical Details

When `USE KB` is called:
1. Checks if KB exists in Qdrant
2. Verifies user has access permissions
3. Loads collection metadata
4. Adds to session's active KB list
5. Updates search context

## Limitations

- Maximum 10 KBs per session
- KB name must exist in Qdrant
- Case-sensitive KB names
- Use `CLEAR KB` to unload specific KB
- Session-scoped (not persistent)

## Error Handling

```basic
TRY
    USE KB "special-docs"
    TALK "Knowledge base loaded successfully"
CATCH "kb_not_found"
    TALK "That knowledge base doesn't exist"
    USE KB "default-docs"  ' Fallback
CATCH "kb_error"
    LOG "Failed to load KB"
    TALK "Having trouble accessing documentation"
END TRY
```

## Performance

- Lazy loading - documents fetched on demand
- Metadata cached in session
- Vector indices remain in Qdrant
- No document duplication in memory

## Best Practices

1. **Load Early**: Load KBs at conversation start
2. **Relevant KBs Only**: Don't load unnecessary collections
3. **Clear When Done**: Use `CLEAR KB` to free resources
4. **Handle Missing KBs**: Always have fallback logic
5. **Name Conventions**: Use descriptive, consistent names

## KB Management

### Check Available KBs
```basic
available = LIST_KBS()
FOR EACH kb IN available
    TALK "Available: " + kb.name + " (" + kb.doc_count + " docs)"
NEXT
```

### Active KBs in Session
```basic
active = GET_ACTIVE_KBS()
TALK "Currently loaded: " + JOIN(active, ", ")
```

## Related Keywords

- **[CLEAR KB](./keyword-clear-kb.md)**: Unload knowledge bases
- **[ADD WEBSITE](./keyword-add-website.md)**: Create KB from website
- **[LLM](./keyword-llm.md)**: Use KB context in responses
- **[FIND](./keyword-find.md)**: Search within loaded KBs

## Advanced Usage

### KB Information
```basic
kb_info = GET_KB_INFO("product-docs")
TALK "KB contains " + kb_info.doc_count + " documents"
TALK "Last updated: " + kb_info.update_date
```

### Language-Specific KBs
```basic
language = GET_USER_LANGUAGE()
USE KB "docs_" + language  ' docs_en, docs_es, docs_fr
```

### Filtered Search
```basic
USE KB "all-products"
' Search only recent products
results = FIND_WITH_FILTER "wireless", "year >= 2023"
```

## Vector Database Integration

Knowledge bases are stored as Qdrant collections:
- Each document is embedded as vectors
- Semantic similarity search enabled
- Metadata filtering supported
- Fast retrieval via HNSW index

## Creating Knowledge Bases

KBs are typically created through:
- `.gbkb` packages in bot folders
- `ADD WEBSITE` command for web content
- Direct Qdrant collection creation
- Import from external sources

## Implementation

Located in `src/basic/keywords/use_kb.rs`

The implementation:
- Validates KB existence in Qdrant
- Manages session KB registry
- Handles concurrent KB access
- Provides search context to LLM