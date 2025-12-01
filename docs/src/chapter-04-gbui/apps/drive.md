# Drive - File Management

> **Your cloud storage workspace**

<img src="../../assets/suite/drive-flow.svg" alt="Drive Flow Diagram" style="max-width: 100%; height: auto;">

---

## Overview

Drive is your personal cloud storage within General Bots Suite. Upload, organize, and share files with a familiar interface inspired by Google Drive. Built with HTMX for smooth interactions and SeaweedFS for reliable object storage.

## Interface Layout

```
┌────────────────┬──────────────────────────────────────────────────┐
│                │  🔍 Search files...                   [⊞] [≡]   │
│  [+ New ▼]     ├──────────────────────────────────────────────────┤
│                │  📁 My Drive > Projects > 2024                   │
│  ─────────────│ ─────────────────────────────────────────────────│
│  🏠 My Drive   │  [☐] Name              Size      Modified        │
│  ⭐ Starred    │  ─────────────────────────────────────────────── │
│  🕐 Recent     │  📁 Reports           -         Today           │
│  🗑 Trash      │  📁 Presentations     -         Yesterday       │
│                │  📄 Budget.xlsx       245 KB    Mar 15          │
│  ─────────────│  📄 Notes.docx        12 KB     Mar 14          │
│  Labels       │  🖼 Logo.png          89 KB     Mar 10          │
│  🔵 Work      │  📊 Sales.csv         156 KB    Mar 8           │
│  🟢 Personal  │                                                  │
│  🟡 Projects  │                                                  │
│                ├──────────────────────────────────────────────────┤
│  ─────────────│  Storage: ████████░░ 4.2 GB of 10 GB            │
│  4.2 GB used  │                                                  │
└────────────────┴──────────────────────────────────────────────────┘
```

## Features

### Upload Files

**Drag and Drop:**
1. Drag files from your computer
2. Drop anywhere in the file area
3. Upload progress shows automatically

**Click to Upload:**
1. Click **+ New** button
2. Select **Upload Files** or **Upload Folder**
3. Choose files from file picker

```html
<button hx-get="/api/v1/drive/upload-modal"
        hx-target="#modal-container"
        hx-swap="innerHTML">
    + New
</button>

<div class="upload-zone"
     ondrop="handleDrop(event)"
     ondragover="handleDragOver(event)">
    <input type="file" 
           multiple
           hx-post="/api/v1/drive/upload"
           hx-encoding="multipart/form-data"
           hx-target="#file-list">
</div>
```

### File Operations

| Action | How to Access | HTMX Attribute |
|--------|---------------|----------------|
| **Open** | Double-click | `hx-get="/api/v1/drive/open"` |
| **Download** | Right-click > Download | `hx-get="/api/v1/drive/download"` |
| **Rename** | Right-click > Rename | `hx-patch="/api/v1/drive/rename"` |
| **Copy** | Right-click > Copy | `hx-post="/api/v1/drive/copy"` |
| **Move** | Right-click > Move to | `hx-post="/api/v1/drive/move"` |
| **Star** | Right-click > Star | `hx-post="/api/v1/drive/star"` |
| **Share** | Right-click > Share | `hx-get="/api/v1/drive/share-modal"` |
| **Delete** | Right-click > Delete | `hx-delete="/api/v1/drive/file"` |

### Context Menu

