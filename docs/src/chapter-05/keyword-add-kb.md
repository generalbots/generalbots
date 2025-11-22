# USE_KB

Load and activate a knowledge base collection for the current conversation.

## Syntax

```basic
USE_KB kb_name
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `kb_name` | String | Name of the knowledge base collection to load |

## Description

The `USE_KB` keyword loads a knowledge base collection into the current session's context, making its documents searchable via `FIND` and available to the LLM for context-aware responses. Knowledge bases are vector collections stored in Qdrant containing indexed documents, FAQs, or other reference materials.

Multiple knowledge bases can be active simultaneously, allowing the bot to access diverse information sources.

## Examples

### Load Single Knowledge Base
```basic
USE_KB "product-docs"
answer = FIND "installation guide"
TALK answer
```

### Load Multiple Knowledge Bases
```basic
USE_KB "company-policies"
USE_KB "hr-handbook"
USE_KB "benefits-guide"

question = HEAR "What's the vacation policy?"
answer = FIND question
TALK answer
```

### Conditional KB Loading
```basic
department = HEAR "Which department are you from?"
IF department = "engineering" THEN
    USE_KB "technical-docs"
    USE_KB "api-reference"
ELSE IF department = "sales" THEN
    USE_KB "product-catalog"
    USE_KB "pricing-guide"
ELSE
    USE_KB "general-info"
END IF
```

### Dynamic KB Selection
```basic
topic = DETECT_TOPIC(user_message)
kb_name = "kb_" + topic
USE_KB kb_name
```

## Knowledge Base Types

Common KB collections include:
- **Documentation**: Product manuals, guides, tutorials
- **FAQs**: Frequently asked questions and answers
- **Policies**: Company policies, procedures, guidelines
- **Products**: Catalogs, specifications, pricing
- **Support**: Troubleshooting guides, known issues
- **Legal**: Terms, contracts, compliance documents

## KB Naming Convention

Knowledge bases follow this naming pattern:
- Format: `category_subcategory_version`
- Examples: `docs_api_v2`, `support_faq_current`, `products_2024`

## Loading Behavior

When `USE_KB` is called:
1. Checks if KB exists in Qdrant
2. Loads vector embeddings into memory
3. Adds to session's active KB list
4. Makes content searchable
5. Updates context for LLM

## Memory Management

- KBs remain loaded for entire session
- Use `CLEAR_KB` to unload specific KB
- Maximum 10 KBs active simultaneously
- Automatically cleared on session end

## Error Handling

```basic
TRY
    USE_KB "special-docs"
    TALK "Knowledge base loaded successfully"
CATCH "kb_not_found"
    TALK "That knowledge base doesn't exist"
    USE_KB "default-docs"  ' Fallback
CATCH "kb_error"
    LOG "Failed to load KB"
    TALK "Having trouble accessing documentation"
END TRY
```

## Performance Considerations

- First load may take 1-2 seconds
- Subsequent queries are cached
- Large KBs (>10,000 documents) may impact response time
- Consider loading only necessary KBs

## KB Content Management

### Creating Knowledge Bases
KBs are created from document collections in `.gbkb` packages:
```
mybot.gbkb/
├── docs/           # Becomes "docs" KB
├── faqs/           # Becomes "faqs" KB
└── policies/       # Becomes "policies" KB
```

### Updating Knowledge Bases
- KBs are indexed during bot deployment
- Updates require re-indexing
- Use version suffixes for updates

## Best Practices

1. **Load Relevant KBs Early**: Load at conversation start
2. **Use Descriptive Names**: Make KB purpose clear
3. **Limit Active KBs**: Don't load unnecessary collections
4. **Clear When Done**: Remove KBs when changing context
5. **Handle Missing KBs**: Always have fallback options
6. **Version Your KBs**: Track KB updates with versions

## Integration with Other Keywords

- **[FIND](./keyword-find.md)**: Search within loaded KBs
- **[CLEAR_KB](./keyword-clear-kb.md)**: Unload knowledge bases
- **[ADD_WEBSITE](./keyword-add-website.md)**: Create KB from website
- **[LLM](./keyword-llm.md)**: Use KB context in responses

## Advanced Usage

### KB Metadata
```basic
kb_info = GET_KB_INFO("product-docs")
TALK "KB contains " + kb_info.doc_count + " documents"
TALK "Last updated: " + kb_info.update_date
```

### Conditional Loading Based on Language
```basic
language = GET_USER_LANGUAGE()
USE_KB "docs_" + language  ' docs_en, docs_es, docs_fr
```

### KB Search with Filters
```basic
USE_KB "all-products"
' Search only recent products
results = FIND_WITH_FILTER "wireless", "year >= 2023"
```

## Troubleshooting

### KB Not Loading
- Verify KB name is correct
- Check if KB was properly indexed
- Ensure Qdrant service is running
- Review bot logs for errors

### Slow Performance
- Reduce number of active KBs
- Optimize KB content (remove duplicates)
- Check Qdrant server resources
- Consider KB partitioning

### Incorrect Results
- Verify KB contains expected content
- Check document quality
- Review indexing settings
- Test with specific queries

## Implementation

Located in `src/basic/keywords/use_kb.rs`

The implementation connects to Qdrant vector database and manages KB collections per session.