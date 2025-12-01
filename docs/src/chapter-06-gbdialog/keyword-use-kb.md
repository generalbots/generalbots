# USE KB

Activate a knowledge base collection for semantic search.

## Syntax

```basic
USE KB "collection_name"
USE KB collection_variable
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `collection_name` | String | Name of folder inside `.gbkb/` |

## Description

Loads a knowledge base collection, enabling automatic semantic search for that content. Once active, the LLM searches this collection when answering questions - no explicit search code needed.

## Examples

### Basic Usage

```basic
USE KB "policies"
' Bot now answers questions using policy documents
```

### Multiple Collections

```basic
USE KB "products"
USE KB "pricing"
USE KB "support"
' All three collections searchable
```

### Conditional Loading

```basic
dept = GET user_department
IF dept = "HR" THEN
  USE KB "hr_policies"
ELSE IF dept = "IT" THEN
  USE KB "it_docs"
END IF
```

### Dynamic Collection

```basic
topic = HEAR "What topic?"
USE KB topic
```

## How It Works

1. User asks question
2. System searches active collections
3. Top matching chunks added to LLM context
4. LLM generates informed response

## Collection Structure

```
bot.gbkb/
├── policies/      → USE KB "policies"
├── products/      → USE KB "products"
└── support/       → USE KB "support"
```

## Supported File Types

PDF, DOCX, TXT, MD, HTML, CSV, JSON

## Performance

- Each collection uses ~50MB RAM when active
- First search: 100-200ms
- Subsequent: 20-50ms (cached)

**Tip:** Load only what's needed, clear when done.

## Common Patterns

### Role-Based

```basic
SWITCH GET user_role
  CASE "manager"
    USE KB "management"
  CASE "developer"
    USE KB "documentation"
  CASE "customer"
    USE KB "products"
END SWITCH
```

### With Context

```basic
USE KB "technical_docs"
SET CONTEXT "You are a technical expert" AS prompt
```

### With Website

```basic
USE WEBSITE "https://docs.example.com"
USE KB "documentation"
' Fresh web content now searchable
```

## Error Handling

```basic
TRY
  USE KB user_requested_kb
CATCH
  TALK "That knowledge base doesn't exist"
END TRY
```

## See Also

- [CLEAR KB](./keyword-clear-kb.md) - Deactivate collections
- [Knowledge Base System](../chapter-03/README.md) - Technical details
- [Semantic Search](../chapter-03/semantic-search.md) - How search works