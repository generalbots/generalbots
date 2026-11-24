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

> **`.gbkb` file indexing stores whole-file records.** The pipeline above does not split a document into segments or overlap windows, so precision degrades on long files; this is a limitation of that path, not a setting. Saved Docs documents take a separate structure-aware path — see "Structure-aware indexing of saved Docs documents" below.

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

## Structure-aware indexing of saved Docs documents

Docs saves are also projected into a structure-aware, in-memory knowledge
index. `DocState::with_in_memory_kb_index`
(`botserver/crates/botdocs/src/state.rs`) converts the saved HTML to Markdown,
splits it at headings, page/slide/cell markers and numbered clauses, and stores
each chunk with a stable ID, a citation and an ACL derived from the document's
tenant and owner. A local lexical embedding fills the vector slot. These points
live only in that process-local index — the Qdrant store described above is not
part of this path.

| Capability | State today |
|---|---|
| Chunk, citation and ACL types | Implemented in `botcore::kb::structured` and used by the Docs save path |
| Indexing on save and autosave | Implemented; an edit first evicts the document's previous points so removed clauses stop matching |
| Removal on delete | Implemented; both index points and their citation identity are dropped |
| Boot backfill | Persisted Docs documents are replayed from Drive into the in-memory index at start-up; a failure is logged, not fatal |
| Permission-filtered search | `GET /api/docs/kb/search?q=<text>&limit=<n>` returns citation identity only — `document_id`, `file`, `version`, `source_ref`, `section`, `score` — and never document content |
| Citation-only chat context | An authenticated user's question about a saved document appends at most three identity lines (`document_id`, `file`, `version`, `section`, `page`) to the system message; an anonymous turn appends none |
| Qdrant projection of these points | Not implemented |

### SourceRef and Citation

`SourceRef` (`citation.rs:26-88`) is a stable address inside a document: a
`SourceKind` (`Page`, `Section`, `Clause`, `Slide`, `Cell`), a `locator` string
and an optional physical `page`. Constructors cover each kind, `with_page`
attaches the page, and `address()` renders a locale-neutral string such as
`clause=7.3 p12` for point IDs, logs and tests.

`Citation` (`citation.rs:90-129`) binds that address to `document_id`, `file`,
`version` and the `heading_path` (the chain of enclosing headings). Its
`label()` is a stable, locale-neutral summary for logs and non-UI surfaces;
the Docs UI is expected to build a localized label from the structured fields
rather than render that string.

### Stable IDs

`stable_id` (`chunk.rs:6-24`) is a 64-bit FNV-1a hash rendered as a 16-character
hex string. The chunker feeds it an explicit, ordered list: tenant ID, document
ID, version, source address, heading path, chunk index and chunk content
(`chunker.rs:163-192`). Re-indexing unchanged text yields the same ID, so point
upserts are idempotent; a change to the version or content yields a new ID and
the index drops the document's previous chunks.

### ACL metadata

`AclMetadata` (`chunk.rs:26-101`) copies the object ACL into every chunk at
index time: `tenant_id`, `is_public`, `allowed_users`, `allowed_roles` and
`allowed_groups`. Retrieval never re-reads the source file. `allows()` first
requires the user's organization to equal `tenant_id`; it then admits public
chunks, authenticated users whose ID or email is listed, or any matching role
or group. A denial is not returned as a "restricted" result — the chunk is
simply absent, so the caller receives neither content nor an existence hint.

### Chunker and in-memory index

`StructureAwareChunker` (`chunker.rs:50-301`) splits extracted text at inline
markers (`[[PAGE:n]]`, `[[SLIDE:n]]`, `[[SHEET:name]]`, `[[CELL:ref]]`), form
feeds and `Page n` lines, Markdown headings and numbered clauses (`7.3`,
`Clause 7.3`, `Section 7`), then windows long sections. Defaults are 900
characters per chunk with 150 characters of overlap (`ChunkOptions`,
`chunker.rs:4-18`). Each chunk keeps its citation and ACL;
`StructuredChunk::to_payload()` (`chunk.rs:128-152`) is the seam for a future
vector indexer and emits `content`, `document_path`, `stable_id`, `tenant_id`,
`citation` and `acl`.

`StructuredKbIndex` (`index.rs:10-177`) is a tenant-scoped in-memory map keyed
by `tenant|document`. Re-indexing removes the previous chunks;
`delete_document` removes chunks and the status row, while `archive_document`
removes chunks but keeps a visible `Archived` status. `retrieve` filters on
tenant and ACL before scoring, scores by query-term overlap rather than
embeddings (`index.rs:189-206`) and returns `RetrievedChunk` values carrying a
`Citation`. `retrieve_with_citations` is the free-function form of the same
call.

### Index status, backfill and test evidence

`IndexStatus` (`index_types.rs:4-10`) is `Indexed`, `Skipped(reason)` or
`Archived`. A `BackfillReport` (`index_types.rs:61-92`) counts total, indexed
and skipped documents, and `accounts_for` checks that every supplied document
appears exactly once.

The inline tests exercise the model's semantics offline: a clause citation with
section and page (`index.rs:259-276`), a denial with no existence hint
(`index.rs:278-289`), replacement on edit (`index.rs:291-318`), immediate
delete/archive (`index.rs:320-337`), backfill accounting including skipped
documents (`index.rs:339-361`) and tenant isolation (`index.rs:363-383`), plus
stable-ID, ACL and payload tests in `chunk.rs`, `citation.rs` and `chunker.rs`.
These are model-level tests, not end-to-end evidence: no server, Drive bucket
or Qdrant collection is involved.

### Still not available

- The index is process-local: it is rebuilt from persisted documents on every
  start-up. There is no Qdrant projection of the structured points and no
  separate index-status or backfill-report UI.
- Only Docs documents are projected today. Slides, Sheets and PDF files are
  not indexed by this structure-aware path.
- There is no opener that highlights a cited page, clause, cell or slide.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Documents not found | Check file is in `.gbkb` folder, verify `USE KB` called |
| Slow indexing | Large PDFs take time; consider splitting documents |
| Outdated content | Set up scheduled crawls for web content |

## See Also

- [Knowledge Base System](./README.md) - Architecture overview
- [Retrieval and RAG](./hybrid-search.md) - How retrieval works
- [Vector Collections](./vector-collections.md) - Collection management