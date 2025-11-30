# HTMX Architecture

## Overview

General Bots Suite uses **HTMX** for its user interface - a modern approach that delivers the interactivity of a single-page application without the complexity of JavaScript frameworks like React, Vue, or Angular.

> **Why HTMX?**
> - Simpler code, easier maintenance
> - Server-rendered HTML (fast, SEO-friendly)
> - Progressive enhancement
> - No build step required
> - Smaller payload than SPA frameworks

---

## How HTMX Works

### Traditional Web vs HTMX

**Traditional (Full Page Reload):**
```
User clicks → Browser requests full page → Server returns entire HTML → Browser replaces everything
```

**HTMX (Partial Update):**
```
User clicks → HTMX requests fragment → Server returns HTML snippet → HTMX updates only that part
```

### Core Concept

HTMX extends HTML with attributes that define:
1. **What triggers the request** (`hx-trigger`)
2. **Where to send it** (`hx-get`, `hx-post`)
3. **What to update** (`hx-target`)
4. **How to update it** (`hx-swap`)

---

## HTMX Attributes Reference

### Request Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `hx-get` | GET request to URL | `hx-get="/api/tasks"` |
| `hx-post` | POST request | `hx-post="/api/tasks"` |
| `hx-put` | PUT request | `hx-put="/api/tasks/1"` |
| `hx-patch` | PATCH request | `hx-patch="/api/tasks/1"` |
| `hx-delete` | DELETE request | `hx-delete="/api/tasks/1"` |

### Trigger Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `hx-trigger` | Event that triggers request | `hx-trigger="click"` |
| | Load on page | `hx-trigger="load"` |
| | Periodic polling | `hx-trigger="every 5s"` |
| | Keyboard event | `hx-trigger="keyup changed delay:300ms"` |

### Target & Swap Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `hx-target` | Element to update | `hx-target="#results"` |
| `hx-swap` | How to insert content | `hx-swap="innerHTML"` |
| | | `hx-swap="outerHTML"` |
| | | `hx-swap="beforeend"` |
| | | `hx-swap="afterbegin"` |

---

## Suite Architecture

### File Structure

```
ui/suite/
├── index.html          # Main entry point with navigation
├── default.gbui        # Full desktop layout
├── single.gbui         # Simple chat layout
├── designer.html       # Visual dialog designer
├── editor.html         # Code editor
├── settings.html       # User settings
├── css/
│   └── app.css         # Global styles
├── js/
│   ├── layout.js       # Layout management
│   └── theme-manager.js # Theme switching
├── chat/
│   ├── chat.html       # Chat component
│   └── chat.css        # Chat styles
├── drive/
│   └── index.html      # File manager
├── tasks/
│   ├── tasks.html      # Task manager
│   └── tasks.css       # Task styles
├── mail/
│   ├── mail.html       # Email client
│   └── mail.css        # Email styles
├── calendar/
│   └── calendar.html   # Calendar view
├── meet/
│   ├── meet.html       # Video meetings
│   └── meet.css        # Meeting styles
├── paper/
│   └── paper.html      # Document editor
├── research/
│   └── research.html   # AI search
├── analytics/
│   └── analytics.html  # Dashboards
├── sources/
│   └── index.html      # Prompts & templates
├── tools/
│   └── compliance.html # Security scanner
└── monitoring/
    └── ...             # System monitoring
```

### Loading Pattern

The Suite uses **lazy loading** - components load only when needed:

```html
<!-- Main navigation in index.html -->
<a href="#chat" 
   data-section="chat"
   hx-get="/ui/suite/chat/chat.html" 
   hx-target="#main-content"
   hx-swap="innerHTML">
    Chat
</a>
```

When user clicks "Chat":
1. HTMX requests `/ui/suite/chat/chat.html`
2. Server returns the Chat HTML fragment
3. HTMX inserts it into `#main-content`
4. Only Chat code loads, not entire app

---

## Component Patterns

### 1. Load on Page View

```html
<!-- Tasks load immediately when component is shown -->
<div id="task-list"
     hx-get="/api/tasks"
     hx-trigger="load"
     hx-swap="innerHTML">
    <div class="loading">Loading tasks...</div>
</div>
```

### 2. Form Submission

