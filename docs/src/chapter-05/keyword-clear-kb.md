# CLEAR KB

Remove knowledge bases from the current session's context.

## Syntax

```basic
CLEAR KB kb_name
```

or to clear all:

```basic
CLEAR KB ALL
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `kb_name` | String | Name of the knowledge base to remove, or "ALL" for all KBs |

## Description

The `CLEAR KB` keyword removes previously loaded knowledge bases from the session's active context. This frees up memory and ensures that subsequent searches don't include unwanted collections.

## Examples

### Clear Specific Knowledge Base
```basic
USE KB "product-docs"
USE KB "user-manuals"

' Later, remove just product-docs
CLEAR KB "product-docs"
' Now only user-manuals is active
```

### Clear All Knowledge Bases
```basic
' Load multiple KBs
USE KB "policies"
USE KB "procedures"
USE KB "faqs"

' Clear everything
CLEAR KB ALL
TALK "All knowledge bases have been unloaded"
```

### Conditional Clearing
```basic
topic = HEAR "What topic interests you?"
IF topic = "technical" THEN
    CLEAR KB "marketing-docs"
    USE KB "technical-docs"
ELSE IF topic = "sales" THEN
    CLEAR KB "technical-docs"
    USE KB "marketing-docs"
END IF
```

### Switch Knowledge Context
```basic
' Start with general KB
USE KB "general-info"
answer = FIND "company overview"
TALK answer

' Switch to specific department
CLEAR KB "general-info"
USE KB "engineering-specs"
answer = FIND "API documentation"
TALK answer
```

### Memory Management
```basic
' Load KBs for initial context
USE KB "onboarding"
USE KB "policies"

' After onboarding complete
CLEAR KB "onboarding"
' Keep policies active but free onboarding memory
```

## Return Value

Returns `true` if the KB was successfully cleared, `false` if the KB wasn't loaded or doesn't exist.

## Error Handling

```basic
result = CLEAR KB "unknown-kb"
IF result = false THEN
    LOG "KB was not loaded or doesn't exist"
END IF
```

## Performance Considerations

- Clearing KBs immediately frees session memory
- Does not delete the actual KB from Qdrant
- Only removes the session association
- Clearing all KBs is faster than clearing individually

## Best Practices

1. **Clear Unused KBs**: Remove KBs when no longer needed
   ```basic
   ' After processing department-specific queries
   CLEAR KB "department-kb"
   ```

2. **Clear Before Loading**: Ensure clean state
   ```basic
   CLEAR KB ALL
   USE KB "fresh-context"
   ```

3. **Memory Optimization**: Clear large KBs after use
   ```basic
   USE KB "large-archive"
   results = FIND query
   CLEAR KB "large-archive"  ' Free memory
   ```

4. **Context Switching**: Clear when changing topics
   ```basic
   ON TOPIC_CHANGE
       CLEAR KB ALL
       USE KB new_topic_kb
   END ON
   ```

## Session Scope

- Clearing only affects the current session
- Other sessions maintain their own KB associations
- KBs remain in Qdrant for future use
- Can reload cleared KBs anytime with `USE KB`

## Monitoring Active KBs

```basic
' Check what's loaded before clearing
active_kbs = GET_ACTIVE_KBS()
TALK "Currently loaded: " + JOIN(active_kbs, ", ")

' Clear specific ones
FOR EACH kb IN active_kbs
    IF kb STARTS WITH "temp_" THEN
        CLEAR KB kb
    END IF
NEXT
```

## Advanced Usage

### Batch Operations
```basic
kbs_to_clear = ["old-docs", "archive-2022", "deprecated"]
FOR EACH kb IN kbs_to_clear
    CLEAR KB kb
NEXT
```

### Scheduled Cleanup
```basic
' Clear all KBs at conversation timeout
ON TIMEOUT
    CLEAR KB ALL
    LOG "Session KBs cleared due to timeout"
END ON
```

### Conditional Preservation
```basic
' Clear all except essential KBs
all_kbs = GET_ACTIVE_KBS()
essential = ["core-policies", "emergency-procedures"]

FOR EACH kb IN all_kbs
    IF kb NOT IN essential THEN
        CLEAR KB kb
    END IF
NEXT
```

## Related Keywords

- [USE KB](./keyword-use-kb.md) - Load knowledge bases
- [ADD WEBSITE](./keyword-add-website.md) - Create KB from website
- [FIND](./keyword-find.md) - Search within loaded KBs
- [LLM](./keyword-llm.md) - Use KB context in responses

## Implementation

Located in `src/basic/keywords/clear_kb.rs`

The implementation:
- Maintains session KB registry
- Removes KB references from context
- Updates search scope
- Handles "ALL" keyword specially
- Returns operation status