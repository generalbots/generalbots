# Semantic Search

Semantic search in BotServer happens automatically when you use `USE KB`. The system searches for relevant information based on meaning, not just keywords, and injects it into the LLM's context.

## How It Works Automatically

1. **User asks a question** - Natural language input
2. **Query converted to vector** - Using the embedding model
3. **Search active collections** - Finds semantically similar content
4. **Inject into context** - Relevant chunks added to LLM prompt
5. **Generate response** - LLM answers using the knowledge

## Activating Semantic Search

Simply use `USE KB` to enable search for a collection:

```basic
USE KB "policies"
USE KB "procedures"
' Both collections are now searchable
' No explicit search commands needed
```

When users ask questions, the system automatically searches these collections and provides relevant context to the LLM.

## How Meaning-Based Search Works

Unlike keyword search, semantic search understands meaning:

- "How many days off do I get?" matches "vacation policy"
- "What's the return policy?" matches "refund procedures"
- "I'm feeling sick" matches "medical leave guidelines"

The system uses vector embeddings to find conceptually similar content, even when exact words don't match.

## Configuration

Search behavior is controlled by `config.csv`:

```csv
prompt-history,2     # How many previous messages to include
prompt-compact,4     # Compact context after N exchanges
```

These settings manage how much context the LLM receives, not the search itself.

## Multiple Collections

When multiple collections are active, the system searches all of them:

```basic
USE KB "products"
USE KB "support"
USE KB "warranty"

' User: "My laptop won't turn on"
' System searches all three collections for relevant info
```

## Search Quality

The quality of semantic search depends on:
- **Document organization** - Well-structured folders help
- **Embedding model** - BGE model works well, can be replaced
- **Content quality** - Clear, descriptive documents work best

## Real Example

```basic
' In start.bas
USE KB "company-handbook"

' User types: "What's the dress code?"
' System automatically:
' 1. Searches company-handbook for dress code info
' 2. Finds relevant sections about attire
' 3. Injects them into LLM context
' 4. LLM generates natural response with the information
```

## Performance

- Search happens in milliseconds
- No configuration needed
- Cached for repeated queries
- Only active collections are searched

## Best Practices

1. **Activate only needed collections** - Don't overload context
2. **Organize content well** - One topic per folder
3. **Use descriptive text** - Helps with matching
4. **Keep documents updated** - Fresh content = better answers

## Common Misconceptions

❌ **Wrong**: You need to call a search function
✅ **Right**: Search happens automatically with `USE KB`

❌ **Wrong**: You need to configure search parameters
✅ **Right**: It works out of the box

❌ **Wrong**: You need special commands to query
✅ **Right**: Users just ask questions naturally

## Troubleshooting

### Not finding relevant content?
- Check the collection is activated with `USE KB`
- Verify documents are in the right folder
- Ensure content is descriptive

### Too much irrelevant content?
- Use fewer collections simultaneously
- Organize documents into more specific folders
- Clear unused collections with `CLEAR KB`

Remember: The beauty of semantic search in BotServer is its simplicity - just `USE KB` and let the system handle the rest!