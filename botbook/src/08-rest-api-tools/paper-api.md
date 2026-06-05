# Paper API

> **Lightweight notes API for quick document creation, AI-powered editing, templates, and multi-format export.**

---

## Base URL

```
/api/ui/paper
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### New Note

**`GET /api/ui/paper/new`**

Creates a new blank note.

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Untitled Note",
  "content": "",
  "created_at": "2026-01-20T15:00:00Z",
  "updated_at": "2026-01-20T15:00:00Z"
}
```

---

### List Notes

**`GET /api/ui/paper/list`**

Returns a list of all notes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `sort` | string | No | Sort: `created_at`, `updated_at`, `name` |
| `order` | string | No | `asc` or `desc` (default: `desc`) |
| `tag` | string | No | Filter by tag |

**Response:**
```json
{
  "notes": [
    {
      "id": "uuid-string",
      "name": "Meeting Notes - Jan 20",
      "preview": "Discussed Q1 targets and team allocation...",
      "word_count": 320,
      "tags": ["meeting", "q1"],
      "pinned": true,
      "created_at": "2026-01-20T10:30:00Z",
      "updated_at": "2026-01-20T14:45:00Z"
    }
  ],
  "total": 48,
  "page": 1,
  "limit": 20
}
```

---

### Search Notes

**`GET /api/ui/paper/search`**

Search notes by name or content.

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
      "name": "Meeting Notes - Jan 20",
      "match_score": 0.88,
      "snippet": "...Q1 revenue targets set at R$ 500K...",
      "tags": ["meeting", "q1"]
    }
  ],
  "total": 5
}
```

---

### Save Note

**`POST /api/ui/paper/save`**

Creates a new note or updates an existing one.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | No | Note ID (omit for new) |
| `name` | string | Yes | Note title |
| `content` | string | Yes | Note content (plain text or Markdown) |
| `tags` | array | No | Array of tag strings |

**Request Body:**
```json
{
  "name": "Meeting Notes - Jan 20",
  "content": "# Meeting Notes\n\n## Attendees\n- Ana\n- Carlos\n\n## Agenda\n1. Q1 targets\n2. Team allocation\n3. Budget review",
  "tags": ["meeting", "q1"]
}
```

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Meeting Notes - Jan 20",
  "saved_at": "2026-01-20T15:00:00Z",
  "word_count": 320
}
```

---

### Autosave Note

**`POST /api/ui/paper/autosave`**

Auto-saves note content (debounced, lightweight).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |
| `content` | string | Yes | Note content |

**Response:**
```json
{
  "autosaved": true,
  "saved_at": "2026-01-20T15:01:00Z"
}
```

---

### Get Note by ID

**`GET /api/ui/paper/:id`**

