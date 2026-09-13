# Document Indexing 🟡 BETA

Documents in `.gbkb` folders are indexed automatically. No manual configuration required.

## Automatic Triggers

Indexing occurs when:
- Files added to `.gbkb` folders
- Files modified or updated
- `USE KB` called for a collection
- `USE WEBSITE` registers URLs for crawling

## Processing Pipeline

```
Document → Extract Text → Embed → Store in Qdrant
```

| Stage | Description |
|-------|-------------|
| **Extract** | Pull text from PDF, DOCX, DOC, XLSX, XLS, ODS, PPTX, PPT, ODP, EPUB, ODT, HTML, MD, TXT, CSV, JSON, YAML, TOML, and more |
| **Embed** | Generate one vector per document with the configured embedding model |
| **Store** | Save to Qdrant with the file path, type, bucket and tag metadata that retrieval filters on |

> **No chunking stage (September 2026).** A document becomes a single record. There is no splitting into segments and no overlap, so precision degrades on long documents. This is a known limitation, not a setting.

## Supported File Types

| Format | Notes |
|--------|-------|
| PDF | Full text extraction, OCR for scanned docs |
| DOCX/DOC | Microsoft Word documents |
| XLSX/XLS/ODS | Spreadsheets (Excel, OpenDocument) — each row indexed |
| PPTX/PPT/ODP | Presentations (PowerPoint, OpenDocument) — slide text extracted |
| EPUB/ODT | E-books and OpenDocument text |
| TXT/MD/RST/ADOC | Plain text, Markdown, reStructuredText, AsciiDoc |
| HTML | Web pages (text only) |
| CSV/TSV | Tabular data — each row indexed separately |
| JSON/JSONL | Structured data |
| YAML/TOML/INI | Configuration files |
| PY/RS/JS/TS/SH/SQL | Source code files |
| CSS/SVG | Style and vector graphics |
| ICS/VCF/EML | Calendar, contacts, email |
| LOG | Log files |
| Any `text/*` MIME | Catch-all for any text-based format (max 100MB) |

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

The embedding endpoint is part of the bot's configuration. Two models are supported:

| Setting | Model | Dimensions |
|---------|-------|-----------|
| Local embedding service (default) | `sentence-transformers/all-MiniLM-L6-v2` | 384 |
| OpenAI | `text-embedding-3-small` | 1536 |

Input is truncated to 600 tokens before embedding with the local model. If no embedding endpoint is configured, retrieval degrades to keyword matching, and if embedding generation fails at query time a non-semantic hash vector is substituted — see [Retrieval and RAG](./hybrid-search.md#embeddings-and-their-fallbacks).

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