```html
<!-- Add task form -->
<form hx-post="/api/tasks"
      hx-target="#task-list"
      hx-swap="afterbegin"
      hx-on::after-request="this.reset()">
    <input type="text" name="text" placeholder="New task..." required>
    <button type="submit">Add</button>
</form>
```

**Flow:**
1. User types task, clicks Add
2. HTMX POSTs form data to `/api/tasks`
3. Server creates task, returns HTML for new task item
4. HTMX inserts at beginning of `#task-list`
5. Form resets automatically

### 3. Click Actions

```html
<!-- Task item with actions -->
<div class="task-item" id="task-123">
    <input type="checkbox" 
           hx-patch="/api/tasks/123"
           hx-vals='{"completed": true}'
           hx-target="#task-123"
           hx-swap="outerHTML">
    <span>Review quarterly report</span>
    <button hx-delete="/api/tasks/123"
            hx-target="#task-123"
            hx-swap="outerHTML"
            hx-confirm="Delete this task?">
        🗑
    </button>
</div>
```

### 4. Search with Debounce

```html
<!-- Search input with 300ms delay -->
<input type="text" 
       name="q"
       placeholder="Search..."
       hx-get="/api/search"
       hx-trigger="keyup changed delay:300ms"
       hx-target="#search-results"
       hx-indicator="#search-spinner">

<span id="search-spinner" class="htmx-indicator">🔄</span>
<div id="search-results"></div>
```

**Flow:**
1. User types in search box
2. After 300ms of no typing, HTMX sends request
3. Spinner shows during request
4. Results replace `#search-results` content

### 5. Real-time Updates (WebSocket)

```html
<!-- Chat with WebSocket -->
<div id="chat-app" hx-ext="ws" ws-connect="/ws">
    <div id="messages"
         hx-get="/api/sessions/current/history"
         hx-trigger="load"
         hx-swap="innerHTML">
    </div>
    
    <form ws-send>
        <input name="content" type="text">
        <button type="submit">Send</button>
    </form>
</div>
```

**Flow:**
1. WebSocket connects on load
2. History loads via HTMX GET
3. New messages sent via WebSocket (`ws-send`)
4. Server pushes updates to all connected clients

### 6. Polling for Updates

```html
<!-- Analytics that refresh every 30 seconds -->
<div class="metric-card"
     hx-get="/api/analytics/messages/count"
     hx-trigger="load, every 30s"
     hx-swap="innerHTML">
    <!-- Content updates automatically -->
</div>
```

### 7. Infinite Scroll

```html
<!-- File list with infinite scroll -->
<div id="file-list">
    <!-- Files here -->
    
    <div hx-get="/api/files?page=2"
         hx-trigger="revealed"
         hx-swap="afterend">
        Loading more...
    </div>
</div>
```

---

## API Response Patterns

### Server Returns HTML Fragments

The server doesn't return JSON - it returns ready-to-display HTML:

**Request:**
```
GET /api/tasks
```

**Response:**
```html
<div class="task-item" id="task-1">
    <input type="checkbox">
    <span>Review quarterly report</span>
</div>
<div class="task-item" id="task-2">
    <input type="checkbox">
    <span>Update documentation</span>
</div>
```

### Swap Strategies

| Strategy | Effect |
|----------|--------|
| `innerHTML` | Replace contents of target |
| `outerHTML` | Replace entire target element |
| `beforeend` | Append inside target (at end) |
| `afterbegin` | Prepend inside target (at start) |
| `beforebegin` | Insert before target |
| `afterend` | Insert after target |
| `delete` | Delete target element |
| `none` | Don't swap (for side effects) |

---

## CSS Integration

### Loading Indicators

```css
/* Hidden by default */
.htmx-indicator {
    display: none;
}

/* Shown during request */
.htmx-request .htmx-indicator {
    display: inline-block;
}

/* Or when indicator IS the requesting element */
.htmx-request.htmx-indicator {
    display: inline-block;
}
```

### Transition Effects

```css
/* Fade in new content */
.htmx-settling {
    opacity: 0;
}

.htmx-swapping {
    opacity: 0;
    transition: opacity 0.2s ease-out;
}
```

---

## JavaScript Integration

### HTMX Events