Returns a single note with full content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Note ID |

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Meeting Notes - Jan 20",
  "content": "# Meeting Notes\n\n## Attendees\n- Ana\n- Carlos\n\n## Agenda\n1. Q1 targets\n2. Team allocation\n3. Budget review",
  "tags": ["meeting", "q1"],
  "pinned": false,
  "word_count": 320,
  "created_at": "2026-01-20T10:30:00Z",
  "updated_at": "2026-01-20T14:45:00Z"
}
```

---

### Delete Note

**`POST /api/ui/paper/:id/delete`**

Deletes a note permanently.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Note ID to delete |

**Response:**
```json
{
  "deleted": true,
  "id": "uuid-string"
}
```

---

## Templates

### Get Blank Template

**`GET /api/ui/paper/template/blank`**

Returns a blank note template.

**Response:**
```json
{
  "name": "Blank Note",
  "content": ""
}
```

---

### Get Meeting Template

**`GET /api/ui/paper/template/meeting`**

Returns a meeting notes template.

**Response:**
```json
{
  "name": "Meeting Notes",
  "content": "# Meeting Notes\n\n**Date:** \n**Attendees:** \n**Location:** \n\n## Agenda\n\n- \n- \n- \n\n## Discussion\n\n\n\n## Action Items\n\n- [ ] \n- [ ] \n\n## Next Meeting\n\n**Date:** \n**Topics:** "
}
```

---

### Get Todo Template

**`GET /api/ui/paper/template/todo`**

Returns a todo list template.

**Response:**
```json
{
  "name": "Todo List",
  "content": "# Todo List\n\n## High Priority\n\n- [ ] \n- [ ] \n\n## Medium Priority\n\n- [ ] \n- [ ] \n\n## Low Priority\n\n- [ ] \n- [ ] \n\n## Completed\n\n- [x] "
}
```

---

### Get Research Template

**`GET /api/ui/paper/template/research`**

Returns a research notes template.

**Response:**
```json
{
  "name": "Research Notes",
  "content": "# Research: [Topic]\n\n## Objective\n\n\n\n## Key Findings\n\n1. \n2. \n3. \n\n## Sources\n\n- \n- \n\n## Questions\n\n- \n\n## Next Steps\n\n- \n"
}
```

---

### Get Report Template

**`GET /api/ui/paper/template/report`**

Returns a simple report template.

**Response:**
```json
{
  "name": "Report",
  "content": "# [Report Title]\n\n**Author:** \n**Date:** \n\n## Executive Summary\n\n\n\n## Introduction\n\n\n\n## Analysis\n\n\n\n## Conclusion\n\n\n\n## Recommendations\n\n1. \n2. \n3. "
}
```

---

### Get Letter Template

**`GET /api/ui/paper/template/letter`**

Returns a letter template.

**Response:**
```json
{
  "name": "Letter",
  "content": "[Your Name]\n[Your Address]\n[City, State ZIP]\n[Date]\n\n[Recipient Name]\n[Recipient Address]\n[City, State ZIP]\n\nDear [Recipient],\n\n[Body of letter]\n\nSincerely,\n[Your Name]"
}
```

---

## AI Operations

### AI Summarize

**`POST /api/ui/paper/ai/summarize`**

Generates a summary of the note content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to summarize |
| `max_words` | integer | No | Maximum summary length (default: 100) |

**Request Body:**
```json
{
  "content": "Long note content about Q1 meeting...",
  "max_words": 75
}
```

**Response:**
```json
{
  "summary": "Q1 meeting covered revenue targets (R$ 500K), team reallocation to Project X, and budget approval of R$ 50K for marketing.",
  "word_count": 28
}
```

---

### AI Expand

**`POST /api/ui/paper/ai/expand`**

Expands a short note or bullet points into fuller prose.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to expand |
| `context` | string | No | Additional context |

**Request Body:**
```json
{
  "content": "- Revenue up 15%\n- New clients: 50\n- Retention: 92%",
  "context": "Q1 performance summary for board meeting"
}
```

**Response:**
```json
{
  "expanded": "In Q1, revenue increased by 15% compared to the previous quarter, driven primarily by acquiring 50 new clients. Additionally, our client retention rate remained strong at 92%, demonstrating the quality of our service delivery."
}
```

---

### AI Improve

**`POST /api/ui/paper/ai/improve`**

Improves writing quality and clarity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to improve |
| `focus` | string | No | `grammar`, `clarity`, `tone`, `all` (default: `all`) |

**Request Body:**
```json
{
  "content": "we need to get the thing done by friday and make sure its good",
  "focus": "grammar"
}
```

**Response:**
```json
{
  "improved": "We need to complete the task by Friday and ensure it meets quality standards.",
  "changes_made": ["Capitalized sentence start", "Added possessive clarity", "Corrected grammar"]
}
```

---

### AI Simplify

**`POST /api/ui/paper/ai/simplify`**

Simplifies complex text for easier comprehension.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to simplify |
| `reading_level` | string | No | `elementary`, `intermediate`, `advanced` |

**Request Body:**
```json
{
  "content": "The implementation of the strategic initiative necessitates comprehensive stakeholder engagement.",
  "reading_level": "intermediate"
}
```

**Response:**
```json
{
  "simplified": "To implement this plan, we need to involve all stakeholders."
}
```

---

### AI Translate

**`POST /api/ui/paper/ai/translate`**

Translates note content to another language.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to translate |
| `target_language` | string | Yes | Target language code |
| `source_language` | string | No | Source language (auto-detect if omitted) |

**Request Body:**
```json
{
  "content": "Reunião agendada para segunda-feira às 14h.",
  "target_language": "en"
}
```

**Response:**
```json
{
  "translated": "Meeting scheduled for Monday at 2 PM.",
  "source_language": "pt",
  "target_language": "en"
}
```

---

### AI Custom

**`POST /api/ui/paper/ai/custom`**

Applies a custom AI instruction to the note.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | Yes | Text to process |
| `instruction` | string | Yes | Custom instruction |

**Request Body:**
```json
{
  "content": "Meeting with client went well. They liked the proposal.",
  "instruction": "Make this more professional and add specific details"
}
```

**Response:**
```json
{
  "result": "The client meeting was productive. The client expressed strong approval of the proposed solution, particularly highlighting the scalability features and competitive pricing structure.",
  "instruction_applied": true
}
```

---

## Export

### Export as PDF

**`GET /api/ui/paper/export/pdf`**

Exports the note as a PDF file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |

**Response:**
```json
{
  "download_url": "/api/ui/paper/export/uuid/pdf",
  "filename": "Meeting Notes - Jan 20.pdf",
  "size_bytes": 48128
}
```

---

### Export as DOCX

**`GET /api/ui/paper/export/docx`**

Exports the note as a Word document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |

**Response:**
```json
{
  "download_url": "/api/ui/paper/export/uuid/docx",
  "filename": "Meeting Notes - Jan 20.docx",
  "size_bytes": 32768
}
```

---

### Export as Markdown

**`GET /api/ui/paper/export/md`**

Returns the note content in Markdown format.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |

**Response:**
```json
{
  "download_url": "/api/ui/paper/export/uuid/md",
  "filename": "Meeting Notes - Jan 20.md",
  "content": "# Meeting Notes\n\n## Attendees\n- Ana\n- Carlos\n\n## Agenda\n1. Q1 targets"
}
```

---

### Export as HTML

**`GET /api/ui/paper/export/html`**

Exports the note as an HTML file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |

**Response:**
```json
{
  "download_url": "/api/ui/paper/export/uuid/html",
  "filename": "Meeting Notes - Jan 20.html",
  "size_bytes": 16384
}
```

---

### Export as Plain Text

**`GET /api/ui/paper/export/txt`**

Returns the note as plain text.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Note ID |

**Response:**
```json
{
  "download_url": "/api/ui/paper/export/uuid/txt",
  "filename": "Meeting Notes - Jan 20.txt",
  "content": "Meeting Notes\n\nAttendees:\n- Ana\n- Carlos\n\nAgenda:\n1. Q1 targets"
}
```

---

## See Also

- [Docs API](docs-api.md) - Rich-text word processor
- [Slides API](slides-api.md) - Presentation creation
- [Research API](research-api.md) - Research collections
