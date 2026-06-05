# Docs API

> **Word processor API for creating, editing, and collaborating on rich-text documents with AI assistance, commenting, track changes, and multi-format export.**

---

## Base URL

```
/api/docs
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Documents

**`GET /api/docs/list`**

Returns a list of all documents accessible to the authenticated user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `sort` | string | No | Sort: `created_at`, `updated_at`, `name` |
| `order` | string | No | `asc` or `desc` (default: `desc`) |

**Response:**
```json
{
  "documents": [
    {
      "id": "uuid-string",
      "name": "Project Proposal",
      "word_count": 2450,
      "page_count": 5,
      "created_at": "2026-01-15T10:30:00Z",
      "updated_at": "2026-01-20T14:45:00Z",
      "has_comments": true,
      "has_track_changes": false
    }
  ],
  "total": 34
}
```

---

### Search Documents

**`GET /api/docs/search`**

Full-text search across document content and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Search query |
| `limit` | integer | No | Max results (default: 10) |

**Response:**
```json
{
  "results": [
    {
      "id": "uuid-string",
      "name": "Project Proposal",
      "match_score": 0.92,
      "snippet": "...the proposed budget of R$ 50,000 covers...",
      "match_count": 3
    }
  ],
  "total": 8
}
```

---

### Load Document

**`GET /api/docs/load`**

Loads the full document with all content, formatting, and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Project Proposal",
  "content": {
    "type": "doc",
    "content": [
      {
        "type": "heading",
        "attrs": {"level": 1},
        "content": [{"type": "text", "text": "Project Proposal"}]
      },
      {
        "type": "paragraph",
        "content": [{"type": "text", "text": "This document outlines the project scope..."}]
      }
    ]
  },
  "metadata": {
    "author": "user-uuid",
    "version": 7,
    "word_count": 2450,
    "page_count": 5
  },
  "track_changes_enabled": false,
  "comments_count": 3
}
```

---

### Save Document

**`POST /api/docs/save`**

Saves document content and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | No | Document ID (omit for new) |
| `name` | string | Yes | Document name |
| `content` | object | Yes | Document content (ProseMirror/JSON format) |

**Request Body:**
```json
{
  "name": "Project Proposal",
  "content": {
    "type": "doc",
    "content": [
      {
        "type": "heading",
        "attrs": {"level": 1},
        "content": [{"type": "text", "text": "Project Proposal"}]
      }
    ]
  }
}
```

**Response:**
```json
{
  "id": "uuid-string",
  "saved_at": "2026-01-20T15:00:00Z",
  "version": 8
}
```

---

### Autosave Document

**`POST /api/docs/autosave`**

Saves document content automatically (debounced, no version increment).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |
| `content` | object | Yes | Document content |

**Response:**
```json
{
  "autosaved": true,
  "saved_at": "2026-01-20T15:01:00Z"
}
```

---

### Delete Document

**`POST /api/docs/delete`**

Deletes a document permanently.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID to delete |

**Response:**
```json
{
  "deleted": true,
  "id": "uuid-string"
}
```

---

### New Document

**`GET /api/docs/new`**

Creates a new blank document.

**Response:**
```json
{
  "id": "new-uuid-string",
  "name": "Untitled Document",
  "content": {
    "type": "doc",
    "content": [
      {"type": "paragraph", "content": []}
    ]
  },
  "created_at": "2026-01-20T15:00:00Z"
}
```

---

### AI Generate

**`POST /api/docs/ai`**

Generates document content using AI based on a prompt.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `prompt` | string | Yes | Content generation prompt |
| `document_id` | string | No | Existing document to append to |
| `length` | string | No | Desired length: `short`, `medium`, `long` |

**Request Body:**
```json
{
  "prompt": "Write a professional project proposal for a mobile app development project",
  "length": "long"
}
```

**Response:**
```json
{
  "content": {
    "type": "doc",
    "content": [...]
  },
  "word_count": 1200,
  "generation_time_ms": 4200
}
```

---

### Get Document by ID

**`GET /api/docs/:id`**