```javascript
// After any HTMX swap
document.body.addEventListener('htmx:afterSwap', (e) => {
    console.log('Content updated:', e.detail.target);
});

// Before request
document.body.addEventListener('htmx:beforeRequest', (e) => {
    console.log('Sending request to:', e.detail.pathInfo.path);
});

// After request completes
document.body.addEventListener('htmx:afterRequest', (e) => {
    if (e.detail.successful) {
        console.log('Request succeeded');
    } else {
        console.error('Request failed');
    }
});

// On WebSocket message
document.body.addEventListener('htmx:wsAfterMessage', (e) => {
    console.log('Received:', e.detail.message);
});
```

### Triggering HTMX from JavaScript

```javascript
// Trigger an HTMX request programmatically
htmx.trigger('#task-list', 'load');

// Make an AJAX request
htmx.ajax('GET', '/api/tasks', {
    target: '#task-list',
    swap: 'innerHTML'
});

// Process new HTMX content
htmx.process(document.getElementById('new-content'));
```

---

## Designer Page Architecture

The visual dialog designer uses a hybrid approach:

### Canvas Management (JavaScript)
```javascript
// State managed in JavaScript
const state = {
    nodes: new Map(),      // Node data
    connections: [],       // Connections between nodes
    zoom: 1,               // Canvas zoom level
    pan: { x: 0, y: 0 }    // Canvas position
};
```

### File Operations (HTMX)
```html
<!-- Load file via HTMX -->
<button hx-get="/api/v1/designer/files"
        hx-target="#file-list-content">
    Open File
</button>

<!-- Save via HTMX -->
<button hx-post="/api/v1/designer/save"
        hx-include="#designer-data">
    Save
</button>
```

### Drag-and-Drop (JavaScript)
```javascript
// Toolbox items are draggable
toolboxItems.forEach(item => {
    item.addEventListener('dragstart', (e) => {
        e.dataTransfer.setData('nodeType', item.dataset.nodeType);
    });
});

// Canvas handles drop
canvas.addEventListener('drop', (e) => {
    const nodeType = e.dataTransfer.getData('nodeType');
    createNode(nodeType, e.clientX, e.clientY);
});
```

---

## Performance Considerations

### 1. Minimize Request Size

Return only what's needed:
```html
<!-- Good: Return just the updated row -->
<tr id="row-123">...</tr>

<!-- Bad: Return entire table -->
<table>...</table>
```

### 2. Use Appropriate Triggers

```html
<!-- Don't poll too frequently -->
hx-trigger="every 30s"  <!-- Good for dashboards -->
hx-trigger="every 1s"   <!-- Too frequent! -->

<!-- Debounce user input -->
hx-trigger="keyup changed delay:300ms"  <!-- Good -->
hx-trigger="keyup"                       <!-- Too many requests -->
```

### 3. Lazy Load Heavy Content

```html
<!-- Load tab content only when tab is clicked -->
<div role="tabpanel" 
     hx-get="/api/heavy-content"
     hx-trigger="intersect once">
</div>
```

### 4. Use `hx-boost` for Navigation

```html
<!-- Boost all links in nav -->
<nav hx-boost="true">
    <a href="/page1">Page 1</a>  <!-- Now uses HTMX -->
    <a href="/page2">Page 2</a>
</nav>
```

---

## Security

### CSRF Protection

HTMX automatically includes CSRF tokens:

```html
<meta name="csrf-token" content="abc123...">
```

```javascript
// Configure HTMX to send CSRF token
document.body.addEventListener('htmx:configRequest', (e) => {
    e.detail.headers['X-CSRF-Token'] = document.querySelector('meta[name="csrf-token"]').content;
});
```

### Content Security

- Server validates all inputs
- HTML is sanitized before rendering
- Authentication checked on every request

---

## Comparison: HTMX vs React

| Aspect | HTMX | React |
|--------|------|-------|
| **Learning Curve** | Low (HTML attributes) | High (JSX, hooks, state) |
| **Bundle Size** | ~14KB | ~40KB + app code |
| **Build Step** | None | Required |
| **Server Load** | More (renders HTML) | Less (returns JSON) |
| **Client Load** | Less | More |
| **SEO** | Excellent | Requires SSR |
| **Complexity** | Simple | Complex |
| **Best For** | Content sites, dashboards | Complex SPAs, offline apps |

---

## Further Reading

- [HTMX Official Documentation](https://htmx.org/docs/)
- [HTMX Examples](https://htmx.org/examples/)
- [Hypermedia Systems (Book)](https://hypermedia.systems/)
- [Chapter 04: UI Reference](./README.md)