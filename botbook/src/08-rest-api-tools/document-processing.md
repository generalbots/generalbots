# Document Processing API 🟡 BETA

botserver provides RESTful endpoints for processing, extracting, and analyzing various document formats including PDFs, Office documents, and images.

## Overview

The Document Processing API enables:
- Text extraction from documents
- OCR for scanned documents
- Metadata extraction
- Document conversion
- Content analysis and summarization

## Base URL

```
http://localhost:8080/api/documents
```

## Authentication

All Document Processing API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Upload Document

**POST** `/upload`

Upload a document for processing.

**Request:**
- Method: `POST`
- Content-Type: `multipart/form-data`

**Form Data:**
- `file` - The document file
- `process_options` - JSON string of processing options

**Example Request:**
```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -F "file=@document.pdf" \
  -F 'process_options={"extract_text":true,"extract_metadata":true}' \
  http://localhost:8080/api/documents/upload
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "filename": "document.pdf",
  "size_bytes": 2048576,
  "mime_type": "application/pdf",
  "status": "processing",
  "uploaded_at": "2024-01-15T10:00:00Z"
}
```

### Process Document

**POST** `/process`

Process an already uploaded document.

**Request Body:**
```json
{
  "document_id": "doc_abc123",
  "operations": [
    "extract_text",
    "extract_metadata",
    "generate_summary",
    "extract_entities"
  ],
  "options": {
    "language": "en",
    "ocr_enabled": true,
    "chunk_size": 1000
  }
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "process_id": "prc_xyz789",
  "status": "processing",
  "estimated_completion": "2024-01-15T10:02:00Z"
}
```

### Get Processing Status

**GET** `/process/{process_id}/status`

Check the status of document processing.

**Response:**
```json
{
  "process_id": "prc_xyz789",
  "document_id": "doc_abc123",
  "status": "completed",
  "progress": 100,
  "completed_at": "2024-01-15T10:01:30Z",
  "results_available": true
}
```

### Get Extracted Text

**GET** `/documents/{document_id}/text`

Retrieve extracted text from a processed document.

**Query Parameters:**
- `page` - Specific page number (optional)
- `format` - Output format: `plain`, `markdown`, `html`

**Response:**
```json
{
  "document_id": "doc_abc123",
  "text": "This is the extracted text from the document...",
  "pages": 10,
  "word_count": 5420,
  "language": "en"
}
```

### Get Document Metadata

**GET** `/documents/{document_id}/metadata`

Retrieve metadata from a document.

**Response:**
```json
{
  "document_id": "doc_abc123",
  "metadata": {
    "title": "Annual Report 2024",
    "author": "John Doe",
    "created_date": "2024-01-10T08:00:00Z",
    "modified_date": "2024-01-14T16:30:00Z",
    "pages": 10,
    "producer": "Microsoft Word",
    "keywords": ["annual", "report", "finance"],
    "custom_properties": {
      "department": "Finance",
      "confidentiality": "Internal"
    }
  }
}
```

### Generate Summary

**POST** `/documents/{document_id}/summarize`

Generate an AI summary of the document.

**Request Body:**
```json
{
  "type": "abstractive",
  "length": "medium",
  "focus_areas": ["key_points", "conclusions"],
  "language": "en"
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "summary": "This document discusses the annual financial performance...",
  "key_points": [
    "Revenue increased by 15%",
    "New market expansion successful",
    "Operating costs reduced"
  ],
  "summary_length": 250
}
```

### Extract Entities

**POST** `/documents/{document_id}/entities`

Extract named entities from the document.

**Request Body:**
```json
{
  "entity_types": ["person", "organization", "location", "date", "money"],
  "confidence_threshold": 0.7
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "entities": [
    {
      "text": "John Smith",
      "type": "person",
      "confidence": 0.95,
      "occurrences": 5
    },
    {
      "text": "New York",
      "type": "location",
      "confidence": 0.88,
      "occurrences": 3
    },
    {
      "text": "$1.5 million",
      "type": "money",
      "confidence": 0.92,
      "occurrences": 2
    }
  ]
}
```

### Convert Document

**POST** `/documents/{document_id}/convert`

Convert document to another format.

**Request Body:**
```json
{
  "target_format": "pdf",
  "options": {
    "compress": true,
    "quality": "high",
    "page_size": "A4"
  }
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "converted_id": "doc_def456",
  "original_format": "docx",
  "target_format": "pdf",
  "download_url": "/api/documents/doc_def456/download"
}
```