Returns document metadata and content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Document ID |

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Project Proposal",
  "content": {...},
  "metadata": {
    "author": "user-uuid",
    "version": 7,
    "word_count": 2450
  }
}
```

---

### Get Blank Template

**`GET /api/docs/template/blank`**

Returns a blank document template.

**Response:**
```json
{
  "name": "Blank Document",
  "content": {
    "type": "doc",
    "content": [
      {"type": "paragraph", "content": []}
    ]
  }
}
```

---

### Get Meeting Template

**`GET /api/docs/template/meeting`**

Returns a meeting notes template.

**Response:**
```json
{
  "name": "Meeting Notes",
  "content": {
    "type": "doc",
    "content": [
      {
        "type": "heading",
        "attrs": {"level": 1},
        "content": [{"type": "text", "text": "Meeting Notes"}]
      },
      {
        "type": "paragraph",
        "content": [{"type": "text", "text": "Date: \nAttendees: \nLocation: "}]
      },
      {
        "type": "heading",
        "attrs": {"level": 2},
        "content": [{"type": "text", "text": "Agenda"}]
      },
      {"type": "bulletList", "content": []},
      {
        "type": "heading",
        "attrs": {"level": 2},
        "content": [{"type": "text", "text": "Action Items"}]
      },
      {"type": "bulletList", "content": []}
    ]
  }
}
```

---

### Get Report Template

**`GET /api/docs/template/report`**

Returns a report template with standard sections.

**Response:**
```json
{
  "name": "Report",
  "content": {
    "type": "doc",
    "content": [
      {
        "type": "heading",
        "attrs": {"level": 1},
        "content": [{"type": "text", "text": "Report Title"}]
      },
      {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Executive Summary"}]},
      {"type": "paragraph", "content": []},
      {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Introduction"}]},
      {"type": "paragraph", "content": []},
      {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Methodology"}]},
      {"type": "paragraph", "content": []},
      {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Results"}]},
      {"type": "paragraph", "content": []},
      {"type": "heading", "attrs": {"level": 2}, "content": [{"type": "text", "text": "Conclusion"}]},
      {"type": "paragraph", "content": []}
    ]
  }
}
```

---

### Get Letter Template

**`GET /api/docs/template/letter`**

Returns a formal letter template.

**Response:**
```json
{
  "name": "Formal Letter",
  "content": {
    "type": "doc",
    "content": [
      {"type": "paragraph", "content": [{"type": "text", "text": "[Your Name]\n[Your Address]\n[City, State ZIP]\n[Date]"}]},
      {"type": "paragraph", "content": []},
      {"type": "paragraph", "content": [{"type": "text", "text": "[Recipient Name]\n[Recipient Address]\n[City, State ZIP]"}]},
      {"type": "paragraph", "content": []},
      {"type": "paragraph", "content": [{"type": "text", "text": "Dear [Recipient],"}]},
      {"type": "paragraph", "content": []},
      {"type": "paragraph", "content": [{"type": "text", "text": "[Body of letter]"}]},
      {"type": "paragraph", "content": []},
      {"type": "paragraph", "content": [{"type": "text", "text": "Sincerely,\n[Your Name]"}]}
    ]
  }
}
```

---

### AI Summarize

**`POST /api/docs/ai/summarize`**

Generates a concise summary of the document content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |
| `max_words` | integer | No | Maximum summary length (default: 200) |

**Request Body:**
```json
{
  "id": "uuid-string",
  "max_words": 150
}
```

**Response:**
```json
{
  "summary": "This document proposes a mobile app development project with a budget of R$ 50,000 and a 3-month timeline. Key features include user authentication, real-time notifications, and payment integration.",
  "word_count": 42
}
```

---

### AI Expand

**`POST /api/docs/ai/expand`**

Expands selected text with additional detail and elaboration.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to expand |
| `context` | string | No | Additional context for expansion |

**Request Body:**
```json
{
  "text": "The app will use modern technology.",
  "context": "Focus on React Native and cloud infrastructure"
}
```

**Response:**
```json
{
  "expanded": "The app will leverage React Native for cross-platform development, ensuring consistent user experience across iOS and Android devices. Backend services will be hosted on a scalable cloud infrastructure using containerized microservices, providing high availability and automatic scaling during peak usage periods."
}
```

---

### AI Improve

**`POST /api/docs/ai/improve`**

Improves writing quality, grammar, and clarity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to improve |
| `focus` | string | No | Focus area: `grammar`, `clarity`, `tone`, `all` |

**Request Body:**
```json
{
  "text": "We need to do the thing with the app and make it good for users.",
  "focus": "clarity"
}
```

**Response:**
```json
{
  "improved": "We need to develop the mobile application with a focus on user experience and functionality.",
  "changes_made": ["Improved sentence clarity", "Removed vague language", "Added specificity"]
}
```

---

### AI Simplify

**`POST /api/docs/ai/simplify`**

Simplifies complex text for easier comprehension.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to simplify |
| `reading_level` | string | No | Target level: `elementary`, `intermediate`, `advanced` |

**Request Body:**
```json
{
  "text": "The implementation of the architectural framework necessitates comprehensive utilization of state-of-the-art technological paradigms.",
  "reading_level": "intermediate"
}
```

**Response:**
```json
{
  "simplified": "The framework design requires using the latest technology tools and methods."
}
```

---

### AI Translate

**`POST /api/docs/ai/translate`**

Translates document content to a target language.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to translate |
| `target_language` | string | Yes | Target language code |
| `source_language` | string | No | Source language (auto-detect if omitted) |

**Request Body:**
```json
{
  "text": "Bem-vindos ao nosso aplicativo móvel.",
  "target_language": "en"
}
```

**Response:**
```json
{
  "translated": "Welcome to our mobile application.",
  "source_language": "pt",
  "target_language": "en"
}
```

---

### AI Custom

**`POST /api/docs/ai/custom`**

Applies a custom AI instruction to the document text.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `text` | string | Yes | Text to process |
| `instruction` | string | Yes | Custom AI instruction |

**Request Body:**
```json
{
  "text": "The project deadline is March 15.",
  "instruction": "Make this more urgent and add consequences for missing the deadline"
}
```

**Response:**
```json
{
  "result": "The project deadline is March 15 — any delay beyond this date will result in additional costs of R$ 5,000 per week and potential loss of the client contract.",
  "instruction_applied": true
}
```

---

### Export as PDF

**`GET /api/docs/export/pdf`**

Exports the document as a PDF file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "download_url": "/api/docs/export/uuid/pdf",
  "filename": "Project Proposal.pdf",
  "size_bytes": 156432
}
```

---

### Export as DOCX

**`GET /api/docs/export/docx`**

Exports the document as a Microsoft Word file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "download_url": "/api/docs/export/uuid/docx",
  "filename": "Project Proposal.docx",
  "size_bytes": 98304
}
```

---

### Export as Markdown

**`GET /api/docs/export/md`**

Exports the document as Markdown format.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "download_url": "/api/docs/export/uuid/md",
  "filename": "Project Proposal.md",
  "content": "# Project Proposal\n\nThis document outlines..."
}
```

---

### Export as HTML

**`GET /api/docs/export/html`**

Exports the document as a standalone HTML file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "download_url": "/api/docs/export/uuid/html",
  "filename": "Project Proposal.html",
  "size_bytes": 45056
}
```

---

### Export as Plain Text

**`GET /api/docs/export/txt`**

Exports the document as plain text.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Document ID |

**Response:**
```json
{
  "download_url": "/api/docs/export/uuid/txt",
  "filename": "Project Proposal.txt",
  "content": "Project Proposal\n\nThis document outlines..."
}
```

---

### Import Document

**`POST /api/docs/import`**

Imports a document from an external file format.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file` | binary | Yes | File to import (docx, pdf, html, md, txt) |
| `name` | string | No | Document name (default: filename) |

**Response:**
```json
{
  "id": "imported-uuid",
  "name": "Imported Document",
  "format_detected": "docx",
  "word_count": 3200,
  "imported_at": "2026-01-20T15:05:00Z"
}
```

---

### Add Comment

**`POST /api/docs/comment`**

Adds a comment to the document at a specific position.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `text` | string | Yes | Comment text |
| `anchor_from` | integer | Yes | Start position in document |
| `anchor_to` | integer | Yes | End position in document |

**Request Body:**
```json
{
  "document_id": "uuid-string",
  "text": "Please verify these numbers with the finance team.",
  "anchor_from": 245,
  "anchor_to": 280
}
```

**Response:**
```json
{
  "comment_id": "comment-uuid",
  "author": "user-uuid",
  "created_at": "2026-01-20T15:10:00Z",
  "text": "Please verify these numbers with the finance team."
}
```

---

### Reply to Comment

**`POST /api/docs/comment/reply`**

Adds a reply to an existing comment.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `comment_id` | string | Yes | Parent comment ID |
| `text` | string | Yes | Reply text |

**Request Body:**
```json
{
  "comment_id": "comment-uuid",
  "text": "Confirmed with finance. Numbers are correct."
}
```

**Response:**
```json
{
  "reply_id": "reply-uuid",
  "comment_id": "comment-uuid",
  "author": "user-uuid",
  "text": "Confirmed with finance. Numbers are correct.",
  "created_at": "2026-01-20T15:15:00Z"
}
```

---

### Resolve Comment

**`POST /api/docs/comment/resolve`**

Marks a comment thread as resolved.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `comment_id` | string | Yes | Comment ID to resolve |

**Response:**
```json
{
  "resolved": true,
  "comment_id": "comment-uuid",
  "resolved_at": "2026-01-20T15:20:00Z",
  "resolved_by": "user-uuid"
}
```

---

### Delete Comment

**`POST /api/docs/comment/delete`**

Deletes a comment and all its replies.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `comment_id` | string | Yes | Comment ID to delete |

**Response:**
```json
{
  "deleted": true,
  "comment_id": "comment-uuid"
}
```

---

### Get Comments

**`GET /api/docs/comments`**

Returns all comments for a document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `include_resolved` | boolean | No | Include resolved comments (default: false) |

**Response:**
```json
{
  "comments": [
    {
      "id": "comment-uuid",
      "text": "Please verify these numbers with the finance team.",
      "author": "user-uuid",
      "author_name": "Ana",
      "anchor_from": 245,
      "anchor_to": 280,
      "created_at": "2026-01-20T15:10:00Z",
      "resolved": false,
      "replies": [
        {
          "id": "reply-uuid",
          "text": "Confirmed with finance. Numbers are correct.",
          "author": "user-uuid-2",
          "author_name": "Carlos",
          "created_at": "2026-01-20T15:15:00Z"
        }
      ]
    }
  ],
  "total": 5,
  "unresolved_count": 2
}
```

---

### Enable Track Changes

**`POST /api/docs/track-changes/enable`**

Enables track changes mode for the document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `enabled` | boolean | Yes | Enable or disable tracking |

**Request Body:**
```json
{
  "document_id": "uuid-string",
  "enabled": true
}
```

**Response:**
```json
{
  "track_changes_enabled": true,
  "document_id": "uuid-string"
}
```

---

### Accept/Reject Track Changes

**`POST /api/docs/track-changes/accept-reject`**

Accepts or rejects a specific track change.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `change_id` | string | Yes | Track change ID |
| `action` | string | Yes | `accept` or `reject` |

**Request Body:**
```json
{
  "document_id": "uuid-string",
  "change_id": "change-uuid",
  "action": "accept"
}
```

**Response:**
```json
{
  "change_id": "change-uuid",
  "action": "accepted",
  "document_version": 9
}
```

---

### Get Track Changes

**`GET /api/docs/track-changes`**

Returns all track changes for a document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |

**Response:**
```json
{
  "track_changes_enabled": true,
  "changes": [
    {
      "id": "change-uuid",
      "type": "insert",
      "author": "user-uuid",
      "author_name": "Ana",
      "position": 245,
      "content": "additional context",
      "created_at": "2026-01-20T15:10:00Z",
      "status": "pending"
    },
    {
      "id": "change-uuid-2",
      "type": "delete",
      "author": "user-uuid-2",
      "author_name": "Carlos",
      "position": 310,
      "content": "old text",
      "created_at": "2026-01-20T15:12:00Z",
      "status": "pending"
    }
  ],
  "total": 7,
  "pending_count": 4
}
```

---

### Generate Table of Contents

**`POST /api/docs/toc/generate`**

Automatically generates a table of contents from document headings.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |

**Response:**
```json
{
  "toc": [
    {"level": 1, "title": "Executive Summary", "position": 0},
    {"level": 2, "title": "Project Scope", "position": 150},
    {"level": 2, "title": "Timeline", "position": 420},
    {"level": 3, "title": "Phase 1", "position": 520},
    {"level": 3, "title": "Phase 2", "position": 680},
    {"level": 1, "title": "Budget", "position": 900},
    {"level": 1, "title": "Conclusion", "position": 1200}
  ],
  "inserted_at_position": 0
}
```

---

### Add Footnote

**`POST /api/docs/footnote`**

Adds a footnote to the document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `text` | string | Yes | Footnote text |
| `position` | integer | Yes | Position in document to attach footnote |

**Request Body:**
```json
{
  "document_id": "uuid-string",
  "text": "Source: IBGE Census 2026",
  "position": 450
}
```

**Response:**
```json
{
  "footnote_id": "fn-uuid",
  "number": 1,
  "text": "Source: IBGE Census 2026",
  "position": 450
}
```

---

### Add Endnote

**`POST /api/docs/endnote`**

Adds an endnote to the document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `document_id` | string | Yes | Document ID |
| `text` | string | Yes | Endnote text |
| `position` | integer | Yes | Position in document to attach endnote |

**Request Body:**
```json
{
  "document_id": "uuid-string",
  "text": "Full methodology available upon request.",
  "position": 1200
}
```

**Response:**
```json
{
  "endnote_id": "en-uuid",
  "number": 1,
  "text": "Full methodology available upon request.",
  "position": 1200
}
```

---

### Get Styles

**`GET /api/docs/styles`**

Returns available document styles and formatting options.

**Response:**
```json
{
  "paragraph_styles": [
    {"id": "normal", "name": "Normal", "font": "Calibri", "size": 11},
    {"id": "heading-1", "name": "Heading 1", "font": "Calibri", "size": 24, "bold": true},
    {"id": "heading-2", "name": "Heading 2", "font": "Calibri", "size": 18, "bold": true},
    {"id": "heading-3", "name": "Heading 3", "font": "Calibri", "size": 14, "bold": true},
    {"id": "quote", "name": "Quote", "font": "Georgia", "size": 12, "italic": true}
  ],
  "character_styles": [
    {"id": "bold", "name": "Bold", "bold": true},
    {"id": "italic", "name": "Italic", "italic": true},
    {"id": "underline", "name": "Underline", "underline": true}
  ],
  "colors": [
    {"id": "primary", "hex": "#1a365d"},
    {"id": "secondary", "hex": "#2d3748"},
    {"id": "accent", "hex": "#3182ce"}
  ]
}
```

---

### Create Custom Style

**`POST /api/docs/style`**

Creates or updates a custom document style.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Style name |
| `font` | string | No | Font family |
| `size` | integer | No | Font size in pt |
| `bold` | boolean | No | Bold text |
| `italic` | boolean | No | Italic text |
| `color` | string | No | Text color hex |

**Request Body:**
```json
{
  "name": "Custom Header",
  "font": "Roboto",
  "size": 16,
  "bold": true,
  "color": "#2d3748"
}
```

**Response:**
```json
{
  "style_id": "style-uuid",
  "name": "Custom Header",
  "created": true
}
```

---

### WebSocket Connection

**`GET /api/docs/ws/:doc_id`** (WebSocket)

Real-time collaborative editing WebSocket endpoint.

**Connection:**
```
ws://localhost:8080/api/docs/ws/{doc_id}
```

**Messages sent (client to server):**
```json
{
  "type": "update",
  "version": 7,
  "changes": [...]
}
```

```json
{
  "type": "cursor",
  "position": 245
}
```

**Messages received (server to client):**
```json
{
  "type": "update",
  "version": 8,
  "changes": [...],
  "author": "user-uuid"
}
```

```json
{
  "type": "cursor",
  "user": "Ana",
  "position": 310,
  "color": "#3182ce"
}
```

```json
{
  "type": "comment",
  "comment": {
    "id": "comment-uuid",
    "text": "New comment added",
    "anchor_from": 245,
    "anchor_to": 280
  }
}
```

---

## See Also

- [Slides API](slides-api.md) - Presentation creation
- [Paper API](paper-api.md) - Lightweight notes
- [Video API](video-api.md) - Video editing
- [Research API](research-api.md) - Research collections