```html
<div class="context-menu" id="context-menu">
    <div class="context-menu-item"
         hx-get="/api/v1/drive/open"
         hx-include="[name='selected-path']">
        📂 Open
    </div>
    <div class="context-menu-item"
         hx-get="/api/v1/drive/download"
         hx-include="[name='selected-path']">
        ⬇️ Download
    </div>
    <div class="context-menu-separator"></div>
    <div class="context-menu-item"
         hx-get="/api/v1/drive/rename-modal"
         hx-target="#modal-container">
        ✏️ Rename
    </div>
    <div class="context-menu-item"
         hx-post="/api/v1/drive/copy"
         hx-include="[name='selected-path']">
        📋 Copy
    </div>
    <div class="context-menu-item"
         hx-get="/api/v1/drive/move-modal"
         hx-target="#modal-container">
        📁 Move to...
    </div>
    <div class="context-menu-separator"></div>
    <div class="context-menu-item"
         hx-post="/api/v1/drive/star"
         hx-include="[name='selected-path']">
        ⭐ Add to Starred
    </div>
    <div class="context-menu-item"
         hx-get="/api/v1/drive/share-modal"
         hx-target="#modal-container">
        🔗 Share
    </div>
    <div class="context-menu-separator"></div>
    <div class="context-menu-item danger"
         hx-delete="/api/v1/drive/file"
         hx-include="[name='selected-path']"
         hx-confirm="Move to trash?">
        🗑 Delete
    </div>
</div>
```

### View Modes

**Grid View (⊞):**
- Large thumbnails for images
- Folder icons with previews
- Best for visual browsing

**List View (≡):**
- Detailed file information
- Sortable columns
- Best for managing many files

```html
<div class="view-toggle">
    <button class="view-toggle-btn active"
            onclick="setView('grid')">
        ⊞
    </button>
    <button class="view-toggle-btn"
            onclick="setView('list')">
        ≡
    </button>
</div>
```

### Navigation

**Sidebar:**
- My Drive - All your files
- Starred - Favorite files
- Recent - Recently accessed
- Trash - Deleted files (30-day retention)

**Breadcrumb:**
```html
<div class="breadcrumb" id="breadcrumb">
    <div class="breadcrumb-item"
         hx-get="/api/v1/drive/list?path=/"
         hx-target="#file-list">
        My Drive
    </div>
    <span class="breadcrumb-separator">/</span>
    <div class="breadcrumb-item"
         hx-get="/api/v1/drive/list?path=/Projects"
         hx-target="#file-list">
        Projects
    </div>
    <span class="breadcrumb-separator">/</span>
    <div class="breadcrumb-item current">
        2024
    </div>
</div>
```

### Labels & Organization

Create colored labels to organize files:

| Label | Color | Use Case |
|-------|-------|----------|
| 🔵 Work | Blue | Business files |
| 🟢 Personal | Green | Personal documents |
| 🟡 Projects | Yellow | Active projects |
| 🔴 Urgent | Red | Priority items |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+U` | Upload files |
| `Ctrl+N` | New folder |
| `Delete` | Move to trash |
| `Ctrl+C` | Copy selected |
| `Ctrl+X` | Cut selected |
| `Ctrl+V` | Paste |
| `Enter` | Open selected |
| `F2` | Rename selected |
| `Ctrl+A` | Select all |
| `Escape` | Deselect all |

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/drive/list` | GET | List files in directory |
| `/api/v1/drive/upload` | POST | Upload files |
| `/api/v1/drive/download` | GET | Download file |
| `/api/v1/drive/file` | DELETE | Delete file |
| `/api/v1/drive/rename` | PATCH | Rename file |
| `/api/v1/drive/move` | POST | Move file |
| `/api/v1/drive/copy` | POST | Copy file |
| `/api/v1/drive/star` | POST | Toggle star |
| `/api/v1/drive/share` | POST | Create share link |
| `/api/v1/drive/folder` | POST | Create folder |

### Query Parameters

```
GET /api/v1/drive/list?path=/Projects&sort=name&order=asc&view=grid
```

| Parameter | Values | Default |
|-----------|--------|---------|
| `path` | Directory path | `/` |
| `sort` | `name`, `size`, `modified`, `type` | `name` |
| `order` | `asc`, `desc` | `asc` |
| `view` | `grid`, `list` | `grid` |

## HTMX Integration

### File Listing

```html
<div id="file-list"
     hx-get="/api/v1/drive/list"
     hx-trigger="load"
     hx-vals='{"path": "/"}'
     hx-swap="innerHTML">
    <div class="htmx-indicator">
        Loading files...
    </div>
</div>
```

### File Upload with Progress

