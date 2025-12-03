# Missing Implementations - UI Apps Backend Integration

## Status Summary

**Frontend (HTML/JS)**: ✅ COMPLETE - All UI shells exist
**Backend (Rust APIs)**: 🔴 INCOMPLETE - Missing handlers

| App | HTML | JavaScript | Backend Routes | Status |
|-----|------|-----------|------------------|--------|
| Chat | ✅ | ✅ basic | ✅ /api/sessions, /ws | COMPLETE |
| Drive | ✅ | ✅ basic | ✅ /api/drive/* | COMPLETE |
| Tasks | ✅ | ✅ basic | ✅ /api/tasks/* | COMPLETE |
| Mail | ✅ | ✅ basic | ✅ /api/email/* | COMPLETE |
| Calendar | ✅ | ✅ basic | ✅ CalDAV, /api/calendar/* | COMPLETE |
| Meet | ✅ | ✅ basic | ✅ /api/meet/*, /ws/meet | COMPLETE |
| **Analytics** | ✅ | ✅ forms | ❌ NONE | **NEEDS BACKEND** |
| **Paper** | ✅ | ✅ editor | ❌ NONE | **NEEDS BACKEND** |
| **Research** | ✅ | ✅ search | ✅ /api/kb/search | **PARTIAL** |
| **Designer** | ✅ | ✅ builder | ❌ NONE | **NEEDS BACKEND** |
| **Sources** | ✅ | ✅ list | ❌ NONE | **NEEDS BACKEND** |
| Monitoring | ✅ | ✅ dashboard | ✅ /api/admin/stats | COMPLETE |

---

## Backend Endpoints That Need Implementation

### 1. Analytics Dashboard (`/api/analytics/`)

**Current URL Definition**:
- `/api/analytics/dashboard` - GET
- `/api/analytics/metric` - GET

**Needed Endpoints**:
```rust
GET  /api/analytics/dashboard?timeRange=day|week|month|year
     → Returns HTML: dashboard cards with metrics

GET  /api/analytics/sessions?start_date=&end_date=
     → Returns HTML: session analytics table

GET  /api/analytics/bots?bot_id=&timeRange=
     → Returns HTML: bot performance metrics

GET  /api/analytics/top-queries
     → Returns HTML: trending queries list

GET  /api/analytics/error-rate?timeRange=
     → Returns HTML: error statistics
```

**Backend Logic Needed**:
- Query `message_history` table for message counts
- Calculate aggregates from `sessions` table  
- Fetch system metrics from monitoring
- Use database connection pool in `AppState`
- Return Askama template rendered as HTML

---

### 2. Paper App - Document Management (`/api/documents/`)

**Needed Endpoints**:
```rust
POST /api/documents
     { title, content, type: "draft" | "note" | "template" }
     → Returns: Document ID + status

GET  /api/documents
     → Returns HTML: document list with previews

GET  /api/documents/:id
     → Returns HTML: full document content

PUT  /api/documents/:id
     { title?, content? }
     → Returns: success status

DELETE /api/documents/:id
       → Returns: 204 No Content

POST /api/documents/:id/export?format=pdf|docx|txt
     → Returns: File binary

POST /api/documents/:id/ai
     { action: "rewrite" | "summarize" | "expand", tone?: string }
     → Returns HTML: AI suggestion panel
```

**Backend Logic Needed**:
- Store documents in Drive (S3) under `.gbdocs/`
- Use LLM module for AI operations (exists at `botserver/src/llm/`)
- Query Drive metadata from AppState
- Use Askama to render document HTML

---

### 3. Designer App - Bot Configuration (`/api/bots/`, `/api/dialogs/`)

**Current URL Definition**:
- `/api/bots` - GET/POST
- `/api/bots/:id` - GET/PUT/DELETE
- `/api/bots/:id/config` - GET/PUT

**Needed Endpoints**:
```rust
GET  /api/bots/:id/dialogs
     → Returns HTML: dialog list

POST /api/bots/:id/dialogs
     { name, content: BASIC code }
     → Returns: success + dialog ID

PUT  /api/bots/:id/dialogs/:dialog_id
     { name?, content? }
     → Returns: success

DELETE /api/bots/:id/dialogs/:dialog_id
       → Returns: 204 No Content

POST /api/bots/:id/dialogs/:dialog_id/validate
     → Returns HTML: validation results (errors/warnings)

POST /api/bots/:id/dialogs/:dialog_id/deploy
     → Returns HTML: deployment status

GET  /api/bots/:id/templates
     → Returns HTML: available template list
```

**Backend Logic Needed**:
- BASIC compiler (exists at `botserver/src/basic/compiler/`)
- Store dialog files in Drive under `.gbdialogs/`
- Parse BASIC syntax for validation
- Use existing database and Drive connections

---

### 4. Sources App - Templates & Prompts (`/api/sources/`)

**Needed Endpoints**:
```rust
GET  /api/sources?category=all|templates|prompts|samples
     → Returns HTML: source card grid

GET  /api/sources/:id
     → Returns HTML: source detail view

POST /api/sources
     { name, content, category, description, tags }
     → Returns: source ID (admin only)

POST /api/sources/:id/clone
     → Returns: new source ID

POST /api/sources/templates/:id/create-bot
     { bot_name, bot_description }
     → Returns HTML: new bot created message
```

**Backend Logic Needed**:
- List files from Drive `.gbai/templates` folder
- Parse template metadata from YAML/comments
- Create new bots by copying template files
- Query Drive for available templates

---

### 5. Research App - Enhancement (`/api/kb/`)

**Current**: `/api/kb/search` exists but returns JSON

**Needed Improvements**:
```rust
GET  /api/kb/search?q=query&limit=10&offset=0
     → Already exists but needs HTMX response format
     → Return HTML partial with results (not JSON)

GET  /api/kb/stats?bot_id=
     → Returns HTML: KB statistics card

POST /api/kb/reindex?bot_id=
     → Returns HTML: reindexing status
```

**Changes Needed**:
- Add Askama template for search results HTML
- Change response from JSON to HTML
- Keep API logic the same, just change rendering

---

## HTMX Integration Pattern

### Frontend Pattern (Already in HTML files)

```html
<!-- Research app example -->
<input type="text" 
       id="researchQuery"
       placeholder="Search knowledge base..."
       hx-get="/api/kb/search"
       hx-target="#researchResults"
       hx-trigger="keyup changed delay:500ms"
       hx-include="[name='limit']"
/>

<div id="researchResults">
  <!-- Backend fills this with HTML -->
</div>
```

### Backend Response Pattern (What to implement)

Instead of returning JSON:
```json
{
  "results": [
    { "id": 1, "title": "Item 1", "snippet": "..." }
  ]
}
```

Return Askama template as HTML:
```html
<div class="result-item" hx-get="/api/kb/1" hx-trigger="click">
  <h3>Item 1</h3>
  <p class="snippet">...</p>
  <span class="meta">relevance: 0.95</span>
</div>
```

---

## Implementation Priority

### 🔴 CRITICAL (Quick Wins)

1. **Analytics Dashboard** - Pure SQL aggregation
   - Effort: 4-6 hours
   - No external dependencies
   - Just query existing tables

2. **Paper Documents** - Reuse Drive module
   - Effort: 2-3 hours
   - Use existing S3 integration
   - Minimal new code

3. **Research HTML Integration** - Change response format
   - Effort: 1-2 hours
   - KB search exists, just render differently
   - Add Askama template

### 🟡 IMPORTANT (Medium Effort)

4. **Sources Templates** - File enumeration
   - Effort: 2-3 hours
   - List Drive templates
   - Parse metadata

5. **Designer Bot Config** - Use existing compiler
   - Effort: 6-8 hours
   - BASIC compiler exists
   - Integrate with Drive storage

---

## Code Locations Reference

| Component | Location |
|-----------|----------|
| Database models | `botserver/src/schema.rs` |
| Existing handlers | `botserver/src/{drive,tasks,email,calendar,meet}/mod.rs` |
| BASIC compiler | `botserver/src/basic/compiler/mod.rs` |
| AppState | `botserver/src/core/shared/state.rs` |
| URL definitions | `botserver/src/core/urls.rs` |
| Askama templates | `botserver/templates/` |
| LLM module | `botserver/src/llm/mod.rs` |
| Drive module | `botserver/src/drive/mod.rs` |

---

## Testing Strategy

### Manual Endpoint Testing

```bash
# Test Analytics (when implemented)
curl -X GET "http://localhost:3000/api/analytics/dashboard?timeRange=day"

# Test Paper documents (when implemented)
curl -X GET "http://localhost:3000/api/documents"

# Test Research (update response format)
curl -X GET "http://localhost:3000/api/kb/search?q=test"

# Test Sources (when implemented)
curl -X GET "http://localhost:3000/api/sources?category=templates"

# Test Designer (when implemented)
curl -X GET "http://localhost:3000/api/bots/bot-id/dialogs"
```

### HTMX Integration Testing

1. Open browser DevTools Network tab
2. Click button in UI that triggers HTMX
3. Verify request goes to correct endpoint
4. Verify response is HTML (not JSON)
5. Verify HTMX swaps content into target element

---

## Key Principles

✅ **Use HTMX for UI interactions** - Let backend render HTML
✅ **Reuse existing modules** - Drive, LLM, compiler already exist
✅ **Minimal JavaScript** - Only `htmx-app.js` and `theme-manager.js` needed
✅ **Return HTML from endpoints** - Use Askama templates
✅ **Leverage AppState** - Database, Drive, LLM all available
✅ **Keep features modular** - Each app independent, can be disabled

---

## Not Implemented (By Design)

- ❌ Player app (media viewer) - Use Drive file previews instead
- ❌ Custom JavaScript per app - HTMX handles all interactions
- ❌ GraphQL - REST API with HTMX is simpler
- ❌ WebAssembly - Rust backend does heavy lifting