# Issue #001: DRIVE — Three conflicting API namespaces (ghost endpoints)

**Severity:** CRITICAL
**Components:** `botui/ui/suite/drive/`, `botserver/src/main_module/server.rs`, `botbook/src/07-user-interface/apps/drive.md`
**Type:** Ghost functionality + Documentation drift

## Description

The Drive app has **three conflicting API namespace definitions**, only one of which is actually implemented in the backend. The other two are **ghost functionality** — they call URLs that don't exist on the server, resulting in silent 404 errors.

---

## The Three Namespaces

### 1. SPA (Modern) — `/api/files/*` ✅ IMPLEMENTED
**File:** `drive.js` (line 8: `const API_BASE = "/api/files"`)
**Backend:** `server.rs:191-239` under `cfg(feature = "drive")`

Working endpoints:
- `GET /api/files/buckets` — list MinIO buckets
- `GET /api/files/list` — list files
- `GET /api/files/quota` — storage quota
- `GET /api/files/recent` — recent files
- `GET /api/files/favorite` — favorites/starred
- `GET /api/files/shared` — shared files
- `GET /api/files/search` — search
- `POST /api/files/write` — upload (base64 JSON)
- `POST /api/files/download` — download (base64) for preview
- `POST /api/files/download-binary` — binary download
- `POST /api/files/createFolder` — create folder
- `POST /api/files/delete` — delete
- `POST /api/files/copy` — copy
- `POST /api/files/move` — move/rename
- `POST /api/files/open` — open file (returns app + url)
- `POST /api/files/ai/chat` — AI assistant

**Verdict:** The modern SPA is correctly wired.

### 2. HTMX Template — `/api/drive/*` ❌ GHOST (not implemented)
**File:** `index.html` (legacy HTMX template, ~1925 lines)
**Backend:** ZERO `/api/drive/*` endpoints exist (grep for `"api/drive"` in `botserver/src/` returns empty).

Ghost endpoints returning 404:
| Method | Endpoint | Feature |
|--------|----------|---------|
| GET | `/api/drive/files?path=/` | Initial file listing |
| GET | `/api/drive/files?filter=shared\|recent\|starred\|trash` | Filters |
| GET | `/api/drive/storage` | Storage used |
| GET | `/api/drive/breadcrumb` | Navigation breadcrumb |
| GET | `/api/drive/folders` | List folders for copy/move modal |
| GET | `/api/drive/download` | HTMX-based download |
| POST | `/api/drive/upload` | Upload (multipart) |
| POST | `/api/drive/folder` | Create folder |
| POST | `/api/drive/share` | Share files |
| DELETE | `/api/drive/files` | Delete files |

### 3. Botbook Documentation — `/api/drive/*` ❌ INCORRECT
**File:** `botbook/src/07-user-interface/apps/drive.md` (lines 216-223)

Documents these endpoints that DON'T exist:
```
/api/drive/list          GET    List files
/api/drive/upload        POST   Upload file
/api/drive/download/:path GET   Download file
/api/drive/delete/:path  DELETE Delete file
/api/drive/move          POST   Move/rename file
/api/drive/copy          POST   Copy file
/api/drive/mkdir         POST   Create folder
/api/drive/share         POST   Share file
```

### 4. No-prefix endpoints — `/docs/*`, `/files/*` ❌ GHOST
**File:** `index.html` HTMX calls without `/api/` prefix

Definitively non-existent:
| Method | Path | Feature |
|--------|------|---------|
| POST | `/docs/merge` | Merge documents |
| POST | `/docs/convert` | Convert format |
| POST | `/docs/fill` | Fill template |
| POST | `/docs/export` | Export document |
| POST | `/docs/import` | Import document |
| GET | `/files/sync/status` | Sync status |
| POST | `/files/sync/start` | Start sync |
| POST | `/files/sync/stop` | Stop sync |
| POST | `/files/copy` | Copy (modal) |
| POST | `/files/move` | Move (modal) |
| POST | `/files/shareFolder` | Share folder |
| GET | `/files/versions` | Version history |
| GET | `/files/permissions` | Permissions |

### 5. drive-sentient — Complete mock, no real API
**Files:** `drive-sentient.html`, `drive-sentient.js`
Makes ZERO network calls. All data is mocked in-memory.

## Impact

- The HTMX template (`index.html`) is **completely broken** — none of its endpoints work.
- Botbook documentation is wrong, leading developers to call non-existent endpoints.
- **Two competing implementations** (SPA + HTMX template) for the same app.
- 17+ ghost endpoints pollute the codebase.

## Suggested Fix

1. **Unify** the API namespace: migrate `index.html` to use `/api/files/*` or implement `/api/drive/*` routes in the backend as aliases.
2. **Fix** botbook documentation to reflect `/api/files/*`.
3. **Remove** or mark the HTMX `index.html` template as deprecated.
4. **Remove** or mark `drive-sentient.*` as prototype (not for production).
5. **Implement or remove** the `/docs/*` and no-prefix `/files/*` endpoints.
