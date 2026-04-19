# Document Indexing

Documents in `.gbkb` folders are indexed automatically. No manual configuration required.

## Automatic Triggers

Indexing occurs when:
- Files added to `.gbkb` folders
- Files modified or updated
- `USE KB` called for a collection
- `USE WEBSITE` registers URLs for crawling

## Processing Pipeline

```
Document → Extract Text → Chunk → Embed → Store in Qdrant
```

| Stage | Description |
|-------|-------------|
| **Extract** | Pull text from PDF, DOCX, HTML, MD, TXT, CSV |
| **Chunk** | Split into ~500 token segments with 50 token overlap |
| **Embed** | Generate vectors using BGE model |
| **Store** | Save to Qdrant with metadata |

## Supported File Types

| Format | Notes |
|--------|-------|
| PDF | Full text extraction, OCR for scanned docs |
| DOCX | Microsoft Word documents |
| TXT/MD | Plain text and Markdown |
| HTML | Web pages (text only) |
| CSV/JSON | Structured data |

## Website Indexing

Schedule regular crawls for web content:

```basic
SET SCHEDULE "0 2 * * *"  ' Daily at 2 AM
USE WEBSITE "https://docs.example.com"
```

### Schedule Examples

| Pattern | Frequency |
|---------|-----------|
| `"0 * * * *"` | Hourly |
| `"*/30 * * * *"` | Every 30 minutes |
| `"0 0 * * 0"` | Weekly (Sunday) |
| `"0 0 1 * *"` | Monthly (1st) |

## Configuration

In `config.csv`:

```csv
name,value
embedding-url,http://localhost:8082
embedding-model,../../../../data/llm/bge-small-en-v1.5-f32.gguf
```

## Using Indexed Content

```basic
USE KB "documentation"
' All documents now searchable
' LLM uses this knowledge automatically
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Documents not found | Check file is in `.gbkb` folder, verify `USE KB` called |
| Slow indexing | Large PDFs take time; consider splitting documents |
| Outdated content | Set up scheduled crawls for web content |

## See Also

- [Knowledge Base System](./README.md) - Architecture overview
- [Semantic Search](./semantic-search.md) - How search works
- [Vector Collections](./vector-collections.md) - Collection management