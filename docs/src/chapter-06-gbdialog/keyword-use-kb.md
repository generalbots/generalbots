# USE KB

**Activate a knowledge base for semantic search.** The USE KB keyword loads a document collection from `.gbkb/` folders, making it available for the LLM to search and reference when answering questions.

## Syntax

```basic
USE KB collection_name
USE KB "collection_name"
```

## Parameters

- `collection_name` - Name of the folder inside `.gbkb/` to activate

## Description

USE KB activates a knowledge base collection for the current session. Once activated, the LLM automatically searches this collection when generating responses, finding relevant information without explicit search commands.

## How It Works

1. **Collection Loading**: Reads documents from `.gbkb/collection_name/`
2. **Vector Search**: Finds semantically similar content to user queries
3. **Context Injection**: Adds relevant chunks to LLM context
4. **Automatic**: No explicit search needed - happens behind the scenes

## Examples

### Basic Usage
```basic
USE KB "policies"
' Now the bot can answer questions about policies

answer = HEAR
' Knowledge is automatically available to the conversation
TALK "Let me search the policies for: " + answer
```

### Multiple Collections
```basic
USE KB "products"
USE KB "pricing"
USE KB "support"
' All three collections are now searchable
```

### Conditional Loading
```basic
department = GET user_department

if department == "HR"
  USE KB "hr_policies"
else if department == "IT"
  USE KB "it_documentation"
else
  USE KB "general"
end
```

### Dynamic Collection Names
```basic
topic = HEAR
USE KB topic  ' Load collection based on user input
```

## Collection Structure

Knowledge bases are organized as folders:

```
bot.gbkb/
├── policies/
│   ├── vacation-policy.pdf
│   ├── code-of-conduct.docx
│   └── employee-handbook.txt
├── products/
│   ├── catalog-2024.pdf
│   └── specifications.xlsx
└── support/
    ├── faq.md
    └── troubleshooting.pdf
```

Each folder becomes a named collection accessible via USE KB.

## Supported File Types

- **PDF** - Automatic text extraction
- **DOCX/DOC** - Word documents
- **TXT** - Plain text files
- **MD** - Markdown files
- **HTML** - Web pages
- **CSV** - Structured data
- **JSON** - Configuration and data files

## Search Behavior

When USE KB is active:

1. **User asks**: "What's the vacation policy?"
2. **System searches**: Finds relevant chunks in policies collection
3. **Context built**: Adds top 5 matching chunks to context
4. **LLM responds**: Uses found information to answer accurately

## Managing Active Collections


### Clear Specific Collection
```basic
CLEAR KB "policies"
' Policies collection no longer searched
```

### Clear All Collections
```basic
CLEAR KB
' All collections deactivated
```

## Performance Considerations

### Memory Usage
Each collection uses ~50MB RAM when active. Load only what's needed:

```basic
' Good - Load for specific task
USE KB "troubleshooting"
' ... handle support issue ...
CLEAR KB "troubleshooting"

' Bad - Load everything
USE KB "collection1"
USE KB "collection2"
USE KB "collection3"
' ... context size intensive ...
```

### Search Speed
- First search: 100-200ms (cold cache)
- Subsequent: 20-50ms (warm cache)
- More collections = slower search

## Advanced Usage

### Scoped Knowledge
```basic
' Customer service flow
ON "customer_service"
  USE KB "products"
  USE KB "policies"
  USE KB "returns"
END

' Technical support flow  
ON "tech_support"
  CLEAR KB
  USE KB "documentation"
  USE KB "known_issues"
END
```

### Dynamic Reloading
```basic
' Refresh collection with new documents
CLEAR KB "news"
USE WEBSITE "https://example.com/news"
USE KB "news"
```

### Collection Validation
```basic
try
  USE KB user_requested_collection
catch error
  TALK "Sorry, that knowledge base doesn't exist"
  collections = LIST_AVAILABLE_KB()
  TALK "Available: " + collections
end
```

## Integration with Other Keywords

### With SET CONTEXT
```basic
USE KB "technical_docs"
SET CONTEXT "You are a technical expert" AS prompt
' LLM now searches technical docs with expert perspective
```

### With USE TOOL
```basic
USE KB "inventory"
USE TOOL "check_stock"
' Tool can access inventory knowledge when executing
```

### With USE WEBSITE
```basic
USE WEBSITE "https://docs.example.com"
USE KB "documentation"
' Fresh web content now searchable
```

## Best Practices

### Do's
✅ Load collections relevant to conversation topic  
✅ Clear unused collections to save memory  
✅ Organize documents into logical collections  
✅ Use descriptive collection names  

### Don'ts
❌ Don't load all collections at once  
❌ Don't assume collections exist - handle errors  
❌ Don't mix unrelated documents in one collection  
❌ Don't forget to clear when switching contexts  

## Common Patterns

### Role-Based Knowledge
```basic
role = GET user_role

switch role
  case "manager"
    USE KB "management"
    USE KB "reports"
  case "developer"  
    USE KB "documentation"
    USE KB "apis"
  case "customer"
    USE KB "products"
    USE KB "support"
end
```

### Time-Based Collections
```basic
month = GET_MONTH()
USE KB "updates_" + month
' Load current month's updates
```

## Error Handling

Common errors and solutions:

**Collection not found**
```basic
Error: Knowledge base 'unknown' not found
Solution: Check collection exists in .gbkb/ folder
```

**Empty collection**
```basic
Warning: Collection 'new' has no documents
Solution: Add documents to the folder
```

**Index not ready**
```basic
Info: Indexing collection 'large'...
Solution: Wait for indexing to complete (automatic)
```

## See Also

- [CLEAR KB](./keyword-clear-kb.md) - Deactivate knowledge bases
- [USE WEBSITE](./keyword-use-website.md) - Associate website with conversation
- [Knowledge Base Guide](../chapter-03/README.md) - Complete KB documentation
- [Vector Collections](../chapter-03/vector-collections.md) - How collections work