### Search Within Document

**POST** `/documents/{document_id}/search`

Search for text within a document.

**Request Body:**
```json
{
  "query": "revenue growth",
  "case_sensitive": false,
  "whole_words": false,
  "regex": false
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "matches": [
    {
      "page": 3,
      "line": 15,
      "context": "...the company achieved significant revenue growth in Q4...",
      "position": 1247
    },
    {
      "page": 7,
      "line": 8,
      "context": "...projecting continued revenue growth for next year...",
      "position": 3892
    }
  ],
  "total_matches": 2
}
```

### Split Document

**POST** `/documents/{document_id}/split`

Split a document into multiple parts.

**Request Body:**
```json
{
  "method": "by_pages",
  "pages_per_split": 5
}
```

**Response:**
```json
{
  "document_id": "doc_abc123",
  "parts": [
    {
      "part_id": "part_001",
      "pages": "1-5",
      "download_url": "/api/documents/part_001/download"
    },
    {
      "part_id": "part_002",
      "pages": "6-10",
      "download_url": "/api/documents/part_002/download"
    }
  ],
  "total_parts": 2
}
```

### Merge Documents

**POST** `/documents/merge`

Merge multiple documents into one.

**Request Body:**
```json
{
  "document_ids": ["doc_abc123", "doc_def456", "doc_ghi789"],
  "output_format": "pdf",
  "preserve_metadata": true
}
```

**Response:**
```json
{
  "merged_document_id": "doc_merged_xyz",
  "source_count": 3,
  "total_pages": 30,
  "download_url": "/api/documents/doc_merged_xyz/download"
}
```

## Supported Formats

### Input Formats
- **Documents**: PDF, DOCX, DOC, ODT, RTF, TXT
- **Spreadsheets**: XLSX, XLS, ODS, CSV
- **Presentations**: PPTX, PPT, ODP
- **Images**: PNG, JPG, JPEG, GIF, BMP, TIFF
- **Web**: HTML, XML, MARKDOWN

### Output Formats
- PDF
- Plain Text
- Markdown
- HTML
- JSON
- CSV (for tabular data)

## Office interoperability and coexistence (status)

General Bots converts Office/ODF formats locally and preserves original OOXML
packages where it can. It is **not** an Office for the web or Google Workspace
client. This section records what exists in the source today; the remaining
items are open work tracked in issue #2463, not shipped features.

| Capability | Status | Source evidence |
|---|---|---|
| DOCX / ODT import into the Docs model | Implemented (HTML intermediate) | `botserver/crates/botdocs/src/handlers_api/import.rs` maps `docx` to `convert_docx_to_html` and `odt` to `odt_zip_to_html`. |
| DOCX / XLSX round-trip package preservation | Implemented | `botserver/crates/botdocs/src/storage_docx_source.rs` (`DocxFidelity::Preserved`, falls back to `Regenerated`) and the `x-gb-docx-fidelity` export header in `handlers_api/export.rs`; zip-level passthrough in `botserver/crates/botsheet/src/storage/xlsx_passthrough.rs` copies unmodelled parts byte-for-byte. |
| ODF exports: ODT, ODS, ODP | Implemented as hand-built ODF 1.2 packages | `botserver/crates/botdocs/src/utils_ooxml.rs` (ODT), `botserver/crates/botsheet/src/export/ods.rs` (ODS), `botserver/crates/botslides/src/ui/odp.rs` (ODP). Each writes a ZIP with a stored `mimetype` and a manifest declaring version 1.2. There is no ODF 1.3 conformance claim and no LibreOffice round-trip validator in the repository. |
| PDF export | Implemented, built-in Helvetica only | `botserver/crates/botdocs/src/pdf/writer.rs` states that only the built-in Helvetica family is used, so no font embedding is required. Routes are registered in `botserver/crates/botdocs/src/lib.rs`. |
| Google Sheets clipboard paste | Implemented, clipboard HTML table only | `botserver/crates/botsheet/src/handlers/paste_html.rs` parses the HTML table flavour that Excel and Google Sheets place on the clipboard. This is not a Google Docs/Sheets/Slides API import connector. |
| WOPI / Office for the web / Office Online | Not implemented | No `wopi` reference exists in `botserver/src`, `botserver/crates`, `botlib/src` or `botui/ui`. |
| Google Workspace import/export connector with a conversion report | Not implemented | No Google API import handler and no conversion-report producer in the docs, sheet or slides crates. |
| Font substitution, metric-compatible fallback, font embedding on export | Not implemented | No substitution code found; the PDF writer uses only built-in Helvetica and embeds no fonts. |
| Embedded objects / OLE round-trip | Not implemented | No embedded-object handling found. `botserver/crates/botdocs/src/import_security.rs` explicitly never reads non-XML parts such as images and embedded fonts. |
| PDF form fill and save | Not implemented | No form-fill code found. |
| Cross-format compare built on the review engine | Not implemented | No such handler found; tracked in issue #2463. |

