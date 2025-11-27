# Vector Collections

A **vector collection** is automatically generated from each folder in `.gbkb`. Each folder becomes a searchable collection that the LLM can use during conversations.

## How Collections Work

Each `.gbkb` folder is automatically:
1. Scanned for documents (PDF, DOCX, TXT, HTML, MD)
2. Text extracted from all files
3. Split into chunks for processing
4. Converted to vector embeddings using BGE model (replaceable)
5. Made available for semantic search

## Folder Structure

```
botname.gbkb/
├── policies/        # Becomes "policies" collection
├── procedures/      # Becomes "procedures" collection
└── faqs/           # Becomes "faqs" collection
```

## Using Collections

Simply activate a collection with `USE KB`:

```basic
USE KB "policies"
' The LLM now has access to all documents in the policies folder
' No need to explicitly search - happens automatically during responses
```

## Multiple Collections

Load multiple collections for comprehensive knowledge:

```basic
USE KB "policies"
USE KB "procedures" 
USE KB "faqs"
' All three collections are now active
' LLM searches across all when generating responses
```

## Automatic Document Indexing

Documents are indexed automatically when:
- Files are added to `.gbkb` folders
- `USE KB` is called for the first time
- The system detects new or modified files

### Indexing Flow

```
     .gbkb/policies/
           │
           ├── vacation.pdf
           ├── handbook.docx
           └── rules.txt
                │
                ▼
    ┌─────────────────────┐
    │   File Detection    │
    │  • New files found  │
    │  • Hash comparison  │
    └──────────┬──────────┘
               │
               ▼
    ┌─────────────────────┐
    │   Text Extraction   │
    │  • PDF → Text       │
    │  • DOCX → Text      │
    │  • HTML → Text      │
    └──────────┬──────────┘
               │
               ▼
    ┌─────────────────────┐
    │     Chunking        │
    │  • Split by size    │
    │  • Overlap windows  │
    │  • ~500 tokens each │
    └──────────┬──────────┘
               │
               ▼
    ┌─────────────────────┐
    │  Generate Embeddings│
    │  • BGE Model        │
    │  • 384 dimensions   │
    │  • Batch processing │
    └──────────┬──────────┘
               │
               ▼
    ┌─────────────────────┐
    │  Store in Vector DB │
    │  • Vector storage   │
    │  • Metadata tags    │
    │  • Collection index │
    └──────────┬──────────┘
               │
               ▼
    ┌─────────────────────┐
    │    Ready to Use     │
    │  USE KB "policies"  │
    └─────────────────────┘
```

## Website Indexing

To keep web content updated, schedule regular crawls:

```basic
' In update-content.bas
SET SCHEDULE "0 3 * * *"  ' Run daily at 3 AM
USE WEBSITE "https://example.com/docs"
' Website is registered for crawling and will be available in conversations
```

## How Search Works

When `USE KB` is active:
1. User asks a question
2. System automatically searches relevant collections
3. Finds semantically similar content
4. Injects relevant chunks into LLM context
5. LLM generates response using the knowledge

**Important**: Search happens automatically - you don't need to call any search function. Just activate the KB with `USE KB` and ask questions naturally.

### Search Pipeline

```
"What's the vacation policy?"
            │
            ▼
    ┌────────────────┐
    │  Query Embed   │
    │  BGE Model     │
    └───────┬────────┘
            │ [0.2, -0.5, 0.8, ...]
            ▼
    ┌────────────────┐
    │  Vector Search │
    │   Vector DB    │
    │  Cosine Sim    │
    └───────┬────────┘
            │
            ├─► Match 1: "Vacation days: 15 annually" (0.92)
            ├─► Match 2: "PTO policy applies to..." (0.87)
            └─► Match 3: "Time off requests via..." (0.83)
                        │
                        ▼
            ┌──────────────────────┐
            │   Context Building   │
            │  • Top 5 matches     │
            │  • 2000 tokens max   │
            │  • Relevance sorted │
            └──────────┬───────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │      LLM Call        │
            │  Context + Question  │
            │  "Based on docs..."  │
            └──────────┬───────────┘
                       │
                       ▼
                 "You get 15 vacation days per year..."
```

## Embeddings Configuration

The system uses BGE embeddings by default:

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

You can replace BGE with any compatible embedding model by changing the model path in config.csv.

## Collection Management

- `USE KB "name"` - Activates a collection for the session
- `CLEAR KB` - Removes all active collections
- `CLEAR KB "name"` - Removes a specific collection

## Best Practices

1. **Organize by topic** - One folder per subject area
2. **Name clearly** - Use descriptive folder names
3. **Update regularly** - Schedule website crawls if using web content
4. **Keep files current** - System auto-indexes changes
5. **Don't overload** - Use only necessary collections per session

## Example: Customer Support Bot

```
support.gbkb/
├── products/        # Product documentation
├── policies/        # Company policies
├── troubleshooting/ # Common issues and solutions
└── contact/         # Contact information
```

In your dialog:
```basic
' Activate all support knowledge
USE KB "products"
USE KB "troubleshooting"
' Bot can now answer product questions and solve issues
```

## Performance Notes

- Collections are cached for fast access
- Only active collections consume memory
- Embeddings are generated once and reused
- Changes trigger automatic re-indexing

No manual configuration needed - just organize your documents in folders and use `USE KB` to activate them!