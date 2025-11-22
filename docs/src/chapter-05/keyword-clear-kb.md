# CLEAR_KB

Clear knowledge base collections from the current session context.

## Syntax

```basic
CLEAR_KB kb_name
```

or

```basic
CLEAR_KB
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `kb_name` | String | Optional. Name of specific knowledge base to remove. If omitted, clears all KBs |

## Description

The `CLEAR_KB` keyword removes knowledge base collections from the current session's context. This is useful for:

- Switching between different knowledge domains
- Reducing context size for performance
- Preventing irrelevant KB interference
- Managing memory usage in long-running sessions

When called without parameters, it clears all active knowledge bases. When called with a specific KB name, it only removes that collection.

## Examples

### Clear Specific Knowledge Base
```basic
USE_KB "product-docs"
USE_KB "user-manual"
' Later, remove just one
CLEAR_KB "product-docs"
' user-manual remains active
```

### Clear All Knowledge Bases
```basic
USE_KB "docs"
USE_KB "faqs"
USE_KB "policies"
' Clear everything
CLEAR_KB
' No KBs active now
```

### Conditional KB Management
```basic
IF topic = "technical" THEN
    CLEAR_KB
    USE_KB "engineering-docs"
ELSE
    CLEAR_KB "engineering-docs"
    USE_KB "general-docs"
END IF
```

### Knowledge Base Rotation
```basic
' Use KB for initial query
USE_KB "current-year-data"
answer = FIND "quarterly results"
TALK answer

' Switch to historical data
CLEAR_KB "current-year-data"
USE_KB "historical-data"
historical = FIND "previous year results"
TALK historical
```

## Return Value

Returns a boolean indicating success:
- `true`: Knowledge base(s) successfully cleared
- `false`: Operation failed or KB not found

## Session Context

The `CLEAR_KB` command affects only the current user session. Other sessions maintain their own KB contexts independently.

## Memory Management

Clearing knowledge bases:
- Removes vector collection references from session
- Frees up context window space
- Does NOT delete the actual KB data (only removes from active context)
- Reduces memory footprint of the session

## Error Handling

- Silently succeeds if specified KB is not currently loaded
- Returns false if KB name is invalid format
- Logs warning if clearing fails due to system constraints

## Performance Considerations

1. **Context Size**: Clearing unused KBs reduces token usage in LLM calls
2. **Query Speed**: Fewer active KBs means faster semantic search
3. **Memory**: Each KB consumes memory for index caching
4. **Relevance**: Too many KBs can introduce irrelevant results

## Best Practices

1. **Clear Before Switching**: Always clear old KBs before loading new ones for different domains
2. **Periodic Cleanup**: In long conversations, periodically clear unused KBs
3. **Domain Separation**: Don't mix unrelated knowledge domains
4. **Check Before Clear**: Optionally check if KB is loaded before clearing

Example of good practice:
```basic
' Clean switch between domains
CLEAR_KB
IF customer_type = "enterprise" THEN
    USE_KB "enterprise-docs"
    USE_KB "sla-policies"
ELSE
    USE_KB "standard-docs"
END IF
```

## Related Keywords

- [USE_KB](./keyword-use-kb.md) - Load knowledge base collections
- [ADD_WEBSITE](./keyword-add-website.md) - Add website content to KB
- [FIND](./keyword-find.md) - Search within loaded KBs
- [LLM](./keyword-llm.md) - Query LLM with KB context

## Implementation Details

Located in `src/basic/keywords/clear_kb.rs`

The implementation:
- Maintains KB references in session state
- Uses HashSet for efficient KB tracking
- Integrates with Qdrant vector store
- Handles concurrent access safely
- Updates session metrics for monitoring