Practical consequence: a DOCX or XLSX that is patched in place keeps its
original package, and ODT/ODS/ODP output is produced locally. However, a file
cannot be opened and saved in Office for the web through WOPI, a Google Docs
file cannot be imported through a connector, a missing corporate font is not
substituted with a metric-compatible fallback, and an embedded workbook inside
a PPTX is not preserved as an object. Those are the coexistence gaps described
in issue #2463.

## Processing Options

### OCR Options
```json
{
  "ocr_enabled": true,
  "ocr_language": "eng",
  "ocr_engine": "tesseract",
  "preprocessing": {
    "deskew": true,
    "remove_noise": true,
    "enhance_contrast": true
  }
}
```

### Text Extraction Options
```json
{
  "preserve_formatting": false,
  "extract_tables": true,
  "extract_images": false,
  "chunk_text": true,
  "chunk_size": 1000,
  "chunk_overlap": 100
}
```

### Summary Options
```json
{
  "summary_type": "extractive",
  "summary_length": "medium",
  "bullet_points": true,
  "include_keywords": true,
  "max_sentences": 5
}
```

## Batch Processing

### Submit Batch

**POST** `/batch/process`

Process multiple documents in batch.

**Request Body:**
```json
{
  "documents": [
    {
      "document_id": "doc_001",
      "operations": ["extract_text", "summarize"]
    },
    {
      "document_id": "doc_002",
      "operations": ["extract_entities"]
    }
  ],
  "notify_on_completion": true,
  "webhook_url": "https://example.com/webhook"
}
```

### Get Batch Status

**GET** `/batch/{batch_id}/status`

Check batch processing status.

**Response:**
```json
{
  "batch_id": "batch_abc123",
  "total_documents": 10,
  "processed": 7,
  "failed": 1,
  "pending": 2,
  "completion_percentage": 70
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "unsupported_format",
  "message": "File format .xyz is not supported",
  "supported_formats": ["pdf", "docx", "txt"]
}
```

### 413 Payload Too Large
```json
{
  "error": "file_too_large",
  "message": "File size exceeds maximum limit",
  "max_size_bytes": 52428800,
  "provided_size_bytes": 104857600
}
```

### 422 Unprocessable Entity
```json
{
  "error": "corrupted_file",
  "message": "The document appears to be corrupted and cannot be processed"
}
```

## Webhooks

Configure webhooks to receive processing notifications:

```json
{
  "event": "document.processed",
  "document_id": "doc_abc123",
  "status": "completed",
  "results": {
    "text_extracted": true,
    "summary_generated": true,
    "entities_extracted": true
  }
}
```

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Upload Document | 50/hour | Per user |
| Process Document | 100/hour | Per user |
| Generate Summary | 20/hour | Per user |
| Batch Processing | 5/hour | Per user |

## Best Practices

1. **Preprocess Documents**: Clean scanned documents before OCR
2. **Use Appropriate Formats**: Choose the right output format for your use case
3. **Batch Similar Documents**: Process similar documents together for efficiency
4. **Handle Large Files**: Use chunking for large documents
5. **Cache Results**: Store processed results to avoid reprocessing
6. **Monitor Processing**: Use webhooks for long-running operations

## Integration Examples

### Python Example

```python
import requests

# Upload and process document
with open('document.pdf', 'rb') as f:
    response = requests.post(
        'http://localhost:8080/api/documents/upload',
        headers={'Authorization': 'Bearer token123'},
        files={'file': f},
        data={'process_options': '{"extract_text": true}'}
    )
    
document_id = response.json()['document_id']

# Get extracted text
text_response = requests.get(
    f'http://localhost:8080/api/documents/{document_id}/text',
    headers={'Authorization': 'Bearer token123'}
)

print(text_response.json()['text'])
```

## Related APIs

- [Storage API](./storage-api.md) - Document storage
- [ML API](./ml-api.md) - Advanced text analysis
- [Knowledge Base API](../03-knowledge-ai/kb-and-tools.md) - Document indexing