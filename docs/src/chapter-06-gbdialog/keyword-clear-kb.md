# CLEAR KB

Remove knowledge bases from the current session.

## Syntax

```basic
CLEAR KB "collection_name"   ' Remove specific collection
CLEAR KB ALL                 ' Remove all collections
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_name` | String | Name of KB to remove (optional) |
| `ALL` | Keyword | Removes all active KBs |

## Description

`CLEAR KB` removes previously loaded knowledge bases from the session's context. This frees memory and ensures subsequent queries don't search unwanted collections.

## Examples

### Clear Specific KB

```basic
USE KB "policies"
USE KB "products"

' Later, remove just policies
CLEAR KB "policies"
' Only products remains active
```

### Clear All KBs

```basic
USE KB "hr-docs"
USE KB "it-docs"
USE KB "finance"

CLEAR KB ALL
' All collections removed
```

### Context Switching

```basic
' Support flow
USE KB "troubleshooting"
USE KB "known-issues"
' ... handle support ...

' Switch to sales
CLEAR KB ALL
USE KB "products"
USE KB "pricing"
```

## Return Value

Returns `true` if cleared successfully, `false` if KB wasn't loaded.

## Best Practices

| Do | Don't |
|----|-------|
| Clear when switching topics | Leave large KBs active unnecessarily |
| Clear before loading new context | Assume collections auto-clear |
| Use `ALL` for clean slate | Clear one-by-one when `ALL` works |

## Session Scope

- Only affects current session
- Other sessions keep their KBs
- KBs remain in database for future use
- Can reload cleared KBs anytime

## See Also

- [USE KB](./keyword-use-kb.md) - Load knowledge bases
- [Knowledge Base System](../chapter-03/README.md) - Technical details