```html
<form hx-post="/api/v1/drive/upload"
      hx-encoding="multipart/form-data"
      hx-target="#file-list"
      hx-swap="innerHTML"
      hx-indicator="#upload-progress">
    <input type="file" name="files" multiple>
    <input type="hidden" name="path" id="current-path">
    <button type="submit">Upload</button>
</form>

<div id="upload-progress" class="htmx-indicator">
    <div class="progress-bar">
        <div class="progress-fill"></div>
    </div>
    <span>Uploading...</span>
</div>
```

### Folder Navigation

```html
<div class="file-card" 
     data-type="folder"
     data-path="/Projects"
     hx-get="/api/v1/drive/list"
     hx-vals='{"path": "/Projects"}'
     hx-target="#file-list"
     hx-trigger="dblclick">
    <div class="file-card-preview folder">
        📁
    </div>
    <div class="file-card-name">Projects</div>
</div>
```

## CSS Classes

```css
.drive-container {
    display: grid;
    grid-template-columns: 250px 1fr;
    height: calc(100vh - 64px);
}

.drive-sidebar {
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 1rem;
}

.drive-main {
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
    padding: 1rem;
}

.file-card {
    border: 2px solid transparent;
    border-radius: 8px;
    padding: 1rem;
    cursor: pointer;
    transition: all 0.2s;
}

.file-card:hover {
    background: var(--surface-hover);
    border-color: var(--border);
}

.file-card.selected {
    background: var(--primary-light);
    border-color: var(--primary);
}

.file-card-preview {
    height: 100px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 3rem;
    border-radius: 4px;
    background: var(--surface);
}

.file-card-preview img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
}

.file-card-name {
    margin-top: 0.5rem;
    font-size: 0.875rem;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.file-list {
    display: flex;
    flex-direction: column;
}

.file-row {
    display: grid;
    grid-template-columns: auto 1fr 100px 100px 150px auto;
    gap: 1rem;
    padding: 0.75rem 1rem;
    align-items: center;
    border-bottom: 1px solid var(--border);
}

.file-row:hover {
    background: var(--surface-hover);
}

.upload-zone {
    border: 2px dashed var(--border);
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    transition: all 0.2s;
}

.upload-zone.dragover {
    border-color: var(--primary);
    background: var(--primary-light);
}

.breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
}

.breadcrumb-item {
    cursor: pointer;
    color: var(--text-secondary);
}

.breadcrumb-item:hover {
    color: var(--primary);
}

.breadcrumb-item.current {
    color: var(--text-primary);
    font-weight: 500;
}

.storage-bar {
    height: 8px;
    background: var(--surface);
    border-radius: 4px;
    overflow: hidden;
}

.storage-bar-fill {
    height: 100%;
    background: var(--primary);
    border-radius: 4px;
    transition: width 0.3s;
}
```

## JavaScript Handlers

