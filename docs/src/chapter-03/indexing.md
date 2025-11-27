# Document Indexing

Document indexing in BotServer is automatic. When documents are added to `.gbkb` folders, they are processed and made searchable without any manual configuration.

## Automatic Indexing

The system automatically indexes documents when:
- Files are added to any `.gbkb` folder
- `USE KB` is called for a collection
- Files are modified or updated
- `USE WEBSITE` registers websites for crawling (preprocessing) and associates them with sessions (runtime)

## How Indexing Works

1. **Document Detection** - System scans `.gbkb` folders for files
2. **Text Extraction** - Content extracted from PDF, DOCX, HTML, MD, TXT
3. **Chunking** - Text split into manageable segments
4. **Embedding Generation** - Chunks converted to vectors using BGE model
5. **Storage** - Vectors stored for semantic search

## Supported File Types

- **PDF** - Full text extraction
- **DOCX** - Microsoft Word documents
- **TXT** - Plain text files
- **HTML** - Web pages (text only)
- **MD** - Markdown documents
- **CSV** - Structured data

## Website Indexing

To keep web content fresh, schedule regular crawls:

```basic
' In update-docs.bas
SET SCHEDULE "0 2 * * *"  ' Run daily at 2 AM

USE WEBSITE "https://docs.example.com"
' Website is registered for crawling during preprocessing
' At runtime, it associates the crawled content with the session
```

### Scheduling Options

```basic
SET SCHEDULE "0 * * * *"     ' Every hour
SET SCHEDULE "*/30 * * * *"  ' Every 30 minutes
SET SCHEDULE "0 0 * * 0"     ' Weekly on Sunday
SET SCHEDULE "0 0 1 * *"     ' Monthly on the 1st
```

## Real-Time Updates

Documents are re-indexed automatically when:
- File content changes
- New files appear in folders
- Files are deleted (removed from index)

## Using Indexed Content

Once indexed, content is automatically available:

```basic
USE KB "documentation"
' All documents in the documentation folder are now searchable
' The LLM will use this knowledge when answering questions
```

You don't need to explicitly search - the system does it automatically when generating responses.

## Configuration

Indexing uses settings from `config.csv`:

```csv
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

The BGE embedding model can be replaced with any compatible model.

## Performance Optimization

The system optimizes indexing by:
- Processing only changed files
- Caching embeddings
- Parallel processing when possible
- Incremental updates

## Example: Knowledge Base Maintenance

Structure your knowledge base:
```
company.gbkb/
├── products/
│   ├── manual-v1.pdf
│   └── specs.docx
├── policies/
│   ├── hr-policy.pdf
│   └── it-policy.md
└── news/
    └── updates.html
```

Schedule regular web updates:
```basic
' In maintenance.bas
SET SCHEDULE "0 1 * * *"

' Register websites for crawling
USE WEBSITE "https://company.com/news"
USE WEBSITE "https://company.com/products"
' Websites are crawled by background service
```

## Best Practices

1. **Organize documents** by topic in separate folders
2. **Schedule updates** for web content
3. **Keep files updated** - system handles re-indexing
4. **Monitor folder sizes** - very large collections may impact performance
5. **Use clear naming** - helps with organization

## Troubleshooting

### Documents Not Appearing
- Check file is in a `.gbkb` folder
- Verify file type is supported
- Ensure `USE KB` was called for that collection

### Slow Indexing
- Large PDFs may take time to process
- Consider splitting very large documents
- Check available system resources

### Outdated Content
- Set up scheduled crawls for web content
- Ensure files are being updated
- Check that re-indexing is triggered

Remember: Indexing is automatic - just add documents to folders and use `USE KB` to activate them!