```javascript
// Drag and drop handling
function initDragAndDrop() {
    const uploadZone = document.querySelector('.upload-zone');
    
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(event => {
        uploadZone.addEventListener(event, preventDefaults);
    });
    
    ['dragenter', 'dragover'].forEach(event => {
        uploadZone.addEventListener(event, () => {
            uploadZone.classList.add('dragover');
        });
    });
    
    ['dragleave', 'drop'].forEach(event => {
        uploadZone.addEventListener(event, () => {
            uploadZone.classList.remove('dragover');
        });
    });
    
    uploadZone.addEventListener('drop', handleDrop);
}

function preventDefaults(e) {
    e.preventDefault();
    e.stopPropagation();
}

function handleDrop(e) {
    const files = e.dataTransfer.files;
    uploadFiles(files);
}

function uploadFiles(files) {
    const formData = new FormData();
    const currentPath = document.getElementById('current-path').value;
    
    formData.append('path', currentPath);
    [...files].forEach(file => {
        formData.append('files', file);
    });
    
    htmx.ajax('POST', '/api/v1/drive/upload', {
        target: '#file-list',
        swap: 'innerHTML',
        values: formData
    });
}

// Context menu
function initContextMenu() {
    const contextMenu = document.getElementById('context-menu');
    
    document.addEventListener('contextmenu', (e) => {
        const fileCard = e.target.closest('.file-card, .file-row');
        if (fileCard) {
            e.preventDefault();
            selectFile(fileCard);
            contextMenu.style.left = e.clientX + 'px';
            contextMenu.style.top = e.clientY + 'px';
            contextMenu.classList.add('visible');
        }
    });
    
    document.addEventListener('click', () => {
        contextMenu.classList.remove('visible');
    });
}

// File selection
let selectedFiles = new Set();

function selectFile(element) {
    if (!event.ctrlKey && !event.metaKey) {
        document.querySelectorAll('.file-card.selected, .file-row.selected')
            .forEach(el => el.classList.remove('selected'));
        selectedFiles.clear();
    }
    
    element.classList.toggle('selected');
    const path = element.dataset.path;
    
    if (element.classList.contains('selected')) {
        selectedFiles.add(path);
    } else {
        selectedFiles.delete(path);
    }
    
    document.querySelector('[name="selected-path"]').value = 
        [...selectedFiles].join(',');
}

// View toggle
function setView(view) {
    const fileList = document.getElementById('file-list');
    fileList.classList.toggle('file-grid', view === 'grid');
    fileList.classList.toggle('file-list-view', view === 'list');
    
    document.querySelectorAll('.view-toggle-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === view);
    });
    
    localStorage.setItem('drive-view', view);
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
            case 'u':
                e.preventDefault();
                document.getElementById('upload-input').click();
                break;
            case 'n':
                e.preventDefault();
                showModal('new-folder-modal');
                break;
            case 'a':
                e.preventDefault();
                document.querySelectorAll('.file-card, .file-row')
                    .forEach(el => el.classList.add('selected'));
                break;
        }
    }
    
    if (e.key === 'Delete' && selectedFiles.size > 0) {
        htmx.ajax('DELETE', '/api/v1/drive/file', {
            target: '#file-list',
            values: { paths: [...selectedFiles].join(',') }
        });
    }
    
    if (e.key === 'Enter' && selectedFiles.size === 1) {
        const path = [...selectedFiles][0];
        htmx.ajax('GET', '/api/v1/drive/open', {
            values: { path }
        });
    }
});
```

## File Type Icons

| Extension | Icon | Category |
|-----------|------|----------|
| `.pdf` | 📕 | Document |
| `.doc`, `.docx` | 📄 | Document |
| `.xls`, `.xlsx` | 📊 | Spreadsheet |
| `.ppt`, `.pptx` | 📽 | Presentation |
| `.jpg`, `.png`, `.gif` | 🖼 | Image |
| `.mp4`, `.mov` | 🎬 | Video |
| `.mp3`, `.wav` | 🎵 | Audio |
| `.zip`, `.rar` | 📦 | Archive |
| `.txt`, `.md` | 📝 | Text |
| Folder | 📁 | Directory |

## Storage Backend

Drive uses **SeaweedFS** for object storage:

- Distributed file system
- Automatic replication
- High availability
- Efficient for large files
- S3-compatible API

File metadata is stored in **PostgreSQL**:
- File names and paths
- Permissions and sharing
- Labels and stars
- Version history

## Troubleshooting

### Upload Fails

1. Check file size limit (default: 100MB)
2. Verify storage quota
3. Check network connection
4. Look for file type restrictions

### Files Not Displaying

1. Refresh the page
2. Check current path is valid
3. Verify permissions
4. Clear browser cache

### Context Menu Not Working

1. Enable JavaScript
2. Check for console errors
3. Try right-clicking on file directly
4. Refresh the page

## See Also

- [HTMX Architecture](../htmx-architecture.md) - How Drive uses HTMX
- [Suite Manual](../suite-manual.md) - Complete user guide
- [Chat App](./chat.md) - Share files in chat
- [Storage API](../../chapter-10-api/storage-api.md) - API reference